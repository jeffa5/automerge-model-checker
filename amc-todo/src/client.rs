use std::borrow::Cow;

use stateright::actor::Id;

use crate::{app::App, trigger::TriggerResponse};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Client {
    pub random_ids: bool,
}

impl amc_core::Application for Client {
    type Input = crate::trigger::TriggerMsg;

    type Output = crate::trigger::TriggerResponse;

    type State = App;

    fn init(&self, id: Id) -> Self::State {
        App::new(id, self.random_ids)
    }

    fn execute(&self, document: &mut Cow<Self::State>, input: Self::Input) -> Self::Output {
        match input {
            crate::trigger::TriggerMsg::CreateTodo(text) => {
                let id = document.to_mut().create_todo(text);
                TriggerResponse::CreateTodo(id)
            }
            crate::trigger::TriggerMsg::ToggleActive(id) => {
                let b = document.to_mut().toggle_active(id);
                TriggerResponse::ToggleActive(b)
            }
            crate::trigger::TriggerMsg::DeleteTodo(id) => {
                let was_present = document.to_mut().delete_todo(id);
                TriggerResponse::DeleteTodo(was_present)
            }
        }
    }
}
