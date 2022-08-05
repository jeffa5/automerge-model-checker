use std::borrow::Cow;

use crate::{Application, ClientMsg, ServerMsg};
use crate::{Server, Trigger};
use stateright::actor::{Actor, Command, Id, Out};

/// The root message type.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum GlobalMsg<A: Application> {
    /// A message specific to the register system's internal protocol.
    Internal(ServerMsg),

    /// A message between clients and servers.
    External(ClientMsg<A>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlobalActor<T, A> {
    Trigger(T),
    Server(Server<A>),
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum GlobalActorState<T: Trigger<A>, A: Application> {
    Trigger(<T as Actor>::State),
    Server(<Server<A> as Actor>::State),
}

impl<T: Trigger<A>, A: Application> Actor for GlobalActor<T, A> {
    type Msg = GlobalMsg<A>;

    type State = GlobalActorState<T, A>;

    fn on_start(&self, id: Id, o: &mut Out<Self>) -> Self::State {
        match self {
            GlobalActor::Trigger(trigger_actor) => {
                let mut trigger_out = Out::new();
                let state = GlobalActorState::Trigger(trigger_actor.on_start(id, &mut trigger_out));
                let mut new_out: Out<Self> = trigger_out
                    .into_iter()
                    .map(|o| match o {
                        Command::CancelTimer => Command::CancelTimer,
                        Command::SetTimer(t) => Command::SetTimer(t),
                        Command::Send(id, msg) => Command::Send(id, GlobalMsg::External(msg)),
                    })
                    .collect();
                o.append(&mut new_out);
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
            (A::Trigger(trigger_actor), S::Trigger(client_state), GlobalMsg::External(tmsg)) => {
                let mut client_state = Cow::Borrowed(client_state);
                let mut client_out = Out::new();
                trigger_actor.on_msg(id, &mut client_state, src, tmsg, &mut client_out);
                if let Cow::Owned(client_state) = client_state {
                    *state = Cow::Owned(GlobalActorState::Trigger(client_state))
                }

                let mut new_out: Out<Self> = client_out
                    .into_iter()
                    .map(|o| match o {
                        Command::CancelTimer => Command::CancelTimer,
                        Command::SetTimer(t) => Command::SetTimer(t),
                        Command::Send(id, msg) => Command::Send(id, GlobalMsg::External(msg)),
                    })
                    .collect();

                o.append(&mut new_out);
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
            (A::Trigger(_), S::Trigger(_), GlobalMsg::Internal(_)) => {}
            (A::Server(_), S::Trigger(_), _) => {}
            (A::Trigger(_), S::Server(_), _) => {}
        }
    }

    fn on_timeout(&self, id: Id, state: &mut Cow<Self::State>, o: &mut Out<Self>) {
        use GlobalActor as A;
        use GlobalActorState as S;
        match (self, &**state) {
            (A::Trigger(_), S::Trigger(_)) => {}
            (A::Trigger(_), S::Server(_)) => {}
            (A::Server(server_actor), S::Server(server_state)) => {
                let mut server_state = Cow::Borrowed(server_state);
                let mut server_out = Out::new();
                server_actor.on_timeout(id, &mut server_state, &mut server_out);
                if let Cow::Owned(server_state) = server_state {
                    *state = Cow::Owned(GlobalActorState::Server(server_state))
                }
                o.append(&mut server_out);
            }
            (A::Server(_), S::Trigger(_)) => {}
        }
    }
}
