mod pdf_ops;

use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::thread;
use std::time::Duration;
use tiny_http::{Header, Response, Server, StatusCode};

const INDEX_HTML: &str = include_str!("../static/index.html");
const STYLE_CSS: &str = include_str!("../static/style.css");
const APP_JS: &str = include_str!("../static/app.js");

#[derive(Deserialize)]
struct MergeReq {
    files: Vec<String>,
}

#[derive(Deserialize)]
struct SplitReq {
    file: String,
    mode: String,
    range: Option<String>,
}

#[derive(Deserialize)]
struct OrganizeReq {
    file: String,
    order: Vec<u32>,
    rotations: Vec<(u32, i64)>,
}

#[derive(Deserialize)]
struct CompressReq {
    file: String,
}

#[derive(Deserialize)]
struct ImagesReq {
    images: Vec<String>,
}

#[derive(Deserialize)]
struct WatermarkReq {
    file: String,
    text: String,
}

#[derive(Deserialize)]
struct PageNumbersReq {
    file: String,
    position: String,
    format: String,
}

#[derive(Deserialize)]
struct ExtractTextReq {
    file: String,
}

#[derive(Serialize)]
struct ApiResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    orig_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

fn json_response(res: ApiResponse) -> Response<Cursor<Vec<u8>>> {
    let json_bytes = serde_json::to_vec(&res).unwrap_or_default();
    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
    Response::from_data(json_bytes).with_header(header)
}

fn main() {
    let port = 8080;
    let addr = format!("127.0.0.1:{}", port);
    let server = Server::http(&addr).expect("Konnte HTTP Server nicht starten");

    println!("============================================================");
    println!("  PDF TOOLKIT RUST - NATIVE AUSFÜHRUNG");
    println!("  Server läuft auf: http://{}", addr);
    println!("  Drücke STRG + C zum Beenden.");
    println!("============================================================");

    // Open browser in background thread
    let open_addr = format!("http://{}", addr);
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(800));
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", &open_addr])
            .spawn();
    });

    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let path = url.split('?').next().unwrap_or("/");

        match (request.method(), path) {
            (&tiny_http::Method::Get, "/") => {
                let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap();
                let _ = request.respond(Response::from_string(INDEX_HTML).with_header(header));
            }
            (&tiny_http::Method::Get, "/style.css") => {
                let header = Header::from_bytes(&b"Content-Type"[..], &b"text/css; charset=utf-8"[..]).unwrap();
                let _ = request.respond(Response::from_string(STYLE_CSS).with_header(header));
            }
            (&tiny_http::Method::Get, "/app.js") => {
                let header = Header::from_bytes(&b"Content-Type"[..], &b"application/javascript; charset=utf-8"[..]).unwrap();
                let _ = request.respond(Response::from_string(APP_JS).with_header(header));
            }

            (&tiny_http::Method::Post, "/api/merge") => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let resp = match serde_json::from_str::<MergeReq>(&body) {
                    Ok(req) => {
                        let mut raw_files = Vec::new();
                        let mut err_flag = false;
                        for b64 in req.files {
                            if let Ok(bytes) = BASE64_STANDARD.decode(&b64) {
                                raw_files.push(bytes);
                            } else {
                                err_flag = true;
                                break;
                            }
                        }
                        if err_flag || raw_files.is_empty() {
                            ApiResponse { ok: false, error: Some("Ungültige Base64-Daten".into()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                        } else {
                            match pdf_ops::merge(raw_files) {
                                Ok(out) => ApiResponse { ok: true, error: None, data: Some(BASE64_STANDARD.encode(out)), filename: Some("zusammengefuegt.pdf".into()), orig_size: None, new_size: None, text: None },
                                Err(e) => ApiResponse { ok: false, error: Some(e), data: None, filename: None, orig_size: None, new_size: None, text: None }
                            }
                        }
                    }
                    Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None },
                };
                let _ = request.respond(json_response(resp));
            }

            (&tiny_http::Method::Post, "/api/split") => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let resp = match serde_json::from_str::<SplitReq>(&body) {
                    Ok(req) => match BASE64_STANDARD.decode(&req.file) {
                        Ok(bytes) => {
                            let range_str = req.range.as_deref().unwrap_or("");
                            match pdf_ops::split(&bytes, &req.mode, range_str) {
                                Ok((out, filename)) => ApiResponse { ok: true, error: None, data: Some(BASE64_STANDARD.encode(out)), filename: Some(filename), orig_size: None, new_size: None, text: None },
                                Err(e) => ApiResponse { ok: false, error: Some(e), data: None, filename: None, orig_size: None, new_size: None, text: None }
                            }
                        }
                        Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                    },
                    Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                };
                let _ = request.respond(json_response(resp));
            }

            (&tiny_http::Method::Post, "/api/organize") => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let resp = match serde_json::from_str::<OrganizeReq>(&body) {
                    Ok(req) => match BASE64_STANDARD.decode(&req.file) {
                        Ok(bytes) => match pdf_ops::organize(&bytes, &req.order, &req.rotations) {
                            Ok(out) => ApiResponse { ok: true, error: None, data: Some(BASE64_STANDARD.encode(out)), filename: Some("organisiert.pdf".into()), orig_size: None, new_size: None, text: None },
                            Err(e) => ApiResponse { ok: false, error: Some(e), data: None, filename: None, orig_size: None, new_size: None, text: None }
                        },
                        Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                    },
                    Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                };
                let _ = request.respond(json_response(resp));
            }

            (&tiny_http::Method::Post, "/api/compress") => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let resp = match serde_json::from_str::<CompressReq>(&body) {
                    Ok(req) => match BASE64_STANDARD.decode(&req.file) {
                        Ok(bytes) => match pdf_ops::compress(&bytes) {
                            Ok((out, orig, new_s)) => ApiResponse { ok: true, error: None, data: Some(BASE64_STANDARD.encode(out)), filename: Some("komprimiert.pdf".into()), orig_size: Some(orig), new_size: Some(new_s), text: None },
                            Err(e) => ApiResponse { ok: false, error: Some(e), data: None, filename: None, orig_size: None, new_size: None, text: None }
                        },
                        Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                    },
                    Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                };
                let _ = request.respond(json_response(resp));
            }

            (&tiny_http::Method::Post, "/api/images-to-pdf") => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let resp = match serde_json::from_str::<ImagesReq>(&body) {
                    Ok(req) => {
                        let mut raw_imgs = Vec::new();
                        for b64 in req.images {
                            if let Ok(b) = BASE64_STANDARD.decode(&b64) {
                                raw_imgs.push(b);
                            }
                        }
                        match pdf_ops::images_to_pdf(raw_imgs) {
                            Ok(out) => ApiResponse { ok: true, error: None, data: Some(BASE64_STANDARD.encode(out)), filename: Some("bilder.pdf".into()), orig_size: None, new_size: None, text: None },
                            Err(e) => ApiResponse { ok: false, error: Some(e), data: None, filename: None, orig_size: None, new_size: None, text: None }
                        }
                    }
                    Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                };
                let _ = request.respond(json_response(resp));
            }

            (&tiny_http::Method::Post, "/api/watermark") => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let resp = match serde_json::from_str::<WatermarkReq>(&body) {
                    Ok(req) => match BASE64_STANDARD.decode(&req.file) {
                        Ok(bytes) => match pdf_ops::watermark(&bytes, &req.text) {
                            Ok(out) => ApiResponse { ok: true, error: None, data: Some(BASE64_STANDARD.encode(out)), filename: Some("wasserzeichen.pdf".into()), orig_size: None, new_size: None, text: None },
                            Err(e) => ApiResponse { ok: false, error: Some(e), data: None, filename: None, orig_size: None, new_size: None, text: None }
                        },
                        Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                    },
                    Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                };
                let _ = request.respond(json_response(resp));
            }

            (&tiny_http::Method::Post, "/api/page-numbers") => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let resp = match serde_json::from_str::<PageNumbersReq>(&body) {
                    Ok(req) => match BASE64_STANDARD.decode(&req.file) {
                        Ok(bytes) => match pdf_ops::page_numbers(&bytes, &req.format, &req.position) {
                            Ok(out) => ApiResponse { ok: true, error: None, data: Some(BASE64_STANDARD.encode(out)), filename: Some("nummeriert.pdf".into()), orig_size: None, new_size: None, text: None },
                            Err(e) => ApiResponse { ok: false, error: Some(e), data: None, filename: None, orig_size: None, new_size: None, text: None }
                        },
                        Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                    },
                    Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                };
                let _ = request.respond(json_response(resp));
            }

            (&tiny_http::Method::Post, "/api/extract-text") => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let resp = match serde_json::from_str::<ExtractTextReq>(&body) {
                    Ok(req) => match BASE64_STANDARD.decode(&req.file) {
                        Ok(bytes) => match pdf_ops::extract_text(&bytes) {
                            Ok(text) => ApiResponse { ok: true, error: None, data: None, filename: None, orig_size: None, new_size: None, text: Some(text) },
                            Err(e) => ApiResponse { ok: false, error: Some(e), data: None, filename: None, orig_size: None, new_size: None, text: None }
                        },
                        Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                    },
                    Err(e) => ApiResponse { ok: false, error: Some(e.to_string()), data: None, filename: None, orig_size: None, new_size: None, text: None }
                };
                let _ = request.respond(json_response(resp));
            }

            _ => {
                let resp = Response::from_string("Not Found").with_status_code(StatusCode(404));
                let _ = request.respond(resp);
            }
        }
    }
}
