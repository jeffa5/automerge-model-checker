use crate::app::AppState;

use super::Application;

/// A client strategy that just puts at a single key into a map.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapSinglePutter;

impl Application for MapSinglePutter {
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
    ) -> Self::Output {
        let value = 'A';
        document.to_mut().put_map(input, value.to_string());
    }
}

/// A client strategy that just puts at the start of a list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListPutter;

impl Application for ListPutter {
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
    ) -> Self::Output {
        let value = 'A';
        document.to_mut().put_list(input, value.to_string());
    }
}
