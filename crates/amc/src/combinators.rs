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
pub struct RepeaterState<D: Clone + 'static> {
    inner: Cow<'static, D>,
    finished_repeats: u8,
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
            inner: Cow::Owned(inner),
            finished_repeats: 0,
            application_id,
        };
        (state, inputs)
    }

    fn handle_output(
        &self,
        state: &mut std::borrow::Cow<Self::State>,
        output: <A as Application>::Output,
    ) -> Vec<<A as Application>::Input> {
        let state = state.to_mut();
        let generated_inputs = self.driver.handle_output(&mut state.inner, output);
        if generated_inputs.is_empty() {
            // no changes, try to repeat
            state.finished_repeats += 1;
            if state.finished_repeats != self.repeats {
                let (driver_state, new_inputs) = self.driver.init(state.application_id);
                state.inner = Cow::Owned(driver_state);
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
