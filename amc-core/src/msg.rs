use crate::{
    client::{ClientFunction, ClientMsg},
    server::ServerMsg,
};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum GlobalMsg<C: ClientFunction> {
    /// A message specific to the register system's internal protocol.
    Internal(ServerMsg),

    /// A message between clients and servers.
    External(ClientMsg<C>),
}
