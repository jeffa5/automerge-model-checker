use std::borrow::Cow;

use stateright::actor::{Actor, Id, Out};

use crate::{
    client::{Application, ApplicationMsg, Client},
    drive::Drive,
    server::{Server, ServerMsg},
};

/// The root message type.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum GlobalMsg<A: Application> {
    /// A message specific to the register system's internal protocol.
    ServerToServer(ServerMsg),

    /// A message between clients and servers.
    ClientToServer(ApplicationMsg<A>),
}

impl<A:Application> GlobalMsg<A> {
    /// Obtain the input to the application, if this was one.
    pub fn input(&self) -> Option<&A::Input> {
        match self {
            Self::ClientToServer(ApplicationMsg::Input(i)) => Some(i),
            Self::ServerToServer(_) | Self::ClientToServer(_) => None,
        }
    }

    /// Obtain the output from the application, if this was one.
    pub fn output(&self) -> Option<&A::Output> {
        match self {
            Self::ClientToServer(ApplicationMsg::Output(o)) => Some(o),
            Self::ServerToServer(_) | Self::ClientToServer(_) => None,
        }
    }
}

/// The root actor type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlobalActor<A, D> {
    /// Actor to trigger behaviour in the application.
    Client(Client<A, D>),
    /// Server that hosts the application.
    Server(Server<A>),
}

/// The root actor state.
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum GlobalActorState<D: Drive<A>, A: Application> {
    /// State for the driver.
    Client(<Client<A, D> as Actor>::State),
    /// State for the application.
    Server(<Server<A> as Actor>::State),
}

impl<A: Application, D: Drive<A>> Actor for GlobalActor<A, D> {
    type Msg = GlobalMsg<A>;

    type State = GlobalActorState<D, A>;

    fn on_start(&self, id: Id, o: &mut Out<Self>) -> Self::State {
        match self {
            GlobalActor::Client(client_actor) => {
                let mut client_out = Out::new();
                let state = GlobalActorState::Client(client_actor.on_start(id, &mut client_out));
                o.append(&mut client_out);
                state
            }
            GlobalActor::Server(server_actor) => {
                let mut server_out = Out::new();
                let state = GlobalActorState::Server(server_actor.on_start(id, &mut server_out));
                o.append(&mut server_out);
                state
            }
        }
    }

    fn on_msg(
        &self,
        id: Id,
        state: &mut Cow<Self::State>,
        src: Id,
        msg: Self::Msg,
        o: &mut Out<Self>,
    ) {
        use GlobalActor as A;
        use GlobalActorState as S;

        match (self, &**state, msg) {
            (A::Client(client_actor), S::Client(client_state), msg) => {
                let mut client_state = Cow::Borrowed(client_state);
                let mut client_out = Out::new();
                client_actor.on_msg(id, &mut client_state, src, msg, &mut client_out);
                if let Cow::Owned(client_state) = client_state {
                    *state = Cow::Owned(GlobalActorState::Client(client_state))
                }

                o.append(&mut client_out);
            }
            (A::Server(server_actor), S::Server(server_state), msg) => {
                let mut server_state = Cow::Borrowed(server_state);
                let mut server_out = Out::new();
                server_actor.on_msg(id, &mut server_state, src, msg, &mut server_out);
                if let Cow::Owned(server_state) = server_state {
                    *state = Cow::Owned(GlobalActorState::Server(server_state))
                }
                o.append(&mut server_out);
            }
            (A::Server(_), S::Client(_), _) => {}
            (A::Client(_), S::Server(_), _) => {}
        }
    }

    fn on_timeout(&self, id: Id, state: &mut Cow<Self::State>, o: &mut Out<Self>) {
        use GlobalActor as A;
        use GlobalActorState as S;
        match (self, &**state) {
            (A::Client(_), S::Client(_)) => {}
            (A::Client(_), S::Server(_)) => {}
            (A::Server(server_actor), S::Server(server_state)) => {
                let mut server_state = Cow::Borrowed(server_state);
                let mut server_out = Out::new();
                server_actor.on_timeout(id, &mut server_state, &mut server_out);
                if let Cow::Owned(server_state) = server_state {
                    *state = Cow::Owned(GlobalActorState::Server(server_state))
                }
                o.append(&mut server_out);
            }
            (A::Server(_), S::Client(_)) => {}
        }
    }
}
