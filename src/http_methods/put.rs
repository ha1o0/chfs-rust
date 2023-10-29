use std::{fs::File, io::Write, path::PathBuf};

use hyper::{body::to_bytes, Body, Request, Response, StatusCode};

use crate::util::{decode_path, map_io_result};

pub async fn handle_resp(
    req: Request<Body>,
    path: &PathBuf,
) -> Result<Response<Body>, Box<dyn std::error::Error>> {
    // 创建响应
    let mut response = Response::new(Body::from(""));
    *response.status_mut() = StatusCode::CREATED;
    // 创建用于写入的文件
    let file_result = File::create(decode_path(path));
    if file_result.is_err() {
        let status_code = map_io_result(file_result, StatusCode::CREATED);
        *response.status_mut() = status_code;
        return Ok(response);
    }
    let mut file = file_result.unwrap();
    let whole_body = to_bytes(req.into_body()).await?;
    let write_result = file.write_all(&whole_body);
    let status_code = map_io_result(write_result, StatusCode::CREATED);
    *response.status_mut() = status_code;
    Ok(response)
}
