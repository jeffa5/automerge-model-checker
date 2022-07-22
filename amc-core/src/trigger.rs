use std::{fmt::Debug, hash::Hash};

use stateright::actor::Actor;

use crate::client::{ClientFunction, ClientMsg};

/// A triggerer of the client functionality. Similar to function invocation.
pub trait Trigger<C: ClientFunction>:
    Actor<Msg = ClientMsg<C>> + Clone + Debug + PartialEq + Hash
{
}
