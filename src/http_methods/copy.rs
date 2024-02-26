use std::path::{Path, PathBuf};

use crate::util::{
    decode_path, decode_uri, empty, extract_relative_path, get_base_dir, get_header, get_server_prefix, map_io_result
};
use async_recursion::async_recursion;
use http_body_util::combinators::BoxBody;
use hyper::{
    body::{Bytes, Incoming},
    Request, Response, StatusCode,
};
use tokio::fs;

pub async fn handle_resp(req: &Request<Incoming>, from_path: &PathBuf) -> Response<BoxBody<Bytes, std::io::Error>> {
    let mut response = Response::new(empty());
    let to_path = get_to_path(req);
    if to_path.is_none() {
        *response.status_mut() = StatusCode::NOT_FOUND;
        return response;
    }
    let status_code;
    if from_path.is_dir() {
        status_code = copy_dir_files(from_path, &to_path.unwrap()).await;
    } else {
        status_code = copy_file(
            from_path.to_str().unwrap(),
            to_path.unwrap().to_str().unwrap(),
        )
        .await;
    }
    *response.status_mut() = status_code;
    response
}

#[async_recursion]
async fn copy_dir_files(from_dir_path: &PathBuf, to_dir_path: &PathBuf) -> StatusCode {
    // log::info!("copy dir: from: {:?}, to: {:?}", from_dir_path, to_dir_path);
    let create_dir_result = fs::create_dir_all(decode_path(to_dir_path)).await;
    if create_dir_result.is_err() {
        return map_io_result(create_dir_result, StatusCode::CREATED);
    }
    let mut entries = fs::read_dir(from_dir_path).await.unwrap();
    let mut status_code = StatusCode::CREATED;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let entry_path = entry.path();
        let entry_name = entry.file_name();
        // log::info!("entry path: {:?}, name: {:?}", entry_path, entry_name);
        let from_abs_path =
            &Path::new(&decode_path(&from_dir_path.join(&entry_name))).to_path_buf();
        let to_abs_path = &Path::new(&decode_path(&to_dir_path.join(&entry_name))).to_path_buf();
        // log::info!(
        //     "copy from_abs_path: {:?}, to_abs_path: {:?}",
        //     from_abs_path,
        //     to_abs_path
        // );
        if from_abs_path == to_abs_path {
            // log::info!("same path");
            continue;
        }
        if entry_path.is_dir() {
            status_code = copy_dir_files(from_abs_path, to_abs_path).await;
        } else {
            if entry_path.to_str().unwrap() == ".DS_Store" {
                continue;
            }
            status_code = copy_file(
                from_abs_path.to_str().unwrap(),
                to_abs_path.to_str().unwrap(),
            )
            .await;
        }
    }
    status_code
}

async fn copy_file(from_path: &str, to_path: &str) -> StatusCode {
    // log::info!("copy file: from: {:?}, to: {:?}", from_path, to_path);
    let copy_result = fs::copy(
        Path::new(&decode_uri(from_path)),
        Path::new(&decode_uri(to_path)),
    )
    .await;
    map_io_result(copy_result, StatusCode::CREATED)
}

fn get_to_path(req: &Request<Incoming>) -> Option<PathBuf> {
    let server_prefix = get_server_prefix(req);
    let base_dir = get_base_dir(req);
    let destination = get_header(req, "destination", "");
    let host = get_header(req, "host", "");
    // log::info!("destination: {}, host: {}", destination, host);
    let rel_path_result = extract_relative_path(destination, host);
    // log::info!("rel_path_result: {:?}", rel_path_result.clone());
    if rel_path_result.is_none() {
        return None;
    }
    let mut rel_path = rel_path_result.unwrap();
    rel_path = rel_path.replacen(&server_prefix, "", 1);
    Some(Path::new(&base_dir).join(rel_path.trim_start_matches('/')))
}
