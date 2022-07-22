use std::borrow::Cow;

use crate::{client::ClientFunction, msg::GlobalMsg, server::Server, trigger::Trigger};
use stateright::actor::{Actor, Command, Id, Out};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MyRegisterActor<T, C> {
    Trigger(T),
    Server(Server<C>),
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum MyRegisterActorState<T: Trigger<C>, C: ClientFunction> {
    Trigger(<T as Actor>::State),
    Server(<Server<C> as Actor>::State),
}

impl<T: Trigger<C>, C: ClientFunction> Actor for MyRegisterActor<T, C> {
    type Msg = GlobalMsg<C>;

    type State = MyRegisterActorState<T, C>;

    fn on_start(&self, id: Id, o: &mut Out<Self>) -> Self::State {
        match self {
            MyRegisterActor::Trigger(trigger_actor) => {
                let mut trigger_out = Out::new();
                let state =
                    MyRegisterActorState::Trigger(trigger_actor.on_start(id, &mut trigger_out));
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
            MyRegisterActor::Server(server_actor) => {
                let mut server_out = Out::new();
                let state =
                    MyRegisterActorState::Server(server_actor.on_start(id, &mut server_out));
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
        use MyRegisterActor as A;
        use MyRegisterActorState as S;

        match (self, &**state, msg) {
            (A::Trigger(trigger_actor), S::Trigger(client_state), GlobalMsg::External(tmsg)) => {
                let mut client_state = Cow::Borrowed(client_state);
                let mut client_out = Out::new();
                trigger_actor.on_msg(id, &mut client_state, src, tmsg, &mut client_out);
                if let Cow::Owned(client_state) = client_state {
                    *state = Cow::Owned(MyRegisterActorState::Trigger(client_state))
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
                    *state = Cow::Owned(MyRegisterActorState::Server(server_state))
                }
                o.append(&mut server_out);
            }
            (A::Trigger(_), S::Trigger(_), GlobalMsg::Internal(_)) => {}
            (A::Server(_), S::Trigger(_), _) => {}
            (A::Trigger(_), S::Server(_), _) => {}
        }
    }

    fn on_timeout(&self, id: Id, state: &mut Cow<Self::State>, o: &mut Out<Self>) {
        use MyRegisterActor as A;
        use MyRegisterActorState as S;
        match (self, &**state) {
            (A::Trigger(_), S::Trigger(_)) => {}
            (A::Trigger(_), S::Server(_)) => {}
            (A::Server(server_actor), S::Server(server_state)) => {
                let mut server_state = Cow::Borrowed(server_state);
                let mut server_out = Out::new();
                server_actor.on_timeout(id, &mut server_state, &mut server_out);
                if let Cow::Owned(server_state) = server_state {
                    *state = Cow::Owned(MyRegisterActorState::Server(server_state))
                }
                o.append(&mut server_out);
            }
            (A::Server(_), S::Trigger(_)) => {}
        }
    }
}
