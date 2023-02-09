use amc::application::DerefDocument;
use amc::application::Document;
use automerge::transaction::Transactable;
use automerge::transaction::UnObserved;
use automerge::ObjType;
use automerge::ReadDoc;
use automerge::ROOT;

use crate::scalar::ScalarValue;

pub const LIST_KEY: &str = "list";
pub const TEXT_KEY: &str = "text";
pub const MAP_KEY: &str = "map";

/// The app that clients work with.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AppState {
    doc: Box<Document>,
}

impl DerefDocument for AppState {
    fn document(&self) -> &Document {
        &self.doc
    }

    fn document_mut(&mut self) -> &mut Document {
        &mut self.doc
    }
}

impl AppState {
    pub fn new(id: usize) -> Self {
        let mut doc = Document::new(id);
        doc.with_initial_change (|txn| {
            // create objects we'll be working in
            txn.put_object(ROOT, MAP_KEY, ObjType::Map).unwrap();
            txn.put_object(ROOT, LIST_KEY, ObjType::List).unwrap();
            txn.put_object(ROOT, TEXT_KEY, ObjType::Text).unwrap();
        });
        Self {
            doc: Box::new(doc),
        }
    }

    fn get_map_obj(tx: &mut automerge::transaction::Transaction<UnObserved>) -> automerge::ObjId {
        tx.get(ROOT, MAP_KEY).ok().flatten().map(|(_, id)| id).unwrap()
    }

    fn get_list_obj(tx: &mut automerge::transaction::Transaction<UnObserved>) -> automerge::ObjId {
        tx.get(ROOT, LIST_KEY).ok().flatten().map(|(_, id)| id).unwrap()
    }

    fn get_text_obj(tx: &mut automerge::transaction::Transaction<UnObserved>) -> automerge::ObjId {
        tx.get(ROOT, TEXT_KEY).ok().flatten().map(|(_, id)| id).unwrap()
    }

    pub fn put_map(&mut self, key: String, value: ScalarValue) {
        let mut tx = self.doc.transaction();
        let map = Self::get_map_obj(&mut tx);
        tx.put(map, key, value).unwrap();
        tx.commit();
    }

    pub fn put_list(&mut self, index: usize, value: ScalarValue) {
        let mut tx = self.doc.transaction();
        let list = Self::get_list_obj(&mut tx);
        if tx.put(list, index, value).is_err() {
            tx.rollback();
            return;
        };
        tx.commit();
    }

    pub fn put_text(&mut self, index: usize, value: String) {
        let mut tx = self.doc.transaction();
        let text = Self::get_text_obj(&mut tx);
        if tx.put(text, index, value).is_err() {
            tx.rollback();
            return;
        };
        tx.commit();
    }

    pub fn insert_list(&mut self, index: usize, value: ScalarValue) {
        let mut tx = self.doc.transaction();
        let list = Self::get_list_obj(&mut tx);
        if tx.insert(list, index, value).is_err() {
            tx.rollback();
            return;
        }
        tx.commit();
    }

    pub fn insert_text(&mut self, index: usize, value: String) {
        let mut tx = self.doc.transaction();
        let text = Self::get_text_obj(&mut tx);
        if tx.insert(text, index, value).is_err() {
            tx.rollback();
            return;
        }
        tx.commit();
    }

    pub fn delete(&mut self, key: &str) {
        let mut tx = self.doc.transaction();
        let map = Self::get_map_obj(&mut tx);
        tx.delete(map, key).unwrap();
        tx.commit();
    }

    pub fn delete_list(&mut self, index: usize) {
        let mut tx = self.doc.transaction();
        let list = Self::get_list_obj(&mut tx);
        if tx.delete(list, index).is_err() {
            tx.rollback();
            return;
        };
        tx.commit();
    }

    pub fn delete_text(&mut self, index: usize) {
        let mut tx = self.doc.transaction();
        let text = Self::get_text_obj(&mut tx);
        if tx.delete(text, index).is_err() {
            tx.rollback();
            self.doc.set_error();
            return;
        };
        tx.commit();
    }

    pub fn splice_list(&mut self, index: usize, delete: usize, values: Vec<ScalarValue>) {
        let mut tx = self.doc.transaction();
        let list = Self::get_list_obj(&mut tx);
        let values = values.into_iter().map(Into::into);
        if tx.splice(list, index, delete, values).is_err() {
            tx.rollback();
            return;
        }
        tx.commit();
    }

    pub fn splice_text(&mut self, index: usize, delete: usize, value: String) {
        let mut tx = self.doc.transaction();
        let text = Self::get_text_obj(&mut tx);
        if tx.splice_text(text, index, delete, &value).is_err() {
            tx.rollback();
            return;
        }
        tx.commit();
    }

    pub fn increment_map(&mut self, key: String, by: i64) {
        let mut tx = self.doc.transaction();
        let map = Self::get_map_obj(&mut tx);
        tx.increment(map, key, by).unwrap();
        tx.commit();
    }

    pub fn increment_list(&mut self, index: usize, by: i64) {
        let mut tx = self.doc.transaction();
        let list = Self::get_list_obj(&mut tx);
        if tx.increment(list, index, by).is_err() {
            tx.rollback();
            return;
        };
        tx.commit();
    }

    pub fn length(&self, key: &str) -> usize {
        self.doc
            .get(ROOT, key)
            .ok()
            .flatten()
            .map(|(_, id)| self.doc.length(id))
            .unwrap_or_default()
    }
}
