use amc_core::Application;
use amc_core::ClientMsg;
use amc_core::DerefDocument;
use amc_core::Document;
use amc_core::GlobalActor;
use amc_core::GlobalActorState;
use amc_core::GlobalMsg;
use amc_core::Reporter;
use amc_core::Server;
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
struct Trigger {
    func: TriggerFunc,
    server: Id,
}
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
enum TriggerFunc {
    Inc,
    Dec,
}

impl amc_core::Trigger<Counter> for Trigger {}
impl Actor for Trigger {
    type Msg = ClientMsg<Counter>;
    type State = ();
    fn on_start(&self, _id: Id, o: &mut Out<Self>) -> Self::State {
        match self.func {
            TriggerFunc::Inc => o.send(self.server, ClientMsg::Request(CounterMsg::Increment)),
            TriggerFunc::Dec => o.send(self.server, ClientMsg::Request(CounterMsg::Decrement)),
        }
    }
}

#[derive(clap::Subcommand, Debug)]
enum SubCmd {
    Serve,
    CheckDfs,
    CheckBfs,
}

/// Methods for syncing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ValueEnum)]
pub enum SyncMethod {
    Changes,
    Messages,
    SaveLoad,
}

#[derive(clap::Parser, Debug)]
struct Opts {
    #[clap(subcommand)]
    command: SubCmd,

    #[clap(long, short, global = true, default_value = "2")]
    servers: usize,

    #[clap(long, global = true, default_value = "1")]
    increments: usize,

    #[clap(long, global = true, default_value = "1")]
    decrements: usize,

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

    let max_value = (opts.servers * opts.increments) - (opts.servers * opts.decrements);
    let mut model = ActorModel::new(Config { max_value }, Vec::new());
    let app = Counter { initial_value: 1 };
    for i in 0..opts.servers {
        model = model.actor(GlobalActor::Server(Server {
            peers: model_peers(i, opts.servers),
            sync_method: amc_core::SyncMethod::Changes,
            app: app.clone(),
        }))
    }

    for i in 0..opts.servers {
        let i = Id::from(i);
        for _ in 0..opts.increments {
            model = model.actor(GlobalActor::Trigger(Trigger {
                func: TriggerFunc::Inc,
                server: i,
            }));
        }
        for _ in 0..opts.decrements {
            model = model.actor(GlobalActor::Trigger(Trigger {
                func: TriggerFunc::Dec,
                server: i,
            }));
        }
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
        if matches!(m.msg, GlobalMsg::ClientToServer(ClientMsg::Request(_))) {
            let mut nh = h.clone();
            nh.push(m.msg.clone());
            Some(nh)
        } else {
            None
        }
    });
    model =
        amc_core::model::with_default_properties(model).init_network(Network::new_ordered(vec![]));
    let model = model.checker().threads(1);

    match opts.command {
        SubCmd::Serve => {
            println!("Serving web ui on http://127.0.0.1:{}", opts.port);
            model.serve(("127.0.0.1", opts.port));
        }
        SubCmd::CheckDfs => {
            model
                .spawn_dfs()
                .report(&mut Reporter::default())
                .join()
                .assert_properties();
        }
        SubCmd::CheckBfs => {
            model
                .spawn_bfs()
                .report(&mut Reporter::default())
                .join()
                .assert_properties();
        }
    }
}
