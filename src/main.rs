use clap::Parser;
use client::Client;
use client::ClientMsg;
use peer::Peer;
use peer::PeerMsg;
use peer::SyncMethod;
use report::Reporter;
use stateright::actor::model_peers;
use stateright::actor::Actor;
use stateright::actor::ActorModel;
use stateright::actor::ActorModelState;
use stateright::actor::Network;
use stateright::actor::Out;
use stateright::Checker;
use stateright::CheckerBuilder;
use stateright::{actor::Id, Model};
use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

mod client;
mod doc;
mod peer;
mod report;

type RequestId = usize;
type Key = String;
type Value = String;

#[derive(Clone, Debug, Eq, PartialEq)]
enum MyRegisterActor {
    Client(Client),
    Server(Peer),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum MyRegisterActorState {
    Client(<Client as Actor>::State),
    Server(<Peer as Actor>::State),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum MyRegisterMsg {
    /// A message specific to the register system's internal protocol.
    Internal(PeerMsg),

    /// Messages originating or destined for clients.
    Client(ClientMsg),
}

impl Actor for MyRegisterActor {
    type Msg = MyRegisterMsg;

    type State = MyRegisterActorState;

    fn on_start(&self, id: Id, o: &mut Out<Self>) -> Self::State {
        match self {
            MyRegisterActor::Client(client_actor) => {
                let mut client_out = Out::new();
                let state =
                    MyRegisterActorState::Client(client_actor.on_start(id, &mut client_out));
                o.append(&mut client_out);
                state
            }
            MyRegisterActor::Server(server_actor) => {
                let mut server_out = Out::new();
                let state =
                    MyRegisterActorState::Server(server_actor.on_start(id, &mut server_out));
                o.append(&mut server_out);
                state
            }
        }
    }

    fn on_msg(
        &self,
        id: Id,
        state: &mut Cow<Self::State>,
        src: Id,
        msg: Self::Msg,
        o: &mut Out<Self>,
    ) {
        use MyRegisterActor as A;
        use MyRegisterActorState as S;

        match (self, &**state) {
            (A::Client(client_actor), S::Client(client_state)) => {
                let mut client_state = Cow::Borrowed(client_state);
                let mut client_out = Out::new();
                client_actor.on_msg(id, &mut client_state, src, msg, &mut client_out);
                if let Cow::Owned(client_state) = client_state {
                    *state = Cow::Owned(MyRegisterActorState::Client(client_state))
                }
                o.append(&mut client_out);
            }
            (A::Server(server_actor), S::Server(server_state)) => {
                let mut server_state = Cow::Borrowed(server_state);
                let mut server_out = Out::new();
                server_actor.on_msg(id, &mut server_state, src, msg, &mut server_out);
                if let Cow::Owned(server_state) = server_state {
                    *state = Cow::Owned(MyRegisterActorState::Server(server_state))
                }
                o.append(&mut server_out);
            }
            (A::Server(_), S::Client(_)) => {}
            (A::Client(_), S::Server(_)) => {}
        }
    }

    fn on_timeout(&self, id: Id, state: &mut Cow<Self::State>, o: &mut Out<Self>) {
        use MyRegisterActor as A;
        use MyRegisterActorState as S;
        match (self, &**state) {
            (A::Client(_), S::Client(_)) => {}
            (A::Client(_), S::Server(_)) => {}
            (A::Server(server_actor), S::Server(server_state)) => {
                let mut server_state = Cow::Borrowed(server_state);
                let mut server_out = Out::new();
                server_actor.on_timeout(id, &mut server_state, &mut server_out);
                if let Cow::Owned(server_state) = server_state {
                    *state = Cow::Owned(MyRegisterActorState::Server(server_state))
                }
                o.append(&mut server_out);
            }
            (A::Server(_), S::Client(_)) => {}
        }
    }
}

struct ModelCfg {
    put_clients: usize,
    delete_clients: usize,
    servers: usize,
    sync_method: SyncMethod,
    message_acks: bool,
}

impl ModelCfg {
    fn into_actor_model(self) -> ActorModel<MyRegisterActor, (), ()> {
        let mut model = ActorModel::new((), ());
        for i in 0..self.servers {
            model = model.actor(MyRegisterActor::Server(Peer {
                peers: model_peers(i, self.servers),
                sync_method: self.sync_method,
                message_acks: self.message_acks,
            }))
        }

        for _ in 0..self.put_clients {
            model = model.actor(MyRegisterActor::Client(Client::MapSinglePutter(
                client::MapSinglePutter {
                    request_count: 2,
                    server_count: self.servers,
                    key: "key".to_owned(),
                },
            )))
        }

        for _ in 0..self.delete_clients {
            model = model.actor(MyRegisterActor::Client(Client::MapSingleDeleter(
                client::MapSingleDeleter {
                    request_count: 2,
                    server_count: self.servers,
                    key: "key".to_owned(),
                },
            )))
        }

        model
            .property(
                stateright::Expectation::Eventually,
                "all actors have the same value for all keys",
                |_, state| all_same_state(&state.actor_states),
            )
            .property(
                stateright::Expectation::Always,
                "in sync when syncing is done and no in-flight requests",
                |_, state| syncing_done_and_in_sync(state),
            )
            .property(
                stateright::Expectation::Always,
                "no errors set (from panics)",
                |_, state| {
                    state.actor_states.iter().all(|s| {
                        if let MyRegisterActorState::Server(s) = &**s {
                            !s.has_error()
                        } else {
                            true
                        }
                    })
                },
            )
            .init_network(Network::new_ordered(vec![]))
    }
}

fn all_same_state(actors: &[Arc<MyRegisterActorState>]) -> bool {
    actors.windows(2).all(|w| match (&*w[0], &*w[1]) {
        (MyRegisterActorState::Client(_), MyRegisterActorState::Client(_)) => true,
        (MyRegisterActorState::Client(_), MyRegisterActorState::Server(_)) => true,
        (MyRegisterActorState::Server(_), MyRegisterActorState::Client(_)) => true,
        (MyRegisterActorState::Server(a), MyRegisterActorState::Server(b)) => {
            a.values() == b.values()
        }
    })
}

fn syncing_done_and_in_sync(state: &ActorModelState<MyRegisterActor>) -> bool {
    // first check that the network has no sync messages in-flight.
    for envelope in state.network.iter_deliverable() {
        match envelope.msg {
            MyRegisterMsg::Internal(PeerMsg::SyncMessage { .. }) => {
                return true;
            }
            MyRegisterMsg::Internal(PeerMsg::SyncChange { .. }) => {
                return true;
            }
            MyRegisterMsg::Internal(PeerMsg::SyncSaveLoad { .. }) => {
                return true;
            }
            MyRegisterMsg::Client(_) => {}
        }
    }

    // next, check that all actors are in the same states (using sub-property checker)
    all_same_state(&state.actor_states)
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(subcommand)]
    command: SubCmd,

    #[clap(long, short, global = true, default_value = "2")]
    put_clients: usize,

    #[clap(long, short, global = true, default_value = "2")]
    delete_clients: usize,

    #[clap(long, short, global = true, default_value = "2")]
    servers: usize,

    #[clap(long, global = true)]
    message_acks: bool,

    #[clap(long, arg_enum, global = true, default_value = "changes")]
    sync_method: SyncMethod,

    #[clap(long, default_value = "8080")]
    port: u16,
}

#[derive(clap::Subcommand, Debug)]
enum SubCmd {
    Serve,
    CheckDfs,
    CheckBfs,
}

fn main() {
    let opts = Opts::parse();

    let model = ModelCfg {
        put_clients: opts.put_clients,
        delete_clients: opts.delete_clients,
        servers: opts.servers,
        sync_method: opts.sync_method,
        message_acks: opts.message_acks,
    }
    .into_actor_model()
    .checker()
    .threads(num_cpus::get());
    run(opts, model)
}

fn run(opts: Opts, model: CheckerBuilder<ActorModel<MyRegisterActor>>) {
    println!("Running with config {:?}", opts);
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
