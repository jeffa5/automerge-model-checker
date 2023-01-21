use amc::driver::Drive;
use stateright::actor::Id;

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

impl Drive<App> for Driver {
    type State = ();

    fn init(
        &self,
        _id: Id,
    ) -> (
        <Self as Drive<App>>::State,
        Vec<<App as amc::prelude::Application>::Input>,
    ) {
        match self.func {
            DriverState::Creater => ((), vec![AppInput::CreateTodo("todo 1".to_owned())]),
            DriverState::Updater => ((), vec![AppInput::ListTodos]),
            DriverState::Toggler => ((), vec![AppInput::ListTodos]),
            DriverState::Deleter => ((), vec![AppInput::ListTodos]),
        }
    }

    fn handle_output(
        &self,
        _state: &mut std::borrow::Cow<Self::State>,
        output: <App as amc::prelude::Application>::Output,
    ) -> Vec<<App as amc::prelude::Application>::Input> {
        match (&self.func, output) {
            (DriverState::Updater, AppOutput::ListTodos(ids)) => ids
                .iter()
                .map(|id| AppInput::Update(*id, "updated todo".to_owned()))
                .collect(),
            (DriverState::Toggler, AppOutput::ListTodos(ids)) => {
                ids.iter().map(|id| AppInput::ToggleActive(*id)).collect()
            }
            (DriverState::Deleter, AppOutput::ListTodos(ids)) => {
                ids.iter().map(|id| AppInput::DeleteTodo(*id)).collect()
            }
            _ => {vec![]}
        }
    }
}
