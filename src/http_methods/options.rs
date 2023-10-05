use http_body_util::Full;
use hyper::{body::Bytes, Response, StatusCode};

pub async fn handle_resp() -> Response<Full<Bytes>> {
    let allow_methods =
        "OPTIONS, GET, HEAD, POST, PUT, DELETE, PROPFIND, MKCOL, COPY, MOVE, MKCOL, DELETE";
    return Response::builder()
        .status(StatusCode::OK)
        .header("Allow", allow_methods)
        .header("DAV", "1")
        .body(Full::new(Bytes::from("")))
        .unwrap();
}
