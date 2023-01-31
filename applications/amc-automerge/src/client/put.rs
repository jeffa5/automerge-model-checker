use crate::app::AppState;

use super::Application;

/// A client strategy that just puts at a single key into a map.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapSinglePutter;

impl Application for MapSinglePutter {
    type Input = (String, String);

    type Output = ();

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Self::State>,
        (key, value): Self::Input,
    ) -> Self::Output {
        document.to_mut().put_map(key, value);
    }
}

/// A client strategy that just puts at the start of a list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListPutter;

impl Application for ListPutter {
    type Input = (usize, String);

    type Output = ();
    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Self::State>,
        (index, value): Self::Input,
    ) -> Self::Output {
        document.to_mut().put_list(index, value);
    }
}
