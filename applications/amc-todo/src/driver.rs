use amc::driver::Drive;
use smol_str::SmolStr;
use tinyvec::TinyVec;
use tracing::debug;

use crate::apphandle::App;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Driver {
    pub func: DriverState,
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
    CreateTodo(SmolStr),
    Update(u32, SmolStr),
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
    ListTodos(TinyVec<[u32; 4]>),
}

impl Drive<App> for Driver {
    type State = ();

    fn init(
        &self,
        _application_id: usize,
    ) -> (
        <Self as Drive<App>>::State,
        Vec<<App as amc::prelude::Application>::Input>,
    ) {
        match self.func {
            DriverState::Creater => ((), vec![AppInput::CreateTodo(SmolStr::new_inline("a"))]),
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
        let inputs = match (&self.func, &output) {
            (DriverState::Updater, AppOutput::ListTodos(ids)) => ids
                .iter()
                .map(|id| AppInput::Update(*id, SmolStr::new_inline("b")))
                .collect(),
            (DriverState::Toggler, AppOutput::ListTodos(ids)) => {
                ids.iter().map(|id| AppInput::ToggleActive(*id)).collect()
            }
            (DriverState::Deleter, AppOutput::ListTodos(ids)) => {
                ids.iter().map(|id| AppInput::DeleteTodo(*id)).collect()
            }
            _ => {
                vec![]
            }
        };
        if !inputs.is_empty() {
            debug!(?output, generated=?inputs, "Handling output");
        }
        inputs
    }
}
