use std::{
    fs::{self},
    path::PathBuf,
};

use http_body_util::Full;
use hyper::{body::Bytes, Response, StatusCode};

use crate::util::map_io_result;

pub async fn handle_resp(path: &PathBuf) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::from("")));
    let file_err = fs::remove_file(path);
    if file_err.is_ok() {
        let status_code = map_io_result(file_err, StatusCode::NO_CONTENT);
        *response.status_mut() = status_code;
        return response;
    }
    let dir_err = fs::remove_dir(path);
    let status_code = map_io_result(dir_err, StatusCode::NO_CONTENT);
    response
        .headers_mut()
        .insert("Content-Length", format!("{}", 0).parse().unwrap());
    *response.status_mut() = status_code;
    response
}
