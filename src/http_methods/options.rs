use std::convert::Infallible;

use http_body_util::combinators::BoxBody;
use hyper::{body::Bytes, Response, StatusCode};

use crate::util::empty;

pub async fn handle_resp() -> Response<BoxBody<Bytes, Infallible>> {
    let allow_methods =
        "OPTIONS, GET, HEAD, POST, PUT, DELETE, PROPFIND, MKCOL, COPY, MOVE, MKCOL, DELETE";
    return Response::builder()
        .status(StatusCode::OK)
        .header("Allow", allow_methods)
        .header("DAV", "1")
        .body(empty())
        .unwrap();
}
