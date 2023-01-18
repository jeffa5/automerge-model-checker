use std::sync::Arc;

use amc::app::{LIST_KEY, MAP_KEY};
use amc::client;
use amc::client::Client;

use amc::trigger::Trigger;
use amc::ObjectType;
use amc_cli::Cli;
use amc_core::GlobalActorState;
use clap::Parser;
use stateright::actor::Id;
use stateright::Property;

#[derive(Parser, Debug)]
struct Opts {
    #[clap(subcommand)]
    command: amc_cli::SubCmd,

    #[clap(long, short, global = true, default_value = "2")]
    servers: usize,

    #[clap(long, global = true, default_value = "changes")]
    sync_method: amc_core::SyncMethod,

    // What object type to check.
    #[clap(long, global = true, default_value = "map")]
    object_type: amc::ObjectType,

    #[clap(long, default_value = "8080")]
    port: u16,
}

type ActorState = GlobalActorState<Trigger, Client>;

const INSERT_REQUEST_COUNT: usize = 2;

impl amc_cli::Cli for Opts {
    type App = Client;

    type Client = Trigger;

    type Config = Config;

    fn application(&mut self, _i: usize) -> Self::App {
        let c = Client {
            map_single_putter: client::MapSinglePutter,
            list_start_putter: client::ListPutter,
            map_single_deleter: client::MapSingleDeleter,
            list_deleter: client::ListDeleter,
            list_inserter: client::ListInserter,
        };
        println!("Adding application {:?}", c);
        c
    }

    fn clients(&mut self, server: usize) -> Vec<Self::Client> {
        let server = Id::from(server);
        let triggers = match self.object_type {
            ObjectType::Map => {
                vec![
                    Trigger {
                        func: amc::trigger::TriggerState::MapSinglePut {
                            request_count: 2,
                            key: "key".to_owned(),
                        },
                        server,
                    },
                    Trigger {
                        func: amc::trigger::TriggerState::MapSingleDelete {
                            request_count: 2,
                            key: "key".to_owned(),
                        },
                        server,
                    },
                ]
            }
            ObjectType::List => {
                vec![
                    Trigger {
                        func: amc::trigger::TriggerState::ListStartPut {
                            request_count: 2,
                            index: 0,
                        },
                        server,
                    },
                    Trigger {
                        func: amc::trigger::TriggerState::ListDelete {
                            request_count: 2,
                            index: 0,
                        },
                        server,
                    },
                    Trigger {
                        func: amc::trigger::TriggerState::ListInsert {
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

    fn config(&self) -> Self::Config {
        let c = Config {
            max_map_size: 1,
            max_list_size: if self.object_type == ObjectType::Map {
                0
            } else {
                self.servers * INSERT_REQUEST_COUNT
            },
        };
        println!("Built config {:?}", c);
        c
    }

    fn properties(
        &self,
    ) -> Vec<
        stateright::Property<
            stateright::actor::ActorModel<
                amc_core::GlobalActor<Self::Client, Self::App>,
                Self::Config,
            >,
        >,
    > {
        type Model = stateright::actor::ActorModel<amc_core::GlobalActor<Trigger, Client>, Config>;
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

    fn servers(&self) -> usize {
        self.servers
    }

    fn sync_method(&self) -> amc_core::SyncMethod {
        self.sync_method
    }

    fn command(&self) -> amc_cli::SubCmd {
        self.command
    }

    fn port(&self) -> u16 {
        self.port
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
    Opts::parse().run();
}
