use std::{fmt::Debug, hash::Hash};

use stateright::actor::Id;

use crate::doc::Document;

pub trait Application: Clone + Eq + Hash + Debug {
    fn new(id: Id) -> Self;

    fn document(&self) -> &Document;
    fn document_mut(&mut self) -> &mut Document;
}
