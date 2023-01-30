use std::fmt::Debug;
use std::hash::Hash;
use std::{collections::BTreeMap, ops::Deref};

use automerge::transaction::{Transaction, UnObserved};
use automerge::{sync, ActorId, Automerge, Change, ChangeHash, ROOT};

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

#[derive(Debug)]
enum Value {
    Map(BTreeMap<String, Value>),
    List(Vec<Value>),
    Scalar(String),
}

fn materialize_root(am: &Automerge) -> Value {
    let mut map = BTreeMap::new();
    for (k, v, id) in am.map_range(ROOT, ..) {
        map.insert(k.to_owned(), materialize_value(am, v, id));
    }
    Value::Map(map)
}

fn materialize_value(am: &Automerge, value: automerge::Value, id: automerge::ObjId) -> Value {
    match value {
        automerge::Value::Object(o) => match o {
            automerge::ObjType::Map => {
                let mut map = BTreeMap::new();
                for (k, v, id) in am.map_range(id, ..) {
                    map.insert(k.to_owned(), materialize_value(am, v, id));
                }
                Value::Map(map)
            }
            automerge::ObjType::List => {
                let mut list = Vec::new();
                for (_, v, id) in am.list_range(id, ..) {
                    list.push(materialize_value(am, v, id));
                }
                Value::List(list)
            }
            automerge::ObjType::Text => todo!(),
            automerge::ObjType::Table => todo!(),
        },
        automerge::Value::Scalar(s) => Value::Scalar(s.to_string()),
    }
}

fn materialize(am: &Automerge) -> Value {
    materialize_root(am)
}

impl Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Document");
        if self.debug_materialize {
            // todo: materialize
            let v = materialize(&self.am);
            s.field("doc", &v)
                .field("heads", &self.get_heads())
                .field("sync_states", &self.sync_states)
                .field("last_sent_heads", &self.last_sent_heads)
                .field("error", &self.error);
        } else {
            s.field("am", &self.am)
                .field("heads", &self.get_heads())
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
    /// Create a new document.
    pub fn new(id: usize) -> Self {
        let mut doc = Automerge::new();
        doc.set_actor(ActorId::from(id.to_be_bytes()));
        Self {
            am: doc,
            sync_states: BTreeMap::new(),
            last_sent_heads: Vec::new(),
            error: false,
            debug_materialize: true,
        }
    }

    /// Create an initial change for the document.
    ///
    /// This ensures that, when syncing, documents have a common root to merge from.
    pub fn with_initial_change<F: Fn(&mut Transaction<'_, UnObserved>)>(&mut self, make_change: F) {
        assert!(self.am.get_changes(&[]).unwrap().is_empty());

        let actor = self.get_actor().clone();
        self.am.set_actor(ActorId::from(999u64.to_be_bytes()));

        let mut txn = self.transaction();
        make_change(&mut txn);
        txn.commit();

        self.am.set_actor(actor);
    }

    /// Check whether this document has errored.
    pub fn has_error(&self) -> bool {
        self.error
    }

    /// Mark this document as having encountered an error.
    pub fn set_error(&mut self) {
        self.error = true;
    }

    /// Apply a change to the document.
    pub fn apply_change(&mut self, change: Change) {
        self.am.apply_changes(std::iter::once(change)).unwrap()
    }

    /// Receive a sync message for this document, automatically handling sync states.
    pub fn receive_sync_message(&mut self, peer: usize, message: sync::Message) {
        let state = self.sync_states.entry(peer).or_default();
        let res = self.am.receive_sync_message(state, message);
        if let Err(_error) = res {
            // set the error
            self.error = true;
        }
    }

    /// Generate a sync message for a peer.
    pub fn generate_sync_message(&mut self, peer: usize) -> Option<sync::Message> {
        let state = self.sync_states.entry(peer).or_default();
        let msg = self.am.generate_sync_message(state);
        if msg.is_some() {
            self.update_last_sent_heads();
        }
        msg
    }

    /// Update the last sent heads to the current ones of the document, returning the previous set
    /// of heads.
    pub fn update_last_sent_heads(&mut self) -> Vec<ChangeHash> {
        // get the new heads that we're syncing
        let mut heads = self.am.get_heads();
        // swap them with the other ones
        std::mem::swap(&mut heads, &mut self.last_sent_heads);
        heads
    }

    /// Get the last local changes since syncing.
    pub fn get_last_local_changes_for_sync(&self) -> impl Iterator<Item = &Change> {
        // get the changes since the heads
        let changes = self.am.get_changes(&self.last_sent_heads).unwrap();
        let actor = self.am.get_actor();
        // and filter them to those that were made by us
        changes.into_iter().filter(move |c| c.actor_id() == actor)
    }

    /// Save the document.
    pub fn save(&mut self) -> Vec<u8> {
        self.am.save()
    }

    /// Merge another document with this one.
    pub fn merge(&mut self, other: &mut Automerge) {
        self.am.merge(other).unwrap();
    }

    /// Create a transaction.
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
