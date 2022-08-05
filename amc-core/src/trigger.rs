use std::{fmt::Debug, hash::Hash};

use stateright::actor::Actor;

use crate::{Application, ClientMsg};

/// A triggerer of the application's behaviour. Similar to function invocation.
pub trait Trigger<A: Application>:
    Actor<Msg = ClientMsg<A>> + Clone + Debug + PartialEq + Hash
{
}
