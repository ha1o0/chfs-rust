use std::{
    collections::HashMap, io::{self, Error}, path::PathBuf
};

use chrono::{DateTime, Utc};
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{body::{Bytes, Incoming}, HeaderMap, Request, StatusCode};
use url::Url;
use urlencoding::{decode, encode};

use crate::{
    cache::get,
    config::{self, Rule},
};

pub fn get_header<'a>(
    req: &'a Request<Incoming>,
    name: &'a str,
    default_value: &'a str,
) -> &'a str {
    let mut result = default_value;
    if let Some(value) = get_header_value(req, name) {
        result = value;
    }
    result
}

pub fn get_protocol(req: &Request<Incoming>) -> &str {
    let mut result = "http";
    if let Some(uri) = req.uri().scheme_str() {
        result = uri;
    }
    result
}

pub fn get_req_path(req: &Request<Incoming>) -> String {
    let path = req.uri().path();
    decode_uri(path)
    // encode_uri(path)
}

pub fn decode_uri(uri: &str) -> String {
    decode(uri).unwrap().to_string()
}

pub fn encode_uri(uri: &str) -> String {
    encode(uri).to_string().replace("%2F", "/")
}

pub fn decode_path(path: &PathBuf) -> String {
    let uri = path.to_str().unwrap();
    decode(uri).unwrap().to_string()
}

pub fn encode_path(path: &PathBuf) -> String {
    let uri = path.to_str().unwrap();
    encode(uri).to_string().replace("%2F", "/")
}

pub fn extract_relative_path(full_url: &str, domain: &str) -> Option<String> {
    // 解析完整的 URL
    if let Ok(url) = Url::parse(full_url) {
        // 获取 URL 的主机部分（域名）
        if let Some(host) = url.host_str() {
            if let Some(port) = url.port_or_known_default() {
                let host_port = format!("{}:{}", host, port);
                // 检查主机是否与给定的域名匹配
                if host_port == domain {
                    // 提取相对路径并克隆到一个 String 中
                    let path = url.path().to_string();
                    return Some(path);
                }
            }
        }
    }
    None
}

pub fn get_header_value<'a>(req: &'a Request<Incoming>, header_name: &'a str) -> Option<&'a str> {
    // 获取HTTP请求的头部
    let headers: &HeaderMap = req.headers();
    // 使用header_name获取特定的头部值
    if let Some(header_value) = headers.get(header_name) {
        // 将头部值转换为字符串
        if let Ok(header_str) = header_value.to_str() {
            return Some(header_str);
        }
    }
    None
}

pub fn is_guest_by_req(req: &Request<Incoming>) -> bool {
    if let Some(guest_server_prefix) = get("guest_server_prefix") {
        return req.uri().to_string().starts_with(&guest_server_prefix);
    }
    false
}

pub fn get_server_prefix(req: &Request<Incoming>) -> String {
    if let Some(rule) = self::get_current_user_rule(req) {
        return rule.server_prefix.to_string();
    }
    "".to_string()
}

pub fn get_base_dir(req: &Request<Incoming>) -> String {
    if let Some(rule) = self::get_current_user_rule(req) {
        return rule.path.to_string();
    }
    "".to_string()
}

pub fn get_current_user_rule(req: &Request<Incoming>) -> Option<&Rule> {
    let cfg = config::get_config();
    let mut user_key = "guest";
    if !self::is_guest_by_req(req) {
        user_key = get_header(req, "Authorization", "");
    }
    cfg.user_rule.get(user_key)
}

// 格式化日期时间为RFC1123格式
pub fn format_date_time(dt: std::time::SystemTime) -> String {
    DateTime::<Utc>::from(dt)
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string()
}

lazy_static::lazy_static! {
    static ref ERROR_KIND_TO_STATUS_CODE: HashMap<io::ErrorKind, StatusCode> = {
        let mut m = HashMap::new();
        m.insert(io::ErrorKind::PermissionDenied, StatusCode::FORBIDDEN);
        m.insert(io::ErrorKind::ConnectionRefused, StatusCode::BAD_GATEWAY);
        m.insert(io::ErrorKind::NotFound, StatusCode::NOT_FOUND);
        m.insert(io::ErrorKind::AlreadyExists, StatusCode::CONFLICT);
        m
    };
}

const DEFAULT_ERROR_STATUS: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;

pub fn map_io_result<T>(result: io::Result<T>, success_status: StatusCode) -> StatusCode {
    match result {
        Ok(_) => success_status,

        Err(ref err) => ERROR_KIND_TO_STATUS_CODE
            .get(&err.kind())
            .cloned()
            .unwrap_or(DEFAULT_ERROR_STATUS),
    }
}

pub fn empty() -> BoxBody<Bytes, Error> {
    Empty::<Bytes>::new().map_err(|e| match e {}).boxed()
}

pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, Error> {
    Full::new(chunk.into()).map_err(|e| match e {}).boxed()
}

// pub fn get_creation_date(file_path: &str) -> String {
//     let os = std::env::consts::OS;
//     if os != "linux" && os != "macos" {
//         return "".to_string();
//     }
//     if let Ok(metadata) = fs::metadata(file_path) {
//         let created = metadata.ctime();

//         // 将Unix时间戳格式化为RFC3339格式
//         let formatted_date = {
//             let secs = created as i64;
//             let datetime = NaiveDateTime::from_timestamp_opt(secs, 0);
//             datetime.expect("invalid or out-of-range datetime")
//         };
//         return formatted_date.format("%Y-%m-%dT%H:%M:%SZ").to_string();
//     }

//     "".to_string()
// }
