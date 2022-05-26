use std::collections::BTreeMap;
use std::hash::Hash;

use automerge::transaction::Transactable;
use automerge::{sync, ActorId, Automerge, Change, ObjType, Value, ROOT};
use stateright::actor::Id;

mod inner;

pub use inner::InnerDoc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Doc {
    am: InnerDoc,
    sync_states: BTreeMap<usize, sync::State>,
    error: bool,
}

impl Doc {
    pub fn new(actor_id: Id, mut doc: InnerDoc) -> Self {
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

    pub fn get(&mut self, key: &str) -> Option<String> {
        self.am.get(&ROOT, key)
    }

    pub fn put(&mut self, key: String, value: String) {
        self.am.put(&ROOT, key, value)
    }

    pub fn put_object(&mut self, key: String, value: ObjType) {
        self.am.put_object(&ROOT, key, value)
    }

    pub fn insert(&mut self, index: usize, value: String) {
        self.am.insert(&ROOT, index, value)
    }

    pub fn delete(&mut self, key: &str) {
        self.am.delete(&ROOT, key)
    }

    pub fn last_local_change(&self) -> Option<&Change> {
        self.am.get_last_local_change()
    }

    pub fn apply_change(&mut self, change: Change) {
        self.am.apply_change(change)
    }

    pub fn values(&self) -> Vec<(&str, Value)> {
        self.am.values()
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

    pub fn merge(&mut self, other: &mut InnerDoc) {
        self.am.merge(other).unwrap();
    }
}
