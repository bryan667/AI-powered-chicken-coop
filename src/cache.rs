use std::collections::HashMap;

pub struct DataCache {
    pub key: String,
    pub store: HashMap<String, String>,
}

impl DataCache {
    pub fn new(key: &str) -> Self {
        println!("Initializing cache with key {}", key);
        DataCache { key: key.to_string(), store: HashMap::new() }
    }

    pub fn store(&mut self, k: &str, v: &str) {
        println!("Storing {} => {} in cache {}", k, v, self.key);
        self.store.insert(k.to_string(), v.to_string());
    }

    pub fn retrieve(&self, k: &str) -> Option<&String> {
        self.store.get(k)
    }
}
