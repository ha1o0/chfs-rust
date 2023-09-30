use std::{
    fs::{self},
    path::PathBuf,
};

use http_body_util::Full;
use hyper::{body::Bytes, Response, StatusCode};

use crate::util::map_io_result;

pub async fn handle_resp(dir_path: &PathBuf) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::from("")));
    let status_code = map_io_result(fs::create_dir_all(dir_path), StatusCode::CREATED);
    response
        .headers_mut()
        .insert("Content-Length", format!("{}", 0).parse().unwrap());
    *response.status_mut() = status_code;
    response
}
