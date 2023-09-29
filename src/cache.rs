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

pub enum CrType {
    INCR,
    DECR,
}

pub fn handle_cr(key: &str, cr_type: CrType) {
    let mut cache = get_lock();
    let mut new_number = 0;
    let original_string = self::get(key);
    if original_string.is_none() {
        cache.insert(key.to_string(), new_number.to_string());
        return;
    }
    if let Ok(original_number) = original_string.unwrap().parse::<u32>() {
        match cr_type {
            CrType::INCR => {
                new_number = original_number + 1;
            }
            CrType::DECR => {
                new_number = original_number - 1;
            }
        }
        cache.insert(key.to_string(), new_number.to_string());
    }
}

pub fn incr(key: &str) {
    self::handle_cr(key, CrType::INCR)
}

pub fn decr(key: &str) {
    self::handle_cr(key, CrType::DECR)
}
