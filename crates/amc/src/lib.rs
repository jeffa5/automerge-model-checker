//! AMC is a collection of utilities to aid in model-checking automerge based CRDTs.
//!
//! The main parts of this library are the [`Application`], [`DerefDocument`] and [`Trigger`]
//! traits.
//! These are used to define your application logic, a way to obtain the automerge document and
//! triggers for actions in your application respectively.
//!
//! The following code is from the [counter example](../examples/counter.rs).
//!
//! ```rust,no_run
//! # use std::borrow::Cow;
//! # use automerge::ROOT;
//! # use automerge::transaction::Transactable;
//! # use amc::Application;
//! # use amc::Document;
//! # use amc::ClientMsg;
//! # use amc::DerefDocument;
//! # use amc::GlobalActor;
//! # use amc::GlobalMsg;
//! # use amc::Server;
//! # use stateright::Model;
//! # use stateright::Checker;
//! # use stateright::actor::Network;
//! # use stateright::actor::ActorModel;
//! # use stateright::actor::Id;
//! # use stateright::actor::model_peers;
//! # use stateright::Expectation;
//! # use stateright::actor::Out;
//! # use stateright::actor::Actor;
//! # use amc::SyncMethod;
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
//! struct Trigger {
//!     func: TriggerFunc,
//!     server: Id,
//! }
//! #[derive(Clone, Hash, Eq, PartialEq, Debug)]
//! enum TriggerFunc {
//!     Inc,
//!     Dec,
//! }
//!
//! impl amc::Trigger<Counter> for Trigger {}
//! impl Actor for Trigger {
//!     type Msg = ClientMsg<Counter>;
//!     type State = ();
//!     fn on_start(&self, _id:Id, o: &mut Out<Self>) -> Self::State {
//!     match self.func {
//!         TriggerFunc::Inc => o.send(self.server, ClientMsg::Request(CounterMsg::Increment)),
//!         TriggerFunc::Dec => o.send(self.server, ClientMsg::Request(CounterMsg::Decrement)),
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
//!         model = model.actor(GlobalActor::Trigger(Trigger {
//!             func: TriggerFunc::Inc,
//!             server: i,
//!         }));
//!         model = model.actor(GlobalActor::Trigger(Trigger {
//!             func: TriggerFunc::Dec,
//!             server: i,
//!         }));
//!     }
//!     model = model.property(Expectation::Always, "captures history", |model, state| {
//!         // TODO
//!         true
//!     });
//!     model = model.record_msg_in(|_, h, m| {
//!         if matches!(m.msg, GlobalMsg::ClientToServer(ClientMsg::Request(_))) {
//!             let mut nh = h.clone();
//!             nh.push(m.msg.clone());
//!             Some(nh)
//!         } else {
//!             None
//!         }
//!     });
//!     model = amc::model::with_default_properties(model).init_network(Network::new_ordered(vec![]));
//!     model.checker().threads(1).spawn_dfs().join().assert_properties();
//! ```

mod bytes;
mod client;
mod document;
mod global;
pub mod model;
mod report;
mod server;
mod trigger;

pub use client::DerefDocument;
pub use client::{Application, ClientMsg};
pub use document::Document;
pub use global::{GlobalActor, GlobalActorState, GlobalMsg};
pub use report::Reporter;
pub use server::{Server, ServerMsg, SyncMethod};
pub use trigger::Trigger;
