use automerge::{sync, ChangeHash};
use automerge::{transaction::Transactable, ObjType};
use automerge::{AutomergeError, Change, ObjId, Value, ROOT};
use std::hash::Hash;

use automerge::{ActorId, AutoCommit, Automerge};

#[derive(Clone, Debug)]
pub enum InnerDoc {
    Automerge(Automerge),
    AutoCommit(AutoCommit),
}

impl PartialEq for InnerDoc {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Automerge(l0), Self::Automerge(r0)) => l0.get_heads() == r0.get_heads(),
            (Self::AutoCommit(l0), Self::AutoCommit(r0)) => l0.get_heads() == r0.get_heads(),
            _ => panic!("mismatching documents"),
        }
    }
}

impl Eq for InnerDoc {}

impl Hash for InnerDoc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            InnerDoc::Automerge(a) => a.get_heads().hash(state),
            InnerDoc::AutoCommit(a) => a.get_heads().hash(state),
        }
    }
}

impl InnerDoc {
    pub fn set_actor(&mut self, id: ActorId) {
        match self {
            InnerDoc::Automerge(a) => {
                a.set_actor(id);
            }
            InnerDoc::AutoCommit(a) => {
                a.set_actor(id);
            }
        }
    }

    pub fn get(&mut self, obj: &ObjId, key: &str) -> Option<String> {
        match self {
            InnerDoc::Automerge(a) => a
                .get(obj, key)
                .unwrap()
                .map(|(v, _)| v.into_string().unwrap()),
            InnerDoc::AutoCommit(a) => a
                .get(obj, key)
                .unwrap()
                .map(|(v, _)| v.into_string().unwrap()),
        }
    }

    pub fn put(&mut self, obj: &ObjId, key: String, value: String) {
        match self {
            InnerDoc::Automerge(a) => {
                let mut tx = a.transaction();
                tx.put(obj, key, value).unwrap();
                tx.commit();
            }
            InnerDoc::AutoCommit(a) => {
                a.put(obj, key, value);
            }
        }
    }

    pub fn put_object(&mut self, obj: &ObjId, key: String, value: ObjType) {
        match self {
            InnerDoc::Automerge(a) => {
                let mut tx = a.transaction();
                tx.put_object(obj, key, value).unwrap();
                tx.commit();
            }
            InnerDoc::AutoCommit(a) => {
                a.put_object(obj, key, value);
            }
        }
    }

    pub fn insert(&mut self, obj: &ObjId, key: usize, value: String) {
        // TODO: use the given objid as the list id
        let list = match self {
            InnerDoc::Automerge(a) => a.get(ROOT, "list"),
            InnerDoc::AutoCommit(a) => a.get(ROOT, "list"),
        };
        match self {
            InnerDoc::Automerge(a) => {
                let mut tx = a.transaction();
                tx.insert(obj, key, value).unwrap();
                tx.commit();
            }
            InnerDoc::AutoCommit(a) => {
                a.insert(obj, key, value);
            }
        }
    }

    pub fn delete(&mut self, obj: &ObjId, key: &str) {
        match self {
            InnerDoc::Automerge(a) => {
                let mut tx = a.transaction();
                tx.delete(obj, key).unwrap();
                tx.commit();
            }
            InnerDoc::AutoCommit(a) => {
                a.delete(obj, key);
            }
        }
    }

    pub fn get_last_local_change(&self) -> Option<&Change> {
        match self {
            InnerDoc::Automerge(a) => a.get_last_local_change(),
            InnerDoc::AutoCommit(a) => a.get_last_local_change(),
        }
    }

    pub fn apply_change(&mut self, change: Change) {
        match self {
            InnerDoc::Automerge(a) => a.apply_changes(std::iter::once(change)).unwrap(),
            InnerDoc::AutoCommit(a) => a.apply_changes(std::iter::once(change)).unwrap(),
        }
    }

    pub fn values(&self) -> Vec<(&str, Value)> {
        match self {
            InnerDoc::Automerge(a) => a
                .map_range(ROOT, ..)
                .map(|(key, value, _)| (key, value))
                .collect(),
            InnerDoc::AutoCommit(a) => a
                .map_range(ROOT, ..)
                .map(|(key, value, _)| (key, value))
                .collect(),
        }
    }

    pub fn receive_sync_message(
        &mut self,
        state: &mut sync::State,
        message: sync::Message,
    ) -> Result<(), AutomergeError> {
        match self {
            InnerDoc::Automerge(a) => a.receive_sync_message(state, message),

            InnerDoc::AutoCommit(a) => a.receive_sync_message(state, message),
        }
    }

    pub fn generate_sync_message(&mut self, state: &mut sync::State) -> Option<sync::Message> {
        match self {
            InnerDoc::Automerge(a) => a.generate_sync_message(state),
            InnerDoc::AutoCommit(a) => a.generate_sync_message(state),
        }
    }

    pub fn save(&mut self) -> Vec<u8> {
        match self {
            InnerDoc::Automerge(a) => a.save(),
            InnerDoc::AutoCommit(a) => a.save(),
        }
    }

    pub fn merge(&mut self, other: &mut Self) -> Result<Vec<ChangeHash>, AutomergeError> {
        match (self, other) {
            (InnerDoc::Automerge(a), InnerDoc::Automerge(b)) => a.merge(b),
            (InnerDoc::AutoCommit(a), InnerDoc::AutoCommit(b)) => a.merge(b),
            (InnerDoc::Automerge(a), InnerDoc::AutoCommit(b)) => {
                panic!("mismatching document types")
            }
            (InnerDoc::AutoCommit(a), InnerDoc::Automerge(b)) => {
                panic!("mismatching document types")
            }
        }
    }
}
