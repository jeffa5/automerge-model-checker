use automerge::sync;
use automerge::Change;
use clap::Parser;
use doc::Doc;
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
use std::str::FromStr;
use std::sync::Arc;

mod doc;

const KEY: &str = "key";

type RequestId = usize;
type Key = String;
type Value = String;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Peer {
    peers: Vec<Id>,
    sync_method: SyncMethod,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum SyncMethod {
    Changes,
    Messages,
}

impl FromStr for SyncMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "changes" => Ok(SyncMethod::Changes),
            "messages" => Ok(SyncMethod::Messages),
            _ => Err("Failed to match sync method".to_owned()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum PeerMsg {
    // TODO: make this use the raw struct to avoid serde overhead
    SyncMessage { message_bytes: Vec<u8> },
    SyncChange { change_bytes: Vec<u8> },
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
            MyRegisterMsg::Put(id, key, value) => {
                // apply the op locally
                state.to_mut().put(key, value);

                // respond to the query (not totally necessary for this)
                o.send(src, MyRegisterMsg::PutOk(id));

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
                    SyncMethod::Messages => todo!(),
                }
            }
            MyRegisterMsg::Get(id, key) => {
                if let Some(value) = state.get(&key) {
                    // respond to the query (not totally necessary for this)
                    o.send(src, MyRegisterMsg::GetOk(id, value))
                }
            }
            MyRegisterMsg::Delete(id, key) => {
                // apply the op locally
                state.to_mut().delete(&key);

                // respond to the query (not totally necessary for this)
                o.send(src, MyRegisterMsg::DeleteOk(id));

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
                    SyncMethod::Messages => todo!(),
                }
            }
            MyRegisterMsg::Internal(PeerMsg::SyncMessage { message_bytes }) => {
                let message = sync::Message::decode(&message_bytes).unwrap();
                // receive the sync message
                state.to_mut().receive_sync_message(message);
                // try and generate a reply
                if let Some(message) = state.to_mut().generate_sync_message() {
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
            MyRegisterMsg::PutOk(_id) => {}
            MyRegisterMsg::GetOk(_id, _value) => {}
            MyRegisterMsg::DeleteOk(_id) => {}
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum MyRegisterActor {
    PutClient {
        put_count: usize,
        /// Whether to send a get request after each mutation
        follow_up_gets: bool,
        server_count: usize,
    },
    DeleteClient {
        delete_count: usize,
        /// Whether to send a get request after each mutation
        follow_up_gets: bool,
        server_count: usize,
    },
    Server(Peer),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum MyRegisterActorState {
    PutClient {
        awaiting: Option<RequestId>,
        op_count: usize,
    },
    DeleteClient {
        awaiting: Option<RequestId>,
        op_count: usize,
    },
    Server(<Peer as Actor>::State),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum MyRegisterMsg {
    /// A message specific to the register system's internal protocol.
    Internal(PeerMsg),

    /// Indicates that a value should be written.
    Put(RequestId, Key, Value),
    /// Indicates that a value should be retrieved.
    Get(RequestId, Key),
    /// Indicates that a value should be deleted.
    Delete(RequestId, Key),

    /// Indicates a successful `Put`. Analogous to an HTTP 2XX.
    PutOk(RequestId),
    /// Indicates a successful `Get`. Analogous to an HTTP 2XX.
    GetOk(RequestId, Value),
    /// Indicates a successful `Delete`. Analogous to an HTTP 2XX.
    DeleteOk(RequestId),
}

impl Actor for MyRegisterActor {
    type Msg = MyRegisterMsg;

    type State = MyRegisterActorState;

    fn on_start(&self, id: Id, o: &mut Out<Self>) -> Self::State {
        match self {
            MyRegisterActor::PutClient {
                put_count,
                // don't issue reads from this so don't worry about this
                follow_up_gets: _,
                server_count,
            } => {
                let server_count = *server_count;

                let index: usize = id.into();
                if index < server_count {
                    panic!("MyRegisterActor clients must be added to the model after servers.");
                }

                if *put_count > 0 {
                    let unique_request_id = index; // next will be 2 * index
                    let value = (b'A' + (index % server_count) as u8) as char;
                    o.send(
                        Id::from(index % server_count),
                        MyRegisterMsg::Put(unique_request_id, KEY.to_owned(), value.to_string()),
                    );
                    MyRegisterActorState::PutClient {
                        awaiting: Some(unique_request_id),
                        op_count: 1,
                    }
                } else {
                    MyRegisterActorState::PutClient {
                        awaiting: None,
                        op_count: 0,
                    }
                }
            }
            MyRegisterActor::DeleteClient {
                delete_count,
                follow_up_gets: _,
                server_count,
            } => {
                let server_count = *server_count;

                let index: usize = id.into();
                if index < server_count {
                    panic!("MyRegisterActor clients must be added to the model after servers.");
                }

                if *delete_count > 0 {
                    let unique_request_id = index; // next will be 2 * index
                    o.send(
                        Id::from(index % server_count),
                        MyRegisterMsg::Delete(unique_request_id, KEY.to_owned()),
                    );
                    MyRegisterActorState::DeleteClient {
                        awaiting: Some(unique_request_id),
                        op_count: 1,
                    }
                } else {
                    MyRegisterActorState::DeleteClient {
                        awaiting: None,
                        op_count: 0,
                    }
                }
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
            (
                A::PutClient {
                    put_count,
                    follow_up_gets,
                    server_count,
                },
                S::PutClient {
                    awaiting: Some(awaiting),
                    op_count,
                },
            ) => {
                let server_count = *server_count;
                match msg {
                    MyRegisterMsg::PutOk(request_id) if &request_id == awaiting => {
                        let index: usize = id.into();
                        let unique_request_id = (op_count + 1) * index;
                        if *op_count < *put_count {
                            let value = (b'Z' - (index % server_count) as u8) as char;
                            o.send(
                                Id::from(index % server_count),
                                MyRegisterMsg::Put(
                                    unique_request_id,
                                    KEY.to_owned(),
                                    value.to_string(),
                                ),
                            );
                            *state = Cow::Owned(MyRegisterActorState::PutClient {
                                awaiting: Some(unique_request_id),
                                op_count: op_count + 1,
                            });
                        } else if *follow_up_gets {
                            o.send(
                                Id::from(index % server_count),
                                MyRegisterMsg::Get(unique_request_id, KEY.to_owned()),
                            );
                            *state = Cow::Owned(MyRegisterActorState::PutClient {
                                awaiting: Some(unique_request_id),
                                op_count: op_count + 1,
                            });
                        } else {
                            *state = Cow::Owned(MyRegisterActorState::PutClient {
                                awaiting: None,
                                op_count: op_count + 1,
                            });
                        }
                    }
                    MyRegisterMsg::GetOk(request_id, _value) if &request_id == awaiting => {
                        // finished
                        *state = Cow::Owned(MyRegisterActorState::PutClient {
                            awaiting: None,
                            op_count: op_count + 1,
                        });
                    }
                    MyRegisterMsg::DeleteOk(request_id) if &request_id == awaiting => {}
                    MyRegisterMsg::PutOk(_) => {}
                    MyRegisterMsg::GetOk(_, _) => {}
                    MyRegisterMsg::DeleteOk(_) => {}
                    MyRegisterMsg::Put(_, _, _) => {}
                    MyRegisterMsg::Get(_, _) => {}
                    MyRegisterMsg::Delete(_, _) => {}
                    MyRegisterMsg::Internal(_) => {}
                }
            }
            (
                A::DeleteClient {
                    delete_count,
                    follow_up_gets,
                    server_count,
                },
                S::DeleteClient {
                    awaiting: Some(awaiting),
                    op_count,
                },
            ) => {
                let server_count = *server_count;
                match msg {
                    MyRegisterMsg::PutOk(_) => {}
                    MyRegisterMsg::GetOk(request_id, _value) if &request_id == awaiting => {
                        // finished
                        *state = Cow::Owned(MyRegisterActorState::DeleteClient {
                            awaiting: None,
                            op_count: op_count + 1,
                        });
                    }
                    MyRegisterMsg::DeleteOk(request_id) if &request_id == awaiting => {
                        let index: usize = id.into();
                        let unique_request_id = (op_count + 1) * index;
                        if *op_count < *delete_count {
                            o.send(
                                Id::from(index % server_count),
                                MyRegisterMsg::Delete(unique_request_id, KEY.to_owned()),
                            );
                        } else if *follow_up_gets {
                            o.send(
                                Id::from(index % server_count),
                                MyRegisterMsg::Get(unique_request_id, KEY.to_owned()),
                            );
                            *state = Cow::Owned(MyRegisterActorState::DeleteClient {
                                awaiting: Some(unique_request_id),
                                op_count: op_count + 1,
                            });
                        } else {
                            *state = Cow::Owned(MyRegisterActorState::DeleteClient {
                                awaiting: None,
                                op_count: op_count + 1,
                            });
                        }
                    }
                    MyRegisterMsg::GetOk(_, _) => {}
                    MyRegisterMsg::DeleteOk(_) => {}
                    MyRegisterMsg::Put(_, _, _) => {}
                    MyRegisterMsg::Get(_, _) => {}
                    MyRegisterMsg::Delete(_, _) => {}
                    MyRegisterMsg::Internal(_) => {}
                }
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
            (A::Server(_), S::PutClient { .. }) => {}
            (A::Server(_), S::DeleteClient { .. }) => {}
            (A::PutClient { .. }, S::Server(_)) => {}
            (A::DeleteClient { .. }, S::Server(_)) => {}
            (
                A::PutClient {
                    put_count: _,
                    follow_up_gets: _,
                    server_count: _,
                },
                S::PutClient {
                    awaiting: None,
                    op_count: _,
                },
            ) => {}
            (
                A::DeleteClient {
                    delete_count: _,
                    follow_up_gets: _,
                    server_count: _,
                },
                S::DeleteClient {
                    awaiting: None,
                    op_count: _,
                },
            ) => {}
            (
                A::PutClient {
                    put_count: _,
                    follow_up_gets: _,
                    server_count: _,
                },
                S::DeleteClient {
                    awaiting: _,
                    op_count: _,
                },
            ) => {}
            (
                A::DeleteClient {
                    delete_count: _,
                    follow_up_gets: _,
                    server_count: _,
                },
                S::PutClient {
                    awaiting: _,
                    op_count: _,
                },
            ) => {}
        }
    }

    fn on_timeout(&self, id: Id, state: &mut Cow<Self::State>, o: &mut Out<Self>) {
        use MyRegisterActor as A;
        use MyRegisterActorState as S;
        match (self, &**state) {
            (A::PutClient { .. }, S::PutClient { .. }) => {}
            (A::PutClient { .. }, S::DeleteClient { .. }) => {}
            (A::DeleteClient { .. }, S::DeleteClient { .. }) => {}
            (A::DeleteClient { .. }, S::PutClient { .. }) => {}
            (A::Server(server_actor), S::Server(server_state)) => {
                let mut server_state = Cow::Borrowed(server_state);
                let mut server_out = Out::new();
                server_actor.on_timeout(id, &mut server_state, &mut server_out);
                if let Cow::Owned(server_state) = server_state {
                    *state = Cow::Owned(MyRegisterActorState::Server(server_state))
                }
                o.append(&mut server_out);
            }
            (A::Server(_), S::PutClient { .. }) => {}
            (A::Server(_), S::DeleteClient { .. }) => {}
            (A::PutClient { .. }, S::Server(_)) => {}
            (A::DeleteClient { .. }, S::Server(_)) => {}
        }
    }
}

struct ModelCfg {
    put_clients: usize,
    delete_clients: usize,
    servers: usize,
    follow_up_gets: bool,
    sync_method: SyncMethod,
}

impl ModelCfg {
    fn into_actor_model(self) -> ActorModel<MyRegisterActor, (), ()> {
        let mut model = ActorModel::new((), ());
        for i in 0..self.servers {
            model = model.actor(MyRegisterActor::Server(Peer {
                peers: model_peers(i, self.servers),
                sync_method: self.sync_method,
            }))
        }

        for _ in 0..self.put_clients {
            model = model.actor(MyRegisterActor::PutClient {
                put_count: 2,
                follow_up_gets: self.follow_up_gets,
                server_count: self.servers,
            })
        }

        for _ in 0..self.delete_clients {
            model = model.actor(MyRegisterActor::DeleteClient {
                delete_count: 2,
                follow_up_gets: self.follow_up_gets,
                server_count: self.servers,
            })
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
            .init_network(Network::new_ordered(vec![]))
    }
}

fn all_same_state(actors: &[Arc<MyRegisterActorState>]) -> bool {
    actors.windows(2).all(|w| match (&*w[0], &*w[1]) {
        (MyRegisterActorState::PutClient { .. }, MyRegisterActorState::PutClient { .. }) => true,
        (MyRegisterActorState::PutClient { .. }, MyRegisterActorState::DeleteClient { .. }) => true,
        (MyRegisterActorState::PutClient { .. }, MyRegisterActorState::Server(_)) => true,
        (MyRegisterActorState::DeleteClient { .. }, MyRegisterActorState::DeleteClient { .. }) => {
            true
        }
        (MyRegisterActorState::DeleteClient { .. }, MyRegisterActorState::PutClient { .. }) => true,
        (MyRegisterActorState::DeleteClient { .. }, MyRegisterActorState::Server(_)) => true,
        (MyRegisterActorState::Server(_), MyRegisterActorState::PutClient { .. }) => true,
        (MyRegisterActorState::Server(_), MyRegisterActorState::DeleteClient { .. }) => true,
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
            MyRegisterMsg::Put(_, _, _)
            | MyRegisterMsg::Get(_, _)
            | MyRegisterMsg::Delete(_, _)
            | MyRegisterMsg::PutOk(_)
            | MyRegisterMsg::GetOk(_, _)
            | MyRegisterMsg::DeleteOk(_) => {}
        }
    }

    // next, check that all actors are in the same states (using sub-property checker)
    all_same_state(&state.actor_states)
}

#[derive(Parser)]
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
    follow_up_gets: bool,

    #[clap(long)]
    sync_method: SyncMethod,
}

#[derive(clap::Subcommand)]
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
        follow_up_gets: opts.follow_up_gets,
        sync_method: opts.sync_method,
    }
    .into_actor_model()
    .checker()
    .threads(num_cpus::get());
    run(opts, model)
}

fn run(opts: Opts, model: CheckerBuilder<ActorModel<MyRegisterActor>>) {
    match opts.command {
        SubCmd::Serve => {
            println!("Serving web ui on http://127.0.0.1:8080");
            model.serve("127.0.0.1:8080");
        }
        SubCmd::CheckDfs => {
            model
                .spawn_dfs()
                .report(&mut std::io::stdout())
                .join()
                .assert_properties();
        }
        SubCmd::CheckBfs => {
            model
                .spawn_bfs()
                .report(&mut std::io::stdout())
                .join()
                .assert_properties();
        }
    }
}
