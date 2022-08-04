use std::{fmt::Debug, hash::Hash};

use stateright::actor::Actor;

use crate::{Application, ClientMsg};

/// A triggerer of the client functionality. Similar to function invocation.
pub trait Trigger<C: Application>:
    Actor<Msg = ClientMsg<C>> + Clone + Debug + PartialEq + Hash
{
}
