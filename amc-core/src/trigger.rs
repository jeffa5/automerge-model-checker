use std::{fmt::Debug, hash::Hash};

use stateright::actor::Actor;

use crate::client::{ClientFunction, ClientMsg};

pub trait Trigger<C: ClientFunction>:
    Actor<Msg = ClientMsg<C>> + Clone + Debug + PartialEq + Hash
{
}
