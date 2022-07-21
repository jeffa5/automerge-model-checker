use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use crate::doc::Doc;
use crate::trigger::TriggerMsg;

mod delete;
mod insert;
mod put;

pub use delete::ListDeleter;
pub use delete::MapSingleDeleter;
pub use insert::ListInserter;
pub use put::ListPutter;
pub use put::MapSinglePutter;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Client {
    pub map_single_putter: put::MapSinglePutter,
    pub list_start_putter: put::ListPutter,
    pub map_single_deleter: delete::MapSingleDeleter,
    pub list_deleter: delete::ListDeleter,
    pub list_inserter: insert::ListInserter,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ClientMsg<C: ClientFunction> {
    /// Message originating from clients to servers.
    Request(C::Input),
    /// Message originating from server to client.
    Response(C::Output),
}

/// A ClientFunction is coupled with a server and implements an atomic action against the document.
/// This ensures that no sync messages are applied within the body of execution.
pub trait ClientFunction: Clone + Hash + Eq + Debug {
    type Input: Clone + Hash + Eq + Debug;
    type Output: Clone + Hash + Eq + Debug;

    fn execute(&self, document: &mut Cow<Box<Doc>>, input: Self::Input) -> Self::Output;
}

impl ClientFunction for Client {
    type Input = TriggerMsg;

    type Output = ();

    fn execute(&self, document: &mut Cow<Box<Doc>>, input: Self::Input) -> Self::Output {
        match input {
            TriggerMsg::MapSinglePut { key } => self.map_single_putter.execute(document, key),
            TriggerMsg::MapSingleDelete { key } => self.map_single_deleter.execute(document, key),
            TriggerMsg::ListPut { index } => self.list_start_putter.execute(document, index),
            TriggerMsg::ListDelete { index } => self.list_deleter.execute(document, index),
            TriggerMsg::ListInsert { index } => self.list_inserter.execute(document, index),
        }
    }
}
