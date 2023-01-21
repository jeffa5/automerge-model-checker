use std::borrow::Cow;

/// amc-todo shows how to implement the application side and client side with a concrete example
///
use crate::apphandle::AppHandle;
use crate::trigger::TriggerMsg;
use crate::trigger::TriggerResponse;
use amc::model::syncing_done;
use amc::Application;
use amc::ClientMsg;
use amc::GlobalActorState;
use amc::GlobalMsg;
use clap::Parser;
use stateright::actor::ActorModel;
use stateright::actor::Envelope;
use stateright::actor::Id;
use stateright::Property;
use trigger::Trigger;
use trigger::TriggerState;

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

type AppHistory = Vec<(GlobalMsg<AppHandle>, GlobalMsg<AppHandle>)>;

pub struct Config {
    pub app: AppHandle,
}

impl amc_cli::Cli for C {
    type App = AppHandle;

    type Client = Trigger;

    type Config = Config;

    type History = AppHistory;

    fn application(&self, _server: usize) -> Self::App {
        AppHandle {
            random_ids: self.random_ids,
        }
    }

    fn clients(&self, server: usize) -> Vec<Self::Client> {
        let i = stateright::actor::Id::from(server);
        vec![
            Trigger {
                func: TriggerState::Creater,
                server: i,
            },
            Trigger {
                func: TriggerState::Updater,
                server: i,
            },
            Trigger {
                func: TriggerState::Toggler,
                server: i,
            },
            Trigger {
                func: TriggerState::Deleter,
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
            ActorModel<amc::GlobalActor<Self::Client, Self::App>, Self::Config, Self::History>,
        >,
    > {
        type Model = stateright::actor::ActorModel<
            amc::GlobalActor<Trigger, AppHandle>,
            Config,
            AppHistory,
        >;
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
                            GlobalMsg::ClientToServer(ClientMsg::Request(req)),
                            GlobalMsg::ClientToServer(ClientMsg::Response(res)),
                        ) => match (req, res) {
                            (TriggerMsg::CreateTodo(_), TriggerResponse::CreateTodo(_)) => {
                                cf.execute(&mut single_app, req.clone());
                            }
                            (TriggerMsg::ToggleActive(_), TriggerResponse::ToggleActive(_)) => {
                                cf.execute(&mut single_app, req.clone());
                            }
                            (
                                TriggerMsg::DeleteTodo(_),
                                TriggerResponse::DeleteTodo(was_present),
                            ) => {
                                if *was_present {
                                    cf.execute(&mut single_app, req.clone());
                                }
                            }
                            (TriggerMsg::Update(_id, _text), TriggerResponse::Update(success)) => {
                                if *success {
                                    cf.execute(&mut single_app, req.clone());
                                }
                            }
                            (TriggerMsg::ListTodos, trigger::TriggerResponse::ListTodos(_ids)) => {}
                            (a, b) => {
                                unreachable!("{:?}, {:?}", a, b)
                            }
                        },
                        (GlobalMsg::ClientToServer(ClientMsg::Response(_)), _) => {}
                        (
                            GlobalMsg::ClientToServer(ClientMsg::Request(_)),
                            GlobalMsg::ClientToServer(ClientMsg::Request(_)),
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
    ) -> fn(cfg: &Config, history: &AppHistory, Envelope<&GlobalMsg<AppHandle>>) -> Option<AppHistory>
    {
        |_, h, m| {
            if matches!(m.msg, GlobalMsg::ClientToServer(ClientMsg::Request(_))) {
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
    ) -> fn(cfg: &Config, history: &AppHistory, Envelope<&GlobalMsg<AppHandle>>) -> Option<AppHistory>
    {
        |_, h, m| {
            if matches!(m.msg, GlobalMsg::ClientToServer(ClientMsg::Response(_))) {
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
