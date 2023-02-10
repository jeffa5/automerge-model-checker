use std::borrow::Cow;

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
    doc: Cow<'static, Document>,
}

impl DerefDocument for AppState {
    fn document(&self) -> &Document {
        &self.doc
    }

    fn document_mut(&mut self) -> &mut Document {
        self.doc.to_mut()
    }
}

impl AppState {
    pub fn new(id: usize) -> Self {
        let mut doc = Document::new(id);
        doc.with_initial_change(|txn| {
            // create objects we'll be working in
            txn.put_object(ROOT, MAP_KEY, ObjType::Map).unwrap();
            txn.put_object(ROOT, LIST_KEY, ObjType::List).unwrap();
            txn.put_object(ROOT, TEXT_KEY, ObjType::Text).unwrap();
        });
        Self {
            doc: Cow::Owned(doc),
        }
    }

    fn get_map_obj(&self) -> automerge::ObjId {
        self.doc
            .get(ROOT, MAP_KEY)
            .ok()
            .flatten()
            .map(|(_, id)| id)
            .unwrap()
    }

    fn get_map_obj_tx(
        tx: &mut automerge::transaction::Transaction<UnObserved>,
    ) -> automerge::ObjId {
        tx.get(ROOT, MAP_KEY)
            .ok()
            .flatten()
            .map(|(_, id)| id)
            .unwrap()
    }

    fn get_list_obj(&self) -> automerge::ObjId {
        self.doc
            .get(ROOT, LIST_KEY)
            .ok()
            .flatten()
            .map(|(_, id)| id)
            .unwrap()
    }

    fn get_list_obj_tx(
        tx: &mut automerge::transaction::Transaction<UnObserved>,
    ) -> automerge::ObjId {
        tx.get(ROOT, LIST_KEY)
            .ok()
            .flatten()
            .map(|(_, id)| id)
            .unwrap()
    }

    fn get_text_obj(&self) -> automerge::ObjId {
        self.doc
            .get(ROOT, TEXT_KEY)
            .ok()
            .flatten()
            .map(|(_, id)| id)
            .unwrap()
    }

    fn get_text_obj_tx(
        tx: &mut automerge::transaction::Transaction<UnObserved>,
    ) -> automerge::ObjId {
        tx.get(ROOT, TEXT_KEY)
            .ok()
            .flatten()
            .map(|(_, id)| id)
            .unwrap()
    }

    pub fn put_map(&mut self, key: String, value: ScalarValue) {
        let mut tx = self.doc.to_mut().transaction();
        let map = Self::get_map_obj_tx(&mut tx);
        tx.put(map, key, value).unwrap();
        tx.commit();
    }

    pub fn put_list(&mut self, index: usize, value: ScalarValue) {
        let list = self.get_list_obj();
        let mut tx = self.doc.to_mut().transaction();
        if tx.put(list, index, value).is_err() {
            tx.rollback();
            return;
        };
        tx.commit();
    }

    pub fn put_text(&mut self, index: usize, value: String) {
        let text = self.get_text_obj();
        let mut tx = self.doc.to_mut().transaction();
        if tx.put(text, index, value).is_err() {
            tx.rollback();
            return;
        };
        tx.commit();
    }

    pub fn insert_list(&mut self, index: usize, value: ScalarValue) {
        let list = self.get_list_obj();
        let mut tx = self.doc.to_mut().transaction();
        if tx.insert(list, index, value).is_err() {
            tx.rollback();
            return;
        }
        tx.commit();
    }

    pub fn insert_text(&mut self, index: usize, value: String) {
        let text = self.get_text_obj();
        let mut tx = self.doc.to_mut().transaction();
        if tx.insert(text, index, value).is_err() {
            tx.rollback();
            return;
        }
        tx.commit();
    }

    pub fn delete(&mut self, key: &str) {
        let mut tx = self.doc.to_mut().transaction();
        let map = Self::get_map_obj_tx(&mut tx);
        tx.delete(map, key).unwrap();
        tx.commit();
    }

    pub fn delete_list(&mut self, index: usize) {
        let mut tx = self.doc.to_mut().transaction();
        let list = Self::get_list_obj_tx(&mut tx);
        if tx.delete(list, index).is_err() {
            tx.rollback();
            return;
        };
        tx.commit();
    }

    pub fn delete_text(&mut self, index: usize) {
        let text = self.get_text_obj();
        let mut tx = self.doc.to_mut().transaction();
        if tx.delete(text, index).is_err() {
            tx.rollback();
            self.doc.to_mut().set_error();
            return;
        };
        tx.commit();
    }

    pub fn splice_list(&mut self, index: usize, delete: usize, values: Vec<ScalarValue>) {
        let mut tx = self.doc.to_mut().transaction();
        let list = Self::get_list_obj_tx(&mut tx);
        let values = values.into_iter().map(Into::into);
        if tx.splice(list, index, delete, values).is_err() {
            tx.rollback();
            return;
        }
        tx.commit();
    }

    pub fn splice_text(&mut self, index: usize, delete: usize, value: String) {
        let mut tx = self.doc.to_mut().transaction();
        let text = Self::get_text_obj_tx(&mut tx);
        if tx.splice_text(text, index, delete, &value).is_err() {
            tx.rollback();
            return;
        }
        tx.commit();
    }

    pub fn increment_map(&mut self, key: String, by: i64) {
        let mut tx = self.doc.to_mut().transaction();
        let map = Self::get_map_obj_tx(&mut tx);
        tx.increment(map, key, by).unwrap();
        tx.commit();
    }

    pub fn increment_list(&mut self, index: usize, by: i64) {
        let list = self.get_list_obj();
        let mut tx = self.doc.to_mut().transaction();
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

    pub fn list_length(&self) -> usize {
        self.doc.length(self.get_list_obj())
    }

    pub fn text_length(&self) -> usize {
        self.doc.length(self.get_text_obj())
    }

    pub fn map_contains(&self, key: &str) -> bool {
        matches!(self.doc.get(self.get_map_obj(), key), Ok(Some(_)))
    }
}
