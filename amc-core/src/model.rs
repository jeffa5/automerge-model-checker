use crate::DerefDocument;
use std::{fmt::Debug, hash::Hash, sync::Arc};

use automerge::Automerge;
use automerge::ROOT;
use stateright::actor::{ActorModel, ActorModelState};

use crate::Application;
use crate::Trigger;
use crate::{GlobalActor, GlobalActorState, GlobalMsg, ServerMsg};

pub fn with_same_state_check<T, A, C, H>(
    model: ActorModel<GlobalActor<T, A>, C, H>,
) -> ActorModel<GlobalActor<T, A>, C, H>
where
    T: Trigger<A>,
    A: Application,
    H: Hash + Debug + Clone,
{
    model.property(
        stateright::Expectation::Eventually,
        "all actors have the same value for all keys",
        |_, state| all_same_state(&state.actor_states),
    )
}

pub fn with_in_sync_check<T, A, C, H>(
    model: ActorModel<GlobalActor<T, A>, C, H>,
) -> ActorModel<GlobalActor<T, A>, C, H>
where
    T: Trigger<A>,
    A: Application,
    H: Hash + Debug + Clone,
{
    model.property(
        stateright::Expectation::Always,
        "in sync when syncing is done and no in-flight requests",
        |_, state| syncing_done_and_in_sync(state),
    )
}

pub fn with_save_load_check<T, A, C, H>(
    model: ActorModel<GlobalActor<T, A>, C, H>,
) -> ActorModel<GlobalActor<T, A>, C, H>
where
    T: Trigger<A>,
    A: Application,
    H: Hash + Debug + Clone,
{
    model.property(
        stateright::Expectation::Always,
        "saving and loading the document gives the same document",
        |_, state| save_load_same(state),
    )
}

pub fn with_error_free_check<T, A, C, H>(
    model: ActorModel<GlobalActor<T, A>, C, H>,
) -> ActorModel<GlobalActor<T, A>, C, H>
where
    T: Trigger<A>,
    A: Application,
    H: Hash + Debug + Clone,
{
    model.property(
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
}

/// Add default properties to a model.
///
/// These include checking for consistent states when syncing is completed, save and load
/// consistency and others.
pub fn with_default_properties<T, A, C, H>(
    mut model: ActorModel<GlobalActor<T, A>, C, H>,
) -> ActorModel<GlobalActor<T, A>, C, H>
where
    T: Trigger<A>,
    A: Application,
    H: Hash + Debug + Clone,
{
    model = with_same_state_check(model);
    model = with_in_sync_check(model);
    model = with_save_load_check(model);
    model = with_error_free_check(model);
    model
}

fn all_same_state<T, A>(actors: &[Arc<GlobalActorState<T, A>>]) -> bool
where
    T: Trigger<A>,
    A: Application,
{
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

/// Check whether syncing is complete at this time. That is, there are no sync messages pending
/// delivery.
pub fn syncing_done<T, A, H>(state: &ActorModelState<GlobalActor<T, A>, H>) -> bool
where
    T: Trigger<A>,
    A: Application,
{
    for envelope in state.network.iter_deliverable() {
        match envelope.msg {
            GlobalMsg::ServerToServer(ServerMsg::SyncMessageRaw { .. }) => {
                return false;
            }
            GlobalMsg::ServerToServer(ServerMsg::SyncChangeRaw { .. }) => {
                return false;
            }
            GlobalMsg::ServerToServer(ServerMsg::SyncSaveLoadRaw { .. }) => {
                return false;
            }
            GlobalMsg::ClientToServer(_) => {}
        }
    }
    true
}

fn syncing_done_and_in_sync<T, A, H>(state: &ActorModelState<GlobalActor<T, A>, H>) -> bool
where
    T: Trigger<A>,
    A: Application,
{
    // first check that the network has no sync messages in-flight.
    // next, check that all actors are in the same states (using sub-property checker)
    !syncing_done(state) || all_same_state(&state.actor_states)
}

fn save_load_same<T, A, H>(state: &ActorModelState<GlobalActor<T, A>, H>) -> bool
where
    T: Trigger<A>,
    A: Application,
{
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
