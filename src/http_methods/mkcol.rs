use std::{
    fs::{self},
    io::ErrorKind,
    path::PathBuf,
};

use http_body_util::Full;
use hyper::{body::Bytes, Response, StatusCode};

pub async fn handle_resp(dir_path: &PathBuf) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::from("")));
    let mut status_code = StatusCode::CREATED;
    if let Err(e) = fs::create_dir_all(dir_path) {
        if e.kind() == ErrorKind::PermissionDenied {
            status_code = StatusCode::FORBIDDEN;
        } else if e.kind() == ErrorKind::AlreadyExists {
            status_code = StatusCode::METHOD_NOT_ALLOWED;
        } else {
            status_code = StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    response
        .headers_mut()
        .insert("Content-Length", format!("{}", 0).parse().unwrap());
    *response.status_mut() = status_code;
    response
}
