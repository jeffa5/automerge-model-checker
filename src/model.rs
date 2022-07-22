use crate::app::LIST_KEY;
use crate::app::MAP_KEY;
use crate::client::Client;
use crate::trigger::Trigger;
use crate::ObjectType;
use amc_core::Application;
use amc_core::GlobalActor;
use amc_core::GlobalActorState;
use amc_core::GlobalMsg;
use amc_core::Server;
use amc_core::ServerMsg;
use amc_core::SyncMethod;
use automerge::Automerge;
use stateright::actor::{model_peers, ActorModel};
use stateright::actor::{ActorModelState, Network};
use std::sync::Arc;

pub type State = GlobalActorState<Trigger, Client>;
pub type Actor = GlobalActor<Trigger, Client>;

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
    pub fn into_actor_model(self) -> ActorModel<GlobalActor<Trigger, Client>, Config, ()> {
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
            model = model.actor(GlobalActor::Server(Server {
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
                    model = model.actor(GlobalActor::Trigger(Trigger {
                        func: crate::trigger::TriggerState::MapSinglePut {
                            request_count: 2,
                            key: "key".to_owned(),
                        },
                        server: i,
                    }));
                    model = model.actor(GlobalActor::Trigger(Trigger {
                        func: crate::trigger::TriggerState::MapSingleDelete {
                            request_count: 2,
                            key: "key".to_owned(),
                        },
                        server: i,
                    }));
                }
                ObjectType::List => {
                    model = model.actor(GlobalActor::Trigger(Trigger {
                        func: crate::trigger::TriggerState::ListStartPut {
                            request_count: 2,
                            index: 0,
                        },
                        server: i,
                    }));
                    model = model.actor(GlobalActor::Trigger(Trigger {
                        func: crate::trigger::TriggerState::ListDelete {
                            request_count: 2,
                            index: 0,
                        },
                        server: i,
                    }));
                    model = model.actor(GlobalActor::Trigger(Trigger {
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
                        if let GlobalActorState::Server(s) = &**s {
                            !s.document().has_error()
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

fn state_has_max_map_size(state: &Arc<State>, cfg: &Config) -> bool {
    let max = cfg.max_map_size;
    if let GlobalActorState::Server(s) = &**state {
        s.length(MAP_KEY) == max
    } else {
        false
    }
}

fn max_map_size_is_the_max(state: &Arc<State>, cfg: &Config) -> bool {
    let max = cfg.max_map_size;
    if let GlobalActorState::Server(s) = &**state {
        s.length(MAP_KEY) <= max
    } else {
        true
    }
}

fn state_has_max_list_size(state: &Arc<State>, cfg: &Config) -> bool {
    let max = cfg.max_list_size;
    if let GlobalActorState::Server(s) = &**state {
        s.length(LIST_KEY) == max
    } else {
        false
    }
}

fn max_list_size_is_the_max(state: &Arc<State>, cfg: &Config) -> bool {
    let max = cfg.max_list_size;
    if let GlobalActorState::Server(s) = &**state {
        s.length(LIST_KEY) <= max
    } else {
        true
    }
}

fn all_same_state(actors: &[Arc<State>]) -> bool {
    actors.windows(2).all(|w| match (&*w[0], &*w[1]) {
        (GlobalActorState::Trigger(_), GlobalActorState::Trigger(_)) => true,
        (GlobalActorState::Trigger(_), GlobalActorState::Server(_)) => true,
        (GlobalActorState::Server(_), GlobalActorState::Trigger(_)) => true,
        (GlobalActorState::Server(a), GlobalActorState::Server(b)) => a.values() == b.values(),
    })
}

fn syncing_done(state: &ActorModelState<Actor>) -> bool {
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

fn syncing_done_and_in_sync(state: &ActorModelState<Actor>) -> bool {
    // first check that the network has no sync messages in-flight.
    // next, check that all actors are in the same states (using sub-property checker)
    !syncing_done(state) || all_same_state(&state.actor_states)
}

fn save_load_same(state: &ActorModelState<Actor>) -> bool {
    for actor in &state.actor_states {
        match &**actor {
            GlobalActorState::Trigger(_) => {
                // clients don't have state to save and load
            }
            GlobalActorState::Server(s) => {
                let bytes = s.clone().document_mut().save();
                let doc = Automerge::load(&bytes).unwrap();
                if doc.get_heads() != s.document().heads() {
                    return false;
                }
            }
        }
    }
    true
}
