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
pub struct TodoOptions {
    /// Whether to use random ids for todo creation.
    #[clap(long, global = true)]
    pub random_ids: bool,

    /// Whether to use generate an initial change.
    #[clap(long, global = true)]
    pub initial_change: bool,

    /// Update existing todos.
    #[clap(long, global = true)]
    pub updater: bool,

    /// Toggle todos.
    #[clap(long, global = true)]
    pub toggler: bool,
}

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    pub todo_options: TodoOptions,

    #[clap(flatten)]
    pub amc_args: amc::cli::RunArgs,
}

type AppHistory = Vec<(GlobalMsg<App>, GlobalMsg<App>)>;

#[derive(Debug, Clone)]
pub struct Config {}

impl amc::model::ModelBuilder for TodoOptions {
    type App = App;

    type Driver = Driver;

    type Config = Config;

    type History = AppHistory;

    fn application(&self, _server: usize, _config: &Config) -> Self::App {
        App {
            random_ids: self.random_ids,
            initial_change: self.initial_change,
        }
    }

    fn drivers(&self, _server: usize, _config: &Config) -> Vec<Self::Driver> {
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
        Config {}
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
            batch_synchronisation: false,
            restarts: false,
            in_sync_check: false,
            save_load_check: false,
            historical_check: false,
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
                Done states=203, unique=162, max_depth=5
                Discovered "all apps have the right number of tasks" counterexample Path[4]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(4), dst: Id(1), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg2FFLKgBTwAIAAAAAAAAAAABAQAAAAgBBAIGFRg0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MBMQljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg7dZRG8BTwAIAAAAAAAAAAEBAQAAAAgBBAIGFRg0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MBMQljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                To explore this path try re-running with `explore 3618709252656414281/3848895781740508824/3030994841380107975/8235390891822971838/6445604413388662187`"#]],
        );
    }

    #[test]
    fn random_ids_partial_fix() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            batch_synchronisation: false,
            restarts: false,
            in_sync_check: false,
            save_load_check: false,
            historical_check: false,
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
                Done states=227, unique=190, max_depth=5
                Discovered "all apps have the right number of tasks" counterexample Path[4]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(4), dst: Id(1), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kgwgf+QYBWAAIAAAAAAAAAAABAQAAAAgBBAIGFSE0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MKMzQ0MjI0MTQwNwljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg7WbvTIBWAAIAAAAAAAAAAEBAQAAAAgBBAIGFSE0AUIEVgVXAXACAAEDAAABfwECAnwFdG9kb3MKMzU0MzE0NDU0NQljb21wbGV0ZWQEdGV4dAQCAAIBAgB+ARZhBAA"] }) }
                To explore this path try re-running with `explore 3618709252656414281/13152949298909582684/18321986597509055663/17700926072447250057/1499795684066723353`"#]],
        );
    }

    #[test]
    fn intial_change_partial_fix() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            batch_synchronisation: false,
            restarts: false,
            in_sync_check: false,
            save_load_check: false,
            historical_check: false,
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
                Done states=203, unique=162, max_depth=5
                Discovered "all apps have the right number of tasks" counterexample Path[4]:
                - Deliver { src: Id(2), dst: Id(0), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(4), dst: Id(1), msg: ClientToServer(Input(CreateTodo("a"))) }
                - Deliver { src: Id(0), dst: Id(1), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9KgxyNDm8BbwHG41V/TSmS0zF9OlKXsY+AGk2OJT4ZQc8yCtzbYlW+IQgAAAAAAAAAAAECAAABCAAAAAAAAAPnCAEEAgQVEjQBQgRWBFcBcAJ/AQIAfwECAn0BMQljb21wbGV0ZWQEdGV4dAN/AAIBfQABFmEDAA"] }) }
                - Deliver { src: Id(1), dst: Id(0), msg: ServerToServer(SyncChangeRaw { missing_changes_bytes: ["hW9Kg75sovgBbwHG41V/TSmS0zF9OlKXsY+AGk2OJT4ZQc8yCtzbYlW+IQgAAAAAAAAAAQECAAABCAAAAAAAAAPnCAEEAgQVEjQBQgRWBFcBcAJ/AQIAfwECAn0BMQljb21wbGV0ZWQEdGV4dAN/AAIBfQABFmEDAA"] }) }
                To explore this path try re-running with `explore 7010877734742148362/10654691187990764641/8397507758268191045/3934490380097970375/4484010949238792495`"#]],
        );
    }

    #[ignore]
    #[test]
    fn both_fixes() {
        let model_opts = ModelOpts {
            servers: 2,
            sync_method: SyncMethod::Changes,
            batch_synchronisation: false,
            restarts: false,
            in_sync_check: false,
            save_load_check: false,
            historical_check: false,
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
