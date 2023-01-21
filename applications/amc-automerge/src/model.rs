use crate::app::LIST_KEY;
use crate::app::MAP_KEY;
use crate::client::App;
use crate::trigger::Driver;
use crate::ObjectType;
use amc::application::server::Server;
use amc::application::server::SyncMethod;
use amc::driver::client::Client;
use amc::global::GlobalActor;
use amc::global::GlobalActorState;
use amc::properties;
use stateright::actor::Network;
use stateright::actor::{model_peers, ActorModel};
use std::marker::PhantomData;
use std::sync::Arc;

pub type State = GlobalActorState<Driver, App>;
pub type Actor = GlobalActor<Driver, App>;

pub struct Config {
    pub max_map_size: usize,
    pub max_list_size: usize,
}

pub struct Builder {
    pub object_type: ObjectType,
    pub servers: usize,
    pub sync_method: SyncMethod,
    pub app: App,
}

impl Builder {
    pub fn into_actor_model(self) -> ActorModel<GlobalActor<App, Driver>, Config, ()> {
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
                app: self.app.clone(),
            }))
        }

        for i in 0..self.servers {
            let i = stateright::actor::Id::from(i);
            match self.object_type {
                ObjectType::Map => {
                    model = model.actor(GlobalActor::Client(Client {
                        server: i,
                        driver: Driver {
                            func: crate::trigger::DriverState::MapSinglePut {
                                request_count: 2,
                                key: "key".to_owned(),
                            },
                            server: i,
                        },
                        _app: PhantomData,
                    }));
                    model = model.actor(GlobalActor::Client(Client {
                        server: i,
                        driver: Driver {
                            func: crate::trigger::DriverState::MapSingleDelete {
                                request_count: 2,
                                key: "key".to_owned(),
                            },
                            server: i,
                        },
                        _app: PhantomData,
                    }));
                }
                ObjectType::List => {
                    model = model.actor(GlobalActor::Client(Client {
                        server: i,
                        driver: Driver {
                            func: crate::trigger::DriverState::ListStartPut {
                                request_count: 2,
                                index: 0,
                            },
                            server: i,
                        },
                        _app: PhantomData,
                    }));
                    model = model.actor(GlobalActor::Client(Client {
                        server: i,

                        driver: Driver {
                            func: crate::trigger::DriverState::ListDelete {
                                request_count: 2,
                                index: 0,
                            },
                            server: i,
                        },
                        _app: PhantomData,
                    }));
                    model = model.actor(GlobalActor::Client(Client {
                        server: i,
                        driver: Driver {
                            func: crate::trigger::DriverState::ListInsert {
                                request_count: insert_request_count,
                                index: 0,
                            },
                            server: i,
                        },
                        _app: PhantomData,
                    }));
                }
            }
        }

        model = properties::with_default_properties(model);
        model
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
