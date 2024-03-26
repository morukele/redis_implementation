use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug, Default)]
pub struct Db {
    pub store: Arc<Mutex<HashMap<String, String>>>,
}

impl Db {
    pub fn new() -> Db {
        Db {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: &str, value: &str) -> Option<String> {
        let result = self
            .store
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());

        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        todo!()
    }
}
