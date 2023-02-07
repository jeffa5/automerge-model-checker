use std::sync::Arc;

use crate::app::{LIST_KEY, MAP_KEY};
use crate::client::App;
use crate::scalar::ScalarValue;
use amc::global::{GlobalActor, GlobalActorState};

use crate::driver::Driver;
use clap::Parser;
use stateright::Property;

mod app;
mod client;
mod driver;
mod scalar;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ValueEnum)]
enum ObjectType {
    /// Use a map object in Automerge.
    Map,
    /// Use a list object in Automerge.
    List,
    /// Use a text object in Automerge.
    Text,
}

#[derive(Parser, Debug)]
struct AutomergeOpts {
    /// What object type to check.
    #[clap(long, global = true, default_value = "map")]
    object_type: crate::ObjectType,

    /// Use the bytes type.
    #[clap(long, global = true)]
    bytes: bool,

    /// Use the string type.
    #[clap(long, global = true)]
    string: bool,

    /// Use the int type.
    #[clap(long, global = true)]
    int: bool,

    /// Use the uint type.
    #[clap(long, global = true)]
    uint: bool,

    /// Use the timestamp type.
    #[clap(long, global = true)]
    timestamp: bool,

    /// Use the boolean type.
    #[clap(long, global = true)]
    boolean: bool,

    /// Use the null type.
    #[clap(long, global = true)]
    null: bool,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(flatten)]
    automerge_opts: AutomergeOpts,

    #[clap(flatten)]
    amc_args: amc::cli::RunArgs,
}

type ActorState = GlobalActorState<Driver, App>;

const INSERT_REQUEST_COUNT: usize = 2;

impl amc::model::ModelBuilder for AutomergeOpts {
    type App = App;

    type Driver = Driver;

    type Config = Config;

    type History = ();

    fn application(&self, _i: usize) -> Self::App {
        let c = App {
            map_single_putter: client::MapSinglePutter,
            map_single_deleter: client::MapSingleDeleter,
            list_start_putter: client::ListPutter,
            list_deleter: client::ListDeleter,
            list_inserter: client::ListInserter,
            text_start_putter: client::TextPutter,
            text_deleter: client::TextDeleter,
            text_inserter: client::TextInserter,
        };
        println!("Adding application {:?}", c);
        c
    }

    fn drivers(&self, server: usize) -> Vec<Self::Driver> {
        let mut drivers = vec![];
        let mut add_drivers = |value: ScalarValue| {
            let mut new_drivers = match self.object_type {
                ObjectType::Map => {
                    vec![
                        Driver {
                            func: crate::driver::DriverState::MapSinglePut {
                                request_count: 2,
                                key: "key".to_owned(),
                                value: value.clone(),
                            },
                        },
                        Driver {
                            func: crate::driver::DriverState::MapSingleDelete {
                                request_count: 2,
                                key: "key".to_owned(),
                            },
                        },
                    ]
                }
                ObjectType::List => {
                    vec![
                        Driver {
                            func: crate::driver::DriverState::ListStartPut {
                                request_count: 2,
                                index: 0,
                                value: value.clone(),
                            },
                        },
                        Driver {
                            func: crate::driver::DriverState::ListDelete {
                                request_count: 2,
                                index: 0,
                            },
                        },
                        Driver {
                            func: crate::driver::DriverState::ListInsert {
                                request_count: INSERT_REQUEST_COUNT,
                                index: 0,
                                value: value.clone(),
                            },
                        },
                    ]
                }
                ObjectType::Text => {
                    if let ScalarValue::Str(s) = value {
                        vec![
                            Driver {
                                func: crate::driver::DriverState::TextStartPut {
                                    request_count: 2,
                                    index: 0,
                                    value: s.clone(),
                                },
                            },
                            Driver {
                                func: crate::driver::DriverState::TextDelete {
                                    request_count: 2,
                                    index: 0,
                                },
                            },
                            Driver {
                                func: crate::driver::DriverState::TextInsert {
                                    request_count: INSERT_REQUEST_COUNT,
                                    index: 0,
                                    value: s.clone(),
                                },
                            },
                        ]
                    } else {
                        panic!("text object wanted but value wasn't a string")
                    }
                }
            };
            drivers.append(&mut new_drivers);
        };
        if self.bytes {
            let value = ScalarValue::Bytes(
                char::from_u32(('a' as u32) + server as u32)
                    .unwrap()
                    .to_string()
                    .into(),
            );
            add_drivers(value);
        }
        if self.string {
            let value = ScalarValue::Str(
                char::from_u32(('a' as u32) + server as u32)
                    .unwrap()
                    .to_string(),
            );
            add_drivers(value);
        }
        if self.int {
            let value = ScalarValue::Int(server as i64);
            add_drivers(value);
        }
        if self.uint {
            let value = ScalarValue::Uint(server as u64);
            add_drivers(value);
        }
        if self.timestamp {
            let value = ScalarValue::Timestamp(server as i64);
            add_drivers(value);
        }
        if self.boolean {
            let value = ScalarValue::Boolean(server % 2 == 0);
            add_drivers(value);
        }
        if self.null {
            let value = ScalarValue::Null;
            add_drivers(value);
        }
        println!("Adding clients {:?}", drivers);
        drivers
    }

    fn config(&self, model_opts: &amc::model::ModelOpts) -> Self::Config {
        let c = Config {
            max_map_size: 1,
            max_list_size: if self.object_type == ObjectType::Map {
                0
            } else {
                model_opts.servers * INSERT_REQUEST_COUNT
            },
        };
        println!("Built config {:?}", c);
        c
    }

    fn history(&self) -> Self::History {}

    fn properties(
        &self,
    ) -> Vec<
        stateright::Property<
            stateright::actor::ActorModel<GlobalActor<Self::App, Self::Driver>, Self::Config>,
        >,
    > {
        type Model = stateright::actor::ActorModel<GlobalActor<App, Driver>, Config>;
        type Prop = Property<Model>;
        vec![
            Prop::sometimes("reach max map size", |model, state| {
                state
                    .actor_states
                    .iter()
                    .any(|s| state_has_max_map_size(s, &model.cfg))
            }),
            Prop::always("max map size is the max", |model, state| {
                state
                    .actor_states
                    .iter()
                    .all(|s| max_map_size_is_the_max(s, &model.cfg))
            }),
            Prop::sometimes("reach max list size", |model, state| {
                state
                    .actor_states
                    .iter()
                    .any(|s| state_has_max_list_size(s, &model.cfg))
            }),
            Prop::always("max list size is the max", |model, state| {
                state
                    .actor_states
                    .iter()
                    .all(|s| max_list_size_is_the_max(s, &model.cfg))
            }),
        ]
    }
}

fn state_has_max_map_size(state: &Arc<ActorState>, cfg: &Config) -> bool {
    let max = cfg.max_map_size;
    if let GlobalActorState::Server(s) = &**state {
        s.length(MAP_KEY) == max
    } else {
        false
    }
}

fn max_map_size_is_the_max(state: &Arc<ActorState>, cfg: &Config) -> bool {
    let max = cfg.max_map_size;
    if let GlobalActorState::Server(s) = &**state {
        s.length(MAP_KEY) <= max
    } else {
        true
    }
}

fn state_has_max_list_size(state: &Arc<ActorState>, cfg: &Config) -> bool {
    let max = cfg.max_list_size;
    if let GlobalActorState::Server(s) = &**state {
        s.length(LIST_KEY) == max
    } else {
        false
    }
}

fn max_list_size_is_the_max(state: &Arc<ActorState>, cfg: &Config) -> bool {
    let max = cfg.max_list_size;
    if let GlobalActorState::Server(s) = &**state {
        s.length(LIST_KEY) <= max
    } else {
        true
    }
}

#[derive(Debug, Clone)]
struct Config {
    pub max_map_size: usize,
    pub max_list_size: usize,
}

fn main() {
    let Args {
        automerge_opts,
        amc_args,
    } = Args::parse();
    amc_args.run(automerge_opts);
}
