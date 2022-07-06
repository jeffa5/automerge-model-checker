use std::borrow::Cow;

use crate::client::ClientMsg;
use crate::doc::Doc;
use crate::MyRegisterMsg;
use automerge::sync;
use automerge::Automerge;
use automerge::Change;
use stateright::actor::Actor;
use stateright::actor::Id;
use stateright::actor::Out;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Peer {
    pub peers: Vec<Id>,
    pub sync_method: SyncMethod,
    pub message_acks: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ArgEnum)]
pub enum SyncMethod {
    Changes,
    Messages,
    SaveLoad,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PeerMsg {
    // TODO: make this use the raw struct to avoid serde overhead
    SyncMessage { message_bytes: Vec<u8> },
    SyncChange { change_bytes: Vec<u8> },
    SyncSaveLoad { doc_bytes: Vec<u8> },
}

impl Actor for Peer {
    type Msg = MyRegisterMsg;

    type State = Box<Doc>;

    fn on_start(&self, id: Id, _o: &mut Out<Self>) -> Self::State {
        Box::new(Doc::new(id))
    }

    fn on_msg(
        &self,
        _id: Id,
        state: &mut std::borrow::Cow<Self::State>,
        src: Id,
        msg: Self::Msg,
        o: &mut Out<Self>,
    ) {
        match msg {
            MyRegisterMsg::Client(ClientMsg::PutMap(id, key, value)) => {
                // apply the op locally
                state.to_mut().put(key, value);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, MyRegisterMsg::Client(ClientMsg::PutOk(id)));
                }

                self.sync(state, o)
            }
            MyRegisterMsg::Client(ClientMsg::PutList(id, index, value)) => {
                // apply the op locally
                state.to_mut().put_list(index, value);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, MyRegisterMsg::Client(ClientMsg::PutOk(id)));
                }

                self.sync(state, o);
            }
            MyRegisterMsg::Client(ClientMsg::PutObject(id, key, obj_type)) => {
                // apply the op locally
                state.to_mut().put_object(key, obj_type);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, MyRegisterMsg::Client(ClientMsg::PutObjectOk(id)));
                }

                self.sync(state, o);
            }
            MyRegisterMsg::Client(ClientMsg::Insert(id, index, value)) => {
                // apply the op locally
                state.to_mut().insert(index, value);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, MyRegisterMsg::Client(ClientMsg::PutObjectOk(id)));
                }

                self.sync(state, o);
            }
            MyRegisterMsg::Client(ClientMsg::Get(id, key)) => {
                if let Some(value) = state.get(&key) {
                    if self.message_acks {
                        // respond to the query (not totally necessary for this)
                        o.send(src, MyRegisterMsg::Client(ClientMsg::GetOk(id, value)))
                    }
                }
            }
            MyRegisterMsg::Client(ClientMsg::DeleteMap(id, key)) => {
                // apply the op locally
                state.to_mut().delete(&key);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, MyRegisterMsg::Client(ClientMsg::DeleteOk(id)));
                }

                self.sync(state, o);
            }
            MyRegisterMsg::Client(ClientMsg::DeleteList(id, index)) => {
                // apply the op locally
                state.to_mut().delete_list(index);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, MyRegisterMsg::Client(ClientMsg::DeleteOk(id)));
                }

                self.sync(state, o);
            }
            MyRegisterMsg::Internal(PeerMsg::SyncMessage { message_bytes }) => {
                let message = sync::Message::decode(&message_bytes).unwrap();
                // receive the sync message
                state.to_mut().receive_sync_message(src.into(), message);
                // try and generate a reply
                if let Some(message) = state.to_mut().generate_sync_message(src.into()) {
                    o.send(
                        src,
                        MyRegisterMsg::Internal(PeerMsg::SyncMessage {
                            message_bytes: message.encode(),
                        }),
                    )
                }
            }
            MyRegisterMsg::Internal(PeerMsg::SyncChange { change_bytes }) => {
                let change = Change::from_bytes(change_bytes).unwrap();
                state.to_mut().apply_change(change)
            }
            MyRegisterMsg::Internal(PeerMsg::SyncSaveLoad { doc_bytes }) => {
                let mut other_doc = Automerge::load(&doc_bytes).unwrap();
                state.to_mut().merge(&mut other_doc);
            }
            MyRegisterMsg::Client(ClientMsg::PutOk(_id)) => {}
            MyRegisterMsg::Client(ClientMsg::PutObjectOk(_id)) => {}
            MyRegisterMsg::Client(ClientMsg::InsertOk(_id)) => {}
            MyRegisterMsg::Client(ClientMsg::GetOk(_id, _value)) => {}
            MyRegisterMsg::Client(ClientMsg::DeleteOk(_id)) => {}
        }
    }
}

impl Peer {
    fn sync(&self, state: &mut Cow<<Self as Actor>::State>, o: &mut Out<Self>) {
        match self.sync_method {
            SyncMethod::Changes => {
                if let Some(change) = state.last_local_change() {
                    o.broadcast(
                        &self.peers,
                        &MyRegisterMsg::Internal(PeerMsg::SyncChange {
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
                            MyRegisterMsg::Internal(PeerMsg::SyncMessage {
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
                    &MyRegisterMsg::Internal(PeerMsg::SyncSaveLoad { doc_bytes: bytes }),
                );
            }
        }
    }
}
