use std::{borrow::Cow, fmt::Debug, hash::Hash};

use crate::application::Application;

/// A ClientFunction is coupled with a server and implements an atomic action against the document.
/// This ensures that no sync messages are applied within the body of execution.
pub trait ClientFunction: Clone + Hash + Eq + Debug {
    type Input: Clone + Hash + Eq + Debug;
    type Output: Clone + Hash + Eq + Debug;
    type Application: Application;

    fn execute(&self, document: &mut Cow<Self::Application>, input: Self::Input) -> Self::Output;
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ClientMsg<C: ClientFunction> {
    /// Message originating from clients to servers.
    Request(C::Input),
    /// Message originating from server to client.
    Response(C::Output),
}
