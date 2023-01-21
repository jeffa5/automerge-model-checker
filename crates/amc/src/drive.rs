use std::{borrow::Cow, fmt::Debug, hash::Hash};

use stateright::actor::Id;

use crate::client::Application;

/// A way of driving the application's behaviour. Similar to function invocation.
pub trait Drive<A: Application>: Clone + Debug + PartialEq + Hash + Send + Sync {
    /// State of the driver.
    type State: Clone + Debug + Hash + PartialEq + Send + Sync;

    /// Initialise a driver, returning any messages to send straight away.
    fn init(&self, id: Id) -> (<Self as Drive<A>>::State, Vec<A::Input>);

    /// Handle an output from the application.
    fn handle_output(&self, state: &mut Cow<Self::State>, output: A::Output) -> Vec<A::Input>;
}
