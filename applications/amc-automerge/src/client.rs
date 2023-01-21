use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use crate::app::AppState;
use crate::trigger::TriggerMsg;

mod delete;
mod insert;
mod put;

use amc::application::Application;
pub use delete::ListDeleter;
pub use delete::MapSingleDeleter;
pub use insert::ListInserter;
pub use put::ListPutter;
pub use put::MapSinglePutter;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct App {
    pub map_single_putter: put::MapSinglePutter,
    pub list_start_putter: put::ListPutter,
    pub map_single_deleter: delete::MapSingleDeleter,
    pub list_deleter: delete::ListDeleter,
    pub list_inserter: insert::ListInserter,
}

impl Application for App {
    type Input = TriggerMsg;

    type Output = ();

    type State = AppState;

    fn init(&self, id: stateright::actor::Id) -> Self::State {
        AppState::new(id)
    }

    fn execute(&self, document: &mut Cow<Self::State>, input: Self::Input) -> Self::Output {
        match input {
            TriggerMsg::MapSinglePut { key } => self.map_single_putter.execute(document, key),
            TriggerMsg::MapSingleDelete { key } => self.map_single_deleter.execute(document, key),
            TriggerMsg::ListPut { index } => self.list_start_putter.execute(document, index),
            TriggerMsg::ListDelete { index } => self.list_deleter.execute(document, index),
            TriggerMsg::ListInsert { index } => self.list_inserter.execute(document, index),
        }
    }
}
