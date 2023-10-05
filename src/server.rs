use crate::exmethod::ExtendMethod;
use crate::http_methods::{copy, delete, exmove, get, head, mkcol, options, propfind, put};
use crate::util::{get_base_dir, get_current_user_rule, get_req_path, get_server_prefix};
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
    let mut resp = Response::new(Full::new(Bytes::from("")));
    let method = req.method().clone();
    if method == Method::OPTIONS {
        resp = options::handle_resp().await;
        return Ok(resp);
    }
    let headers = req.headers().clone();
    // Basic Authentication
    let current_user_rule = get_current_user_rule(&req);
    if method != Method::OPTIONS && current_user_rule.is_none() {
        *resp.status_mut() = StatusCode::UNAUTHORIZED;
        resp.headers_mut().insert(
            WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm='Restricted'"),
        );
        return Ok(resp);
    }
    // 要挂载的目录
    let base_dir = get_base_dir(&req);
    if base_dir.len() == 0 {
        *resp.status_mut() = StatusCode::NOT_FOUND;
        return Ok(resp);
    }
    // webdav 访问路径前缀
    let server_prefix = get_server_prefix(&req);
    let req_path = get_req_path(&req);
    let mut path = req_path.to_string();
    path.replace_range(0..server_prefix.len(), "");
    // 被访问资源绝对路径
    let file_path = Path::new(&base_dir).join(path.trim_start_matches('/'));
    if (method != Method::from(ExtendMethod::MKCOL)
        && method != Method::PUT
        && !file_path.exists()
        && !file_path.is_dir())
        || !req_path.starts_with(&server_prefix)
    {
        *resp.status_mut() = StatusCode::NOT_FOUND;
        return Ok(resp);
    }

    // 权限校验
    let permission = current_user_rule.unwrap().permission.to_uppercase();
    let mut has_permission = true;
    let read_methods = [
        Method::GET,
        Method::HEAD,
        Method::from(ExtendMethod::PROPFIND),
    ];
    if !read_methods.contains(&method) {
        if method == Method::DELETE {
            has_permission = permission.contains("D");
        } else {
            has_permission = permission.contains("W");
        }
    }
    if !has_permission {
        log::error!("111114");
        *resp.status_mut() = StatusCode::FORBIDDEN;
        return Ok(resp);
    }

    // 实现各个 HTTP 方法
    if method == Method::from(ExtendMethod::PROPFIND) {
        resp = propfind::handle_resp(&req, file_path).await;
    } else if method == Method::from(ExtendMethod::COPY) {
        resp = copy::handle_resp(&req, &file_path).await;
    } else if method == Method::from(ExtendMethod::MKCOL) {
        resp = mkcol::handle_resp(&file_path).await;
    } else if method == Method::from(ExtendMethod::MOVE) {
        resp = exmove::handle_resp(&req, &file_path).await;
    } else {
        match method {
            Method::GET => {
                resp = get::handle_resp(&req, &file_path).await;
            }
            Method::HEAD => {
                resp = head::handle_resp(&file_path).await;
            }
            Method::DELETE => {
                resp = delete::handle_resp(&file_path).await;
            }
            Method::PUT => {
                resp = put::handle_resp(req, &file_path).await.unwrap();
            }
            _ => {
                *resp.status_mut() = StatusCode::OK;
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

    if headers.contains_key(CONNECTION) {
        resp.headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    }
    log::info!("resp header: {:?}", resp.headers());
    Ok(resp)
}
