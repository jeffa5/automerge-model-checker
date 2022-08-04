use crate::app::App;

use super::Application;

/// A client strategy that just deletes a single key in a map.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapSingleDeleter;

impl Application for MapSingleDeleter {
    type Input = String;

    type Output = ();

    type State = App;

    fn init(&self, id: stateright::actor::Id) -> Self::State {
        App::new(id)
    }

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Self::State>,
        input: Self::Input,
    ) -> Self::Output {
        document.to_mut().delete(&input);
    }
}

/// A client strategy that just deletes the first element in a list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListDeleter;

impl Application for ListDeleter {
    type Input = usize;

    type Output = ();

    type State = App;

    fn init(&self, id: stateright::actor::Id) -> Self::State {
        App::new(id)
    }

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Self::State>,
        input: Self::Input,
    ) -> Self::Output {
        document.to_mut().delete_list(input);
    }
}
