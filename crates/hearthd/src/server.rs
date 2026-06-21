//! The local server — the seam between the runtime (`hearthd`) and the shell (the UI).
//!
//! A minimal HTTP/1.1 server on `std::net` (no dependency, stays lean) bound to localhost.
//! It serves the UI at `/` and exposes the steward as an API the shell drives:
//!
//! - `GET  /`            → the UI (so the shell and the API are same-origin)
//! - `POST /api/intent`  → run one turn (`{intent, approve}`) → the structured result
//! - `GET  /api/brain`   → the Brain's curated pages ("what do you know about me?")
//! - `POST /api/forget`  → forget a curated page (`{page}`) → snapshot-first, undoable
//!
//! Single-threaded by design: one owner, one steward, requests handled in order.

use crate::Hearthd;
use anyhow::Result;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

pub fn serve(h: Hearthd, addr: &str, ui: Option<PathBuf>) -> Result<()> {
    let listener =
        TcpListener::bind(addr).map_err(|e| anyhow::anyhow!("can't bind {addr}: {e}"))?;
    let ui = ui.filter(|p| p.exists());

    println!("The Hearth is listening on http://{addr}");
    match &ui {
        Some(p) => println!("  UI   http://{addr}/   (serving {})", p.display()),
        None => println!("  UI   (none — pass --ui <the-hearth.html> to serve the shell)"),
    }
    println!("  API  POST /api/intent   GET /api/brain");
    println!("  (Ctrl-C to stop)");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                if let Err(e) = handle(&h, s, ui.as_deref()) {
                    eprintln!("· request error: {e}");
                }
            }
            Err(_) => continue,
        }
    }
    Ok(())
}

fn handle(h: &Hearthd, mut stream: TcpStream, ui: Option<&Path>) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    let mut it = request_line.split_whitespace();
    let method = it.next().unwrap_or("").to_string();
    let path = it.next().unwrap_or("/").to_string();

    let mut content_length = 0usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            break;
        }
        let header = header.trim_end();
        if header.is_empty() {
            break;
        }
        if let Some(v) = header.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let (status, ctype, payload) = respond(h, &method, &path, &body, ui);

    let head = format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: {ctype}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        payload.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;
    Ok(())
}

fn respond(
    h: &Hearthd,
    method: &str,
    path: &str,
    body: &[u8],
    ui: Option<&Path>,
) -> (&'static str, &'static str, Vec<u8>) {
    if method == "OPTIONS" {
        return ("204 No Content", "text/plain", vec![]);
    }
    match (method, path) {
        ("GET", "/") => match ui {
            Some(p) => match std::fs::read(p) {
                Ok(bytes) => ("200 OK", "text/html; charset=utf-8", bytes),
                Err(_) => ("500 Internal Server Error", "text/plain", b"could not read the UI".to_vec()),
            },
            None => (
                "200 OK",
                "text/html; charset=utf-8",
                b"<!doctype html><meta charset=utf-8><body style='font-family:system-ui;background:#06080f;color:#ecf0f8;padding:3rem'><h1>The Hearth is live.</h1><p><code>POST /api/intent</code> &middot; <code>GET /api/brain</code></p>".to_vec(),
            ),
        },
        ("GET", "/api/brain") => json_result(h.brain_pages()),
        ("POST", "/api/forget") => {
            let v: serde_json::Value =
                serde_json::from_slice(body).unwrap_or_else(|_| serde_json::json!({}));
            let page = v.get("page").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
            if page.is_empty() {
                return ("400 Bad Request", "application/json", br#"{"error":"no page"}"#.to_vec());
            }
            json_result(h.forget(&page))
        }
        ("POST", "/api/intent") => {
            let v: serde_json::Value =
                serde_json::from_slice(body).unwrap_or_else(|_| serde_json::json!({}));
            let intent = v.get("intent").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let approve = v.get("approve").and_then(|x| x.as_bool()).unwrap_or(false);
            if intent.trim().is_empty() {
                return ("400 Bad Request", "application/json", br#"{"error":"no intent"}"#.to_vec());
            }
            json_result(h.turn(&intent, approve))
        }
        _ => ("404 Not Found", "application/json", br#"{"error":"not found"}"#.to_vec()),
    }
}

fn json_result<T: serde::Serialize>(r: Result<T>) -> (&'static str, &'static str, Vec<u8>) {
    match r {
        Ok(v) => match serde_json::to_vec(&v) {
            Ok(b) => ("200 OK", "application/json", b),
            Err(e) => error_json(&e.to_string()),
        },
        Err(e) => error_json(&e.to_string()),
    }
}

fn error_json(msg: &str) -> (&'static str, &'static str, Vec<u8>) {
    let body = serde_json::to_vec(&serde_json::json!({ "error": msg })).unwrap_or_default();
    ("500 Internal Server Error", "application/json", body)
}
