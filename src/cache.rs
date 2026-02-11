use std::collections::HashMap;

pub struct DataCache {
    pub key: String,
    pub store: HashMap<String, String>,
}

fn redact_key(key: &str) -> String {
    let shown: String = key.chars().take(4).collect();
    format!("{shown}***")
}

impl DataCache {
    pub fn new(key: &str) -> Self {
        println!("Initializing cache with key {}", redact_key(key));
        DataCache {
            key: key.to_string(),
            store: HashMap::new(),
        }
    }

    pub fn store(&mut self, k: &str, v: &str) {
        println!("Storing {} => {} in cache {}", k, v, redact_key(&self.key));
        self.store.insert(k.to_string(), v.to_string());
    }

    pub fn retrieve(&self, k: &str) -> Option<&String> {
        self.store.get(k)
    }
}

#[cfg(test)]
mod tests {
    use super::DataCache;

    #[test]
    fn cache_store_and_retrieve_round_trip() {
        let mut cache = DataCache::new("CACHE_KEY");
        cache.store("last_temp", "32.0C");

        assert_eq!(cache.retrieve("last_temp"), Some(&"32.0C".to_string()));
        assert_eq!(cache.retrieve("missing"), None);
    }
}
