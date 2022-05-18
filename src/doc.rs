use std::hash::Hash;

use automerge::transaction::Transactable;
use automerge::{ActorId, Automerge, Change, ROOT};
use stateright::actor::Id;

#[derive(Clone, Debug)]
pub struct Doc {
    am: Automerge,
}

impl PartialEq for Doc {
    fn eq(&self, other: &Self) -> bool {
        self.values() == other.values()
    }
}

impl Eq for Doc {}

impl Hash for Doc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.values().hash(state);
    }
}

impl Doc {
    pub fn new(actor_id: Id) -> Self {
        let mut doc = Automerge::new();
        let id: usize = actor_id.into();
        doc.set_actor(ActorId::from(id.to_be_bytes()));
        Self { am: doc }
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
}
