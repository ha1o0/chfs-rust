use chrono::{DateTime, Utc};
use hyper::{Body, HeaderMap, Request};
use urlencoding::{decode, encode};

pub fn get_depth(req: &Request<Body>) -> String {
    let depth;
    if let Some(depth_value) = get_header_value(req, "depth") {
        depth = depth_value;
    } else {
        depth = "0".to_string();
    }
    depth
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
    encode(uri).to_string()
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

pub fn get_header_value(req: &Request<Body>, header_name: &str) -> Option<String> {
    // 获取HTTP请求的头部
    let headers: &HeaderMap = req.headers();
    // 使用header_name获取特定的头部值
    if let Some(header_value) = headers.get(header_name) {
        // 将头部值转换为字符串
        if let Ok(header_str) = header_value.to_str() {
            return Some(header_str.to_string());
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
