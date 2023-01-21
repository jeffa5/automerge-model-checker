use std::{fmt::Debug, hash::Hash};

use stateright::actor::Actor;

use crate::client::{Application, ApplicationMsg};

/// A way of driving the application's behaviour. Similar to function invocation.
pub trait Drive<A: Application>:
    Actor<Msg = ApplicationMsg<A>> + Clone + Debug + PartialEq + Hash + Send + Sync
{
}
