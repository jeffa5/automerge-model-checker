use std::hash::Hash;

use amc_core::Application;
use amc_core::Document;
use automerge::transaction::Transactable;
use automerge::ObjType;
use automerge::ROOT;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

/// The app that clients work with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct App {
    doc: Box<Document>,
    seed: u64,
    rng: StdRng,
}

impl Hash for App {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.doc.hash(state);
        self.seed.hash(state);
    }
}

impl Application for App {
    fn new(id: stateright::actor::Id) -> Self {
        Self {
            doc: Box::new(Document::new(id)),
            seed: usize::from(id) as u64,
            rng: StdRng::seed_from_u64(usize::from(id) as u64),
        }
    }

    fn document(&self) -> &Document {
        &self.doc
    }

    fn document_mut(&mut self) -> &mut Document {
        &mut self.doc
    }
}

impl App {
    // create a todo in the document and return its id
    pub fn create_todo(&mut self, text: String) -> u32 {
        let mut tx = self.doc.transaction();
        let new_id: u32 = self.rng.gen();
        // let last_id = tx.keys(ROOT).next_back();
        // let new_id = if let Some(last_id) = last_id.and_then(|id| id.parse::<u32>().ok()) {
        //     last_id + 1
        // } else {
        //     1
        // };
        let todo = tx
            .put_object(ROOT, new_id.to_string(), ObjType::Map)
            .unwrap();
        tx.put(&todo, "completed", false).unwrap();
        tx.put(&todo, "text", text).unwrap();
        tx.commit();
        new_id
    }

    // toggle whether the given todo is active and return the new status
    pub fn toggle_active(&mut self, id: u32) -> bool {
        let mut tx = self.doc.transaction();
        if let Some((_, todo)) = tx.get(ROOT, id.to_string()).unwrap() {
            tx.put(&todo, "completed", true).unwrap();
            tx.commit();
            self.doc
                .get(&todo, "completed")
                .unwrap()
                .unwrap()
                .0
                .to_bool()
                .unwrap()
        } else {
            // missing todos can't be active
            false
        }
    }

    pub fn delete_todo(&mut self, id: u32) -> bool {
        let mut tx = self.doc.transaction();
        let is_present = tx.get(ROOT, id.to_string()).unwrap().is_some();
        tx.delete(ROOT, id.to_string()).unwrap();
        tx.commit();
        is_present
    }

    pub fn num_todos(&self) -> usize {
        self.doc.length(ROOT)
    }
}
