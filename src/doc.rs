use std::collections::BTreeMap;
use std::hash::Hash;

use automerge::transaction::Transactable;
use automerge::{sync, ActorId, Automerge, Change, ROOT};
use stateright::actor::Id;

#[derive(Clone, Debug)]
pub struct Doc {
    am: Automerge,
    sync_states: BTreeMap<usize, sync::State>,
    error: bool,
}

impl PartialEq for Doc {
    fn eq(&self, other: &Self) -> bool {
        self.values() == other.values()
            // && self.sync_states == other.sync_states
            && self.error == other.error
    }
}

impl Eq for Doc {}

impl Hash for Doc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.values().hash(state);
        // self.sync_states.hash(state);
        self.error.hash(state);
    }
}

impl Doc {
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

    pub fn get(&self, key: &str) -> Option<String> {
        self.am
            .get(ROOT, key)
            .unwrap()
            .map(|(v, _)| v.into_string().unwrap())
    }

    pub fn put(&mut self, key: String, value: String) {
        let mut tx = self.am.transaction();
        tx.put(ROOT, key, value).unwrap();
        tx.commit();
    }

    pub fn delete(&mut self, key: &str) {
        let mut tx = self.am.transaction();
        tx.delete(ROOT, key).unwrap();
        tx.commit();
    }

    pub fn last_local_change(&self) -> Option<&Change> {
        self.am.get_last_local_change()
    }

    pub fn apply_change(&mut self, change: Change) {
        self.am.apply_changes(std::iter::once(change)).unwrap()
    }

    pub fn values(&self) -> Vec<(&str, String)> {
        self.am
            .map_range(ROOT, ..)
            .map(|(key, value, _)| (key, value.into_string().unwrap()))
            .collect()
    }

    pub fn receive_sync_message(&mut self, peer: usize, message: sync::Message) {
        let state = self.sync_states.entry(peer).or_default();
        let res = self.am.receive_sync_message(state, message);
        if let Err(error) = res {
            // set the error
            self.error = true;
        }
    }

    pub fn generate_sync_message(&mut self, peer: usize) -> Option<sync::Message> {
        let state = self.sync_states.entry(peer).or_default();
        self.am.generate_sync_message(state)
    }
}
