use crate::app::AppState;

use super::Application;

/// A client strategy that just deletes a single key in a map.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapSingleDeleter;

impl Application for MapSingleDeleter {
    type Input = String;

    type Output = ();

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Self::State>,
        input: Self::Input,
    ) -> Option<()> {
        document.to_mut().delete(&input);
        None
    }
}

/// A client strategy that just deletes the first element in a list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListDeleter;

impl Application for ListDeleter {
    type Input = usize;

    type Output = ();

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Self::State>,
        input: Self::Input,
    ) -> Option<()> {
        document.to_mut().delete_list(input);
        None
    }
}

/// A client strategy that just deletes the first element in a text object.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TextDeleter;

impl Application for TextDeleter {
    type Input = usize;

    type Output = ();

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Self::State>,
        input: Self::Input,
    ) -> Option<()> {
        document.to_mut().delete_text(input);
        None
    }
}
