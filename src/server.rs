use std::borrow::Cow;

use crate::client::ClientMsg;
use crate::client::Request;
use crate::client::Response;
use crate::doc::Doc;
use crate::register::GlobalMsg;
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
pub struct Server {
    pub peers: Vec<Id>,
    pub sync_method: SyncMethod,
    pub message_acks: bool,
}

/// Methods for syncing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ArgEnum)]
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

impl Actor for Server {
    type Msg = GlobalMsg;

    type State = Box<Doc>;

    /// Servers don't do things on their own unless told to.
    fn on_start(&self, id: Id, _o: &mut Out<Self>) -> Self::State {
        Box::new(Doc::new(id))
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
            GlobalMsg::External(ClientMsg::Request(Request::PutMap(key, value))) => {
                // apply the op locally
                state.to_mut().put_map(key, value);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, GlobalMsg::External(ClientMsg::Response(Response::Ack)));
                }

                self.sync(state, o)
            }
            GlobalMsg::External(ClientMsg::Request(Request::PutList(index, value))) => {
                // apply the op locally
                state.to_mut().put_list(index, value);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, GlobalMsg::External(ClientMsg::Response(Response::Ack)));
                }

                self.sync(state, o);
            }
            GlobalMsg::External(ClientMsg::Request(Request::PutObjectMap(key, obj_type))) => {
                // apply the op locally
                state.to_mut().put_object(key, obj_type);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, GlobalMsg::External(ClientMsg::Response(Response::Ack)));
                }

                self.sync(state, o);
            }
            GlobalMsg::External(ClientMsg::Request(Request::PutObjectList(index, obj_type))) => {
                // apply the op locally
                state.to_mut().put_object_list(index, obj_type);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, GlobalMsg::External(ClientMsg::Response(Response::Ack)));
                }

                self.sync(state, o);
            }
            GlobalMsg::External(ClientMsg::Request(Request::Insert(index, value))) => {
                // apply the op locally
                state.to_mut().insert(index, value);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, GlobalMsg::External(ClientMsg::Response(Response::Ack)));
                }

                self.sync(state, o);
            }
            GlobalMsg::External(ClientMsg::Request(Request::InsertObject(index, objtype))) => {
                // apply the op locally
                state.to_mut().insert_object(index, objtype);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, GlobalMsg::External(ClientMsg::Response(Response::Ack)));
                }

                self.sync(state, o);
            }
            GlobalMsg::External(ClientMsg::Request(Request::GetMap(key))) => {
                if let Some(value) = state.get(&key) {
                    if self.message_acks {
                        // respond to the query (not totally necessary for this)
                        o.send(
                            src,
                            GlobalMsg::External(ClientMsg::Response(Response::AckWithValue(value))),
                        )
                    }
                }
            }
            GlobalMsg::External(ClientMsg::Request(Request::GetList(index))) => {
                if let Some(value) = state.get_list(index) {
                    if self.message_acks {
                        // respond to the query (not totally necessary for this)
                        o.send(
                            src,
                            GlobalMsg::External(ClientMsg::Response(Response::AckWithValue(value))),
                        )
                    }
                }
            }
            GlobalMsg::External(ClientMsg::Request(Request::DeleteMap(key))) => {
                // apply the op locally
                state.to_mut().delete(&key);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, GlobalMsg::External(ClientMsg::Response(Response::Ack)));
                }

                self.sync(state, o);
            }
            GlobalMsg::External(ClientMsg::Request(Request::DeleteList(index))) => {
                // apply the op locally
                state.to_mut().delete_list(index);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, GlobalMsg::External(ClientMsg::Response(Response::Ack)));
                }

                self.sync(state, o);
            }
            GlobalMsg::Internal(ServerMsg::SyncMessageRaw { message_bytes }) => {
                let message = sync::Message::decode(&message_bytes).unwrap();
                // receive the sync message
                state.to_mut().receive_sync_message(src.into(), message);
                // try and generate a reply
                if let Some(message) = state.to_mut().generate_sync_message(src.into()) {
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
                state.to_mut().apply_change(change)
            }
            GlobalMsg::Internal(ServerMsg::SyncSaveLoadRaw { doc_bytes }) => {
                let mut other_doc = Automerge::load(&doc_bytes).unwrap();
                state.to_mut().merge(&mut other_doc);
            }
            GlobalMsg::External(ClientMsg::Response(Response::AckWithValue(_value))) => {}
            GlobalMsg::External(ClientMsg::Response(Response::Ack)) => {}
        }
    }
}

impl Server {
    /// Handle generating a sync message after some changes have been made.
    fn sync(&self, state: &mut Cow<<Self as Actor>::State>, o: &mut Out<Self>) {
        match self.sync_method {
            SyncMethod::Changes => {
                if let Some(change) = state.last_local_change() {
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
                    if let Some(message) = state.to_mut().generate_sync_message((*peer).into()) {
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
                let bytes = state.to_mut().save();
                o.broadcast(
                    &self.peers,
                    &GlobalMsg::Internal(ServerMsg::SyncSaveLoadRaw { doc_bytes: bytes }),
                );
            }
        }
    }
}
