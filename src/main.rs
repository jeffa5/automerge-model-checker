use automerge::sync;
use automerge::Automerge;
use automerge::Change;
use clap::Parser;
use client::Client;
use client::ClientMsg;
use doc::Doc;
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
mod report;

type RequestId = usize;
type Key = String;
type Value = String;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Peer {
    peers: Vec<Id>,
    sync_method: SyncMethod,
    message_acks: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ArgEnum)]
enum SyncMethod {
    Changes,
    Messages,
    SaveLoad,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PeerMsg {
    // TODO: make this use the raw struct to avoid serde overhead
    SyncMessage { message_bytes: Vec<u8> },
    SyncChange { change_bytes: Vec<u8> },
    SyncSaveLoad { doc_bytes: Vec<u8> },
}

impl Actor for Peer {
    type Msg = MyRegisterMsg;

    type State = Doc;

    fn on_start(&self, id: Id, _o: &mut Out<Self>) -> Self::State {
        Self::State::new(id)
    }

    fn on_msg(
        &self,
        _id: Id,
        state: &mut std::borrow::Cow<Self::State>,
        src: Id,
        msg: Self::Msg,
        o: &mut Out<Self>,
    ) {
        match msg {
            MyRegisterMsg::Client(ClientMsg::Put(id, key, value)) => {
                // apply the op locally
                state.to_mut().put(key, value);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, MyRegisterMsg::Client(ClientMsg::PutOk(id)));
                }

                match self.sync_method {
                    SyncMethod::Changes => {
                        if let Some(change) = state.last_local_change() {
                            o.broadcast(
                                &self.peers,
                                &MyRegisterMsg::Internal(PeerMsg::SyncChange {
                                    change_bytes: change.raw_bytes().to_vec(),
                                }),
                            )
                        }
                    }
                    SyncMethod::Messages => {
                        // each peer has a specific state to manage in the sync connection
                        for peer in &self.peers {
                            if let Some(message) =
                                state.to_mut().generate_sync_message((*peer).into())
                            {
                                o.send(
                                    *peer,
                                    MyRegisterMsg::Internal(PeerMsg::SyncMessage {
                                        message_bytes: message.encode(),
                                    }),
                                )
                            }
                        }
                    }
                    SyncMethod::SaveLoad => {
                        let bytes = state.to_mut().save();
                        o.broadcast(
                            &self.peers,
                            &MyRegisterMsg::Internal(PeerMsg::SyncSaveLoad { doc_bytes: bytes }),
                        );
                    }
                }
            }
            MyRegisterMsg::Client(ClientMsg::Get(id, key)) => {
                if let Some(value) = state.get(&key) {
                    if self.message_acks {
                        // respond to the query (not totally necessary for this)
                        o.send(src, MyRegisterMsg::Client(ClientMsg::GetOk(id, value)))
                    }
                }
            }
            MyRegisterMsg::Client(ClientMsg::Delete(id, key)) => {
                // apply the op locally
                state.to_mut().delete(&key);

                if self.message_acks {
                    // respond to the query (not totally necessary for this)
                    o.send(src, MyRegisterMsg::Client(ClientMsg::DeleteOk(id)));
                }

                match self.sync_method {
                    SyncMethod::Changes => {
                        if let Some(change) = state.last_local_change() {
                            o.broadcast(
                                &self.peers,
                                &MyRegisterMsg::Internal(PeerMsg::SyncChange {
                                    change_bytes: change.raw_bytes().to_vec(),
                                }),
                            )
                        }
                    }
                    SyncMethod::Messages => {
                        // each peer has a specific state to manage in the sync connection
                        for peer in &self.peers {
                            if let Some(message) =
                                state.to_mut().generate_sync_message((*peer).into())
                            {
                                o.send(
                                    *peer,
                                    MyRegisterMsg::Internal(PeerMsg::SyncMessage {
                                        message_bytes: message.encode(),
                                    }),
                                )
                            }
                        }
                    }
                    SyncMethod::SaveLoad => {
                        let bytes = state.to_mut().save();
                        o.broadcast(
                            &self.peers,
                            &MyRegisterMsg::Internal(PeerMsg::SyncSaveLoad { doc_bytes: bytes }),
                        );
                    }
                }
            }
            MyRegisterMsg::Internal(PeerMsg::SyncMessage { message_bytes }) => {
                let message = sync::Message::decode(&message_bytes).unwrap();
                // receive the sync message
                state.to_mut().receive_sync_message(src.into(), message);
                // try and generate a reply
                if let Some(message) = state.to_mut().generate_sync_message(src.into()) {
                    o.send(
                        src,
                        MyRegisterMsg::Internal(PeerMsg::SyncMessage {
                            message_bytes: message.encode(),
                        }),
                    )
                }
            }
            MyRegisterMsg::Internal(PeerMsg::SyncChange { change_bytes }) => {
                let change = Change::from_bytes(change_bytes).unwrap();
                state.to_mut().apply_change(change)
            }
            MyRegisterMsg::Internal(PeerMsg::SyncSaveLoad { doc_bytes }) => {
                let mut other_doc = Automerge::load(&doc_bytes).unwrap();
                state.to_mut().merge(&mut other_doc);
            }
            MyRegisterMsg::Client(ClientMsg::PutOk(_id)) => {}
            MyRegisterMsg::Client(ClientMsg::GetOk(_id, _value)) => {}
            MyRegisterMsg::Client(ClientMsg::DeleteOk(_id)) => {}
        }
    }
}

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
            println!("Serving web ui on http://127.0.0.1:8080");
            model.serve("127.0.0.1:8080");
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
