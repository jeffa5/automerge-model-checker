use std::hash::Hash;
use std::{collections::BTreeMap, ops::Deref};

use automerge::transaction::Transaction;
use automerge::{sync, ActorId, Automerge, Change, ChangeHash};
use stateright::actor::Id;

#[derive(Clone, Debug)]
pub struct Document {
    am: Automerge,
    sync_states: BTreeMap<usize, sync::State>,
    /// Whether this document has encountered an error (indicates an application failure).
    error: bool,
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.am.get_heads() == other.am.get_heads()
            && self.sync_states == other.sync_states
            && self.error == other.error
    }
}

impl Eq for Document {}

impl Hash for Document {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.am.get_heads().hash(state);
        self.sync_states.hash(state);
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
            error: false,
        }
    }

    pub fn has_error(&self) -> bool {
        self.error
    }

    pub fn set_error(&mut self) {
        self.error = true;
    }

    pub fn heads(&self) -> Vec<ChangeHash> {
        self.am.get_heads()
    }

    pub fn last_local_change(&self) -> Option<&Change> {
        self.am.get_last_local_change()
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
        self.am.generate_sync_message(state)
    }

    pub fn save(&mut self) -> Vec<u8> {
        self.am.save()
    }

    pub fn merge(&mut self, other: &mut Automerge) {
        self.am.merge(other).unwrap();
    }

    pub fn transaction(&mut self) -> Transaction {
        self.am.transaction()
    }
}

impl Deref for Document {
    type Target = Automerge;

    fn deref(&self) -> &Self::Target {
        &self.am
    }
}
