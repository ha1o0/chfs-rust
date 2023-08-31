use hyper::{Body, Method, Request, Response, StatusCode};
use std::convert::Infallible;

// pub struct WebdavServer {
//     path: String,
// }

// impl WebdavServer {
//     pub fn new(path: String) -> WebdavServer {
//         WebdavServer { path }
//     }

//     pub async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
//         let resp;
//         match req.method() {
//             &Method::GET => {
//                 resp = Response::new(Body::from("Hello, Webdav, GET"));
//             }
//             &Method::PUT => {
//                 resp = Response::new(Body::from("Hello, Webdav, PUT"));
//             }
//             _ => {
//                 resp = Response::new(Body::from("Hello, Webdav"));
//             }
//         }
//         Ok(resp)
//     }
// }

pub async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let resp;
    let method = req.method();
    println!("Method: {}", req.method());
    if method == Method::from_bytes(b"PROPFIND").unwrap() {
        let response_xml = r#"
            <?xml version="1.0"?>
            <D:multistatus xmlns:D="DAV:">
                <D:response>
                    <D:href>/test.txt</D:href>
                    <D:propstat>
                        <D:prop>
                            <D:getcontenttype>text/plain</D:getcontenttype>
                        </D:prop>
                        <D:status>HTTP/1.1 200 OK</D:status>
                    </D:propstat>
                </D:response>
            </D:multistatus>
        "#;
        resp = Response::builder()
            .status(StatusCode::MULTI_STATUS)
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(Body::from(response_xml))
            .unwrap();
        println!("Resp: {:?}", resp);
    } else if method == Method::from_bytes(b"COPY").unwrap() {
        resp = Response::new(Body::from("Hello, Webdav, COPY"));
    } else {
        match req.method() {
            &Method::GET => {
                resp = Response::new(Body::from("Hello, Webdav, GET"));
            }
            &Method::PUT => {
                resp = Response::new(Body::from("Hello, Webdav, PUT"));
            }
            _ => {
                println!("{}", req.method());
                resp = Response::new(Body::from("Hello, Webdav"));
            }
        }
    }
    Ok(resp)
}

pub enum DavMethod {
    // 将资源从一个URI复制到另一个URI
    COPY,
    // 锁定一个资源。WebDAV支持共享锁和互斥锁。
    LOCK,
    // 创建集合（即目录）
    MKCOL,
    // 将资源从一个URI移动到另一个URI
    MOVE,
    // 从Web资源中检索以XML格式存储的属性。它也被重载，以允许一个检索远程系统的集合结构（也叫目录层次结构）
    PROPFIND,
    // 在单个原子性动作中更改和删除资源的多个属性
    PROPPATCH,
    // 解除资源的锁定
    UNLOCK,
}
