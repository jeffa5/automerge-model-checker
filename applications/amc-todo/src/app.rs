use amc::application::DerefDocument;
use amc::application::Document;
use automerge::transaction::Transactable;
use automerge::transaction::Transaction;
use automerge::transaction::UnObserved;
use automerge::ObjId;
use automerge::ObjType;
use automerge::ReadDoc;
use automerge::ROOT;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use smol_str::SmolStr;
use std::hash::Hash;
use tinyvec::TinyVec;

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
            doc.with_initial_change(|tx| {
                Self::todos_map_tx(tx);
            });
        }
        Self {
            doc,
            random_ids,
            seed,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn todos_map_tx(tx: &mut Transaction<UnObserved>) -> ObjId {
        if let Ok(Some((_, id))) = tx.get(ROOT, "todos") {
            return id;
        }
        
        tx.put_object(ROOT, "todos", ObjType::Map).unwrap()
    }

    fn todos_map(&self) -> Option<ObjId> {
        if let Ok(Some((_, id))) = self.doc.get(ROOT, "todos") {
            return Some(id);
        }
        None
    }

    // create a todo in the document and return its id
    pub fn create_todo(&mut self, text: SmolStr) -> u32 {
        let mut tx = self.doc.transaction();
        let todos_map = Self::todos_map_tx(&mut tx);
        let new_id: u32 = if self.random_ids {
            self.rng.gen()
        } else {
            let last_id = tx.keys(&todos_map).next_back();
            if let Some(last_id) = last_id.and_then(|id| id.parse::<u32>().ok()) {
                last_id + 1
            } else {
                1
            }
        };
        let todo = tx
            .put_object(&todos_map, new_id.to_string(), ObjType::Map)
            .unwrap();
        tx.put(&todo, "completed", false).unwrap();
        tx.put(&todo, "text", text.as_str()).unwrap();
        tx.commit();
        new_id
    }

    pub fn update_text(&mut self, id: u32, text: SmolStr) -> bool {
        let mut tx = self.doc.transaction();
        let todos_map = Self::todos_map_tx(&mut tx);
        if let Some((_, todo)) = tx.get(&todos_map, id.to_string()).unwrap() {
            tx.put(todo, "text", text.as_str()).unwrap();
            tx.commit();
            true
        } else {
            false
        }
    }

    // toggle whether the given todo is active and return the new status
    pub fn toggle_active(&mut self, id: u32) -> bool {
        let mut tx = self.doc.transaction();
        let todos_map = Self::todos_map_tx(&mut tx);
        if let Some((_, todo)) = tx.get(&todos_map, id.to_string()).unwrap() {
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
        let todos_map = Self::todos_map_tx(&mut tx);
        let is_present = tx.get(&todos_map, id.to_string()).unwrap().is_some();
        tx.delete(&todos_map, id.to_string()).unwrap();
        tx.commit();
        is_present
    }

    pub fn num_todos(&self) -> usize {
        let todos_map = self.todos_map();
        todos_map.map(|m| self.doc.length(m)).unwrap_or_default()
    }

    pub fn list_todos(&self) -> TinyVec<[u32; 4]> {
        let todos_map = self.todos_map();
        todos_map
            .map(|m| {
                self.doc
                    .map_range(&m, ..)
                    .map(|(k, _, _)| k.parse().unwrap())
                    .collect()
            })
            .unwrap_or_default()
    }
}
