use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub id: u32,
}

#[derive(Default)]
pub struct IdentTable {
    str_to_id: HashMap<Box<str>, u32>,
}

impl IdentTable {
    pub fn new() -> IdentTable {
        Default::default()
    }

    pub fn insert(&mut self, key: &str) -> Symbol {
        if let Some(&id) = self.str_to_id.get(key) {
            return Symbol { id };
        }

        let id = u32::try_from(self.str_to_id.len()).unwrap();
        self.str_to_id.insert(Box::from(key), id);

        Symbol { id }
    }

    pub fn len(&self) -> usize {
        self.str_to_id.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.str_to_id.is_empty()
    }
}
