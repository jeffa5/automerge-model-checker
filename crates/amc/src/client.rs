use std::{borrow::Cow, fmt::Debug, hash::Hash};

use stateright::actor::Id;

use crate::document::Document;

/// An Application is coupled with a server and implements an atomic action against the document.
/// This ensures that no sync messages are applied within the body of execution.
pub trait Application: Clone + Hash + Eq + Debug + Send + Sync {
    /// Inputs that the application accepts to trigger behaviour.
    type Input: Clone + Hash + Eq + Debug + Send + Sync;

    /// Outputs that the behaviour returns.
    type Output: Clone + Hash + Eq + Debug + Send + Sync;

    /// State that the application runs with, including an Automerge document.
    type State: DerefDocument + Send + Sync;

    /// Initialise an application, performing any setup logic.
    fn init(&self, id: Id) -> Self::State;

    /// Execute an application, triggering some behaviour with a given input, expecting a
    /// corresponding output.
    fn execute(&self, state: &mut Cow<Self::State>, input: Self::Input) -> Self::Output;
}

/// Get access to a document.
pub trait DerefDocument: Clone + Hash + Eq + Debug {
    /// Get the document.
    fn document(&self) -> &Document;

    /// Get a mutable reference to the document.
    fn document_mut(&mut self) -> &mut Document;
}

/// A ClientMsg contains the request or response to or from a client's execution.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ClientMsg<C: Application> {
    /// Message originating from clients to servers.
    Request(C::Input),
    /// Message originating from server to client.
    Response(C::Output),
}
