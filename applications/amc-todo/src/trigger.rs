use amc::driver::{ApplicationMsg, Drive};
use stateright::actor::{Actor, Id};

use crate::apphandle::App;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Driver {
    pub func: DriverState,
    pub server: Id,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DriverState {
    Creater,
    Updater,
    Toggler,
    Deleter,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum AppInput {
    CreateTodo(String),
    Update(u32, String),
    ToggleActive(u32),
    DeleteTodo(u32),
    ListTodos,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum AppOutput {
    /// The id of the created task.
    CreateTodo(u32),
    Update(bool),
    ToggleActive(bool),
    DeleteTodo(bool),
    ListTodos(Vec<u32>),
}

impl Drive<App> for Driver {}

impl Actor for Driver {
    type Msg = ApplicationMsg<App>;

    type State = ();

    fn on_start(
        &self,
        _id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        match self.func {
            DriverState::Creater => {
                o.send(
                    self.server,
                    ApplicationMsg::Input(AppInput::CreateTodo("todo 1".to_owned())),
                );
            }
            DriverState::Updater => {
                o.send(self.server, ApplicationMsg::Input(AppInput::ListTodos));
            }
            DriverState::Toggler => {
                o.send(self.server, ApplicationMsg::Input(AppInput::ListTodos));
            }
            DriverState::Deleter => {
                o.send(self.server, ApplicationMsg::Input(AppInput::ListTodos));
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
            ApplicationMsg::Input(_) => unreachable!("clients don't handle requests"),
            ApplicationMsg::Output(r) => match (&self.func, r) {
                (DriverState::Updater, AppOutput::ListTodos(ids)) => {
                    for id in ids {
                        o.send(
                            self.server,
                            ApplicationMsg::Input(AppInput::Update(id, "updated todo".to_owned())),
                        )
                    }
                }
                (DriverState::Toggler, AppOutput::ListTodos(ids)) => {
                    for id in ids {
                        o.send(
                            self.server,
                            ApplicationMsg::Input(AppInput::ToggleActive(id)),
                        );
                    }
                }
                (DriverState::Deleter, AppOutput::ListTodos(ids)) => {
                    for id in ids {
                        o.send(self.server, ApplicationMsg::Input(AppInput::DeleteTodo(id)));
                    }
                }
                _ => {}
            },
        }
    }
}
