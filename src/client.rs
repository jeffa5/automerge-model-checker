use std::borrow::Cow;

use crate::MyRegisterMsg;
use crate::{Key, RequestId, Value};
use stateright::actor::{Actor, Id};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RequestType {
    Put,
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Client {
    // count of requests to send
    pub count: usize,
    /// Whether to send a get request after each mutation
    pub follow_up_gets: bool,
    // number of servers in the system
    pub server_count: usize,
    // type of requests to send to the servers
    pub request_type: RequestType,
    // Whether messages we send will get acknowledgements or not
    pub message_acks: bool,
    /// The key to interact with.
    pub key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ClientState {
    pub awaiting: Option<RequestId>,
    pub op_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ClientMsg {
    /// Indicates that a value should be written.
    Put(RequestId, Key, Value),
    /// Indicates that a value should be retrieved.
    Get(RequestId, Key),
    /// Indicates that a value should be deleted.
    Delete(RequestId, Key),

    /// Indicates a successful `Put`. Analogous to an HTTP 2XX.
    PutOk(RequestId),
    /// Indicates a successful `Get`. Analogous to an HTTP 2XX.
    GetOk(RequestId, Value),
    /// Indicates a successful `Delete`. Analogous to an HTTP 2XX.
    DeleteOk(RequestId),
}

impl Actor for Client {
    type Msg = MyRegisterMsg;

    type State = ClientState;

    fn on_start(
        &self,
        id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        let index: usize = id.into();
        if index < self.server_count {
            panic!("MyRegisterActor clients must be added to the model after servers.");
        }

        if self.message_acks {
            // we'll wait for acks before sending more messages

            if self.count > 0 {
                let unique_request_id = index; // next will be 2 * index
                let value = (b'A' + (index % self.server_count) as u8) as char;
                let msg = match self.request_type {
                    RequestType::Put => {
                        ClientMsg::Put(unique_request_id, self.key.to_owned(), value.to_string())
                    }
                    RequestType::Delete => {
                        ClientMsg::Delete(unique_request_id, self.key.to_owned())
                    }
                };
                o.send(
                    Id::from(index % self.server_count),
                    MyRegisterMsg::Client(msg),
                );
                ClientState {
                    awaiting: Some(unique_request_id),
                    op_count: 1,
                }
            } else {
                ClientState {
                    awaiting: None,
                    op_count: 0,
                }
            }
        } else {
            for i in 0..self.count {
                let unique_request_id = (i + 1) * index; // next will be 2 * index
                let value = (b'A' + (index % self.server_count) as u8) as char;
                let msg = match self.request_type {
                    RequestType::Put => {
                        ClientMsg::Put(unique_request_id, self.key.to_owned(), value.to_string())
                    }
                    RequestType::Delete => {
                        ClientMsg::Delete(unique_request_id, self.key.to_owned())
                    }
                };
                o.send(
                    Id::from(index % self.server_count),
                    MyRegisterMsg::Client(msg),
                );
            }

            ClientState {
                awaiting: None,
                op_count: 0,
            }
        }
    }

    fn on_msg(
        &self,
        id: Id,
        state: &mut std::borrow::Cow<Self::State>,
        _src: Id,
        msg: Self::Msg,
        o: &mut stateright::actor::Out<Self>,
    ) {
        if let Some(awaiting) = state.awaiting {
            match msg {
                MyRegisterMsg::Client(ClientMsg::PutOk(request_id)) => {
                    assert_eq!(request_id, awaiting);
                    let index: usize = id.into();
                    let unique_request_id = (state.op_count + 1) * index;
                    if state.op_count < self.count {
                        let value = (b'Z' - (index % self.server_count) as u8) as char;
                        o.send(
                            Id::from(index % self.server_count),
                            MyRegisterMsg::Client(ClientMsg::Put(
                                unique_request_id,
                                self.key.to_owned(),
                                value.to_string(),
                            )),
                        );
                        *state = Cow::Owned(ClientState {
                            awaiting: Some(unique_request_id),
                            op_count: state.op_count + 1,
                        });
                    } else if self.follow_up_gets {
                        o.send(
                            Id::from(index % self.server_count),
                            MyRegisterMsg::Client(ClientMsg::Get(
                                unique_request_id,
                                self.key.to_owned(),
                            )),
                        );
                        *state = Cow::Owned(ClientState {
                            awaiting: Some(unique_request_id),
                            op_count: state.op_count + 1,
                        });
                    } else {
                        *state = Cow::Owned(ClientState {
                            awaiting: None,
                            op_count: state.op_count + 1,
                        });
                    }
                }
                MyRegisterMsg::Client(ClientMsg::DeleteOk(request_id)) => {
                    assert_eq!(request_id, awaiting);
                    let index: usize = id.into();
                    let unique_request_id = (state.op_count + 1) * index;
                    if state.op_count < self.count {
                        o.send(
                            Id::from(index % self.server_count),
                            MyRegisterMsg::Client(ClientMsg::Delete(
                                unique_request_id,
                                self.key.to_owned(),
                            )),
                        );
                        *state = Cow::Owned(ClientState {
                            awaiting: Some(unique_request_id),
                            op_count: state.op_count + 1,
                        });
                    } else if self.follow_up_gets {
                        o.send(
                            Id::from(index % self.server_count),
                            MyRegisterMsg::Client(ClientMsg::Get(
                                unique_request_id,
                                self.key.to_owned(),
                            )),
                        );
                        *state = Cow::Owned(ClientState {
                            awaiting: Some(unique_request_id),
                            op_count: state.op_count + 1,
                        });
                    } else {
                        *state = Cow::Owned(ClientState {
                            awaiting: None,
                            op_count: state.op_count + 1,
                        });
                    }
                }
                MyRegisterMsg::Client(ClientMsg::GetOk(request_id, _value)) => {
                    assert_eq!(request_id, awaiting);
                    // finished
                    *state = Cow::Owned(ClientState {
                        awaiting: None,
                        op_count: state.op_count + 1,
                    });
                }
                MyRegisterMsg::Client(ClientMsg::Put(_, _, _)) => {}
                MyRegisterMsg::Client(ClientMsg::Get(_, _)) => {}
                MyRegisterMsg::Client(ClientMsg::Delete(_, _)) => {}
                MyRegisterMsg::Internal(_) => {}
            }
        }
    }
}
