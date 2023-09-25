use hyper::Method;
use std::fmt;
use std::str::FromStr;

impl From<ExtendMethod> for Method {
    fn from(extend_method: ExtendMethod) -> Self {
        match extend_method {
            ExtendMethod::COPY => Method::from_bytes(b"COPY").unwrap(),
            ExtendMethod::LOCK => Method::from_bytes(b"LOCK").unwrap(),
            ExtendMethod::MKCOL => Method::from_bytes(b"MKCOL").unwrap(),
            ExtendMethod::MOVE => Method::from_bytes(b"MOVE").unwrap(),
            ExtendMethod::PROPFIND => Method::from_bytes(b"PROPFIND").unwrap(),
            ExtendMethod::PROPPATCH => Method::from_bytes(b"PROPPATCH").unwrap(),
            ExtendMethod::UNLOCK => Method::from_bytes(b"UNLOCK").unwrap(),
        }
    }
}

impl FromStr for ExtendMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "COPY" => Ok(ExtendMethod::COPY),
            "LOCK" => Ok(ExtendMethod::LOCK),
            "MKCOL" => Ok(ExtendMethod::MKCOL),
            "MOVE" => Ok(ExtendMethod::MOVE),
            "PROPFIND" => Ok(ExtendMethod::PROPFIND),
            "PROPPATCH" => Ok(ExtendMethod::PROPPATCH),
            "UNLOCK" => Ok(ExtendMethod::UNLOCK),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ExtendMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExtendMethod::COPY => write!(f, "COPY"),
            ExtendMethod::LOCK => write!(f, "LOCK"),
            ExtendMethod::MKCOL => write!(f, "MKCOL"),
            ExtendMethod::MOVE => write!(f, "MOVE"),
            ExtendMethod::PROPFIND => write!(f, "PROPFIND"),
            ExtendMethod::PROPPATCH => write!(f, "PROPPATCH"),
            ExtendMethod::UNLOCK => write!(f, "UNLOCK"),
        }
    }
}

pub enum ExtendMethod {
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
