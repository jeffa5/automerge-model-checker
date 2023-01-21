use std::borrow::Cow;

/// amc-todo shows how to implement the application side and client side with a concrete example
///
use crate::apphandle::App;
use crate::trigger::AppInput;
use crate::trigger::AppOutput;
use amc::application::Application;
use amc::global::GlobalActor;
use amc::global::GlobalActorState;
use amc::global::GlobalMsg;
use amc::properties::syncing_done;
use amc::driver::ApplicationMsg;
use clap::Parser;
use stateright::actor::ActorModel;
use stateright::actor::Envelope;
use stateright::actor::Id;
use stateright::Property;
use trigger::DriverState;
use trigger::Driver;

mod app;
mod apphandle;
mod trigger;

#[derive(Parser, Debug)]
struct C {
    /// Whether to use random ids for todo creation.
    #[clap(long, global = true)]
    random_ids: bool,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(flatten)]
    c: C,

    #[clap(flatten)]
    lib_opts: amc_cli::Opts,
}

type AppHistory = Vec<(GlobalMsg<App>, GlobalMsg<App>)>;

pub struct Config {
    pub app: App,
}

impl amc_cli::ModelBuilder for C {
    type App = App;

    type Driver = Driver;

    type Config = Config;

    type History = AppHistory;

    fn application(&self, _server: usize) -> Self::App {
        App {
            random_ids: self.random_ids,
        }
    }

    fn drivers(&self, server: usize) -> Vec<Self::Driver> {
        let i = stateright::actor::Id::from(server);
        vec![
            Driver {
                func: DriverState::Creater,
                server: i,
            },
            Driver {
                func: DriverState::Updater,
                server: i,
            },
            Driver {
                func: DriverState::Toggler,
                server: i,
            },
            Driver {
                func: DriverState::Deleter,
                server: i,
            },
        ]
    }

    fn config(&self, _cli_opts: &amc_cli::Opts) -> Self::Config {
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
            ActorModel<GlobalActor<Self::Driver, Self::App>, Self::Config, Self::History>,
        >,
    > {
        type Model =
            stateright::actor::ActorModel<GlobalActor<Driver, App>, Config, AppHistory>;
        type Prop = Property<Model>;
        vec![Prop::always(
            "all apps have the right number of tasks",
            |model, state| {
                if !syncing_done(state) {
                    return true;
                }

                let cf = &model.cfg.app;
                let mut single_app = Cow::Owned(cf.init(Id::from(0)));

                for m in &state.history {
                    match m {
                        (GlobalMsg::ServerToServer(_), _) => unreachable!(),
                        (GlobalMsg::ClientToServer(_), GlobalMsg::ServerToServer(_)) => {
                            unreachable!()
                        }
                        (
                            GlobalMsg::ClientToServer(ApplicationMsg::Input(req)),
                            GlobalMsg::ClientToServer(ApplicationMsg::Output(res)),
                        ) => match (req, res) {
                            (AppInput::CreateTodo(_), AppOutput::CreateTodo(_)) => {
                                cf.execute(&mut single_app, req.clone());
                            }
                            (AppInput::ToggleActive(_), AppOutput::ToggleActive(_)) => {
                                cf.execute(&mut single_app, req.clone());
                            }
                            (
                                AppInput::DeleteTodo(_),
                                AppOutput::DeleteTodo(was_present),
                            ) => {
                                if *was_present {
                                    cf.execute(&mut single_app, req.clone());
                                }
                            }
                            (AppInput::Update(_id, _text), AppOutput::Update(success)) => {
                                if *success {
                                    cf.execute(&mut single_app, req.clone());
                                }
                            }
                            (AppInput::ListTodos, trigger::AppOutput::ListTodos(_ids)) => {}
                            (a, b) => {
                                unreachable!("{:?}, {:?}", a, b)
                            }
                        },
                        (GlobalMsg::ClientToServer(ApplicationMsg::Output(_)), _) => {}
                        (
                            GlobalMsg::ClientToServer(ApplicationMsg::Input(_)),
                            GlobalMsg::ClientToServer(ApplicationMsg::Input(_)),
                        ) => {}
                    }
                }

                state.actor_states.iter().all(|s| {
                    if let GlobalActorState::Server(s) = &**s {
                        s.num_todos() == single_app.num_todos()
                    } else {
                        true
                    }
                })
            },
        )]
    }

    fn record_request(
        &self,
    ) -> fn(cfg: &Config, history: &AppHistory, Envelope<&GlobalMsg<App>>) -> Option<AppHistory>
    {
        |_, h, m| {
            if matches!(m.msg, GlobalMsg::ClientToServer(ApplicationMsg::Input(_))) {
                let mut nh = h.clone();
                nh.push((m.msg.clone(), m.msg.clone()));
                Some(nh)
            } else {
                None
            }
        }
    }

    fn record_response(
        &self,
    ) -> fn(cfg: &Config, history: &AppHistory, Envelope<&GlobalMsg<App>>) -> Option<AppHistory>
    {
        |_, h, m| {
            if matches!(m.msg, GlobalMsg::ClientToServer(ApplicationMsg::Output(_))) {
                let mut nh = h.clone();
                nh.last_mut().unwrap().1 = m.msg.clone();
                Some(nh)
            } else {
                None
            }
        }
    }
}

fn main() {
    let Opts {
        c: c_opts,
        lib_opts,
    } = Opts::parse();
    lib_opts.run(c_opts);
}
