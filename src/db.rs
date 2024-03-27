use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default)]
pub struct Database {
    pub store: HashMap<String, SetObject>,
}

impl Database {
    pub fn new() -> Database {
        Database {
            store: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str, ttl: Option<Duration>) -> Option<SetObject> {
        match ttl {
            Some(ttl) => {
                let expiry_time = Instant::now()
                    .checked_add(ttl)
                    .expect("Failed to store ttl");
                let set_object = SetObject::new(value.to_string(), Some(expiry_time));
                self.store.insert(key.to_string(), set_object)
            }
            None => {
                let set_object = SetObject::new(value.to_string(), None);
                self.store.insert(key.to_string(), set_object)
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<SetObject> {
        let result = self.store.get(key);

        result.cloned()
    }
}

#[derive(Clone, Debug)]
pub struct SetObject {
    pub value: String,
    pub duration: Option<Instant>,
}

impl SetObject {
    pub fn new(value: String, duration: Option<Instant>) -> Self {
        SetObject { value, duration }
    }
}
