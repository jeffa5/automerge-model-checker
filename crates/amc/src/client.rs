use std::{borrow::Cow, fmt::Debug, hash::Hash, marker::PhantomData};
use tracing::debug;

use stateright::actor::{Actor, Id};

use crate::{
    document::Document,
    driver::Drive,
    global::{GlobalMsg, GlobalTimer},
};

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
    fn init(&self, id: usize) -> Self::State;

    /// Execute an application, triggering some behaviour with a given input, expecting a
    /// corresponding output.
    fn execute(&self, state: &mut Cow<Self::State>, input: Self::Input) -> Option<Self::Output>;
}

/// Get access to a document.
pub trait DerefDocument: Clone + Hash + Eq + Debug {
    /// Get the document.
    fn document(&self) -> &Document;

    /// Get a mutable reference to the document.
    fn document_mut(&mut self) -> &mut Document;
}

/// Contains the input to or output from the application.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ApplicationMsg<A: Application> {
    /// Message to feed to the application.
    Input(A::Input),
    /// Message resulting from the application.
    Output(A::Output),
}

/// A wrapper for driver logic.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Client<A, D> {
    /// The id of the server that messages will be sent to.
    pub server: Id,
    /// The driver.
    pub driver: D,
    /// The app we're working with.
    pub _app: PhantomData<A>,
}

impl<A: Application, D: Drive<A>> Actor for Client<A, D> {
    type Msg = GlobalMsg<A>;

    type State = D::State;

    type Timer = GlobalTimer;

    fn on_start(&self, id: Id, o: &mut stateright::actor::Out<Self>) -> Self::State {
        let (state, messages) = self.driver.init(usize::from(id));
        for message in messages {
            o.send(
                self.server,
                GlobalMsg::ClientToServer(ApplicationMsg::Input(message)),
            );
        }
        state
    }

    fn on_msg(
        &self,
        _id: Id,
        state: &mut Cow<Self::State>,
        _src: Id,
        msg: Self::Msg,
        o: &mut stateright::actor::Out<Self>,
    ) {
        match msg {
            GlobalMsg::ServerToServer(_) | GlobalMsg::ClientToServer(ApplicationMsg::Input(_)) => {
                unreachable!()
            }
            GlobalMsg::ClientToServer(ApplicationMsg::Output(output)) => {
                let messages = self.driver.handle_output(state, output);
                if !messages.is_empty() {
                    debug!(
                        new_input_count = messages.len(),
                        "new inputs generated in response to output"
                    );
                }
                for message in messages {
                    o.send(
                        self.server,
                        GlobalMsg::ClientToServer(ApplicationMsg::Input(message)),
                    );
                }
            }
        }
    }
}
