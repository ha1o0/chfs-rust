use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

lazy_static::lazy_static! {
    static ref CACHE: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

fn get_lock() -> MutexGuard<'static, HashMap<String, String>> {
    CACHE.lock().unwrap()
}

pub fn set(key: &str, value: &str) {
    let mut cache = get_lock();
    cache.insert(key.to_string(), value.to_string());
}

pub fn get(key: &str) -> Option<String> {
    let cache = get_lock();
    cache.get(key).cloned()
}

pub fn exist(key: &str) -> bool {
    let cache = get_lock();
    cache.contains_key(key)
}
