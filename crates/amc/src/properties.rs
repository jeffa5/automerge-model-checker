use std::{fmt::Debug, hash::Hash, sync::Arc};

use automerge::Automerge;
use automerge::ReadDoc;
use automerge::ROOT;
use stateright::actor::{ActorModel, ActorModelState};

use crate::client::Application;
use crate::client::DerefDocument;
use crate::drive::Drive;
use crate::global::GlobalMsg;
use crate::global::{GlobalActor, GlobalActorState};
use crate::server::ServerMsg;

/// Add default properties to a model.
///
/// These include checking for consistent states when syncing is completed, save and load
/// consistency and others.
///
/// **Warning**: This may add significant performance overhead, just for the checking of internal automerge
/// properties, you could probably do without these if you're assuming automerge is correct.
pub fn with_default_properties<A, D, C, H>(
    mut model: ActorModel<GlobalActor<A, D>, C, H>,
) -> ActorModel<GlobalActor<A, D>, C, H>
where
    A: Application,
    D: Drive<A>,
    H: Hash + Debug + Clone,
{
    model = with_in_sync_check(model);
    model = with_save_load_check(model);
    model = with_error_free_check(model);
    model
}

/// Ensure that all applications have the same state when there is no syncing to be done.
pub fn with_in_sync_check<A, D, C, H>(
    model: ActorModel<GlobalActor<A, D>, C, H>,
) -> ActorModel<GlobalActor<A, D>, C, H>
where
    A: Application,
    D: Drive<A>,
    H: Hash + Debug + Clone,
{
    model.property(
        stateright::Expectation::Always,
        "in sync when syncing is done and no in-flight requests",
        |_, state| syncing_done_and_in_sync(state),
    )
}

/// Ensure that after each application step, saving and loading the document gives the same
/// document.
///
/// **Warning**: Saving and loading are comparatively expensive to be run in model checking so it
/// might be best not to include this unless you really want it.
pub fn with_save_load_check<A, D, C, H>(
    model: ActorModel<GlobalActor<A, D>, C, H>,
) -> ActorModel<GlobalActor<A, D>, C, H>
where
    A: Application,
    D: Drive<A>,
    H: Hash + Debug + Clone,
{
    model.property(
        stateright::Expectation::Always,
        "saving and loading the document gives the same document",
        |_, state| save_load_same(state),
    )
}

/// Ensure that the application logic doesn't panic.
///
/// This might get removed if panics get better handling in our underlying model checker
/// (Stateright).
pub fn with_error_free_check<A, D, C, H>(
    model: ActorModel<GlobalActor<A, D>, C, H>,
) -> ActorModel<GlobalActor<A, D>, C, H>
where
    A: Application,
    D: Drive<A>,
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

/// Check that all servers have the same document heads.
pub fn all_same_heads<T, A>(actors: &[Arc<GlobalActorState<T, A>>]) -> bool
where
    T: Drive<A>,
    A: Application,
{
    actors.windows(2).all(|w| match (&*w[0], &*w[1]) {
        (GlobalActorState::Client(_), GlobalActorState::Client(_)) => true,
        (GlobalActorState::Client(_), GlobalActorState::Server(_)) => true,
        (GlobalActorState::Server(_), GlobalActorState::Client(_)) => true,
        (GlobalActorState::Server(a), GlobalActorState::Server(b)) => {
            a.document().get_heads() == b.document().get_heads()
        }
    })
}

fn all_same_state<T, A>(actors: &[Arc<GlobalActorState<T, A>>]) -> bool
where
    T: Drive<A>,
    A: Application,
{
    actors.windows(2).all(|w| match (&*w[0], &*w[1]) {
        (GlobalActorState::Client(_), GlobalActorState::Client(_)) => true,
        (GlobalActorState::Client(_), GlobalActorState::Server(_)) => true,
        (GlobalActorState::Server(_), GlobalActorState::Client(_)) => true,
        (GlobalActorState::Server(a), GlobalActorState::Server(b)) => {
            let a_vals = a.document().values(ROOT).collect::<Vec<_>>();
            let b_vals = b.document().values(ROOT).collect::<Vec<_>>();
            a_vals == b_vals
        }
    })
}

/// Check whether syncing is complete at this time. That is, there are no sync messages pending
/// delivery and all documents have the same heads.
pub fn syncing_done<A, D, H>(state: &ActorModelState<GlobalActor<A, D>, H>) -> bool
where
    A: Application,
    D: Drive<A>,
{
    let all_documents_same_heads = all_same_heads(&state.actor_states);

    let network_contains_sync_messages = state.network.iter_deliverable().any(|e| match e.msg {
        GlobalMsg::ServerToServer(s2s) => match s2s {
            ServerMsg::SyncMessageRaw { message_bytes: _ } => true,
            ServerMsg::SyncChangeRaw {
                missing_changes_bytes: _,
            } => true,
            ServerMsg::SyncSaveLoadRaw { doc_bytes: _ } => true,
        },
        GlobalMsg::ClientToServer(_) => false,
    });

    all_documents_same_heads && !network_contains_sync_messages
}

fn syncing_done_and_in_sync<A, D, H>(state: &ActorModelState<GlobalActor<A, D>, H>) -> bool
where
    A: Application,
    D: Drive<A>,
{
    // first check that the network has no sync messages in-flight.
    // next, check that all actors are in the same states (using sub-property checker)
    !syncing_done(state) || all_same_state(&state.actor_states)
}

fn save_load_same<A, D, H>(state: &ActorModelState<GlobalActor<A, D>, H>) -> bool
where
    A: Application,
    D: Drive<A>,
{
    for actor in &state.actor_states {
        match &**actor {
            GlobalActorState::Client(_) => {
                // clients don't have state to save and load
            }
            GlobalActorState::Server(s) => {
                let bytes = s.clone().document_mut().save();
                let doc = Automerge::load(&bytes).unwrap();
                if doc.get_heads() != s.document().get_heads() {
                    return false;
                }
            }
        }
    }
    true
}
