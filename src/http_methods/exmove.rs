use std::path::{Path, PathBuf};

use crate::util::{
    decode_uri, extract_relative_path, get_base_dir, get_header, get_server_prefix, map_io_result,
};
use hyper::{Body, Request, Response, StatusCode};
use tokio::fs;

pub async fn handle_resp(req: &Request<Body>, from_path: &PathBuf) -> Response<Body> {
    // 创建响应
    let mut response = Response::new(Body::from(""));
    let destination = get_header(req, "destination", "");
    let host = get_header(req, "host", "");
    // log::info!("destination: {}, host: {}", destination, host);
    let rel_path_result = extract_relative_path(destination, host);
    // log::info!("rel_path_result: {:?}", rel_path_result.clone());
    if rel_path_result.is_none() {
        *response.status_mut() = StatusCode::NOT_FOUND;
        return response;
    }
    let mut rel_path = rel_path_result.unwrap();
    let server_prefix = get_server_prefix(req);
    let base_dir = get_base_dir(req);
    rel_path = rel_path.replacen(&server_prefix, "", 1);
    let to_path = Path::new(&base_dir).join(rel_path.trim_start_matches('/'));
    // log::info!("to path: {:?}", to_path);
    let move_result = fs::rename(
        decode_uri(from_path.to_str().unwrap()),
        decode_uri(to_path.to_str().unwrap()),
    )
    .await;
    let status_code = map_io_result(move_result, StatusCode::CREATED);
    *response.status_mut() = status_code;
    response
}
