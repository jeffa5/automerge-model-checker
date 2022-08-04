use crate::client::Client;
use crate::trigger::Trigger;
use crate::trigger::TriggerMsg;
use crate::trigger::TriggerResponse;
use crate::trigger::TriggerState;
use amc_core::Application;
use amc_core::ClientMsg;
use amc_core::DerefDocument;
use amc_core::GlobalActor;
use amc_core::GlobalActorState;
use amc_core::GlobalMsg;
use amc_core::Server;
use amc_core::ServerMsg;
use amc_core::SyncMethod;
use automerge::Automerge;
use automerge::ROOT;
use stateright::actor::Id;
use stateright::actor::{model_peers, ActorModel};
use stateright::actor::{ActorModelState, Network};
use stateright::Expectation;
use std::borrow::Cow;
use std::sync::Arc;

pub type State = GlobalActorState<Trigger, Client>;
pub type Actor = GlobalActor<Trigger, Client>;
pub type History = Vec<(GlobalMsg<Client>, GlobalMsg<Client>)>;

pub struct Config {
    client_function: Client,
}

pub struct Builder {
    pub servers: usize,
    pub sync_method: SyncMethod,
    pub message_acks: bool,
    pub client_function: Client,
}

impl Builder {
    pub fn into_actor_model(self) -> ActorModel<GlobalActor<Trigger, Client>, Config, History> {
        let config = Config {
            client_function: self.client_function.clone(),
        };
        let mut model = ActorModel::new(config, Vec::new());
        for i in 0..self.servers {
            model = model.actor(GlobalActor::Server(Server {
                peers: model_peers(i, self.servers),
                sync_method: self.sync_method,
                message_acks: self.message_acks,
                client_function: self.client_function.clone(),
            }))
        }

        for i in 0..self.servers {
            let i = stateright::actor::Id::from(i);
            model = model.actor(GlobalActor::Trigger(Trigger {
                func: TriggerState::Creater,
                server: i,
            }));
            model = model.actor(GlobalActor::Trigger(Trigger {
                func: TriggerState::Toggler(1),
                server: i,
            }));
            model = model.actor(GlobalActor::Trigger(Trigger {
                func: TriggerState::Deleter(1),
                server: i,
            }));
        }

        model
            .property(
                stateright::Expectation::Eventually,
                "all actors have the same value for all keys",
                |_, state| all_same_state(&state.actor_states),
            )
            .property(
                stateright::Expectation::Always,
                "in sync when syncing is done and no in-flight requests",
                |_, state| syncing_done_and_in_sync(state),
            )
            .property(
                stateright::Expectation::Always,
                "saving and loading the document gives the same document",
                |_, state| save_load_same(state),
            )
            .property(
                stateright::Expectation::Always,
                "no errors set (from panics)",
                |_, state| {
                    state.actor_states.iter().all(|s| {
                        if let GlobalActorState::Server(s) = &**s {
                            !s.document().has_error()
                        } else {
                            true
                        }
                    })
                },
            )
            // TODO: run the client history against a single instance of the application to test if
            // it has the same structure.
            // This might not work for every situation but should work for creation since we
            // shouldn't clash.
            .property(
                Expectation::Always,
                "all apps have the right number of tasks",
                |model, state| {
                    if !syncing_done(state) {
                        return true;
                    }

                    let cf = &model.cfg.client_function;
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
                                _ => {
                                    unreachable!()
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

fn all_same_state(actors: &[Arc<State>]) -> bool {
    actors.windows(2).all(|w| match (&*w[0], &*w[1]) {
        (GlobalActorState::Trigger(_), GlobalActorState::Trigger(_)) => true,
        (GlobalActorState::Trigger(_), GlobalActorState::Server(_)) => true,
        (GlobalActorState::Server(_), GlobalActorState::Trigger(_)) => true,
        (GlobalActorState::Server(a), GlobalActorState::Server(b)) => {
            let a_vals = a.document().values(ROOT).collect::<Vec<_>>();
            let b_vals = b.document().values(ROOT).collect::<Vec<_>>();
            a_vals == b_vals
        }
    })
}

fn syncing_done(state: &ActorModelState<Actor, History>) -> bool {
    for envelope in state.network.iter_deliverable() {
        match envelope.msg {
            GlobalMsg::Internal(ServerMsg::SyncMessageRaw { .. }) => {
                return false;
            }
            GlobalMsg::Internal(ServerMsg::SyncChangeRaw { .. }) => {
                return false;
            }
            GlobalMsg::Internal(ServerMsg::SyncSaveLoadRaw { .. }) => {
                return false;
            }
            GlobalMsg::External(_) => {}
        }
    }
    true
}

fn syncing_done_and_in_sync(state: &ActorModelState<Actor, History>) -> bool {
    // first check that the network has no sync messages in-flight.
    // next, check that all actors are in the same states (using sub-property checker)
    !syncing_done(state) || all_same_state(&state.actor_states)
}

fn save_load_same(state: &ActorModelState<Actor, History>) -> bool {
    for actor in &state.actor_states {
        match &**actor {
            GlobalActorState::Trigger(_) => {
                // clients don't have state to save and load
            }
            GlobalActorState::Server(s) => {
                let bytes = s.clone().document_mut().save();
                let doc = Automerge::load(&bytes).unwrap();
                if doc.get_heads() != s.document().heads() {
                    return false;
                }
            }
        }
    }
    true
}
