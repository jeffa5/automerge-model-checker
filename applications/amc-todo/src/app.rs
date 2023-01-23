use amc::application::DerefDocument;
use amc::application::Document;
use automerge::transaction::Transactable;
use automerge::ObjType;
use automerge::ROOT;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::hash::Hash;

/// The app that clients work with.
#[derive(Clone, Debug, Eq)]
pub struct AppState {
    doc: Document,
    random_ids: bool,
    seed: u64,
    rng: StdRng,
}

impl Hash for AppState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.doc.hash(state);
        self.seed.hash(state);
    }
}

impl PartialEq for AppState {
    fn eq(&self, other: &Self) -> bool {
        self.doc == other.doc && self.seed == other.seed
    }
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
    pub fn new(id: usize, random_ids: bool, initial_change: bool) -> Self {
        let seed = id as u64;
        let mut doc = Document::new(id);
        if initial_change {
            doc.with_initial_change(|_| {});
        }
        Self {
            doc,
            random_ids,
            seed,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    // create a todo in the document and return its id
    pub fn create_todo(&mut self, text: String) -> u32 {
        let mut tx = self.doc.transaction();
        let new_id: u32 = if self.random_ids {
            self.rng.gen()
        } else {
            let last_id = tx.keys(ROOT).next_back();
            if let Some(last_id) = last_id.and_then(|id| id.parse::<u32>().ok()) {
                last_id + 1
            } else {
                1
            }
        };
        let todo = tx
            .put_object(ROOT, new_id.to_string(), ObjType::Map)
            .unwrap();
        tx.put(&todo, "completed", false).unwrap();
        tx.put(&todo, "text", text).unwrap();
        tx.commit();
        new_id
    }

    pub fn update_text(&mut self, id: u32, text: String) -> bool {
        let mut tx = self.doc.transaction();
        if let Some((_, todo)) = tx.get(ROOT, id.to_string()).unwrap() {
            tx.put(todo, "text", text).unwrap();
            tx.commit();
            true
        } else {
            false
        }
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

    pub fn list_todos(&self) -> Vec<u32> {
        self.doc
            .map_range(ROOT, ..)
            .map(|(k, _, _)| k.parse().unwrap())
            .collect()
    }
}
