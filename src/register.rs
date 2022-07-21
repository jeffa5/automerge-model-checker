use std::borrow::Cow;

use stateright::actor::{Actor, Id, Out};

use crate::{
    client::{Client, ClientFunction, ClientMsg},
    server::{Server, ServerMsg},
    trigger::Trigger,
};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum GlobalMsg<C: ClientFunction> {
    /// A message specific to the register system's internal protocol.
    Internal(ServerMsg),

    /// A message between clients and servers.
    External(ClientMsg<C>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MyRegisterActor {
    Trigger(Trigger),
    Server(Server<Client>),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum MyRegisterActorState {
    Trigger(<Trigger as Actor>::State),
    Server(<Server<Client> as Actor>::State),
}

impl Actor for MyRegisterActor {
    type Msg = GlobalMsg<Client>;

    type State = MyRegisterActorState;

    fn on_start(&self, id: Id, o: &mut Out<Self>) -> Self::State {
        match self {
            MyRegisterActor::Trigger(trigger_actor) => {
                let mut trigger_out = Out::new();
                let state =
                    MyRegisterActorState::Trigger(trigger_actor.on_start(id, &mut trigger_out));
                o.append(&mut trigger_out);
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

        match (self, &**state) {
            (A::Trigger(client_actor), S::Trigger(client_state)) => {
                let mut client_state = Cow::Borrowed(client_state);
                let mut client_out = Out::new();
                client_actor.on_msg(id, &mut client_state, src, msg, &mut client_out);
                if let Cow::Owned(client_state) = client_state {
                    *state = Cow::Owned(MyRegisterActorState::Trigger(client_state))
                }
                o.append(&mut client_out);
            }
            (A::Server(server_actor), S::Server(server_state)) => {
                let mut server_state = Cow::Borrowed(server_state);
                let mut server_out = Out::new();
                server_actor.on_msg(id, &mut server_state, src, msg, &mut server_out);
                if let Cow::Owned(server_state) = server_state {
                    *state = Cow::Owned(MyRegisterActorState::Server(server_state))
                }
                o.append(&mut server_out);
            }
            (A::Server(_), S::Trigger(_)) => {}
            (A::Trigger(_), S::Server(_)) => {}
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
