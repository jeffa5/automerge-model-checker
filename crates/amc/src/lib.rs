#![deny(missing_docs)]

//! AMC is a collection of utilities to aid in model-checking automerge based CRDTs.
//!
//! The main parts of this library are the [`Application`](application::Application), [`DerefDocument`](application::DerefDocument) and [`Drive`](driver::Drive)
//! traits.
//! These are used to define your application logic, a way to obtain the automerge document and
//! drivers for actions in your application respectively.

mod bytes;
mod client;
mod document;
mod drive;
mod server;

/// All global utilities.
pub mod global;

/// Utilities for built-in properties.
pub mod properties;

/// User application implementations.
pub mod application {
    pub use crate::client::Application;
    pub use crate::client::DerefDocument;
    pub use crate::document::Document;

    /// Wrappers around applications to handle syncing.
    pub mod server {
        pub use crate::server::{Server, ServerMsg, SyncMethod};
    }
}

/// Drivers of application functionality.
pub mod driver {
    pub use crate::client::ApplicationMsg;
    pub use crate::drive::Drive;

    /// Wrappers around drivers.
    pub mod client {
        pub use crate::client::Client;
    }
}

/// Combination of useful items.
///
/// Use with `use amc::prelude::*;`.
pub mod prelude {
    pub use crate::application::Application;
    pub use crate::client::ApplicationMsg;
    pub use crate::client::DerefDocument;
    pub use crate::document::Document;
    pub use crate::drive::Drive;
}
