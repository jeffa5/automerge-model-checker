use std::borrow::Cow;

use amc::application::Application;
use stateright::actor::Id;

use crate::{app::AppState, trigger::AppOutput};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct App {
    pub random_ids: bool,
}

impl Application for App {
    type Input = crate::trigger::AppInput;

    type Output = crate::trigger::AppOutput;

    type State = AppState;

    fn init(&self, id: Id) -> Self::State {
        AppState::new(id, self.random_ids)
    }

    fn execute(&self, document: &mut Cow<Self::State>, input: Self::Input) -> Self::Output {
        match input {
            crate::trigger::AppInput::CreateTodo(text) => {
                let id = document.to_mut().create_todo(text);
                AppOutput::CreateTodo(id)
            }
            crate::trigger::AppInput::Update(id, text) => {
                let success = document.to_mut().update_text(id, text);
                AppOutput::Update(success)
            }
            crate::trigger::AppInput::ToggleActive(id) => {
                let b = document.to_mut().toggle_active(id);
                AppOutput::ToggleActive(b)
            }
            crate::trigger::AppInput::DeleteTodo(id) => {
                let was_present = document.to_mut().delete_todo(id);
                AppOutput::DeleteTodo(was_present)
            }
            crate::trigger::AppInput::ListTodos => {
                let ids = document.list_todos();
                AppOutput::ListTodos(ids)
            }
        }
    }
}
