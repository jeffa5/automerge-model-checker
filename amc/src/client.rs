use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use crate::app::App;
use crate::trigger::TriggerMsg;

mod delete;
mod insert;
mod put;

use amc_core::ClientFunction;
pub use delete::ListDeleter;
pub use delete::MapSingleDeleter;
pub use insert::ListInserter;
pub use put::ListPutter;
pub use put::MapSinglePutter;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Client {
    pub map_single_putter: put::MapSinglePutter,
    pub list_start_putter: put::ListPutter,
    pub map_single_deleter: delete::MapSingleDeleter,
    pub list_deleter: delete::ListDeleter,
    pub list_inserter: insert::ListInserter,
}

impl ClientFunction for Client {
    type Input = TriggerMsg;

    type Output = ();

    type State = App;

    fn init(&self, id: stateright::actor::Id) -> Self::State {
        App::new(id)
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
