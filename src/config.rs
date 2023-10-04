use lazy_static::lazy_static;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub path: String,
    pub log: String,
    pub user: String,
    pub pwd: String,
    pub mode: String,
    pub server_prefix: String,
    // config: String,
}

lazy_static! {
    static ref CONFIG: Config = load_config();
}

fn load_config() -> Config {
    let mut config = Config {
        port: 8000,
        path: "/".to_string(),
        log: "error".to_string(),
        user: "".to_string(),
        pwd: "".to_string(),
        mode: "".to_string(),
        server_prefix: "/webdav".to_string(),
        // config: "".to_string(),
    };

    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in args {
        let (key, value) = arg.split_once('=').unwrap();
        match key {
            "port" => config.port = value.parse().unwrap(),
            "path" => config.path = value.into(),
            "log" => config.log = value.into(),
            "user" => config.user = value.into(),
            "pwd" => config.pwd = value.into(),
            "mode" => config.mode = value.into(),
            "server_prefix" => config.server_prefix = value.into(),
            _ => {}
        }
    }

    config
}

pub fn get_config() -> &'static Config {
    &CONFIG
}

pub fn get_server_prefix() -> &'static str {
    let cfg = self::get_config();
    &cfg.server_prefix
}

pub fn get_base_dir() -> &'static str {
    let cfg = self::get_config();
    &cfg.path
}
