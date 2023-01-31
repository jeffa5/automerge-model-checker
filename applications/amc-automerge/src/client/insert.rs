use std::borrow::Cow;

use crate::app::AppState;

use super::Application;

/// A client strategy that just inserts at the start of the list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListInserter;

impl Application for ListInserter {
    type Input = (usize, String);

    type Output = ();

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(&self, document: &mut Cow<Self::State>, (index, value): Self::Input) -> Self::Output {
        document.to_mut().insert(index, value);
    }
}
