use crate::cache::exist;
use crate::config;
use crate::util::{encode_uri, format_date_time, get_header, get_req_path};
use chrono::Local;
use http::HeaderValue;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::header::{CONNECTION, WWW_AUTHENTICATE};
use hyper::http;
use hyper::{Method, Request, Response, StatusCode};
use md5;
use mime_guess::from_path;
use std::convert::Infallible;
use std::fs::{self, File};
use std::io::{self, Read, Seek};
use std::path::{Path, PathBuf};

pub async fn handle_request(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    log::info!("req: {:?}", &req);
    let method = req.method();
    let mut resp = Response::new(Full::new(Bytes::from("")));
    let auth_header = get_header(&req, "Authorization", "");
    // Basic Authentication
    if method != Method::OPTIONS
        && method != Method::HEAD
        && exist("need_login")
        && !exist(auth_header)
    {
        *resp.status_mut() = StatusCode::UNAUTHORIZED;
        resp.headers_mut().insert(
            WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm='Restricted'"),
        );
        return Ok(resp);
    }
    // webdav 访问路径前缀
    let server_prefix = "/webdav";
    let req_path = get_req_path(&req);
    // 要挂载的目录
    let cfg = config::get_config();
    let base_dir = cfg.path.as_str();
    let mut path = req_path.to_string();
    path.replace_range(0..server_prefix.len(), "");
    // 被访问资源绝对路径
    let file_path = Path::new(&base_dir).join(path.trim_start_matches('/'));
    if !file_path.exists() && !file_path.is_dir() || !req_path.starts_with("/webdav") {
        *resp.status_mut() = StatusCode::NOT_FOUND;
        return Ok(resp);
    }

    // 实现各个 HTTP 方法

    if method == Method::from_bytes(b"PROPFIND").unwrap() {
        // log::info!("prop: {}, {:?}", path, file_path);
        let multistatus_xml = handle_propfind_resp(&req, file_path, server_prefix, base_dir);
        resp = Response::builder()
            .status(StatusCode::MULTI_STATUS)
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(Full::new(Bytes::from(multistatus_xml)))
            .unwrap();
    } else if method == Method::from_bytes(b"COPY").unwrap() {
        resp = Response::new(Full::new(Bytes::from("Hello, Webdav, COPY")));
    } else {
        match method {
            &Method::OPTIONS => {
                let allow_methods = "OPTIONS, GET, HEAD, POST, PUT, DELETE, PROPFIND";
                resp = Response::builder()
                    .status(StatusCode::OK)
                    .header("Allow", allow_methods)
                    .header("DAV", "1")
                    .body(Full::new(Bytes::from("")))
                    .unwrap();
            }
            &Method::GET => {
                resp = handle_get_resp(&req, &file_path).await;
            }
            &Method::PUT => {
                resp = Response::new(Full::new(Bytes::from("Hello, Webdav, PUT")));
            }
            _ => {
                resp = Response::new(Full::new(Bytes::from("Hello, Webdav")));
            }
        }
    }

    if method != Method::GET {
        log::info!(
            "{}---resp: {:?}",
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            resp
        );
    } else {
        log::info!("{:?}", resp.headers());
    }

    // 如果客户端请求keep-alive,也在响应中带上
    if req.headers().contains_key(CONNECTION) {
        resp.headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    }
    log::info!("resp header: {:?}", resp.headers());
    Ok(resp)
}

fn handle_propfind_resp(
    req: &Request<Incoming>,
    file_path: PathBuf,
    server_prefix: &str,
    base_dir: &str,
) -> String {
    let depth = get_header(req, "depth", "0");
    let mut multistatus_xml = String::new();
    multistatus_xml.push_str(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    multistatus_xml.push_str(r#"<D:multistatus xmlns:D="DAV:">"#);
    if depth == "0" {
        generate_content_xml(
            req,
            &mut multistatus_xml,
            file_path,
            base_dir,
            server_prefix,
        );
    } else {
        for entry in fs::read_dir(&file_path).unwrap() {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                // log::info!("sub: {:?}, {}", entry_path, base_dir);
                generate_content_xml(
                    req,
                    &mut multistatus_xml,
                    entry_path,
                    base_dir,
                    server_prefix,
                );
            }
        }
    }

    multistatus_xml.push_str("</D:multistatus>\n");
    multistatus_xml
}

async fn handle_get_resp(req: &Request<Incoming>, file_path: &PathBuf) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::from("")));
    let range = get_header(req, "range", "");
    if range.len() > 0 {
        let mut file = File::open(file_path).unwrap();
        let metadata = file.metadata().unwrap();
        let file_len = metadata.len();
        let mime_type = from_path(file_path).first_or_octet_stream();
        response
            .headers_mut()
            .insert("Content-Type", format!("{}", mime_type).parse().unwrap());
        let mut start = 0;
        let end: u64;
        let bounds = range.strip_prefix("bytes=").unwrap();
        let max_chunk_size = (file_len / 1024) * 50;
        // log::info!("file_len: {}, max chunk size: {}", file_len, max_chunk_size);
        if bounds.contains("-") {
            let parts = bounds.split('-').collect::<Vec<_>>();
            start = parts[0].parse::<u64>().unwrap();
            if parts.len() == 1 || parts[1] == "" {
                end = file_len - 1;
            } else {
                end = parts[1].parse::<u64>().unwrap();
            }
        } else {
            end = bounds.parse::<u64>().unwrap();
        }
        if (end - start) > max_chunk_size {
            *response.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
        } else {
            file.seek(io::SeekFrom::Start(start)).unwrap();
            let mut stream = Vec::with_capacity((end - start + 1) as usize);
            file.take((end - start + 1) as u64)
                .read_to_end(&mut stream)
                .unwrap();
            *response.status_mut() = StatusCode::PARTIAL_CONTENT;
            response.headers_mut().insert(
                "Content-Range",
                format!("bytes {}-{}/{}", start, end, file_len)
                    .parse()
                    .unwrap(),
            );
            *response.body_mut() = Full::new(Bytes::from(stream));
        }
    } else {
        response = handle_get_all_resp(file_path).await;
    }

    response
}

async fn handle_get_all_resp(file_path: &PathBuf) -> Response<Full<Bytes>> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            log::error!("not found file");
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("")))
                .unwrap();
        }
    };
    let mime_type = from_path(file_path).first_or_octet_stream();
    // 读取文件内容
    let mut buffer = Vec::new();
    match file.take(usize::MAX as u64).read_to_end(&mut buffer) {
        Ok(_) => {
            let mut response = Response::new(Full::new(Bytes::from("")));
            response.headers_mut().insert(
                "Content-Type",
                format!("{}", mime_type.as_ref()).parse().unwrap(),
            );
            *response.body_mut() = Full::new(Bytes::from(buffer));
            response
        }
        Err(_) => {
            log::error!("not write buffer");
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from("")))
                .unwrap();
        }
    }
}

fn generate_content_xml(
    _req: &Request<Incoming>,
    multistatus_xml: &mut String,
    entry_path: PathBuf,
    base_dir: &str,
    server_prefix: &str,
) {
    let mut server_prefix_with_suffix = server_prefix.to_string();
    if !server_prefix_with_suffix.ends_with("/") {
        server_prefix_with_suffix += "/";
    }
    let mut relative_path = entry_path.to_string_lossy().to_owned().to_string();
    relative_path.replace_range(0..base_dir.len(), &server_prefix_with_suffix);
    // log::info!(
    //     "inner: {}---{}==={:?}",
    //     server_prefix_with_suffix, base_dir, entry_path
    // );
    multistatus_xml.push_str("<D:response>\n");
    let encode_relative_path = encode_uri(&relative_path);
    multistatus_xml
        .push_str(format!("<D:href>{}</D:href>\n", format!("{}", encode_relative_path)).as_str());
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
        multistatus_xml.push_str(format!("<D:supportedlock/>\n").as_str());
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
    // let creationdate = get_creation_date(&entry_path.to_string_lossy());
    // if creationdate.len() > 0 {
    //     multistatus_xml
    //         .push_str(format!("<D:creationdate>{}</D:creationdate>\n", creationdate).as_str());
    // }
    multistatus_xml.push_str(
        format!(
            "<D:displayname>{}</D:displayname>\n",
            entry_path.file_name().unwrap().to_string_lossy()
        )
        .as_str(),
    );
    multistatus_xml.push_str("</D:prop>\n");
    multistatus_xml.push_str("<D:status>HTTP/1.1 200 OK</D:status>\n");
    multistatus_xml.push_str("</D:propstat>\n");
    multistatus_xml.push_str("</D:response>\n");
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
