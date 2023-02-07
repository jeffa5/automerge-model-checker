use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use crate::app::AppState;
use crate::driver::DriverMsg;

mod delete;
mod insert;
mod put;

use amc::application::Application;
pub use delete::ListDeleter;
pub use delete::MapSingleDeleter;
pub use delete::TextDeleter;
pub use insert::ListInserter;
pub use insert::TextInserter;
pub use put::ListPutter;
pub use put::MapSinglePutter;
pub use put::TextPutter;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct App {
    pub map_single_putter: put::MapSinglePutter,
    pub map_single_deleter: delete::MapSingleDeleter,
    pub list_start_putter: put::ListPutter,
    pub list_deleter: delete::ListDeleter,
    pub list_inserter: insert::ListInserter,
    pub text_start_putter: put::TextPutter,
    pub text_deleter: delete::TextDeleter,
    pub text_inserter: insert::TextInserter,
}

impl Application for App {
    type Input = DriverMsg;

    type Output = ();

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(&self, document: &mut Cow<Self::State>, input: Self::Input) -> Self::Output {
        match input {
            DriverMsg::MapSinglePut { key, value } => {
                self.map_single_putter.execute(document, (key, value))
            }
            DriverMsg::MapSingleDelete { key } => self.map_single_deleter.execute(document, key),
            DriverMsg::ListPut { index, value } => {
                self.list_start_putter.execute(document, (index, value))
            }
            DriverMsg::ListInsert { index, value } => {
                self.list_inserter.execute(document, (index, value))
            }
            DriverMsg::ListDelete { index } => self.list_deleter.execute(document, index),
            DriverMsg::TextPut { index, value } => {
                self.text_start_putter.execute(document, (index, value))
            }
            DriverMsg::TextInsert { index, value } => {
                self.text_inserter.execute(document, (index, value))
            }
            DriverMsg::TextDelete { index } => self.text_deleter.execute(document, index),
        }
    }
}
