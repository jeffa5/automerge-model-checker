use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use crate::app::Application;
use crate::client::ClientFunction;
use crate::client::ClientMsg;
use crate::msg::GlobalMsg;
use automerge::sync;
use automerge::Automerge;
use automerge::Change;
use stateright::actor::Actor;
use stateright::actor::Id;
use stateright::actor::Out;

/// A peer in the automerge network.
///
/// Servers can be thought of user's applications.
/// They keep state over restarts and can process operations from clients, as well as sync these to
/// other peers.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Server<C> {
    pub peers: Vec<Id>,
    pub sync_method: SyncMethod,
    pub message_acks: bool,
    pub client_function: C,
}

/// Methods for syncing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SyncMethod {
    Changes,
    Messages,
    SaveLoad,
}

/// Messages that servers send to each other.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ServerMsg {
    // TODO: make this use the raw struct to avoid serde overhead
    SyncMessageRaw { message_bytes: Vec<u8> },
    SyncChangeRaw { change_bytes: Vec<u8> },
    SyncSaveLoadRaw { doc_bytes: Vec<u8> },
}

impl<C: ClientFunction> Actor for Server<C> {
    type Msg = GlobalMsg<C>;

    type State = C::Application;

    /// Servers don't do things on their own unless told to.
    fn on_start(&self, id: Id, _o: &mut Out<Self>) -> Self::State {
        C::Application::new(id)
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
            GlobalMsg::External(ClientMsg::Request(request)) => {
                let output = self.client_function.execute(state, request);
                o.send(src, GlobalMsg::External(ClientMsg::Response(output)));

                self.sync(state, o)
            }
            GlobalMsg::Internal(ServerMsg::SyncMessageRaw { message_bytes }) => {
                let message = sync::Message::decode(&message_bytes).unwrap();
                let document = state.to_mut().document_mut();
                // receive the sync message
                document.receive_sync_message(src.into(), message);
                // try and generate a reply
                if let Some(message) = document.generate_sync_message(src.into()) {
                    o.send(
                        src,
                        GlobalMsg::Internal(ServerMsg::SyncMessageRaw {
                            message_bytes: message.encode(),
                        }),
                    )
                }
            }
            GlobalMsg::Internal(ServerMsg::SyncChangeRaw { change_bytes }) => {
                let change = Change::from_bytes(change_bytes).unwrap();
                state.to_mut().document_mut().apply_change(change)
            }
            GlobalMsg::Internal(ServerMsg::SyncSaveLoadRaw { doc_bytes }) => {
                let mut other_doc = Automerge::load(&doc_bytes).unwrap();
                state.to_mut().document_mut().merge(&mut other_doc);
            }
            GlobalMsg::External(ClientMsg::Response(_)) => {
                // we shouldn't be receiving responses
            }
        }
    }
}

impl<C: ClientFunction> Server<C> {
    /// Handle generating a sync message after some changes have been made.
    fn sync(&self, state: &mut Cow<<Self as Actor>::State>, o: &mut Out<Self>) {
        match self.sync_method {
            SyncMethod::Changes => {
                if let Some(change) = state.document().last_local_change() {
                    o.broadcast(
                        &self.peers,
                        &GlobalMsg::Internal(ServerMsg::SyncChangeRaw {
                            change_bytes: change.raw_bytes().to_vec(),
                        }),
                    )
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
                            GlobalMsg::Internal(ServerMsg::SyncMessageRaw {
                                message_bytes: message.encode(),
                            }),
                        )
                    }
                }
            }
            SyncMethod::SaveLoad => {
                let bytes = state.to_mut().document_mut().save();
                o.broadcast(
                    &self.peers,
                    &GlobalMsg::Internal(ServerMsg::SyncSaveLoadRaw { doc_bytes: bytes }),
                );
            }
        }
    }
}
