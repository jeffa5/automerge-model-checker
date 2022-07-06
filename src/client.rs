use crate::MyRegisterMsg;
use crate::{Key, RequestId, Value};
use automerge::ObjType;
use stateright::actor::{Actor, Out};

mod delete;
mod insert;
mod put;

pub use delete::ListStartDeleter;
pub use delete::MapSingleDeleter;
pub use insert::ListStartInserter;
pub use put::ListStartPutter;
pub use put::MapSinglePutter;

/// A client that generates actions for peers to process.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Client {
    MapSinglePutter(put::MapSinglePutter),
    ListStartPutter(put::ListStartPutter),
    MapSingleDeleter(delete::MapSingleDeleter),
    ListStartDeleter(delete::ListStartDeleter),
    ListStartInserter(insert::ListStartInserter),
}

/// Messages that clients send to peers.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ClientMsg {
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

    /// Indicates a successful `Put`. Analogous to an HTTP 2XX.
    PutOk(RequestId),
    /// Indicates a successful `PutObject`. Analogous to an HTTP 2XX.
    PutObjectOk(RequestId),
    /// Indicates a successful `Insert`. Analogous to an HTTP 2XX.
    InsertOk(RequestId),
    /// Indicates a successful `Get`. Analogous to an HTTP 2XX.
    GetOk(RequestId, Value),
    /// Indicates a successful `Delete`. Analogous to an HTTP 2XX.
    DeleteOk(RequestId),
}

impl Actor for Client {
    type Msg = MyRegisterMsg;

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
                o.append(&mut out);
            }
            Client::ListStartPutter(p) => {
                let mut out = Out::new();
                p.on_start(id, &mut out);
                o.append(&mut out);
            }
            Client::MapSingleDeleter(d) => {
                let mut out = Out::new();
                d.on_start(id, &mut out);
                o.append(&mut out);
            }
            Client::ListStartDeleter(d) => {
                let mut out = Out::new();
                d.on_start(id, &mut out);
                o.append(&mut out);
            }
            Client::ListStartInserter(a) => {
                let mut out = Out::new();
                a.on_start(id, &mut out);
                o.append(&mut out);
            }
        }
    }
}
