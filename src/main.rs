use automerge::Automerge;
use clap::Parser;
use client::Client;
use client::ClientMsg;
use doc::LIST_KEY;
use doc::MAP_KEY;
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
pub enum GlobalMsg {
    /// A message specific to the register system's internal protocol.
    Internal(PeerMsg),

    /// Messages originating or destined for clients.
    Client(ClientMsg),
}

impl Actor for MyRegisterActor {
    type Msg = GlobalMsg;

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

struct ModelBuilder {
    put_clients: usize,
    delete_clients: usize,
    insert_clients: usize,
    object_type: ObjectType,
    servers: usize,
    sync_method: SyncMethod,
    message_acks: bool,
}

impl ModelBuilder {
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
            match self.object_type {
                ObjectType::Map => {
                    model = model.actor(MyRegisterActor::Client(Client::MapSinglePutter(
                        client::MapSinglePutter {
                            request_count: 2,
                            server_count: self.servers,
                            key: "key".to_owned(),
                        },
                    )))
                }
                ObjectType::List => {
                    model = model.actor(MyRegisterActor::Client(Client::ListStartPutter(
                        client::ListStartPutter {
                            request_count: 2,
                            server_count: self.servers,
                        },
                    )))
                }
            }
        }

        for _ in 0..self.delete_clients {
            match self.object_type {
                ObjectType::Map => {
                    model = model.actor(MyRegisterActor::Client(Client::MapSingleDeleter(
                        client::MapSingleDeleter {
                            request_count: 2,
                            server_count: self.servers,
                            key: "key".to_owned(),
                        },
                    )))
                }
                ObjectType::List => {
                    model = model.actor(MyRegisterActor::Client(Client::ListDeleter(
                        client::ListDeleter {
                            index: 0,
                            request_count: 2,
                            server_count: self.servers,
                        },
                    )))
                }
            }
        }

        for _ in 0..self.insert_clients {
            match self.object_type {
                ObjectType::List => {
                    model = model.actor(MyRegisterActor::Client(Client::ListInserter(
                        client::ListInserter {
                            index: 0,
                            request_count: 2,
                            server_count: self.servers,
                        },
                    )))
                }
                ObjectType::Map => {
                    println!(
                        "had {} insert_clients but using a map object, no insert clients will be used", self.insert_clients
                    );
                    break;
                }
            }
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
                "saving and loading the document gives the same document",
                |_, state| save_load_same(state),
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
            .property(
                stateright::Expectation::Sometimes,
                "reach max map size",
                |model, state| {
                    state
                        .actor_states
                        .iter()
                        .any(|s| state_has_max_map_size(&model.actors, s))
                },
            )
            .property(
                stateright::Expectation::Always,
                "max map size is the max",
                |model, state| {
                    state
                        .actor_states
                        .iter()
                        .all(|s| max_map_size_is_the_max(&model.actors, s))
                },
            )
            .property(
                stateright::Expectation::Sometimes,
                "reach max list size",
                |model, state| {
                    state
                        .actor_states
                        .iter()
                        .any(|s| state_has_max_list_size(&model.actors, s))
                },
            )
            .property(
                stateright::Expectation::Always,
                "max list size is the max",
                |model, state| {
                    state
                        .actor_states
                        .iter()
                        .all(|s| max_list_size_is_the_max(&model.actors, s))
                },
            )
            .init_network(Network::new_ordered(vec![]))
    }
}

// TODO: move this to a precalculated field on a config struct that is shared.
fn max_map_size(actors: &[MyRegisterActor]) -> usize {
    actors
        .iter()
        .map(|a| match a {
            MyRegisterActor::Client(c) => match c {
                Client::MapSinglePutter(c) => c.request_count,
                Client::MapSingleDeleter(_)
                | Client::ListStartPutter(_)
                | Client::ListDeleter(_)
                | Client::ListInserter(_) => 0,
            },
            MyRegisterActor::Server(_) => 0,
        })
        .sum()
}

// TODO: move this to a precalculated field on a config struct that is shared.
fn max_list_size(actors: &[MyRegisterActor]) -> usize {
    actors
        .iter()
        .map(|a| match a {
            MyRegisterActor::Client(c) => match c {
                Client::MapSinglePutter(_)
                | Client::MapSingleDeleter(_)
                | Client::ListStartPutter(_)
                | Client::ListDeleter(_) => 0,
                Client::ListInserter(c) => c.request_count,
            },
            MyRegisterActor::Server(_) => 0,
        })
        .sum()
}

fn state_has_max_map_size(actors: &[MyRegisterActor], state: &Arc<MyRegisterActorState>) -> bool {
    let max = max_map_size(actors);
    if let MyRegisterActorState::Server(s) = &**state {
        s.length(MAP_KEY) == max
    } else {
        false
    }
}

fn max_map_size_is_the_max(actors: &[MyRegisterActor], state: &Arc<MyRegisterActorState>) -> bool {
    let max = max_map_size(actors);
    if let MyRegisterActorState::Server(s) = &**state {
        s.length(MAP_KEY) <= max
    } else {
        true
    }
}

fn state_has_max_list_size(actors: &[MyRegisterActor], state: &Arc<MyRegisterActorState>) -> bool {
    let max = max_list_size(actors);
    if let MyRegisterActorState::Server(s) = &**state {
        s.length(LIST_KEY) == max
    } else {
        false
    }
}

fn max_list_size_is_the_max(actors: &[MyRegisterActor], state: &Arc<MyRegisterActorState>) -> bool {
    let max = max_list_size(actors);
    if let MyRegisterActorState::Server(s) = &**state {
        s.length(LIST_KEY) <= max
    } else {
        true
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

fn syncing_done(state: &ActorModelState<MyRegisterActor>) -> bool {
    for envelope in state.network.iter_deliverable() {
        match envelope.msg {
            GlobalMsg::Internal(PeerMsg::SyncMessageRaw { .. }) => {
                return false;
            }
            GlobalMsg::Internal(PeerMsg::SyncChangeRaw { .. }) => {
                return false;
            }
            GlobalMsg::Internal(PeerMsg::SyncSaveLoadRaw { .. }) => {
                return false;
            }
            GlobalMsg::Client(_) => {}
        }
    }
    true
}

fn syncing_done_and_in_sync(state: &ActorModelState<MyRegisterActor>) -> bool {
    // first check that the network has no sync messages in-flight.
    // next, check that all actors are in the same states (using sub-property checker)
    !syncing_done(state) || all_same_state(&state.actor_states)
}

fn save_load_same(state: &ActorModelState<MyRegisterActor>) -> bool {
    for actor in &state.actor_states {
        match &**actor {
            MyRegisterActorState::Client(_) => {
                // clients don't have state to save and load
            }
            MyRegisterActorState::Server(s) => {
                let bytes = s.clone().save();
                let doc = Automerge::load(&bytes).unwrap();
                if doc.get_heads() != s.heads() {
                    return false;
                }
            }
        }
    }
    true
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
    insert_clients: usize,

    #[clap(long, short, global = true, default_value = "2")]
    servers: usize,

    #[clap(long, global = true)]
    message_acks: bool,

    #[clap(long, arg_enum, global = true, default_value = "changes")]
    sync_method: SyncMethod,

    // What object type to check.
    #[clap(long, arg_enum, global = true, default_value = "map")]
    object_type: ObjectType,

    #[clap(long, default_value = "8080")]
    port: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ArgEnum)]
pub enum ObjectType {
    Map,
    List,
}

#[derive(clap::Subcommand, Debug)]
enum SubCmd {
    Serve,
    CheckDfs,
    CheckBfs,
}

fn main() {
    let opts = Opts::parse();

    let model = ModelBuilder {
        put_clients: opts.put_clients,
        delete_clients: opts.delete_clients,
        insert_clients: opts.insert_clients,
        servers: opts.servers,
        sync_method: opts.sync_method,
        message_acks: opts.message_acks,
        object_type: opts.object_type,
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
