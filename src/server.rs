use crate::cache::exist;
use crate::config;
use crate::exmethod::ExtendMethod;
use crate::http_methods::{get, head, mkcol, options, propfind};
use crate::util::{get_header, get_req_path};
use chrono::Local;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::header::{CONNECTION, WWW_AUTHENTICATE};
use hyper::http::HeaderValue;
use hyper::{Method, Request, Response, StatusCode};
use std::convert::Infallible;
use std::path::Path;

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
    if (method != Method::from(ExtendMethod::MKCOL) && !file_path.exists() && !file_path.is_dir())
        || !req_path.starts_with("/webdav")
    {
        *resp.status_mut() = StatusCode::NOT_FOUND;
        return Ok(resp);
    }

    // 实现各个 HTTP 方法
    if method == Method::from(ExtendMethod::PROPFIND) {
        resp = propfind::handle_resp(&req, file_path, server_prefix, base_dir);
    } else if method == Method::from(ExtendMethod::COPY) {
        resp = Response::new(Full::new(Bytes::from("Hello, Webdav, COPY")));
    } else if method == Method::from(ExtendMethod::MKCOL) {
        resp = mkcol::handle_resp(&file_path).await;
    } else {
        match method {
            &Method::OPTIONS => {
                resp = options::handle_resp();
            }
            &Method::GET => {
                resp = get::handle_resp(&req, &file_path).await;
            }
            &Method::HEAD => {
                resp = head::handle_resp(&file_path).await;
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

    if req.headers().contains_key(CONNECTION) {
        resp.headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    }
    log::info!("resp header: {:?}", resp.headers());
    Ok(resp)
}
