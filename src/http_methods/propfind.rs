use std::{convert::Infallible, fs, path::PathBuf};

use http_body_util::combinators::BoxBody;
use hyper::{
    body::{Bytes, Incoming},
    Request, Response, StatusCode,
};
use mime_guess::from_path;

use crate::util::{encode_uri, format_date_time, full, get_base_dir, get_header, get_server_prefix};

pub async fn handle_resp(req: &Request<Incoming>, file_path: PathBuf) -> Response<BoxBody<Bytes, Infallible>> {
    let depth = get_header(req, "depth", "0");
    let mut multistatus_xml = String::new();
    multistatus_xml.push_str(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    multistatus_xml.push_str(r#"<D:multistatus xmlns:D="DAV:">"#);
    if depth == "0" {
        generate_content_xml(req, &mut multistatus_xml, file_path);
    } else {
        for entry in fs::read_dir(&file_path).unwrap() {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                generate_content_xml(req, &mut multistatus_xml, entry_path);
            }
        }
    }
    multistatus_xml.push_str("</D:multistatus>\n");

    return Response::builder()
        .status(StatusCode::MULTI_STATUS)
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(full(Bytes::from(multistatus_xml)))
        .unwrap();
}

fn generate_content_xml(
    req: &Request<Incoming>,
    multistatus_xml: &mut String,
    entry_path: PathBuf,
) {
    let base_dir = get_base_dir(req);
    let mut server_prefix_with_suffix = get_server_prefix(req).to_string();
    if !server_prefix_with_suffix.ends_with("/") {
        server_prefix_with_suffix += "/";
    }
    let mut relative_path = entry_path.to_string_lossy().to_owned().to_string();
    relative_path = relative_path.replacen(&base_dir, &server_prefix_with_suffix, 1);
    multistatus_xml.push_str("<D:response>\n");
    let encode_relative_path = encode_uri(&relative_path);
    multistatus_xml
        .push_str(format!("<D:href>{}</D:href>\n", format!("{}", encode_relative_path)).as_str());
    multistatus_xml.push_str("<D:propstat>\n");
    multistatus_xml.push_str("<D:prop>\n");

    let is_dir = entry_path.is_dir();
    let metadata = fs::metadata(&entry_path).unwrap();
    let last_modified = metadata.modified().unwrap();
    if is_dir {
        multistatus_xml.push_str("<D:resourcetype><D:collection/></D:resourcetype>\n");
    } else {
        let mime_type = from_path(&entry_path).first_or_octet_stream().to_string();
        let content_length = metadata.len();
        multistatus_xml.push_str(format!("<D:resourcetype/>\n").as_str());
        multistatus_xml.push_str(format!("<D:supportedlock/>\n").as_str());
        multistatus_xml.push_str(
            format!(
                "<D:getcontentlength>{}</D:getcontentlength>\n",
                content_length
            )
            .as_str(),
        );
        multistatus_xml
            .push_str(format!("<D:getcontenttype>{}</D:getcontenttype>\n", mime_type).as_str());
    }
    let mtime = last_modified.duration_since(std::time::UNIX_EPOCH).unwrap();
    let mtime_secs = mtime.as_secs();
    let etag = md5::compute(mtime_secs.to_string());
    multistatus_xml.push_str(format!("<D:getetag>{:?}</D:getetag>\n", etag).as_str());
    multistatus_xml.push_str(
        format!(
            "<D:getlastmodified>{}</D:getlastmodified>\n",
            format_date_time(last_modified)
        )
        .as_str(),
    );
    // let creationdate = get_creation_date(&entry_path.to_string_lossy());
    // if creationdate.len() > 0 {
    //     multistatus_xml
    //         .push_str(format!("<D:creationdate>{}</D:creationdate>\n", creationdate).as_str());
    // }
    multistatus_xml.push_str(
        format!(
            "<D:displayname>{}</D:displayname>\n",
            entry_path.file_name().unwrap().to_str().unwrap()
        )
        .as_str(),
    );
    multistatus_xml.push_str("</D:prop>\n");
    multistatus_xml.push_str("<D:status>HTTP/1.1 200 OK</D:status>\n");
    multistatus_xml.push_str("</D:propstat>\n");
    multistatus_xml.push_str("</D:response>\n");
}
