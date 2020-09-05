use serde::{Serialize, Deserialize};
use serde_json::Value;
use tide::{Response, Result, StatusCode};
use std::path::Path;
use std::ffi::OsStr;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub (crate) struct Page {
    pub size: usize,
    pub page: usize,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            size: 25,
            page: 1,
        }
    }
}


pub (crate) struct RespUtil;

impl RespUtil {

  pub fn ok(data: Value) -> Result<Response> {
    Self::with_status(StatusCode::Ok, data)
  }

  pub fn  bad_data(data: Value) -> Result<Response> {
    Self::with_status(StatusCode::BadRequest, data)
  }

  pub fn  not_found(data: Value) -> Result<Response> {
    Self::with_status(StatusCode::NotFound, data)
  }

  pub fn  no_auth(data: Value) -> Result<Response> {
    Self::with_status(StatusCode::Unauthorized, data)
  }

  pub fn with_status(status: StatusCode, data: Value) 
    -> Result<Response> {
    let mut resp = Response::new(status);
    resp.set_body(data);
    Ok(resp)
  }

  pub fn static_not_found() -> Result<Response> {
    let mut resp = Response::new(StatusCode::NotFound);
    resp.set_content_type("text/html");
    resp.set_body("Not Found!");
    Ok(resp)
  }

  pub fn static_ok(data: &[u8], path: &String) -> Result<Response> {
    let mut resp = Response::new(StatusCode::Ok);
    resp.set_content_type(MimeUtils::guess(path));
    resp.set_body(data);
    Ok(resp)
  }

}

pub (crate) struct MimeUtils;

impl MimeUtils {

  pub fn guess(path: &String) -> &str {
    match Path::new(path).extension()
      .and_then(OsStr::to_str) {
        Some("html") => "text/html",
        Some("js") | Some("mjs") | Some("jsonp") => "text/javascript",
        Some("json") => "text/json",
        Some("css") => "text/css",
        Some("svg") => "text/svg",
        Some("xml") => "text/xml",
        Some("ico") | Some("cur") => "image/x-icon",
        Some("bmp") => "image/bmp",
        Some("git") => "image/gif",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => "application/octet-stream",
    }
  }

}

