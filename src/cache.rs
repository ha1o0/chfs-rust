use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

lazy_static::lazy_static! {
    static ref CACHE: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

fn get_cache_lock() -> MutexGuard<'static, HashMap<String, String>> {
    CACHE.lock().unwrap()
}

pub fn set_cache(key: &str, value: &str) {
    let mut cache = get_cache_lock();
    cache.insert(key.to_string(), value.to_string());
}

pub fn get_cache(key: &str) -> Option<String> {
    let cache = get_cache_lock();
    cache.get(key).cloned()
}

pub fn exist_cache(key: &str) -> bool {
    let cache = get_cache_lock();
    cache.contains_key(key)
}
