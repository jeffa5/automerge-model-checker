use std::{borrow::Cow, fmt::Debug, hash::Hash};

use stateright::actor::Id;

use crate::Application;

/// A ClientFunction is coupled with a server and implements an atomic action against the document.
/// This ensures that no sync messages are applied within the body of execution.
pub trait ClientFunction: Clone + Hash + Eq + Debug {
    type Input: Clone + Hash + Eq + Debug;
    type Output: Clone + Hash + Eq + Debug;
    type State: Application;

    fn init(&self, id: Id) -> Self::State;

    fn execute(&self, state: &mut Cow<Self::State>, input: Self::Input) -> Self::Output;
}

/// A ClientMsg contains the request or response to or from a client's execution.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ClientMsg<C: ClientFunction> {
    /// Message originating from clients to servers.
    Request(C::Input),
    /// Message originating from server to client.
    Response(C::Output),
}
