use std::borrow::Cow;

use amc::application::Application;
use stateright::actor::Id;

use crate::{app::AppState, driver::AppOutput};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct App {
    pub random_ids: bool,
}

impl Application for App {
    type Input = crate::driver::AppInput;

    type Output = crate::driver::AppOutput;

    type State = AppState;

    fn init(&self, id: Id) -> Self::State {
        AppState::new(id, self.random_ids)
    }

    fn execute(&self, document: &mut Cow<Self::State>, input: Self::Input) -> Self::Output {
        match input {
            crate::driver::AppInput::CreateTodo(text) => {
                let id = document.to_mut().create_todo(text);
                AppOutput::CreateTodo(id)
            }
            crate::driver::AppInput::Update(id, text) => {
                let success = document.to_mut().update_text(id, text);
                AppOutput::Update(success)
            }
            crate::driver::AppInput::ToggleActive(id) => {
                let b = document.to_mut().toggle_active(id);
                AppOutput::ToggleActive(b)
            }
            crate::driver::AppInput::DeleteTodo(id) => {
                let was_present = document.to_mut().delete_todo(id);
                AppOutput::DeleteTodo(was_present)
            }
            crate::driver::AppInput::ListTodos => {
                let ids = document.list_todos();
                AppOutput::ListTodos(ids)
            }
        }
    }
}
