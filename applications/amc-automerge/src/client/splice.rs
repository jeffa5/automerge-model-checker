use std::borrow::Cow;

use crate::{app::AppState, scalar::ScalarValue};

use super::Application;

/// A client strategy that just splices into the list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListSplicer;

impl Application for ListSplicer {
    type Input = (usize, usize, Vec<ScalarValue>);

    type Output = ();

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(
        &self,
        document: &mut Cow<Self::State>,
        (index, delete, values): Self::Input,
    ) -> Option<()> {
        document.to_mut().splice_list(index, delete, values);
        None
    }
}

/// A client strategy that just splices into the text.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TextSplicer;

impl Application for TextSplicer {
    type Input = (usize, usize, String);

    type Output = ();

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(
        &self,
        document: &mut Cow<Self::State>,
        (index, delete, text): Self::Input,
    ) -> Option<()> {
        document.to_mut().splice_text(index, delete, text);
        None
    }
}
