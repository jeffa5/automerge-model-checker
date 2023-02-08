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

    /// Update existing todos.
    #[clap(long, global = true)]
    updater: bool,

    /// Toggle todos.
    #[clap(long, global = true)]
    toggler: bool,
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
        let mut drivers = vec![
            Driver {
                func: DriverState::Creater,
            },
            Driver {
                func: DriverState::Deleter,
            },
        ];
        if self.updater {
            drivers.push(Driver {
                func: DriverState::Updater,
            });
        }
        if self.toggler {
            drivers.push(Driver {
                func: DriverState::Toggler,
            });
        }
        drivers
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

                let mut present_tasks = Vec::new();

                for (i, o) in &state.history {
                    match (i.input(), o.output()) {
                        (Some(req), Some(res)) => match (req, res) {
                            (AppInput::CreateTodo(_), AppOutput::CreateTodo(id)) => {
                                present_tasks.push(id);
                            }
                            (AppInput::ToggleActive(_), AppOutput::ToggleActive(_)) => {}
                            (AppInput::DeleteTodo(id), AppOutput::DeleteTodo(_)) => {
                                if let Some(index_to_remove) =
                                    present_tasks.iter().position(|&x| x == id)
                                {
                                    present_tasks.swap_remove(index_to_remove);
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

                let expected_task_count = present_tasks.len();
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
            restarts: false,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let todo_opts = TodoOptions {
            random_ids: false,
            initial_change: false,
            updater: false,
            toggler: false,
        };

        amc_test::check_bfs(
            model_opts,
            todo_opts,
            expect![[r#"
                Done states=1010, unique=651, max_depth=7
                Discovered "all apps have the right number of tasks" counterexample Path[6]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(4), dst: Id(1), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Timeout(Id(0), Server(Synchronise))
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg2FFLKgBTwAIAAAAAAAAAAABAQAAAAgBBAIGFRg0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MBMQljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                - Timeout(Id(1), Server(Synchronise))
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg7dZRG8BTwAIAAAAAAAAAAEBAQAAAAgBBAIGFRg0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MBMQljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                To explore this path try re-running with `explore 15482871843547519777/8203200380951402305/8829712127819938698/9203449192000354338/1513831609101728924/10211774821601995043/3348636454154444996`"#]],
        );
    }

    #[test]
    fn random_ids_partial_fix() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            restarts: false,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let todo_opts = TodoOptions {
            random_ids: true,
            initial_change: false,
            updater: false,
            toggler: false,
        };

        amc_test::check_bfs(
            model_opts,
            todo_opts,
            expect![[r#"
                Done states=1311, unique=856, max_depth=7
                Discovered "all apps have the right number of tasks" counterexample Path[6]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(4), dst: Id(1), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Timeout(Id(0), Server(Synchronise))
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kgwgf+QYBWAAIAAAAAAAAAAABAQAAAAgBBAIGFSE0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MKMzQ0MjI0MTQwNwljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                - Timeout(Id(1), Server(Synchronise))
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg7WbvTIBWAAIAAAAAAAAAAEBAQAAAAgBBAIGFSE0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MKMzU0MzE0NDU0NQljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                To explore this path try re-running with `explore 15482871843547519777/18165317894337760192/694225343320960706/7251433598226590557/3823086620333083307/10110955772386490526/18101974323185263970`"#]],
        );
    }

    #[test]
    fn intial_change_partial_fix() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            restarts: false,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let todo_opts = TodoOptions {
            random_ids: false,
            initial_change: true,
            updater: false,
            toggler: false,
        };

        amc_test::check_bfs(
            model_opts,
            todo_opts,
            expect![[r#"
                Done states=1010, unique=651, max_depth=7
                Discovered "all apps have the right number of tasks" counterexample Path[6]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(4), dst: Id(1), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Timeout(Id(0), Server(Synchronise))
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9KgxyNDm8BbwHG41V/TSmS0zF9OlKXsY+AGk2OJT4ZQc8yCtzbYlW+IQgAAAAAAAAAAAECAAABCAAAAAAAAAPnCAEEAgQVEjQBQgRWBFcBcAJ/AQIAfwECAn0BMQljb21wbGV0ZWQEdGV4dAN/AAIBfQABFmEDAA"] }) }
                - Timeout(Id(1), Server(Synchronise))
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg75sovgBbwHG41V/TSmS0zF9OlKXsY+AGk2OJT4ZQc8yCtzbYlW+IQgAAAAAAAAAAQECAAABCAAAAAAAAAPnCAEEAgQVEjQBQgRWBFcBcAJ/AQIAfwECAn0BMQljb21wbGV0ZWQEdGV4dAN/AAIBfQABFmEDAA"] }) }
                To explore this path try re-running with `explore 17492868438200066151/2505832214583032789/1929226361012236889/16042632906015736619/15462423237410012875/16262740734369261783/10226122137356237458`"#]],
        );
    }

    #[ignore]
    #[test]
    fn both_fixes() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            restarts: false,
            in_sync_check: false,
            save_load_check: false,
            error_free_check: false,
        };
        let counter_opts = TodoOptions {
            random_ids: true,
            initial_change: true,
            updater: false,
            toggler: false,
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
