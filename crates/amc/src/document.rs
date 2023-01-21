use std::fmt::Debug;
use std::hash::Hash;
use std::{collections::BTreeMap, ops::Deref};

use automerge::transaction::{Transaction, UnObserved};
use automerge::{sync, ActorId, Automerge, Change, ChangeHash, ROOT};
use stateright::actor::Id;

/// A document that holds an automerge object and also the sync states for peers.
#[derive(Clone)]
pub struct Document {
    am: Automerge,
    /// States for the syncing.
    sync_states: BTreeMap<usize, sync::State>,
    /// Heads of the last sync operation.
    last_sent_heads: Vec<ChangeHash>,
    /// Whether this document has encountered an error (indicates an application failure).
    error: bool,
    /// Whether to show the json repr in debug.
    debug_materialize: bool,
}

impl Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Document");
        if self.debug_materialize {
            // todo: materialize
            let mut m = BTreeMap::new();
            for (k, v, _) in self.am.map_range(ROOT, ..) {
                m.insert(k, v);
            }
            s.field("doc", &m)
                .field("sync_states", &self.sync_states)
                .field("last_sent_heads", &self.last_sent_heads)
                .field("error", &self.error);
        } else {
            s.field("am", &self.am)
                .field("sync_states", &self.sync_states)
                .field("last_sent_heads", &self.last_sent_heads)
                .field("error", &self.error);
        }
        s.finish()
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.am.get_heads() == other.am.get_heads()
            && self.sync_states == other.sync_states
            && self.last_sent_heads == other.last_sent_heads
            && self.error == other.error
    }
}

impl Eq for Document {}

impl Hash for Document {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.am.get_heads().hash(state);
        self.sync_states.hash(state);
        self.last_sent_heads.hash(state);
        self.error.hash(state);
    }
}

impl Document {
    pub fn new(actor_id: Id) -> Self {
        let mut doc = Automerge::new();
        let id: usize = actor_id.into();
        doc.set_actor(ActorId::from(id.to_be_bytes()));
        Self {
            am: doc,
            sync_states: BTreeMap::new(),
            last_sent_heads: Vec::new(),
            error: false,
            debug_materialize: true,
        }
    }

    pub fn has_error(&self) -> bool {
        self.error
    }

    pub fn set_error(&mut self) {
        self.error = true;
    }

    pub fn apply_change(&mut self, change: Change) {
        self.am.apply_changes(std::iter::once(change)).unwrap()
    }

    pub fn receive_sync_message(&mut self, peer: usize, message: sync::Message) {
        let state = self.sync_states.entry(peer).or_default();
        let res = self.am.receive_sync_message(state, message);
        if let Err(_error) = res {
            // set the error
            self.error = true;
        }
    }

    pub fn generate_sync_message(&mut self, peer: usize) -> Option<sync::Message> {
        let state = self.sync_states.entry(peer).or_default();
        let msg = self.am.generate_sync_message(state);
        if msg.is_some() {
            self.update_last_sent_heads();
        }
        msg
    }

    /// Check whether this document has finished sending things to peers.
    pub fn finished_sending_changes(&self) -> bool {
        self.get_heads() == self.last_sent_heads
    }

    pub fn update_last_sent_heads(&mut self) -> Vec<ChangeHash> {
        // get the new heads that we're syncing
        let mut heads = self.am.get_heads();
        // swap them with the other ones
        std::mem::swap(&mut heads, &mut self.last_sent_heads);
        heads
    }

    pub fn get_last_local_changes_for_sync(&self) -> impl Iterator<Item = &Change> {
        // get the changes since the heads
        let changes = self.am.get_changes(&self.last_sent_heads).unwrap();
        let actor = self.am.get_actor();
        // and filter them to those that were made by us
        changes.into_iter().filter(move |c| c.actor_id() == actor)
    }

    pub fn save(&mut self) -> Vec<u8> {
        self.am.save()
    }

    pub fn merge(&mut self, other: &mut Automerge) {
        self.am.merge(other).unwrap();
    }

    pub fn transaction(&mut self) -> Transaction<UnObserved> {
        self.am.transaction()
    }
}

impl Deref for Document {
    type Target = Automerge;

    fn deref(&self) -> &Self::Target {
        &self.am
    }
}
