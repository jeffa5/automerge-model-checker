use std::borrow::Cow;

use crate::{app::App, trigger::TriggerResponse};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Client {}

impl amc_core::ClientFunction for Client {
    type Input = crate::trigger::TriggerMsg;

    type Output = crate::trigger::TriggerResponse;

    type Application = App;

    fn execute(&self, document: &mut Cow<Self::Application>, input: Self::Input) -> Self::Output {
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
