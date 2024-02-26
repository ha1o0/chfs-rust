use std::{convert::Infallible, fs::File, path::PathBuf};

use http_body_util::combinators::BoxBody;
use hyper::{body::Bytes, Response};
use mime_guess::from_path;

use crate::util::{empty, format_date_time};

pub async fn handle_resp(file_path: &PathBuf) -> Response<BoxBody<Bytes, Infallible>> {
    let mut response = Response::new(empty());
    let file = File::open(file_path).unwrap();
    let metadata = file.metadata().unwrap();
    let file_len = metadata.len();
    let mime_type = from_path(file_path).first_or_octet_stream();
    let last_modified = metadata.modified().unwrap();
    response
        .headers_mut()
        .insert("Content-Type", format!("{}", mime_type).parse().unwrap());
    response
        .headers_mut()
        .insert("Content-Length", format!("{}", file_len).parse().unwrap());
    response.headers_mut().insert(
        "Last-Modified",
        format!("{}", format_date_time(last_modified))
            .parse()
            .unwrap(),
    );

    response
}
