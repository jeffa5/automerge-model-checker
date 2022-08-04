use stateright::actor::{Actor, Id};

use crate::client::Client;
use amc_core::ClientMsg;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Trigger {
    pub func: TriggerState,
    pub server: Id,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TriggerState {
    Creater,
    Toggler(u32),
    Deleter(u32),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TriggerMsg {
    CreateTodo(String),
    ToggleActive(u32),
    DeleteTodo(u32),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TriggerResponse {
    /// The id of the created task.
    CreateTodo(u32),
    ToggleActive(bool),
    DeleteTodo(bool),
}

impl amc_core::Trigger<Client> for Trigger {}

impl Actor for Trigger {
    type Msg = ClientMsg<Client>;

    type State = ();

    fn on_start(
        &self,
        _id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        match self.func {
            TriggerState::Creater => {
                o.send(
                    self.server,
                    ClientMsg::Request(TriggerMsg::CreateTodo("todo 1".to_owned())),
                );
            }
            TriggerState::Toggler(id) => {
                o.send(
                    self.server,
                    ClientMsg::Request(TriggerMsg::ToggleActive(id)),
                );
            }
            TriggerState::Deleter(id) => {
                o.send(self.server, ClientMsg::Request(TriggerMsg::DeleteTodo(id)));
            }
        }
    }

    fn on_msg(
        &self,
        _id: Id,
        _state: &mut std::borrow::Cow<Self::State>,
        _src: Id,
        msg: Self::Msg,
        _o: &mut stateright::actor::Out<Self>,
    ) {
        match msg {
            ClientMsg::Request(_) => unreachable!("clients don't handle requests"),
            ClientMsg::Response(r) => match r {
                TriggerResponse::CreateTodo(_) => {}
                TriggerResponse::ToggleActive(_) => {}
                TriggerResponse::DeleteTodo(_was_present) => {}
            },
        }
    }
}
