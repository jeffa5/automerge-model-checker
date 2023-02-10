use std::borrow::Cow;
use tracing::debug;

use amc::application::Application;

use crate::{app::AppState, driver::AppOutput};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct App {
    pub random_ids: bool,
    pub initial_change: bool,
}

impl Application for App {
    type Input = crate::driver::AppInput;

    type Output = crate::driver::AppOutput;

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id, self.random_ids, self.initial_change)
    }

    fn execute(&self, document: &mut Cow<Self::State>, input: Self::Input) -> Option<AppOutput> {
        let output = match &input {
            crate::driver::AppInput::CreateTodo(text) => {
                let id = document.to_mut().create_todo(text.clone());
                AppOutput::CreateTodo(id)
            }
            crate::driver::AppInput::Update(id, text) => {
                let success = document.to_mut().update_text(*id, text.clone());
                AppOutput::Update(success)
            }
            crate::driver::AppInput::ToggleActive(id) => {
                let b = document.to_mut().toggle_active(*id);
                AppOutput::ToggleActive(b)
            }
            crate::driver::AppInput::DeleteTodo(id) => {
                let was_present = document.to_mut().delete_todo(*id);
                AppOutput::DeleteTodo(was_present)
                // AppOutput::DeleteTodo(false)
            }
            crate::driver::AppInput::ListTodos => {
                let ids = document.list_todos();
                AppOutput::ListTodos(ids)
            }
        };
        debug!(?input, ?output, "Executing new input");
        Some(output)
    }
}
