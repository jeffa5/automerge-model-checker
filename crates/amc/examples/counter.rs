//! An example of implementing an application to be tested with AMC.
//!
//! Run with `cargo run --release --example counter -- --help`
//!
//! The counter that this models is very simple, having an increment and decrement action.

use amc::application::Application;
use amc::application::DerefDocument;
use amc::application::Document;
use amc::application::server::Server;
use amc::application::server::SyncMethod;
use amc::global::GlobalActor;
use amc::global::GlobalActorState;
use amc::global::GlobalMsg;
use amc::driver::ApplicationMsg;
use amc::driver::Drive;
use automerge::transaction::Transactable;
use automerge::ROOT;
use stateright::actor::model_peers;
use stateright::actor::Actor;
use stateright::actor::ActorModel;
use stateright::actor::Id;
use stateright::actor::Network;
use stateright::actor::Out;
use stateright::Checker;
use stateright::Expectation;
use stateright::Model;
use std::borrow::Cow;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct Counter {
    initial_value: usize,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct CounterState {
    value: usize,
    doc: Document,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
enum CounterMsg {
    Increment,
    Decrement,
}

impl Application for Counter {
    type Input = CounterMsg;
    type Output = ();
    type State = CounterState;

    fn init(&self, id: Id) -> Self::State {
        CounterState {
            value: self.initial_value,
            doc: Document::new(id),
        }
    }

    fn execute(&self, state: &mut Cow<Self::State>, input: Self::Input) -> Self::Output {
        match input {
            CounterMsg::Increment => {
                let value = state
                    .doc
                    .get(ROOT, "counter")
                    .unwrap()
                    .and_then(|(v, _)| v.to_i64())
                    .unwrap_or_default();
                let state = state.to_mut();
                let mut txn = state.doc.transaction();
                txn.put(ROOT, "counter", value + 1).unwrap();
                txn.commit();
            }
            CounterMsg::Decrement => {
                let value = state
                    .doc
                    .get(ROOT, "counter")
                    .unwrap()
                    .and_then(|(v, _)| v.to_i64())
                    .unwrap_or_default();
                let state = state.to_mut();
                let mut txn = state.doc.transaction();
                txn.put(ROOT, "counter", value - 1).unwrap();
                txn.commit();
            }
        }
    }
}

impl DerefDocument for CounterState {
    fn document(&self) -> &Document {
        &self.doc
    }
    fn document_mut(&mut self) -> &mut Document {
        &mut self.doc
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct Driver {
    func: DriverFunc,
    server: Id,
}

/// Action for the application to perform.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
enum DriverFunc {
    /// Number of times to send an increment.
    Inc(u8),
    /// Number of times to send a decrement.
    Dec(u8),
}

impl Drive<Counter> for Driver {}
impl Actor for Driver {
    type Msg = ApplicationMsg<Counter>;
    type State = ();
    fn on_start(&self, _id: Id, o: &mut Out<Self>) -> Self::State {
        match self.func {
            DriverFunc::Inc(n) => {
                for _ in 0..n {
                    o.send(self.server, ApplicationMsg::Input(CounterMsg::Increment))
                }
            }
            DriverFunc::Dec(n) => {
                for _ in 0..n {
                    o.send(self.server, ApplicationMsg::Input(CounterMsg::Decrement))
                }
            }
        }
    }
}

#[derive(clap::Subcommand, Debug)]
enum SubCmd {
    Serve,
    CheckDfs,
    CheckBfs,
}

#[derive(clap::Parser, Debug)]
struct Opts {
    #[clap(subcommand)]
    command: SubCmd,

    #[clap(long, short, global = true, default_value = "2")]
    servers: usize,

    #[clap(long, global = true, default_value = "1")]
    increments: u8,

    #[clap(long, global = true, default_value = "1")]
    decrements: u8,

    #[clap(long, global = true, default_value = "changes")]
    sync_method: SyncMethod,

    #[clap(long, default_value = "8080")]
    port: u16,

    /// Whether to use random ids for todo creation.
    #[clap(long, global = true)]
    random_ids: bool,
}

struct Config {
    max_value: usize,
}

fn main() {
    use clap::Parser;
    let opts = Opts::parse();

    let max_value =
        (opts.servers * opts.increments as usize) - (opts.servers * opts.decrements as usize);
    let mut model = ActorModel::new(Config { max_value }, Vec::new());
    let app = Counter { initial_value: 1 };
    for i in 0..opts.servers {
        model = model.actor(GlobalActor::Server(Server {
            peers: model_peers(i, opts.servers),
            sync_method: SyncMethod::Changes,
            app: app.clone(),
        }))
    }

    for i in 0..opts.servers {
        let i = Id::from(i);
        model = model.actor(GlobalActor::Driver(Driver {
            func: DriverFunc::Inc(opts.increments),
            server: i,
        }));
        model = model.actor(GlobalActor::Driver(Driver {
            func: DriverFunc::Dec(opts.decrements),
            server: i,
        }));
    }
    model = model.property(Expectation::Eventually, "max value", |model, state| {
        for actor in &state.actor_states {
            if let GlobalActorState::Server(s) = &**actor {
                if s.document()
                    .get(ROOT, "counter")
                    .unwrap()
                    .and_then(|(v, _)| v.to_i64())
                    .unwrap_or_default()
                    == model.cfg.max_value as i64
                {
                    return true;
                }
            }
        }
        false
    });
    model = model.record_msg_in(|_, h, m| {
        if matches!(m.msg, GlobalMsg::ClientToServer(ApplicationMsg::Input(_))) {
            let mut nh = h.clone();
            nh.push(m.msg.clone());
            Some(nh)
        } else {
            None
        }
    });
    model = model.init_network(Network::new_ordered(vec![]));
    let model = model.checker().threads(num_cpus::get());

    match opts.command {
        SubCmd::Serve => {
            println!("Serving web ui on http://127.0.0.1:{}", opts.port);
            model.serve(("127.0.0.1", opts.port));
        }
        SubCmd::CheckDfs => {
            model
                .spawn_dfs()
                .join()
                .assert_properties();
        }
        SubCmd::CheckBfs => {
            model
                .spawn_bfs()
                .join()
                .assert_properties();
        }
    }
}
