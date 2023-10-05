use std::{collections::HashMap, env, fs};

use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;

use crate::cache::set;

lazy_static! {
    static ref CONFIG: Config = load_config();
}

pub fn get_config() -> &'static Config {
    &CONFIG
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub log: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub user_rule: HashMap<String, Rule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub permission: String,
    #[serde(default)]
    pub server_prefix: String,
}

fn load_config() -> Config {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut config_json_path = "".to_string();
    for arg in args {
        let (key, value) = &arg.split_once('=').unwrap();
        match key {
            &"config" => config_json_path = value.to_string(),
            _ => {}
        }
    }
    let json_str = &fs::read_to_string(config_json_path).unwrap();
    let mut config: Config = serde_json::from_str(json_str).unwrap();
    init_user(&mut config);
    config
}

pub fn init_user(config: &mut Config) {
    let rules = &config.rules;
    for rule in rules {
        let server_prefix = &rule.server_prefix;
        let user = &rule.user;
        let password = &rule.password;
        // guest is no user and no password
        if user.len() > 0 && password.len() > 0 {
            let b64 = general_purpose::STANDARD.encode(user.to_string() + ":" + &password);
            let auth_b64 = &("Basic ".to_string() + &b64.to_string());
            config.user_rule.insert(auth_b64.to_string(), rule.clone());
        } else {
            config.user_rule.insert("guest".to_string(), rule.clone());
            set("guest_server_prefix", &server_prefix);
        }
    }
}
