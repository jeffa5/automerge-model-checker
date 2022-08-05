use crate::apphandle::AppHandle;
use crate::trigger::Trigger;
use crate::trigger::TriggerMsg;
use crate::trigger::TriggerResponse;
use crate::trigger::TriggerState;
use amc_core::model;
use amc_core::Application;
use amc_core::ClientMsg;
use amc_core::GlobalActor;
use amc_core::GlobalActorState;
use amc_core::GlobalMsg;
use amc_core::Server;
use amc_core::SyncMethod;
use stateright::actor::Id;
use stateright::actor::Network;
use stateright::actor::{model_peers, ActorModel};
use stateright::Expectation;
use std::borrow::Cow;

pub type Actor = GlobalActor<Trigger, AppHandle>;
pub type History = Vec<(GlobalMsg<AppHandle>, GlobalMsg<AppHandle>)>;

pub struct Config {
    app: AppHandle,
}

pub struct Builder {
    pub servers: usize,
    pub sync_method: SyncMethod,
    pub message_acks: bool,
    pub app: AppHandle,
}

impl Builder {
    pub fn into_actor_model(self) -> ActorModel<GlobalActor<Trigger, AppHandle>, Config, History> {
        let config = Config {
            app: self.app.clone(),
        };
        let mut model = ActorModel::new(config, Vec::new());
        for i in 0..self.servers {
            model = model.actor(GlobalActor::Server(Server {
                peers: model_peers(i, self.servers),
                sync_method: self.sync_method,
                message_acks: self.message_acks,
                app: self.app.clone(),
            }))
        }

        for i in 0..self.servers {
            let i = stateright::actor::Id::from(i);
            model = model.actor(GlobalActor::Trigger(Trigger {
                func: TriggerState::Creater,
                server: i,
            }));
            model = model.actor(GlobalActor::Trigger(Trigger {
                func: TriggerState::Updater,
                server: i,
            }));
            model = model.actor(GlobalActor::Trigger(Trigger {
                func: TriggerState::Toggler,
                server: i,
            }));
            model = model.actor(GlobalActor::Trigger(Trigger {
                func: TriggerState::Deleter(1),
                server: i,
            }));
        }

        model = model::with_default_properties(model);
        model
            .property(
                Expectation::Always,
                "all apps have the right number of tasks",
                |model, state| {
                    if !model::syncing_done(state) {
                        return true;
                    }

                    let cf = &model.cfg.app;
                    let mut single_app = Cow::Owned(cf.init(Id::from(0)));

                    for m in &state.history {
                        match m {
                            (GlobalMsg::Internal(_), _) => unreachable!(),
                            (GlobalMsg::External(_), GlobalMsg::Internal(_)) => unreachable!(),
                            (
                                GlobalMsg::External(ClientMsg::Request(req)),
                                GlobalMsg::External(ClientMsg::Response(res)),
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
                                (
                                    TriggerMsg::Update(_id, _text),
                                    TriggerResponse::Update(success),
                                ) => {
                                    if *success {
                                        cf.execute(&mut single_app, req.clone());
                                    }
                                }
                                (TriggerMsg::ListTodos, TriggerResponse::ListTodos(_ids)) => {}
                                (a, b) => {
                                    unreachable!("{:?}, {:?}", a, b)
                                }
                            },
                            (GlobalMsg::External(ClientMsg::Response(_)), _) => {}
                            (
                                GlobalMsg::External(ClientMsg::Request(_)),
                                GlobalMsg::External(ClientMsg::Request(_)),
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
            )
            .record_msg_in(|_, h, m| {
                // only record external messages
                if matches!(m.msg, GlobalMsg::External(_)) {
                    let mut nh = h.clone();
                    nh.push((m.msg.clone(), m.msg.clone()));
                    Some(nh)
                } else {
                    None
                }
            })
            .record_msg_out(|_, h, m| {
                // only record external messages
                if matches!(m.msg, GlobalMsg::External(ClientMsg::Response(_))) {
                    let mut nh = h.clone();
                    nh.last_mut().unwrap().1 = m.msg.clone();
                    Some(nh)
                } else {
                    None
                }
            })
            .init_network(Network::new_ordered(vec![]))
    }
}
