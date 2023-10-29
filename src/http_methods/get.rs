use std::{
    io::{self, Read, Seek},
    path::PathBuf,
};

use hyper::{Body, Request, Response, StatusCode};
use mime_guess::from_path;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::{config, util::get_header};

static NOTFOUND: &[u8] = b"Not Found";

pub async fn handle_resp(req: &Request<Body>, file_path: &PathBuf) -> Response<Body> {
    let mut response = Response::new(Body::from(""));
    let range = get_header(req, "range", "");
    if range.len() > 0 {
        let mut file = std::fs::File::open(file_path).unwrap();
        let metadata = file.metadata().unwrap();
        let file_len = metadata.len();
        let mime_type = from_path(file_path).first_or_octet_stream();
        response
            .headers_mut()
            .insert("Content-Type", format!("{}", mime_type).parse().unwrap());
        let mut start = 0;
        let end: u64;
        let bounds = range.strip_prefix("bytes=").unwrap();
        let max_chunk_size = (file_len / 1024) * 30;
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
        // let mut concurrent_count = 0;
        // if let Some(concurrent_count_string) = get("over_size") {
        //     let concurrent_count_result = concurrent_count_string.parse::<u32>();
        //     if !concurrent_count_result.is_err() {
        //         concurrent_count = concurrent_count_result.unwrap();
        //     }
        // }

        let is_over_size = (end - start) > max_chunk_size;
        let cfg = config::get_config();
        if is_over_size && cfg.mode == "dev" {
            *response.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
            return response;
        }
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
        *response.body_mut() = Body::from(stream);
    } else {
        response = handle_get_all_resp(file_path).await;
    }

    response
}

async fn handle_get_all_resp(file_path: &PathBuf) -> Response<Body> {
    let mime_type = from_path(file_path).first_or_octet_stream();
    let mut response = Response::new(Body::from(""));
    response.headers_mut().insert(
        "Content-Type",
        format!("{}", mime_type.as_ref()).parse().unwrap(),
    );
    if let Ok(file) = File::open(file_path).await {
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);
        return Response::new(body);
    }

    not_found()
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}
