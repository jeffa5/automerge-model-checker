#![deny(missing_docs)]

//! AMC is a collection of utilities to aid in model-checking automerge based CRDTs.
//!
//! The main parts of this library are the [`Application`](application::Application), [`DerefDocument`](application::DerefDocument) and [`Drive`](driver::Drive)
//! traits.
//! These are used to define your application logic, a way to obtain the automerge document and
//! drivers for actions in your application respectively.
//!
//! The following code is from the [counter example](../examples/counter.rs).
//!
//! ```rust,no_run
//! # use std::borrow::Cow;
//! # use automerge::ROOT;
//! # use automerge::transaction::Transactable;
//! # use amc::application::Application;
//! # use amc::application::Document;
//! # use amc::driver::ApplicationMsg;
//! # use amc::application::DerefDocument;
//! # use amc::global::GlobalActor;
//! # use amc::global::GlobalMsg;
//! # use amc::application::server::Server;
//! # use stateright::Model;
//! # use stateright::Checker;
//! # use stateright::actor::Network;
//! # use stateright::actor::ActorModel;
//! # use stateright::actor::Id;
//! # use stateright::actor::model_peers;
//! # use stateright::Expectation;
//! # use stateright::actor::Out;
//! # use stateright::actor::Actor;
//! # use amc::application::server::SyncMethod;
//! #
//! #[derive(Clone, Hash, Eq, PartialEq, Debug)]
//! struct Counter {
//!     initial_value: usize,
//! }
//!
//! #[derive(Clone, Hash, Eq, PartialEq, Debug)]
//! struct CounterState {
//!     value: usize,
//!     doc: Document,
//! }
//!
//! #[derive(Clone, Hash, Eq, PartialEq, Debug)]
//! enum CounterMsg {
//!     Increment,
//!     Decrement,
//! }
//!
//! impl Application for Counter {
//!     type Input = CounterMsg;
//!     type Output = ();
//!     type State = CounterState;
//!
//!     fn init(&self, id: Id) -> Self::State {
//!         CounterState { value: self.initial_value, doc: Document::new(id) }
//!     }
//!
//!     fn execute(&self, state: &mut Cow<Self::State>, input: Self::Input) -> Self::Output {
//!         match input {
//!             CounterMsg::Increment => {
//!                 let value = state.doc.get(ROOT, "counter").unwrap().and_then(|(v,_)| v.to_i64()).unwrap_or_default();
//!                 let state = state.to_mut();
//!                 let mut txn = state.doc.transaction();
//!                 txn.put(ROOT, "counter", value + 1).unwrap();
//!                 txn.commit();
//!             }
//!             CounterMsg::Decrement => {
//!                 let value = state.doc.get(ROOT, "counter").unwrap().and_then(|(v,_)| v.to_i64()).unwrap_or_default();
//!                 let state = state.to_mut();
//!                 let mut txn = state.doc.transaction();
//!                 txn.put(ROOT, "counter", value - 1).unwrap();
//!                 txn.commit();
//!             }
//!         }
//!     }
//! }
//!
//! impl DerefDocument for CounterState {
//!     fn document(&self) -> &Document {
//!         &self.doc
//!     }
//!     fn document_mut(&mut self) -> &mut Document {
//!         &mut self.doc
//!     }
//! }
//!
//! #[derive(Clone, Hash, Eq, PartialEq, Debug)]
//! struct Driver {
//!     func: TriggerFunc,
//!     server: Id,
//! }
//! #[derive(Clone, Hash, Eq, PartialEq, Debug)]
//! enum TriggerFunc {
//!     Inc,
//!     Dec,
//! }
//!
//! impl amc::driver::Drive<Counter> for Driver {}
//! impl Actor for Driver {
//!     type Msg = ApplicationMsg<Counter>;
//!     type State = ();
//!     fn on_start(&self, _id:Id, o: &mut Out<Self>) -> Self::State {
//!     match self.func {
//!         TriggerFunc::Inc => o.send(self.server, ApplicationMsg::Input(CounterMsg::Increment)),
//!         TriggerFunc::Dec => o.send(self.server, ApplicationMsg::Input(CounterMsg::Decrement)),
//!     }
//!     }
//! }
//!
//!     let mut model = ActorModel::new((), Vec::new());
//!     let num_servers = 2;
//!     let app = Counter{initial_value:1};
//!     for i in 0..num_servers {
//!         model = model.actor(GlobalActor::Server(Server {
//!             peers: model_peers(i, num_servers),
//!             sync_method: SyncMethod::Changes,
//!             app: app.clone(),
//!         }))
//!     }
//!
//!     for i in 0..num_servers {
//!         let i = Id::from(i);
//!         model = model.actor(GlobalActor::Driver(Driver {
//!             func: TriggerFunc::Inc,
//!             server: i,
//!         }));
//!         model = model.actor(GlobalActor::Driver(Driver {
//!             func: TriggerFunc::Dec,
//!             server: i,
//!         }));
//!     }
//!     model = model.property(Expectation::Always, "captures history", |model, state| {
//!         // TODO
//!         true
//!     });
//!     model = model.record_msg_in(|_, h, m| {
//!         if matches!(m.msg, GlobalMsg::ClientToServer(ApplicationMsg::Input(_))) {
//!             let mut nh = h.clone();
//!             nh.push(m.msg.clone());
//!             Some(nh)
//!         } else {
//!             None
//!         }
//!     });
//!     model = amc::properties::with_default_properties(model).init_network(Network::new_ordered(vec![]));
//!     model.checker().threads(1).spawn_dfs().join().assert_properties();
//! ```

mod bytes;
mod client;
mod document;
mod server;
mod drive;

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
}
