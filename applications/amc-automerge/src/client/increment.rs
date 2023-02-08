use crate::app::AppState;

use super::Application;

/// A client strategy that increments a counter in a map.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapIncrementer;

impl Application for MapIncrementer {
    type Input = (String, i64);

    type Output = ();

    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Self::State>,
        (key, by): Self::Input,
    ) -> Self::Output {
        document.to_mut().increment_map(key, by);
    }
}

/// A client strategy that increments a counter in a list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListIncrementer;

impl Application for ListIncrementer {
    type Input = (usize, i64);

    type Output = ();
    type State = AppState;

    fn init(&self, id: usize) -> Self::State {
        AppState::new(id)
    }

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Self::State>,
        (index, by): Self::Input,
    ) -> Self::Output {
        document.to_mut().increment_list(index, by);
    }
}
