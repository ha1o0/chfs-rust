use hyper::{Body, Response, StatusCode};
use std::{
    fs::{self},
    path::PathBuf,
};

use crate::util::map_io_result;

pub async fn handle_resp(dir_path: &PathBuf) -> Response<Body> {
    let mut response = Response::new(Body::from(""));
    let status_code = map_io_result(fs::create_dir_all(dir_path), StatusCode::CREATED);
    response
        .headers_mut()
        .insert("Content-Length", format!("{}", 0).parse().unwrap());
    *response.status_mut() = status_code;
    response
}
