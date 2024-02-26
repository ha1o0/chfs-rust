use std::{
    fs, path::PathBuf
};

use http_body_util::combinators::BoxBody;
use hyper::{body::Bytes, Response, StatusCode};

use crate::util::{empty, map_io_result};

pub async fn handle_resp(path: &PathBuf) -> Response<BoxBody<Bytes, std::io::Error>> {
    let mut response = Response::new(empty());
    let file_result = fs::remove_file(path);
    if file_result.is_ok() {
        let status_code = map_io_result(file_result, StatusCode::NO_CONTENT);
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
