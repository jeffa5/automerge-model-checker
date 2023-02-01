use std::borrow::Cow;

use crate::{client::Application, drive::Drive};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// Repeater, see `repeat` on `Drive`.
pub struct Repeater<D> {
    driver: D,
    repeats: u8,
}

/// State for the repeater.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RepeaterState<S: Clone + 'static> {
    inner: S,
    current_repeat: u8,
    application_id: usize,
}

impl<A, D> Drive<A> for Repeater<D>
where
    A: Application,
    D: Drive<A>,
    D::State: 'static,
{
    type State = RepeaterState<D::State>;

    fn init(
        &self,
        application_id: usize,
    ) -> (<Self as Drive<A>>::State, Vec<<A as Application>::Input>) {
        let (inner, inputs) = self.driver.init(application_id);
        let state = RepeaterState {
            inner,
            current_repeat: 0,
            application_id,
        };
        (state, inputs)
    }

    fn handle_output(
        &self,
        state: &mut std::borrow::Cow<Self::State>,
        output: <A as Application>::Output,
    ) -> Vec<<A as Application>::Input> {
        let mut inner_state = Cow::Borrowed(&state.inner);
        let generated_inputs = self.driver.handle_output(&mut inner_state, output);
        if let Cow::Owned(inner_state) = inner_state {
            *state = Cow::Owned(RepeaterState {
                inner: inner_state,
                current_repeat: state.current_repeat,
                application_id: state.application_id,
            })
        }
        if generated_inputs.is_empty() {
            // no changes, try to repeat
            if state.current_repeat + 1 != self.repeats {
                let (driver_state, new_inputs) = self.driver.init(state.application_id);
                state.to_mut().current_repeat += 1;
                state.to_mut().inner = driver_state;
                return new_inputs;
            }
        }
        generated_inputs
    }
}

/// Repeat a driver's logic.
pub trait Repeat {
    /// Repeat this drivers logic `repeats` times.
    fn repeat(self, repeats: u8) -> Repeater<Self>
    where
        Self: Sized;
}

impl<D> Repeat for D {
    fn repeat(self, repeats: u8) -> Repeater<Self>
    where
        Self: Sized,
    {
        Repeater {
            driver: self,
            repeats,
        }
    }
}
