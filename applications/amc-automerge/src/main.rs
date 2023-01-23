use std::sync::Arc;

use crate::app::{LIST_KEY, MAP_KEY};
use crate::client::App;
use amc::global::{GlobalActor, GlobalActorState};

use crate::driver::Driver;
use clap::Parser;
use stateright::Property;

mod app;
mod client;
mod driver;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ValueEnum)]
enum ObjectType {
    Map,
    List,
}

#[derive(Parser, Debug)]
struct AutomergeOpts {
    // What object type to check.
    #[clap(long, global = true, default_value = "map")]
    object_type: crate::ObjectType,
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
            list_start_putter: client::ListPutter,
            map_single_deleter: client::MapSingleDeleter,
            list_deleter: client::ListDeleter,
            list_inserter: client::ListInserter,
        };
        println!("Adding application {:?}", c);
        c
    }

    fn drivers(&self, _server: usize) -> Vec<Self::Driver> {
        let drivers = match self.object_type {
            ObjectType::Map => {
                vec![
                    Driver {
                        func: crate::driver::DriverState::MapSinglePut {
                            request_count: 2,
                            key: "key".to_owned(),
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
                        },
                    },
                ]
            }
        };
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

#[derive(Debug)]
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
