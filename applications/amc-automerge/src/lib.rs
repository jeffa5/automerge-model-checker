use std::sync::Arc;

use crate::app::{LIST_KEY, MAP_KEY};
use crate::client::App;
use crate::scalar::ScalarValue;
use amc::global::{GlobalActor, GlobalActorState};
use app::TEXT_KEY;

use crate::driver::Driver;
use clap::Parser;
use stateright::Property;

mod app;
mod client;
mod driver;
mod scalar;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ValueEnum)]
pub enum ObjectType {
    /// Use a map object in Automerge.
    Map,
    /// Use a list object in Automerge.
    List,
    /// Use a text object in Automerge.
    Text,
}

#[derive(Parser, Debug)]
pub struct AutomergeOpts {
    /// What object type to check.
    #[clap(long, global = true, default_value = "map")]
    pub object_type: ObjectType,

    /// Use the bytes type.
    #[clap(long, global = true)]
    pub bytes: bool,

    /// Use the string type.
    #[clap(long, global = true)]
    pub string: bool,

    /// Use the int type.
    #[clap(long, global = true)]
    pub int: bool,

    /// Use the uint type.
    #[clap(long, global = true)]
    pub uint: bool,

    /// Use the timestamp type.
    #[clap(long, global = true)]
    pub timestamp: bool,

    /// Use the boolean type.
    #[clap(long, global = true)]
    pub boolean: bool,

    /// Use the null type.
    #[clap(long, global = true)]
    pub null: bool,

    /// Keys to use if using a map.
    #[clap(long, global = true, default_value = "foo", value_delimiter = ',')]
    pub keys: Vec<String>,

    /// Indices to target for list operations, should include 0 at least otherwise the list won't
    /// be built up.
    #[clap(long, global = true, default_value = "0", value_delimiter = ',')]
    pub indices: Vec<usize>,

    /// Whether to add splice operations for lists.
    #[clap(long, global = true)]
    pub splice: bool,

    /// Times to repeat each request.
    #[clap(long, global = true, default_value = "1")]
    pub repeats: u8,
}

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    pub automerge_opts: AutomergeOpts,

    #[clap(flatten)]
    pub amc_args: amc::cli::RunArgs,
}

type ActorState = GlobalActorState<Driver, App>;

impl amc::model::ModelBuilder for AutomergeOpts {
    type App = App;

    type Driver = Driver;

    type Config = Config;

    type History = ();

    fn application(&self, _i: usize, _config: &Config) -> Self::App {
        let c = App {
            map_single_putter: client::MapSinglePutter,
            map_single_deleter: client::MapSingleDeleter,
            map_incrementer: client::MapIncrementer,
            list_putter: client::ListPutter,
            list_deleter: client::ListDeleter,
            list_inserter: client::ListInserter,
            list_splicer: client::ListSplicer,
            list_incrementer: client::ListIncrementer,
            text_putter: client::TextPutter,
            text_deleter: client::TextDeleter,
            text_inserter: client::TextInserter,
            text_splicer: client::TextSplicer,
        };
        println!("Adding application {:?}", c);
        c
    }

    fn drivers(&self, server: usize, _config: &Config) -> Vec<Self::Driver> {
        let mut drivers = vec![];
        let mut add_drivers = |value: ScalarValue| {
            let mut new_drivers: Vec<_> = match self.object_type {
                ObjectType::Map => self
                    .keys
                    .iter()
                    .flat_map(|k| {
                        let mut d = vec![
                            Driver {
                                func: crate::driver::DriverState::MapSinglePut {
                                    key: k.to_owned(),
                                    value: value.clone(),
                                },
                                repeats: self.repeats,
                            },
                            Driver {
                                func: crate::driver::DriverState::MapSingleDelete {
                                    key: k.to_owned(),
                                },
                                repeats: self.repeats,
                            },
                        ];
                        if value.is_counter() {
                            d.push(Driver {
                                func: crate::driver::DriverState::MapIncrement {
                                    key: k.to_owned(),
                                    by: 1,
                                },
                                repeats: self.repeats,
                            });
                        }
                        d
                    })
                    .collect(),
                ObjectType::List => self
                    .indices
                    .iter()
                    .copied()
                    .flat_map(|index| {
                        let mut d = vec![
                            Driver {
                                func: crate::driver::DriverState::ListPut {
                                    index,
                                    value: value.clone(),
                                },
                                repeats: self.repeats,
                            },
                            Driver {
                                func: crate::driver::DriverState::ListDelete { index },
                                repeats: self.repeats,
                            },
                            Driver {
                                func: crate::driver::DriverState::ListInsert {
                                    index,
                                    value: value.clone(),
                                },
                                repeats: self.repeats,
                            },
                        ];
                        if self.splice {
                            d.push(Driver {
                                func: crate::driver::DriverState::ListSplice {
                                    index,
                                    delete: 2,
                                    values: vec![value.clone(); 2],
                                },
                                repeats: self.repeats,
                            });
                        }
                        if value.is_counter() {
                            d.push(Driver {
                                func: crate::driver::DriverState::ListIncrement { index, by: 1 },
                                repeats: self.repeats,
                            });
                        }
                        d
                    })
                    .collect(),
                ObjectType::Text => {
                    if let ScalarValue::Str(s) = value {
                        self.indices
                            .iter()
                            .copied()
                            .flat_map(|index| {
                                let mut d = vec![
                                    Driver {
                                        func: crate::driver::DriverState::TextPut {
                                            index,
                                            value: s.clone(),
                                        },
                                        repeats: self.repeats,
                                    },
                                    Driver {
                                        func: crate::driver::DriverState::TextDelete { index },
                                        repeats: self.repeats,
                                    },
                                    Driver {
                                        func: crate::driver::DriverState::TextInsert {
                                            index,
                                            value: s.clone(),
                                        },
                                        repeats: self.repeats,
                                    },
                                ];
                                if self.splice {
                                    d.push(Driver {
                                        func: crate::driver::DriverState::TextSplice {
                                            index,
                                            delete: 2,
                                            text: format!("{}{}", s, s),
                                        },
                                        repeats: self.repeats,
                                    });
                                }
                                d
                            })
                            .collect()
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
            max_map_size: if self.object_type == ObjectType::Map {
                self.keys.len()
            } else {
                // don't add to it otherwise
                0
            },
            max_list_size: if self.object_type == ObjectType::List {
                // each server performs an insert to the indices repeated some number of times
                model_opts.servers * self.repeats as usize * self.indices.len()
            } else {
                0
            },
            max_text_size: if self.object_type == ObjectType::Text {
                // each server performs an insert to the indices repeated some number of times
                model_opts.servers * self.repeats as usize * self.indices.len()
            } else {
                0
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
            Prop::sometimes("reach max text size", |model, state| {
                state
                    .actor_states
                    .iter()
                    .any(|s| state_has_max_text_size(s, &model.cfg))
            }),
            Prop::always("max text size is the max", |model, state| {
                state
                    .actor_states
                    .iter()
                    .all(|s| max_text_size_is_the_max(s, &model.cfg))
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

fn state_has_max_text_size(state: &Arc<ActorState>, cfg: &Config) -> bool {
    let max = cfg.max_text_size;
    if let GlobalActorState::Server(s) = &**state {
        s.length(TEXT_KEY) == max
    } else {
        false
    }
}

fn max_text_size_is_the_max(state: &Arc<ActorState>, cfg: &Config) -> bool {
    let max = cfg.max_text_size;
    if let GlobalActorState::Server(s) = &**state {
        s.length(TEXT_KEY) <= max
    } else {
        true
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub max_map_size: usize,
    pub max_list_size: usize,
    pub max_text_size: usize,
}
