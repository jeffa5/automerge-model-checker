use crate::register::GlobalMsg;
use automerge::ObjType;
use stateright::actor::Command;
use stateright::actor::{Actor, Out};

mod delete;
mod insert;
mod put;

pub use delete::ListDeleter;
pub use delete::MapSingleDeleter;
pub use insert::ListInserter;
pub use put::ListStartPutter;
pub use put::MapSinglePutter;

type RequestId = usize;
type Key = String;
type Value = String;

/// A client that generates actions for peers to process.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Client {
    MapSinglePutter(put::MapSinglePutter),
    ListStartPutter(put::ListStartPutter),
    MapSingleDeleter(delete::MapSingleDeleter),
    ListDeleter(delete::ListDeleter),
    ListInserter(insert::ListInserter),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ClientMsg {
    /// Message originating from clients to servers.
    Request(Request),

    /// Message originating from server to client.
    Response(Response),
}

/// Messages that clients send to servers.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Request {
    /// Indicates that a value should be written.
    PutMap(RequestId, Key, Value),
    /// Indicates that a list element should be overwritten.
    PutList(RequestId, usize, Value),
    /// Indicates that an object should be created.
    PutObject(RequestId, Key, ObjType),
    /// Indicates that a value should be inserted into the list.
    Insert(RequestId, usize, Value),
    /// Indicates that a value should be retrieved.
    Get(RequestId, Key),
    /// Indicates that a value should be deleted.
    DeleteMap(RequestId, Key),
    /// Indicates that a list element should be deleted.
    DeleteList(RequestId, usize),
}

/// Messages that servers send to clients.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Response {
    /// Indicates a successful request with a value response. Analogous to an HTTP 2XX.
    AckWithValue(RequestId, Value),
    /// Indicates a successful request with no value response. Analogous to an HTTP 2XX.
    Ack(RequestId),
}

impl Actor for Client {
    type Msg = GlobalMsg;

    type State = ();

    /// Clients generate all of their actions on start so we only need to initialise them.
    fn on_start(
        &self,
        id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        match self {
            Client::MapSinglePutter(p) => {
                let mut out = Out::new();
                p.on_start(id, &mut out);
                let mut out: Out<Client> = out
                    .into_iter()
                    .map(|m| match m {
                        Command::Send(id, msg) => {
                            Command::Send(id, GlobalMsg::External(ClientMsg::Request(msg)))
                        }
                        Command::SetTimer(r) => Command::SetTimer(r),
                        Command::CancelTimer => Command::CancelTimer,
                    })
                    .collect();
                o.append(&mut out);
            }
            Client::ListStartPutter(p) => {
                let mut out = Out::new();
                p.on_start(id, &mut out);
                let mut out: Out<Client> = out
                    .into_iter()
                    .map(|m| match m {
                        Command::Send(id, msg) => {
                            Command::Send(id, GlobalMsg::External(ClientMsg::Request(msg)))
                        }
                        Command::SetTimer(r) => Command::SetTimer(r),
                        Command::CancelTimer => Command::CancelTimer,
                    })
                    .collect();
                o.append(&mut out);
            }
            Client::MapSingleDeleter(d) => {
                let mut out = Out::new();
                d.on_start(id, &mut out);
                let mut out: Out<Client> = out
                    .into_iter()
                    .map(|m| match m {
                        Command::Send(id, msg) => {
                            Command::Send(id, GlobalMsg::External(ClientMsg::Request(msg)))
                        }
                        Command::SetTimer(r) => Command::SetTimer(r),
                        Command::CancelTimer => Command::CancelTimer,
                    })
                    .collect();
                o.append(&mut out);
            }
            Client::ListDeleter(d) => {
                let mut out = Out::new();
                d.on_start(id, &mut out);
                let mut out: Out<Client> = out
                    .into_iter()
                    .map(|m| match m {
                        Command::Send(id, msg) => {
                            Command::Send(id, GlobalMsg::External(ClientMsg::Request(msg)))
                        }
                        Command::SetTimer(r) => Command::SetTimer(r),
                        Command::CancelTimer => Command::CancelTimer,
                    })
                    .collect();
                o.append(&mut out);
            }
            Client::ListInserter(a) => {
                let mut out = Out::new();
                a.on_start(id, &mut out);
                let mut out: Out<Client> = out
                    .into_iter()
                    .map(|m| match m {
                        Command::Send(id, msg) => {
                            Command::Send(id, GlobalMsg::External(ClientMsg::Request(msg)))
                        }
                        Command::SetTimer(r) => Command::SetTimer(r),
                        Command::CancelTimer => Command::CancelTimer,
                    })
                    .collect();
                o.append(&mut out);
            }
        }
    }
}
