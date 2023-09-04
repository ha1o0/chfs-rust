use std::{fs, os::unix::prelude::MetadataExt};

use chrono::{DateTime, NaiveDateTime, Utc};
use hyper::{Body, HeaderMap, Request};
use urlencoding::{decode, encode};

pub fn get_range(req: &Request<Body>) -> &str {
    let mut result = "";
    if let Some(value) = get_header_value(req, "range") {
        result = value;
    }
    result
}

pub fn get_depth(req: &Request<Body>) -> &str {
    let mut result = "0";
    if let Some(value) = get_header_value(req, "depth") {
        result = value;
    }
    result
}

pub fn get_protocol(req: &Request<Body>) -> &str {
    let mut result = "http";
    if let Some(uri) = req.uri().scheme_str() {
        result = uri;
    }
    result
}

pub fn get_host(req: &Request<Body>) -> &str {
    let mut result = "";
    if let Some(value) = get_header_value(req, "host") {
        result = value;
    }
    result
}

pub fn get_req_path(req: &Request<Body>) -> String {
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

// pub fn generate_body(status: StatusCode) -> Response<Body> {
//     let resp;
//     match status {
//         StatusCode::NOT_FOUND => {
//             resp = Response::builder()
//                 .status(StatusCode::NOT_FOUND)
//                 .body(Body::empty())
//                 .unwrap();
//         }
//     }
//     resp
// }

pub fn get_header_value<'a>(req: &'a Request<Body>, header_name: &'a str) -> Option<&'a str> {
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

pub fn get_creation_date(file_path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        if let Ok(metadata) = fs::metadata(file_path) {
            if let Ok(created) = metadata.created() {
                // 将Windows文件时间格式化为RFC3339格式
                return created.format("%Y-%m-%dT%H:%M:%SZ").to_string();
            }
        }
    }

    #[cfg(all(not(target_os = "windows"), target_family = "unix"))]
    {
        if let Ok(metadata) = fs::metadata(file_path) {
            let created = metadata.ctime();

            // 将Unix时间戳格式化为RFC3339格式
            let formatted_date = {
                let secs = created as i64;
                let datetime = NaiveDateTime::from_timestamp_opt(secs, 0);
                datetime.expect("invalid or out-of-range datetime")
            };
            return formatted_date.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        }
    }

    "".to_string()
}
