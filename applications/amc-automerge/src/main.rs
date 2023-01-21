use std::sync::Arc;

use amc::global::{GlobalActor, GlobalActorState};
use amc_automerge::app::{LIST_KEY, MAP_KEY};
use amc_automerge::client;
use amc_automerge::client::App;

use amc_automerge::trigger::Driver;
use amc_automerge::ObjectType;
use clap::Parser;
use stateright::actor::Id;
use stateright::Property;

#[derive(Parser, Debug)]
struct C {
    // What object type to check.
    #[clap(long, global = true, default_value = "map")]
    object_type: amc_automerge::ObjectType,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(flatten)]
    c: C,

    #[clap(flatten)]
    lib_opts: amc_cli::Opts,
}

type ActorState = GlobalActorState<Driver, App>;

const INSERT_REQUEST_COUNT: usize = 2;

impl amc_cli::ModelBuilder for C {
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

    fn drivers(&self, server: usize) -> Vec<Self::Driver> {
        let server = Id::from(server);
        let triggers = match self.object_type {
            ObjectType::Map => {
                vec![
                    Driver {
                        func: amc_automerge::trigger::DriverState::MapSinglePut {
                            request_count: 2,
                            key: "key".to_owned(),
                        },
                        server,
                    },
                    Driver {
                        func: amc_automerge::trigger::DriverState::MapSingleDelete {
                            request_count: 2,
                            key: "key".to_owned(),
                        },
                        server,
                    },
                ]
            }
            ObjectType::List => {
                vec![
                    Driver {
                        func: amc_automerge::trigger::DriverState::ListStartPut {
                            request_count: 2,
                            index: 0,
                        },
                        server,
                    },
                    Driver {
                        func: amc_automerge::trigger::DriverState::ListDelete {
                            request_count: 2,
                            index: 0,
                        },
                        server,
                    },
                    Driver {
                        func: amc_automerge::trigger::DriverState::ListInsert {
                            request_count: INSERT_REQUEST_COUNT,
                            index: 0,
                        },
                        server,
                    },
                ]
            }
        };
        println!("Adding clients {:?}", triggers);
        triggers
    }

    fn config(&self, cli_opts: &amc_cli::Opts) -> Self::Config {
        let c = Config {
            max_map_size: 1,
            max_list_size: if self.object_type == ObjectType::Map {
                0
            } else {
                cli_opts.servers * INSERT_REQUEST_COUNT
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
    let Opts {
        c: c_opts,
        lib_opts,
    } = Opts::parse();
    lib_opts.run(c_opts);
}
