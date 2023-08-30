use std::convert::Infallible;
use hyper::{ Request, Body, Method, Response};

pub struct WebdavServer {
  path: String
}

impl WebdavServer {
  pub fn new(path: String) -> WebdavServer {
      WebdavServer {
        path
      }
  }

  pub async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
      let resp;
      match req.method() {
          &Method::GET => {
              resp = Response::new(Body::from("Hello, Webdav, GET"));
          },
          &Method::PUT => {
              resp = Response::new(Body::from("Hello, Webdav, PUT"));
          },
          _ => {
              resp = Response::new(Body::from("Hello, Webdav"));
          }
      }
      Ok(resp)
  }
}

enum DavMethod {
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
