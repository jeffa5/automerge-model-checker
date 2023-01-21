use amc::triggers::{ClientMsg, Trigger};
use stateright::actor::{Actor, Id};

use crate::apphandle::AppHandle;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Triggerer {
    pub func: TriggerState,
    pub server: Id,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TriggerState {
    Creater,
    Updater,
    Toggler,
    Deleter,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TriggerMsg {
    CreateTodo(String),
    Update(u32, String),
    ToggleActive(u32),
    DeleteTodo(u32),
    ListTodos,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TriggerResponse {
    /// The id of the created task.
    CreateTodo(u32),
    Update(bool),
    ToggleActive(bool),
    DeleteTodo(bool),
    ListTodos(Vec<u32>),
}

impl Trigger<AppHandle> for Triggerer {}

impl Actor for Triggerer {
    type Msg = ClientMsg<AppHandle>;

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
            TriggerState::Updater => {
                o.send(self.server, ClientMsg::Request(TriggerMsg::ListTodos));
            }
            TriggerState::Toggler => {
                o.send(self.server, ClientMsg::Request(TriggerMsg::ListTodos));
            }
            TriggerState::Deleter => {
                o.send(self.server, ClientMsg::Request(TriggerMsg::ListTodos));
            }
        }
    }

    fn on_msg(
        &self,
        _id: Id,
        _state: &mut std::borrow::Cow<Self::State>,
        _src: Id,
        msg: Self::Msg,
        o: &mut stateright::actor::Out<Self>,
    ) {
        match msg {
            ClientMsg::Request(_) => unreachable!("clients don't handle requests"),
            ClientMsg::Response(r) => match (&self.func, r) {
                (TriggerState::Updater, TriggerResponse::ListTodos(ids)) => {
                    for id in ids {
                        o.send(
                            self.server,
                            ClientMsg::Request(TriggerMsg::Update(id, "updated todo".to_owned())),
                        )
                    }
                }
                (TriggerState::Toggler, TriggerResponse::ListTodos(ids)) => {
                    for id in ids {
                        o.send(
                            self.server,
                            ClientMsg::Request(TriggerMsg::ToggleActive(id)),
                        );
                    }
                }
                (TriggerState::Deleter, TriggerResponse::ListTodos(ids)) => {
                    for id in ids {
                        o.send(self.server, ClientMsg::Request(TriggerMsg::DeleteTodo(id)));
                    }
                }
                _ => {}
            },
        }
    }
}
