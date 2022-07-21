use crate::client::Client;
use crate::doc::LIST_KEY;
use crate::doc::MAP_KEY;
use crate::register::GlobalMsg;
use crate::server::Server;
use crate::server::ServerMsg;
use crate::trigger::Trigger;
use crate::{
    register::MyRegisterActor, register::MyRegisterActorState, server::SyncMethod, ObjectType,
};
use automerge::Automerge;
use stateright::actor::{model_peers, ActorModel};
use stateright::actor::{ActorModelState, Network};
use std::sync::Arc;

pub struct Config {
    pub max_map_size: usize,
    pub max_list_size: usize,
}

pub struct Builder {
    pub object_type: ObjectType,
    pub servers: usize,
    pub sync_method: SyncMethod,
    pub message_acks: bool,
    pub client_function: Client,
}

impl Builder {
    pub fn into_actor_model(self) -> ActorModel<MyRegisterActor, Config, ()> {
        let insert_request_count = 2;
        let config = Config {
            max_map_size: 1,
            max_list_size: if self.object_type == ObjectType::Map {
                0
            } else {
                self.servers * insert_request_count
            },
        };
        let mut model = ActorModel::new(config, ());
        for i in 0..self.servers {
            model = model.actor(MyRegisterActor::Server(Server {
                peers: model_peers(i, self.servers),
                sync_method: self.sync_method,
                message_acks: self.message_acks,
                client_function: self.client_function.clone(),
            }))
        }

        for i in 0..self.servers {
            let i = stateright::actor::Id::from(i);
            match self.object_type {
                ObjectType::Map => {
                    model = model.actor(MyRegisterActor::Trigger(Trigger {
                        func: crate::trigger::TriggerState::MapSinglePut {
                            request_count: 2,
                            key: "key".to_owned(),
                        },
                        server: i,
                    }));
                    model = model.actor(MyRegisterActor::Trigger(Trigger {
                        func: crate::trigger::TriggerState::MapSingleDelete {
                            request_count: 2,
                            key: "key".to_owned(),
                        },
                        server: i,
                    }));
                }
                ObjectType::List => {
                    model = model.actor(MyRegisterActor::Trigger(Trigger {
                        func: crate::trigger::TriggerState::ListStartPut {
                            request_count: 2,
                            index: 0,
                        },
                        server: i,
                    }));
                    model = model.actor(MyRegisterActor::Trigger(Trigger {
                        func: crate::trigger::TriggerState::ListDelete {
                            request_count: 2,
                            index: 0,
                        },
                        server: i,
                    }));
                    model = model.actor(MyRegisterActor::Trigger(Trigger {
                        func: crate::trigger::TriggerState::ListInsert {
                            request_count: insert_request_count,
                            index: 0,
                        },
                        server: i,
                    }));
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
                        .any(|s| state_has_max_map_size(s, &model.cfg))
                },
            )
            .property(
                stateright::Expectation::Always,
                "max map size is the max",
                |model, state| {
                    state
                        .actor_states
                        .iter()
                        .all(|s| max_map_size_is_the_max(s, &model.cfg))
                },
            )
            .property(
                stateright::Expectation::Sometimes,
                "reach max list size",
                |model, state| {
                    state
                        .actor_states
                        .iter()
                        .any(|s| state_has_max_list_size(s, &model.cfg))
                },
            )
            .property(
                stateright::Expectation::Always,
                "max list size is the max",
                |model, state| {
                    state
                        .actor_states
                        .iter()
                        .all(|s| max_list_size_is_the_max(s, &model.cfg))
                },
            )
            .init_network(Network::new_ordered(vec![]))
    }
}

fn state_has_max_map_size(state: &Arc<MyRegisterActorState>, cfg: &Config) -> bool {
    let max = cfg.max_map_size;
    if let MyRegisterActorState::Server(s) = &**state {
        s.length(MAP_KEY) == max
    } else {
        false
    }
}

fn max_map_size_is_the_max(state: &Arc<MyRegisterActorState>, cfg: &Config) -> bool {
    let max = cfg.max_map_size;
    if let MyRegisterActorState::Server(s) = &**state {
        s.length(MAP_KEY) <= max
    } else {
        true
    }
}

fn state_has_max_list_size(state: &Arc<MyRegisterActorState>, cfg: &Config) -> bool {
    let max = cfg.max_list_size;
    if let MyRegisterActorState::Server(s) = &**state {
        s.length(LIST_KEY) == max
    } else {
        false
    }
}

fn max_list_size_is_the_max(state: &Arc<MyRegisterActorState>, cfg: &Config) -> bool {
    let max = cfg.max_list_size;
    if let MyRegisterActorState::Server(s) = &**state {
        s.length(LIST_KEY) <= max
    } else {
        true
    }
}

fn all_same_state(actors: &[Arc<MyRegisterActorState>]) -> bool {
    actors.windows(2).all(|w| match (&*w[0], &*w[1]) {
        (MyRegisterActorState::Trigger(_), MyRegisterActorState::Trigger(_)) => true,
        (MyRegisterActorState::Trigger(_), MyRegisterActorState::Server(_)) => true,
        (MyRegisterActorState::Server(_), MyRegisterActorState::Trigger(_)) => true,
        (MyRegisterActorState::Server(a), MyRegisterActorState::Server(b)) => {
            a.values() == b.values()
        }
    })
}

fn syncing_done(state: &ActorModelState<MyRegisterActor>) -> bool {
    for envelope in state.network.iter_deliverable() {
        match envelope.msg {
            GlobalMsg::Internal(ServerMsg::SyncMessageRaw { .. }) => {
                return false;
            }
            GlobalMsg::Internal(ServerMsg::SyncChangeRaw { .. }) => {
                return false;
            }
            GlobalMsg::Internal(ServerMsg::SyncSaveLoadRaw { .. }) => {
                return false;
            }
            GlobalMsg::External(_) => {}
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
            MyRegisterActorState::Trigger(_) => {
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
