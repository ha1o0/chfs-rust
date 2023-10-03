use std::{fs::File, io::Write, path::PathBuf};

use http_body_util::{BodyExt, Full};
use hyper::{
    body::{Bytes, Incoming},
    Request, Response, StatusCode,
};

use crate::util::map_io_result;

pub async fn handle_resp(
    req: Request<Incoming>,
    path: &PathBuf,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error>> {
    // 创建响应
    let mut response = Response::new(Full::new(Bytes::from("")));
    *response.status_mut() = StatusCode::CREATED;
    // 创建用于写入的文件
    let file_result = File::create(path);
    if file_result.is_err() {
        let status_code = map_io_result(file_result, StatusCode::CREATED);
        *response.status_mut() = status_code;
        return Ok(response);
    }
    let mut file = file_result.unwrap();
    let whole_body = req.collect().await?.to_bytes();
    let write_result = file.write_all(&whole_body);
    let status_code = map_io_result(write_result, StatusCode::CREATED);
    *response.status_mut() = status_code;
    Ok(response)
}
