use std::{
    collections::HashMap,
    io::{self},
};

use chrono::{DateTime, Utc};
use hyper::{body::Incoming, HeaderMap, Request, StatusCode};
use urlencoding::{decode, encode};

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

pub fn map_io_result(result: io::Result<()>, success_status: StatusCode) -> StatusCode {
    match result {
        Ok(_) => success_status,

        Err(ref err) => ERROR_KIND_TO_STATUS_CODE
            .get(&err.kind())
            .cloned()
            .unwrap_or(DEFAULT_ERROR_STATUS),
    }
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
