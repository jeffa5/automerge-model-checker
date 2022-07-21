use super::ClientFunction;

/// A client strategy that just puts at a single key into a map.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapSinglePutter;

impl ClientFunction for MapSinglePutter {
    type Input = String;

    type Output = ();

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Box<crate::doc::Doc>>,
        input: Self::Input,
    ) -> Self::Output {
        let value = 'A';
        document.to_mut().put_map(input, value.to_string());
    }
}

/// A client strategy that just puts at the start of a list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListPutter;

impl ClientFunction for ListPutter {
    type Input = usize;

    type Output = ();

    fn execute(
        &self,
        document: &mut std::borrow::Cow<Box<crate::doc::Doc>>,
        input: Self::Input,
    ) -> Self::Output {
        let value = 'A';
        document.to_mut().put_list(input, value.to_string());
    }
}
