use crate::app::App;

use super::ClientFunction;

/// A client strategy that just inserts at the start of the list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListInserter;

impl ClientFunction for ListInserter {
    type Input = usize;

    type Output = ();

    type Application = App;

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Self::Application>,
        input: Self::Input,
    ) -> Self::Output {
        let value = 'A';
        document.to_mut().insert(input, value.to_string());
    }
}
