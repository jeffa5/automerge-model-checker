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
    use amc::{application::server::SyncMethod, model::ModelOpts};

    use expect_test::expect;

    use super::*;

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

        amc_test::check_bfs(
            model_opts,
            todo_opts,
            expect![[r#"
                Done states=55117, unique=22075, max_depth=7
                Discovered "all apps have the right number of tasks" counterexample Path[6]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(6), dst: Id(1), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Timeout(Id(0), Server(Synchronise))
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg2FFLKgBTwAIAAAAAAAAAAABAQAAAAgBBAIGFRg0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MBMQljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                - Timeout(Id(1), Server(Synchronise))
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg7dZRG8BTwAIAAAAAAAAAAEBAQAAAAgBBAIGFRg0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MBMQljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                To explore this path try re-running with `explore 15452957160119689351/2382815500850056710/8111381211804313161/3626831637270537747/15919421438256274104/10561930214489353045/3860228754695809727`"#]],
        );
    }

    #[test]
    fn random_ids_partial_fix() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let todo_opts = TodoOptions {
            random_ids: true,
            initial_change: false,
        };

        amc_test::check_bfs(
            model_opts,
            todo_opts,
            expect![[r#"
                Done states=68430, unique=29147, max_depth=7
                Discovered "all apps have the right number of tasks" counterexample Path[6]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(6), dst: Id(1), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Timeout(Id(0), Server(Synchronise))
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kgwgf+QYBWAAIAAAAAAAAAAABAQAAAAgBBAIGFSE0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MKMzQ0MjI0MTQwNwljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                - Timeout(Id(1), Server(Synchronise))
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg7WbvTIBWAAIAAAAAAAAAAEBAQAAAAgBBAIGFSE0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MKMzU0MzE0NDU0NQljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                To explore this path try re-running with `explore 15452957160119689351/12382573908977832907/18145475013656775376/10648721681414430409/3627556555054181984/3972318222558024843/1755286072116325268`"#]],
        );
    }

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

        amc_test::check_bfs(
            model_opts,
            todo_opts,
            expect![[r#"
                Done states=55117, unique=22075, max_depth=7
                Discovered "all apps have the right number of tasks" counterexample Path[6]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(6), dst: Id(1), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Timeout(Id(0), Server(Synchronise))
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9KgxyNDm8BbwHG41V/TSmS0zF9OlKXsY+AGk2OJT4ZQc8yCtzbYlW+IQgAAAAAAAAAAAECAAABCAAAAAAAAAPnCAEEAgQVEjQBQgRWBFcBcAJ/AQIAfwECAn0BMQljb21wbGV0ZWQEdGV4dAN/AAIBfQABFmEDAA"] }) }
                - Timeout(Id(1), Server(Synchronise))
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg75sovgBbwHG41V/TSmS0zF9OlKXsY+AGk2OJT4ZQc8yCtzbYlW+IQgAAAAAAAAAAQECAAABCAAAAAAAAAPnCAEEAgQVEjQBQgRWBFcBcAJ/AQIAfwECAn0BMQljb21wbGV0ZWQEdGV4dAN/AAIBfQABFmEDAA"] }) }
                To explore this path try re-running with `explore 16996445121915788513/15048590559404185920/10822610882158225664/17700366276199367581/8763046345997887752/3447151373637425228/5534433402016584600`"#]],
        );
    }

    #[ignore]
    #[test]
    fn both_fixes() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let counter_opts = TodoOptions {
            random_ids: true,
            initial_change: true,
        };

        amc_test::check_bfs(
            model_opts,
            counter_opts,
            expect![[r#"
                {}
            "#]],
        );
    }
}
