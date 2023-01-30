//! amc-todo shows how to implement the application side and client side with a concrete example

use crate::apphandle::App;
use crate::driver::AppInput;
use crate::driver::AppOutput;
use amc::global::GlobalActor;
use amc::global::GlobalActorState;
use amc::global::GlobalMsg;
use amc::properties::syncing_done;
use clap::Parser;
use driver::Driver;
use driver::DriverState;
use stateright::actor::ActorModel;
use stateright::actor::Envelope;
use stateright::Property;
use tracing::trace;

mod app;
mod apphandle;
mod driver;

#[derive(Parser, Debug)]
struct TodoOptions {
    /// Whether to use random ids for todo creation.
    #[clap(long, global = true)]
    random_ids: bool,

    /// Whether to use generate an initial change.
    #[clap(long, global = true)]
    initial_change: bool,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(flatten)]
    todo_options: TodoOptions,

    #[clap(flatten)]
    amc_args: amc::cli::RunArgs,
}

type AppHistory = Vec<(GlobalMsg<App>, GlobalMsg<App>)>;

#[derive(Debug, Clone)]
pub struct Config {
    pub app: App,
}

impl amc::model::ModelBuilder for TodoOptions {
    type App = App;

    type Driver = Driver;

    type Config = Config;

    type History = AppHistory;

    fn application(&self, _server: usize) -> Self::App {
        App {
            random_ids: self.random_ids,
            initial_change: self.initial_change,
        }
    }

    fn drivers(&self, _server: usize) -> Vec<Self::Driver> {
        vec![
            Driver {
                func: DriverState::Creater,
            },
            Driver {
                func: DriverState::Updater,
            },
            Driver {
                func: DriverState::Toggler,
            },
            Driver {
                func: DriverState::Deleter,
            },
        ]
    }

    fn config(&self, _model_opts: &amc::model::ModelOpts) -> Self::Config {
        Config {
            app: self.application(0),
        }
    }

    fn history(&self) -> Self::History {
        Vec::new()
    }

    fn properties(
        &self,
    ) -> Vec<
        stateright::Property<
            ActorModel<GlobalActor<Self::App, Self::Driver>, Self::Config, Self::History>,
        >,
    > {
        type Model = stateright::actor::ActorModel<GlobalActor<App, Driver>, Config, AppHistory>;
        type Prop = Property<Model>;
        vec![Prop::always(
            "all apps have the right number of tasks",
            |_model, state| {
                if !syncing_done(state) {
                    return true;
                }

                let mut expected_task_count = 0;

                for (i, o) in &state.history {
                    match (i.input(), o.output()) {
                        (Some(req), Some(res)) => match (req, res) {
                            (AppInput::CreateTodo(_), AppOutput::CreateTodo(_)) => {
                                expected_task_count += 1;
                            }
                            (AppInput::ToggleActive(_), AppOutput::ToggleActive(_)) => {}
                            (AppInput::DeleteTodo(_), AppOutput::DeleteTodo(was_present)) => {
                                if *was_present {
                                    expected_task_count -= 1;
                                }
                            }
                            (AppInput::Update(_id, _text), AppOutput::Update(_success)) => {}
                            (AppInput::ListTodos, driver::AppOutput::ListTodos(_ids)) => {}
                            (a, b) => {
                                unreachable!("{:?}, {:?}", a, b)
                            }
                        },
                        (Some(_), None) | (None, Some(_)) | (None, None) => unreachable!(),
                    }
                }

                state.actor_states.iter().all(|s| {
                    if let GlobalActorState::Server(s) = &**s {
                        s.num_todos() == expected_task_count
                    } else {
                        true
                    }
                })
            },
        )]
    }

    fn record_input(
        &self,
    ) -> fn(cfg: &Config, history: &AppHistory, Envelope<&GlobalMsg<App>>) -> Option<AppHistory>
    {
        |_, h, m| {
            if m.msg.input().is_some() {
                let mut nh = h.clone();
                trace!(envelope=?m, "Recording input");
                nh.push((m.msg.clone(), m.msg.clone()));
                Some(nh)
            } else {
                None
            }
        }
    }

    fn record_output(
        &self,
    ) -> fn(cfg: &Config, history: &AppHistory, Envelope<&GlobalMsg<App>>) -> Option<AppHistory>
    {
        |_, h, m| {
            if m.msg.output().is_some() {
                let mut nh = h.clone();
                trace!(envelope=?m, "Recording output");
                nh.last_mut().unwrap().1 = m.msg.clone();
                Some(nh)
            } else {
                None
            }
        }
    }
}

fn main() {
    let Args {
        todo_options,
        amc_args,
    } = Args::parse();
    amc_args.run(todo_options);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use amc::{application::server::SyncMethod, model::ModelOpts};
    use stateright::{Checker, Model};

    use expect_test::{expect, Expect};

    use super::*;

    fn check(model_opts: ModelOpts, todo_opts: TodoOptions, expected: Expect) {
        let model = model_opts.to_model(&todo_opts);
        let checker = model.checker().spawn_bfs().join();

        let discoveries: BTreeMap<_, _> = checker
            .discoveries()
            .into_iter()
            .map(|(n, p)| (n, p.into_actions()))
            .collect();

        expected.assert_debug_eq(&discoveries);
    }

    #[test]
    fn fully_broken() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let todo_opts = TodoOptions {
            random_ids: false,
            initial_change: false,
        };

        check(
            model_opts,
            todo_opts,
            expect![[r#"
                {
                    "all apps have the right number of tasks": [
                        Deliver {
                            src: Id(2),
                            dst: Id(0),
                            msg: ClientToServer(
                                Input(
                                    CreateTodo(
                                        "a",
                                    ),
                                ),
                            ),
                        },
                        Deliver {
                            src: Id(6),
                            dst: Id(1),
                            msg: ClientToServer(
                                Input(
                                    CreateTodo(
                                        "a",
                                    ),
                                ),
                            ),
                        },
                        Timeout(
                            Id(0),
                        ),
                        Deliver {
                            src: Id(0),
                            dst: Id(1),
                            msg: ServerToServer(
                                SyncChangeRaw {
                                    missing_changes_bytes: [
                                        "hW9Kg3sU3+4BRgAIAAAAAAAAAAABAQAAAAgBBAIEFRI0AUIEVgRXAXACAAECAAABAgF9ATEJY29tcGxldGVkBHRleHQDfwACAX0AARZhAwA",
                                    ],
                                },
                            ),
                        },
                        Timeout(
                            Id(1),
                        ),
                        Deliver {
                            src: Id(1),
                            dst: Id(0),
                            msg: ServerToServer(
                                SyncChangeRaw {
                                    missing_changes_bytes: [
                                        "hW9Kg1W34z4BRgAIAAAAAAAAAAEBAQAAAAgBBAIEFRI0AUIEVgRXAXACAAECAAABAgF9ATEJY29tcGxldGVkBHRleHQDfwACAX0AARZhAwA",
                                    ],
                                },
                            ),
                        },
                    ],
                }
            "#]],
        );
    }

    // TODO: enable this once we can get it quick enough
    // #[test]
    // fn random_ids_partial_fix() {
    //     let model_opts = ModelOpts {
    //         servers: 2,
    //         sync_method: SyncMethod::Changes,
    //         in_sync_check: false,
    //         save_load_check: false,
    //         error_free_check: false,
    //     };
    //     let todo_opts = TodoOptions {
    //         random_ids: true,
    //         initial_change: false,
    //     };
    //
    //     check(
    //         model_opts,
    //         todo_opts,
    //         expect![[r#"
    //             {
    //                 "correct value": [
    //                     Deliver {
    //                         src: Id(2),
    //                         dst: Id(0),
    //                         msg: ClientToServer(
    //                             Input(
    //                                 Increment,
    //                             ),
    //                         ),
    //                     },
    //                     Deliver {
    //                         src: Id(4),
    //                         dst: Id(1),
    //                         msg: ClientToServer(
    //                             Input(
    //                                 Increment,
    //                             ),
    //                         ),
    //                     },
    //                     Timeout(
    //                         Id(0),
    //                     ),
    //                     Deliver {
    //                         src: Id(0),
    //                         dst: Id(1),
    //                         msg: ServerToServer(
    //                             SyncChangeRaw {
    //                                 missing_changes_bytes: [
    //                                     "hW9Kg8uC6w0BOQAIAAAAAAAAAAABAQAAAAgVCTQBQgNWA1cCcANxAnMCAgdjb3VudGVyAn4BBX4YFAABfgABfwB/AQ",
    //                                 ],
    //                             },
    //                         ),
    //                     },
    //                     Timeout(
    //                         Id(1),
    //                     ),
    //                     Deliver {
    //                         src: Id(1),
    //                         dst: Id(0),
    //                         msg: ServerToServer(
    //                             SyncChangeRaw {
    //                                 missing_changes_bytes: [
    //                                     "hW9Kg5SFxa4BOQAIAAAAAAAAAAEBAQAAAAgVCTQBQgNWA1cCcANxAnMCAgdjb3VudGVyAn4BBX4YFAABfgABfwB/AQ",
    //                                 ],
    //                             },
    //                         ),
    //                     },
    //                 ],
    //             }
    //         "#]],
    //     );
    // }

    #[test]
    fn intial_change_partial_fix() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let todo_opts = TodoOptions {
            random_ids: false,
            initial_change: true,
        };

        check(
            model_opts,
            todo_opts,
            expect![[r#"
                {
                    "all apps have the right number of tasks": [
                        Deliver {
                            src: Id(2),
                            dst: Id(0),
                            msg: ClientToServer(
                                Input(
                                    CreateTodo(
                                        "a",
                                    ),
                                ),
                            ),
                        },
                        Deliver {
                            src: Id(6),
                            dst: Id(1),
                            msg: ClientToServer(
                                Input(
                                    CreateTodo(
                                        "a",
                                    ),
                                ),
                            ),
                        },
                        Timeout(
                            Id(0),
                        ),
                        Deliver {
                            src: Id(0),
                            dst: Id(1),
                            msg: ServerToServer(
                                SyncChangeRaw {
                                    missing_changes_bytes: [
                                        "hW9KgzEuzhgBZgGihieAmuu1Im/vM2WKUP9eOl19e4lwZghwlxtNesBrSggAAAAAAAAAAAEBAAAACAEEAgQVEjQBQgRWBFcBcAIAAQIAAAECAX0BMQljb21wbGV0ZWQEdGV4dAN/AAIBfQABFmEDAA",
                                    ],
                                },
                            ),
                        },
                        Timeout(
                            Id(1),
                        ),
                        Deliver {
                            src: Id(1),
                            dst: Id(0),
                            msg: ServerToServer(
                                SyncChangeRaw {
                                    missing_changes_bytes: [
                                        "hW9Kg3oqwacBZgGihieAmuu1Im/vM2WKUP9eOl19e4lwZghwlxtNesBrSggAAAAAAAAAAQEBAAAACAEEAgQVEjQBQgRWBFcBcAIAAQIAAAECAX0BMQljb21wbGV0ZWQEdGV4dAN/AAIBfQABFmEDAA",
                                    ],
                                },
                            ),
                        },
                    ],
                }
            "#]],
        );
    }

    // TODO: enable this when it gets quick enough
    // #[test]
    // fn both_fixes() {
    //     let model_opts = ModelOpts {
    //         servers: 2,
    //         sync_method: SyncMethod::Changes,
    //         in_sync_check: false,
    //         save_load_check: false,
    //         error_free_check: false,
    //     };
    //     let counter_opts = TodoOptions {
    //         random_ids: true,
    //         initial_change: true,
    //     };
    //
    //     check(
    //         model_opts,
    //         counter_opts,
    //         expect![[r#"
    //             {}
    //         "#]],
    //     );
    // }
}
