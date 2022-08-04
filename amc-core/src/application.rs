use std::{fmt::Debug, hash::Hash};

use crate::Document;

/// A users application that runs alongside the document, implementing the business logic.
///
/// Internally it holds the automerge document and so provides accessors.
pub trait Application: Clone + Eq + Hash + Debug {
    /// Get the document.
    fn document(&self) -> &Document;

    /// Get a mutable reference to the document.
    fn document_mut(&mut self) -> &mut Document;
}
