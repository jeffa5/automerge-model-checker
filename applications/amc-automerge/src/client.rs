use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use crate::app::AppState;
use crate::driver::DriverMsg;

mod delete;
mod increment;
mod insert;
mod put;
mod splice;

use amc::application::Application;
pub use delete::ListDeleter;
pub use delete::MapSingleDeleter;
pub use delete::TextDeleter;
pub use increment::ListIncrementer;
pub use increment::MapIncrementer;
pub use insert::ListInserter;
pub use insert::TextInserter;
pub use put::ListPutter;
pub use put::MapSinglePutter;
pub use put::TextPutter;
pub use splice::ListSplicer;
pub use splice::TextSplicer;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct App {
    pub map_single_putter: put::MapSinglePutter,
    pub map_single_deleter: delete::MapSingleDeleter,
    pub map_incrementer: increment::MapIncrementer,
    pub list_putter: put::ListPutter,
    pub list_deleter: delete::ListDeleter,
    pub list_inserter: insert::ListInserter,
    pub list_splicer: splice::ListSplicer,
    pub list_incrementer: increment::ListIncrementer,
    pub text_putter: put::TextPutter,
    pub text_deleter: delete::TextDeleter,
    pub text_inserter: insert::TextInserter,
    pub text_splicer: splice::TextSplicer,
}

impl Application for App {
    type Input = DriverMsg;

    type Output = ();

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(&self, document: &mut Cow<Self::State>, input: Self::Input) -> Option<()> {
        match input {
            DriverMsg::MapSinglePut { key, value } => {
                self.map_single_putter.execute(document, (key, value))
            }
            DriverMsg::MapSingleDelete { key } => self.map_single_deleter.execute(document, key),
            DriverMsg::MapIncrement { key, by } => {
                self.map_incrementer.execute(document, (key, by))
            }
            DriverMsg::ListPut { index, value } => {
                self.list_putter.execute(document, (index, value))
            }
            DriverMsg::ListInsert { index, value } => {
                self.list_inserter.execute(document, (index, value))
            }
            DriverMsg::ListDelete { index } => self.list_deleter.execute(document, index),
            DriverMsg::ListSplice {
                index,
                delete,
                values,
            } => self.list_splicer.execute(document, (index, delete, values)),
            DriverMsg::ListIncrement { index, by } => {
                self.list_incrementer.execute(document, (index, by))
            }
            DriverMsg::TextPut { index, value } => {
                self.text_putter.execute(document, (index, value))
            }
            DriverMsg::TextInsert { index, value } => {
                self.text_inserter.execute(document, (index, value))
            }
            DriverMsg::TextDelete { index } => self.text_deleter.execute(document, index),
            DriverMsg::TextSplice {
                index,
                delete,
                text,
            } => self.text_splicer.execute(document, (index, delete, text)),
        }
    }
}
