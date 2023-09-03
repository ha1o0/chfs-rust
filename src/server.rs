use chrono::{DateTime, Utc};
use hyper::{Body, HeaderMap, Method, Request, Response, StatusCode};
use md5;
use mime_guess::from_path;
use std::convert::Infallible;
use std::fs::{self};
use std::path::{Path, PathBuf};

pub async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let resp;
    // webdav 访问路径前缀
    let server_prefix = "/webdav";
    let req_path = get_req_path(&req);
    // 要挂载的目录
    let base_dir = "/Users/halo/webdav/";
    let mut path = req_path.to_string();
    path.replace_range(0..server_prefix.len(), "");
    // 被访问资源绝对路径
    let full_path = Path::new(base_dir).join(path.trim_start_matches('/'));
    if !full_path.exists() && !full_path.is_dir() {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap());
    }
    println!("req: {:?}", &req);
    if !req_path.starts_with("/webdav") {
        resp = Response::new(Body::from("Hello"));
    } else {
        let method = req.method();
        if method == Method::from_bytes(b"PROPFIND").unwrap() {
            let multistatus_xml = handle_propfind_resp(&req, path, full_path, server_prefix);
            resp = Response::builder()
                .status(StatusCode::MULTI_STATUS)
                .header("Content-Type", "application/xml; charset=utf-8")
                .body(Body::from(multistatus_xml))
                .unwrap();
        } else if method == Method::from_bytes(b"COPY").unwrap() {
            resp = Response::new(Body::from("Hello, Webdav, COPY"));
        } else {
            match req.method() {
                &Method::OPTIONS => {
                    let allow_methods = "OPTIONS, GET, HEAD, POST, PUT, DELETE, PROPFIND";
                    resp = Response::builder()
                        .status(StatusCode::OK)
                        .header("Allow", allow_methods)
                        .header("DAV", "1")
                        .body(Body::empty())
                        .unwrap();
                }
                &Method::GET => {
                    resp = Response::new(Body::from("Hello, Webdav, GET"));
                }
                &Method::PUT => {
                    resp = Response::new(Body::from("Hello, Webdav, PUT"));
                }
                _ => {
                    resp = Response::new(Body::from("Hello, Webdav"));
                }
            }
        }
    }
    println!("resp: {:?}", resp);
    Ok(resp)
}

fn get_depth(req: &Request<Body>) -> String {
    let depth;
    if let Some(depth_value) = get_header_value(req, "depth") {
        depth = depth_value;
    } else {
        depth = "0".to_string();
    }
    depth
}

fn get_req_path(req: &Request<Body>) -> String {
    req.uri().path().to_string()
}

fn handle_propfind_resp(
    req: &Request<Body>,
    path: String,
    full_path: PathBuf,
    server_prefix: &str,
) -> String {
    let depth = get_depth(req);
    let mut multistatus_xml = String::new();
    multistatus_xml.push_str(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    multistatus_xml.push_str(r#"<D:multistatus xmlns:D="DAV:">"#);
    if depth == "0" {
        generate_content_xml(&mut multistatus_xml, full_path, path, server_prefix.clone());
    } else {
        for entry in fs::read_dir(&full_path).unwrap() {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let entry_name = entry.file_name().to_string_lossy().into_owned();
                generate_content_xml(
                    &mut multistatus_xml,
                    entry_path,
                    entry_name,
                    server_prefix.clone(),
                );
            }
        }
    }

    multistatus_xml.push_str("</D:multistatus>\n");
    multistatus_xml
}

fn generate_content_xml(
    multistatus_xml: &mut String,
    entry_path: PathBuf,
    entry_name: String,
    server_prefix: &str,
) {
    let mut server_prefix_with_suffix = server_prefix.to_string();
    if !server_prefix_with_suffix.ends_with("/") {
        server_prefix_with_suffix += "/";
    }
    multistatus_xml.push_str("<D:response>\n");
    multistatus_xml.push_str(
        format!(
            "<D:href>{}</D:href>\n",
            format!("{}{}", server_prefix_with_suffix, entry_name)
        )
        .as_str(),
    );
    multistatus_xml.push_str("<D:propstat>\n");
    multistatus_xml.push_str("<D:prop>\n");

    let is_dir = entry_path.is_dir();
    let metadata = fs::metadata(&entry_path).unwrap();
    let last_modified = metadata.modified().unwrap();
    if is_dir {
        multistatus_xml.push_str("<D:resourcetype><D:collection/></D:resourcetype>\n");
    } else {
        let mime_type = from_path(&entry_path).first_or_octet_stream().to_string();
        let content_length = metadata.len();
        multistatus_xml.push_str(format!("<D:resourcetype/>\n").as_str());
        multistatus_xml.push_str(
            format!(
                "<D:getcontentlength>{}</D:getcontentlength>\n",
                content_length
            )
            .as_str(),
        );
        multistatus_xml
            .push_str(format!("<D:getcontenttype>{}</D:getcontenttype>\n", mime_type).as_str());
    }
    let mtime = last_modified.duration_since(std::time::UNIX_EPOCH).unwrap();
    let mtime_secs = mtime.as_secs();
    let etag = md5::compute(mtime_secs.to_string());
    multistatus_xml.push_str(format!("<D:getetag>{:?}</D:getetag>\n", etag).as_str());
    multistatus_xml.push_str(
        format!(
            "<D:getlastmodified>{}</D:getlastmodified>\n",
            format_date_time(last_modified)
        )
        .as_str(),
    );
    multistatus_xml.push_str("<D:displayname/>\n");
    multistatus_xml.push_str("</D:prop>\n");
    multistatus_xml.push_str("<D:status>HTTP/1.1 200 OK</D:status>\n");
    multistatus_xml.push_str("</D:propstat>\n");
    multistatus_xml.push_str("</D:response>\n");
}

fn get_header_value(req: &Request<Body>, header_name: &str) -> Option<String> {
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
fn format_date_time(dt: std::time::SystemTime) -> String {
    DateTime::<Utc>::from(dt)
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string()
}

pub enum DavMethod {
    // 将资源从一个URI复制到另一个URI
    COPY,
    // 锁定一个资源。WebDAV支持共享锁和互斥锁。
    LOCK,
    // 创建集合（即目录）
    MKCOL,
    // 将资源从一个URI移动到另一个URI
    MOVE,
    // 从Web资源中检索以XML格式存储的属性。它也被重载，以允许一个检索远程系统的集合结构（也叫目录层次结构）
    PROPFIND,
    // 在单个原子性动作中更改和删除资源的多个属性
    PROPPATCH,
    // 解除资源的锁定
    UNLOCK,
}
