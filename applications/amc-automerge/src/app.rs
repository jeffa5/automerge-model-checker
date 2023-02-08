use amc::application::DerefDocument;
use amc::application::Document;
use automerge::transaction::Transactable;
use automerge::transaction::UnObserved;
use automerge::ObjType;
use automerge::ReadDoc;
use automerge::Value;
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
        Self {
            doc: Box::new(Document::new(id)),
        }
    }

    pub fn get(&self, key: &str) -> Option<ScalarValue> {
        let (_, map) = self.doc.get(ROOT, MAP_KEY).ok().flatten()?;
        self.doc
            .get(map, key)
            .unwrap()
            .map(|(v, _)| v.into_scalar().unwrap().into())
    }

    pub fn get_list(&self, index: usize) -> Option<ScalarValue> {
        let (_, map) = self.doc.get(ROOT, LIST_KEY).ok().flatten()?;
        self.doc
            .get(map, index)
            .unwrap()
            .map(|(v, _)| v.into_scalar().unwrap().into())
    }

    fn get_map_obj(tx: &mut automerge::transaction::Transaction<UnObserved>) -> automerge::ObjId {
        if let Some((_, id)) = tx.get(ROOT, MAP_KEY).ok().flatten() {
            id
        } else {
            tx.put_object(ROOT, MAP_KEY, ObjType::Map).unwrap()
        }
    }

    fn get_list_obj(tx: &mut automerge::transaction::Transaction<UnObserved>) -> automerge::ObjId {
        if let Some((_, id)) = tx.get(ROOT, LIST_KEY).ok().flatten() {
            id
        } else {
            tx.put_object(ROOT, LIST_KEY, ObjType::List).unwrap()
        }
    }

    fn get_text_obj(tx: &mut automerge::transaction::Transaction<UnObserved>) -> automerge::ObjId {
        if let Some((_, id)) = tx.get(ROOT, TEXT_KEY).ok().flatten() {
            id
        } else {
            tx.put_object(ROOT, TEXT_KEY, ObjType::Text).unwrap()
        }
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
            self.doc.set_error();
            return;
        };
        tx.commit();
    }

    pub fn put_text(&mut self, index: usize, value: String) {
        let mut tx = self.doc.transaction();
        let text = Self::get_text_obj(&mut tx);
        if tx.put(text, index, value).is_err() {
            tx.rollback();
            self.doc.set_error();
            return;
        };
        tx.commit();
    }

    pub fn put_object(&mut self, key: String, value: ObjType) {
        let mut tx = self.doc.transaction();
        tx.put_object(ROOT, key, value).unwrap();
        tx.commit();
    }

    pub fn put_object_list(&mut self, index: usize, value: ObjType) {
        let mut tx = self.doc.transaction();
        tx.put_object(ROOT, index, value).unwrap();
        tx.commit();
    }

    pub fn insert_list(&mut self, index: usize, value: ScalarValue) {
        let mut tx = self.doc.transaction();
        let list = match tx.get(ROOT, LIST_KEY) {
            Ok(Some((Value::Object(ObjType::List), list))) => list,
            _ => {
                tx.rollback();
                self.doc.set_error();
                return;
            }
        };
        tx.insert(list, index, value).unwrap();
        tx.commit();
    }

    pub fn insert_text(&mut self, index: usize, value: String) {
        let mut tx = self.doc.transaction();
        let text = match tx.get(ROOT, TEXT_KEY) {
            Ok(Some((Value::Object(ObjType::Text), text))) => text,
            _ => {
                tx.rollback();
                self.doc.set_error();
                return;
            }
        };
        tx.insert(text, index, value).unwrap();
        tx.commit();
    }

    pub fn insert_object(&mut self, index: usize, value: ObjType) {
        let mut tx = self.doc.transaction();
        let list = match tx.get(ROOT, LIST_KEY) {
            Ok(Some((Value::Object(ObjType::List), list))) => list,
            _ => {
                tx.rollback();
                self.doc.set_error();
                return;
            }
        };
        tx.insert_object(list, index, value).unwrap();
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
            self.doc.set_error();
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
        let list = match tx.get(ROOT, LIST_KEY) {
            Ok(Some((Value::Object(ObjType::List), list))) => list,
            _ => {
                tx.rollback();
                self.doc.set_error();
                return;
            }
        };
        let values = values.into_iter().map(Into::into);
        tx.splice(list, index, delete, values).unwrap();
        tx.commit();
    }

    pub fn splice_text(&mut self, index: usize, delete:usize, value: String) {
        let mut tx = self.doc.transaction();
        let text = match tx.get(ROOT, TEXT_KEY) {
            Ok(Some((Value::Object(ObjType::Text), text))) => text,
            _ => {
                tx.rollback();
                self.doc.set_error();
                return;
            }
        };
        tx.splice_text(text, index, delete, &value).unwrap();
        tx.commit();
    }

    pub fn values(&self) -> Vec<(&str, Value)> {
        self.doc
            .map_range(ROOT, ..)
            .map(|(key, value, _)| (key, value))
            .collect()
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
