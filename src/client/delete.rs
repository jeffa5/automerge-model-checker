use super::ClientFunction;

/// A client strategy that just deletes a single key in a map.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapSingleDeleter;

impl ClientFunction for MapSingleDeleter {
    type Input = String;

    type Output = ();

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Box<crate::doc::Doc>>,
        input: Self::Input,
    ) -> Self::Output {
        document.to_mut().delete(&input);
    }
}

/// A client strategy that just deletes the first element in a list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListDeleter;

impl ClientFunction for ListDeleter {
    type Input = usize;

    type Output = ();

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Box<crate::doc::Doc>>,
        input: Self::Input,
    ) -> Self::Output {
        document.to_mut().delete_list(input);
    }
}
