use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

use crate::bytes::Bytes;
use crate::client::Application;
use crate::client::ApplicationMsg;
use crate::client::DerefDocument;
use crate::global::GlobalMsg;
use crate::global::GlobalTimer;
use automerge::sync;
use automerge::Automerge;
use automerge::Change;
use stateright::actor::model_timeout;
use stateright::actor::Actor;
use stateright::actor::Id;
use stateright::actor::Out;

/// A peer in the automerge network.
///
/// Servers can be thought of as user's applications.
/// They keep state over restarts and can process operations from clients, as well as sync these to
/// other peers.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Server<A> {
    /// Ids of peers of this server.
    pub peers: Vec<Id>,
    /// Method to synchronise with peers.
    pub sync_method: SyncMethod,
    /// Whether to trigger restarts in the application.
    pub restarts: bool,
    /// Application logic.
    pub app: A,
}

/// Methods for syncing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ValueEnum)]
pub enum SyncMethod {
    /// Broadcast changes produced locally in this document to peers.
    Changes,
    /// Use the Automerge sync protocol to send changes to peers.
    Messages,
    /// Save the current document and send its entirety to peers for merging.
    SaveLoad,
}

/// Messages that servers send to each other.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ServerMsg {
    /// A message containing multiple changes.
    SyncChangeRaw {
        /// Bytes of the changes.
        missing_changes_bytes: Vec<Bytes>,
    },
    /// A regular sync message.
    SyncMessageRaw {
        /// The encoded message.
        message_bytes: Bytes,
    },
    /// A saved document.
    SyncSaveLoadRaw {
        /// Bytes of the saved document.
        doc_bytes: Bytes,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Timer {
    Synchronise,
    Restart,
}

impl<A: Application> Actor for Server<A> {
    type Msg = GlobalMsg<A>;

    type State = A::State;

    type Timer = GlobalTimer;

    /// Servers don't do things on their own unless told to.
    fn on_start(&self, id: Id, o: &mut Out<Self>) -> Self::State {
        // Start a timer for periodic syncing.
        o.set_timer(GlobalTimer::Server(Timer::Synchronise), model_timeout());
        if self.restarts {
            o.set_timer(GlobalTimer::Server(Timer::Restart), model_timeout());
        }
        self.app.init(usize::from(id))
    }

    /// Process a message from another peer or client.
    fn on_msg(
        &self,
        _id: Id,
        state: &mut std::borrow::Cow<Self::State>,
        src: Id,
        msg: Self::Msg,
        o: &mut Out<Self>,
    ) {
        match msg {
            GlobalMsg::ClientToServer(ApplicationMsg::Input(request)) => {
                let output = self.app.execute(state, request);
                o.send(
                    src,
                    GlobalMsg::ClientToServer(ApplicationMsg::Output(output)),
                );
            }
            GlobalMsg::ServerToServer(ServerMsg::SyncMessageRaw { message_bytes }) => {
                let message = sync::Message::decode(&message_bytes.0).unwrap();
                let document = state.to_mut().document_mut();
                // receive the sync message
                document.receive_sync_message(src.into(), message);
                // try and generate a reply
                if let Some(message) = document.generate_sync_message(src.into()) {
                    o.send(
                        src,
                        GlobalMsg::ServerToServer(ServerMsg::SyncMessageRaw {
                            message_bytes: Bytes(message.encode()),
                        }),
                    )
                }
            }
            GlobalMsg::ServerToServer(ServerMsg::SyncChangeRaw {
                missing_changes_bytes,
            }) => {
                for change_bytes in missing_changes_bytes {
                    let change = Change::from_bytes(change_bytes.0).unwrap();
                    state.to_mut().document_mut().apply_change(change)
                }
            }
            GlobalMsg::ServerToServer(ServerMsg::SyncSaveLoadRaw { doc_bytes }) => {
                let mut other_doc = Automerge::load(&doc_bytes.0).unwrap();
                state.to_mut().document_mut().merge(&mut other_doc);
            }
            GlobalMsg::ClientToServer(ApplicationMsg::Output(_)) => {
                // we shouldn't be receiving responses
                unreachable!()
            }
        }
    }

    /// Handle a timeout, used to trigger syncing events as this gets interleaved when checking.
    fn on_timeout(
        &self,
        _id: Id,
        state: &mut Cow<Self::State>,
        timer: &Self::Timer,
        o: &mut Out<Self>,
    ) {
        match timer {
            GlobalTimer::Server(Timer::Synchronise) => {
                o.set_timer(
                    timer.clone(),
                    Duration::from_secs(1)..Duration::from_secs(2),
                );
                self.sync(state, o)
            }
            GlobalTimer::Server(Timer::Restart) => {
                o.set_timer(
                    timer.clone(),
                    Duration::from_secs(1)..Duration::from_secs(2),
                );
                self.restart(state)
            }
        }
    }
}

impl<A: Application> Server<A> {
    /// Handle generating a sync message after some changes have been made.
    fn sync(&self, state: &mut Cow<<Self as Actor>::State>, o: &mut Out<Self>) {
        match &self.sync_method {
            SyncMethod::Changes => {
                let new_changes_from_us = state
                    .document()
                    .get_last_local_changes_for_sync()
                    .map(|c| Bytes(c.raw_bytes().to_vec()))
                    .collect::<Vec<_>>();
                if !new_changes_from_us.is_empty() {
                    o.broadcast(
                        &self.peers,
                        &GlobalMsg::ServerToServer(ServerMsg::SyncChangeRaw {
                            missing_changes_bytes: new_changes_from_us,
                        }),
                    );
                    state.to_mut().document_mut().update_last_sent_heads();
                }
            }
            SyncMethod::Messages => {
                // each peer has a specific state to manage in the sync connection
                for peer in &self.peers {
                    if let Some(message) = state
                        .to_mut()
                        .document_mut()
                        .generate_sync_message((*peer).into())
                    {
                        o.send(
                            *peer,
                            GlobalMsg::ServerToServer(ServerMsg::SyncMessageRaw {
                                message_bytes: Bytes(message.encode()),
                            }),
                        )
                    }
                }
            }
            SyncMethod::SaveLoad => {
                let state = state.to_mut();
                let bytes = state.document_mut().save();
                state.document_mut().update_last_sent_heads();
                o.broadcast(
                    &self.peers,
                    &GlobalMsg::ServerToServer(ServerMsg::SyncSaveLoadRaw {
                        doc_bytes: Bytes(bytes),
                    }),
                );
            }
        }
    }

    fn restart(&self, state: &mut Cow<<Self as Actor>::State>) {
        let document = state.to_mut().document_mut();
        let bytes = document.save();
        document.load(&bytes);
        document.reload_sync_states()
    }
}
