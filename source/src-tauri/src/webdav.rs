use base64::Engine;
use once_cell::sync::Lazy;
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::header::{self, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};
use std::fs;
use tauri::Emitter;
use uuid::Uuid;

use crate::commands::FileEntry;

fn fmt_err(e: impl Error) -> String {
    let mut msg = format!("{}", e);
    let mut source = e.source();
    while let Some(s) = source {
        msg.push_str(&format!("\n  → {}", s));
        source = s.source();
    }
    msg
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub username: String,
}

#[derive(Clone)]
#[allow(dead_code)]
struct SessionState {
    client: reqwest::Client,
    base_url: String,
    name: String,
    username: String,
    password: String,
}

static SESSIONS: Lazy<Mutex<HashMap<String, SessionState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn get_temp_dir() -> PathBuf {
    let base = std::env::temp_dir().join("tafiles").join("webdav");
    let _ = fs::create_dir_all(&base);
    base
}

pub fn connect(
    url: &str,
    name: &str,
    username: &str,
    password: &str,
) -> Result<ConnectionInfo, String> {
    let base_url = url.trim_end_matches('/').to_string();

    let mut headers = header::HeaderMap::new();
    if !username.is_empty() {
        let auth = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username, password));
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Basic {}", auth))
                .map_err(|e| format!("认证头错误: {}", e))?,
        );
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let id = Uuid::new_v4().to_string();

    let session = SessionState {
        client,
        base_url: base_url.clone(),
        name: name.to_string(),
        username: username.to_string(),
        password: password.to_string(),
    };

    SESSIONS
        .lock()
        .map_err(|e| format!("锁错误: {}", e))?
        .insert(id.clone(), session);

    Ok(ConnectionInfo {
        id,
        name: name.to_string(),
        url: base_url,
        username: username.to_string(),
    })
}

pub fn disconnect(session_id: &str) -> Result<(), String> {
    SESSIONS
        .lock()
        .map_err(|e| format!("锁错误: {}", e))?
        .remove(session_id);
    Ok(())
}

pub fn list_sessions() -> Result<Vec<ConnectionInfo>, String> {
    let sessions = SESSIONS.lock().map_err(|e| format!("锁错误: {}", e))?;
    Ok(sessions
        .iter()
        .map(|(id, s)| ConnectionInfo {
            id: id.clone(),
            name: s.name.clone(),
            url: s.base_url.clone(),
            username: s.username.clone(),
        })
        .collect())
}

fn get_session(session_id: &str) -> Result<SessionState, String> {
    SESSIONS
        .lock()
        .map_err(|e| format!("锁错误: {}", e))?
        .get(session_id)
        .cloned()
        .ok_or_else(|| "会话不存在或已断开".to_string())
}

fn remote_url(session: &SessionState, path: &str) -> String {
    let p = path.trim_start_matches('/');
    format!("{}/{}", session.base_url, p)
}

pub async fn list(session_id: &str, path: &str) -> Result<Vec<FileEntry>, String> {
    let session = get_session(session_id)?;
    let url = remote_url(&session, path);

    let body = r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:displayname/>
    <D:resourcetype/>
    <D:getcontentlength/>
    <D:getcontenttype/>
    <D:getlastmodified/>
  </D:prop>
</D:propfind>"#;

    let resp = session
        .client
        .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
        .header("Depth", "1")
        .header("Content-Type", "application/xml")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("PROPFIND 请求失败:\n{}", fmt_err(e)))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    if !status.is_success() {
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err("认证失败，请检查用户名和密码".to_string());
        }
        return Err(format!("服务器返回错误: {} - {}", status.as_u16(), &text[..text.len().min(200)]));
    }

    let entries = parse_propfind_response(&text, &session.base_url, path)?;
    let sid = session_id.to_string();
    Ok(entries
        .into_iter()
        .map(|mut e| {
            if e.path.starts_with("webdav://") {
                let rel = &e.path["webdav://".len()..].trim_start_matches('/');
                e.path = format!("webdav://{}/{}", sid, rel);
            }
            e
        })
        .collect())
}

fn parse_propfind_response(
    xml: &str,
    base_url: &str,
    req_path: &str,
) -> Result<Vec<FileEntry>, String> {
    let base_path = base_url
        .find("://")
        .and_then(|i| base_url[i + 3..].find('/').map(|j| &base_url[i + 3 + j..]))
        .unwrap_or("")
        .trim_end_matches('/')
        .to_string();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut entries = Vec::new();
    let mut in_response = false;
    let mut in_propstat = false;
    let mut propstat_ok = false;
    let mut in_prop = false;
    let mut current_href = String::new();
    let mut current_name = String::new();
    let mut current_size = 0u64;
    let mut current_is_dir = false;
    let mut current_mime = String::new();
    let mut collecting = String::new();
    let mut in_collection = false;
    let mut current_tag = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let qname = e.name();
                let name = local_name(qname.as_ref());
                current_tag.push(name.to_vec());
                collecting.clear();
                match name {
                    b"response" => {
                        in_response = true;
                        current_href.clear();
                        current_name.clear();
                        current_size = 0;
                        current_is_dir = false;
                        current_mime.clear();
                        propstat_ok = false;
                        in_collection = false;
                    }
                    b"propstat" => {
                        in_propstat = true;
                    }
                    b"prop" => {
                        in_prop = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let qname = e.name();
                let name = local_name(qname.as_ref());
                if in_response && in_prop {
                    if name == b"collection" {
                        in_collection = true;
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Ok(text) = e.unescape() {
                    collecting = text.to_string();
                }
            }
            Ok(Event::End(ref e)) => {
                let qname = e.name();
                let name = local_name(qname.as_ref());
                current_tag.pop();

                if in_response {
                    match name {
                        b"href" => {
                            if !collecting.is_empty() {
                                current_href = collecting.clone();
                            }
                        }
                        b"displayname" => {
                            if !collecting.is_empty() {
                                current_name = collecting.clone();
                            }
                        }
                        b"getcontentlength" => {
                            current_size = collecting.parse::<u64>().unwrap_or(0);
                        }
                        b"getcontenttype" => {
                            current_mime = collecting.clone();
                        }
                        b"status" => {
                            if collecting.contains("200") {
                                propstat_ok = true;
                            }
                        }
                        b"response" => {
                            if !current_href.is_empty() {
                                let decoded_href = url_decode(&current_href);
                                let href_path = decoded_href
                                    .trim_start_matches('/')
                                    .trim_end_matches('/');

                                let req_path_clean = req_path
                                    .trim_start_matches('/')
                                    .trim_end_matches('/');

                                if href_path == req_path_clean && current_is_dir {
                                    continue;
                                }

                                let name = if !current_name.is_empty() {
                                    current_name.clone()
                                } else {
                                    href_path
                                        .rsplit('/')
                                        .next()
                                        .unwrap_or(href_path)
                                        .to_string()
                                };

                                let ext = if current_is_dir {
                                    String::new()
                                } else {
                                    Path::new(&name)
                                        .extension()
                                        .map(|e| e.to_string_lossy().to_lowercase())
                                        .unwrap_or_default()
                                };

                                let relative = if !base_path.is_empty() && href_path.starts_with(&base_path) {
                                    let r = &href_path[base_path.len()..];
                                    if r.is_empty() { "/".to_string() } else { r.to_string() }
                                } else {
                                    format!("/{}", href_path)
                                };
                                let clean = relative.trim_start_matches('/');

                                entries.push(FileEntry {
                                    name,
                                    path: format!("webdav:///{}", clean),
                                    is_dir: current_is_dir,
                                    ext,
                                    size: current_size,
                                    modified: String::new(),
                                });
                            }
                            in_response = false;
                            in_propstat = false;
                            in_prop = false;
                        }
                        _ => {}
                    }
                }

                match name {
                    b"propstat" => {
                        in_propstat = false;
                    }
                    b"prop" => {
                        in_prop = false;
                        if in_propstat && propstat_ok {
                            current_is_dir = in_collection;
                        }
                    }
                    _ => {}
                }
                collecting.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML 解析错误: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            if a.is_dir {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(entries)
}

fn local_name<'a>(name: &'a [u8]) -> &'a [u8] {
    if let Some(pos) = name.iter().position(|&b| b == b':') {
        &name[pos + 1..]
    } else {
        name
    }
}

fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hi = chars.next().and_then(|c| c.to_digit(16)).unwrap_or(0);
            let lo = chars.next().and_then(|c| c.to_digit(16)).unwrap_or(0);
            result.push(char::from((hi * 16 + lo) as u8));
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

pub async fn read_binary(session_id: &str, path: &str) -> Result<Vec<u8>, String> {
    let session = get_session(session_id)?;
    let url = remote_url(&session, path);

    let resp = session
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("下载文件失败:\n{}", fmt_err(e)))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("下载文件失败: HTTP {}", status.as_u16()));
    }

    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("读取文件内容失败:\n{}", fmt_err(e)))
}

pub async fn read_text(session_id: &str, path: &str) -> Result<String, String> {
    let data = read_binary(session_id, path).await?;
    String::from_utf8(data).map_err(|e| format!("文件不是有效的 UTF-8 文本: {}", e))
}

pub async fn upload(
    session_id: &str,
    local_path: &str,
    remote_path: &str,
) -> Result<(), String> {
    let session = get_session(session_id)?;
    let url = remote_url(&session, remote_path);

    let data = fs::read(local_path).map_err(|e| format!("读取本地文件失败: {}", e))?;

    let resp = session
        .client
        .put(&url)
        .header("Content-Type", "application/octet-stream")
        .body(data)
        .send()
        .await
        .map_err(|e| format!("上传文件失败: {}", e))?;

    let status = resp.status();
    if !status.is_success() && status.as_u16() != 201 && status.as_u16() != 204 {
        return Err(format!("上传失败: HTTP {}", status.as_u16()));
    }

    Ok(())
}

pub async fn download(
    session_id: &str,
    remote_path: &str,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let data = read_binary(session_id, remote_path).await?;

    let remote_clean = remote_path.trim_start_matches('/');
    let temp_path = get_temp_dir().join(session_id).join(remote_clean);

    if let Some(parent) = temp_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建临时目录失败: {}", e))?;
    }

    fs::write(&temp_path, &data).map_err(|e| format!("写入临时文件失败: {}", e))?;

    let path_str = temp_path.to_string_lossy().to_string();
    let sid = session_id.to_string();
    let rpath = remote_path.to_string();

    tokio::spawn(async move {
        monitor_file_changes(path_str, sid, rpath, app_handle).await;
    });

    Ok(temp_path.to_string_lossy().to_string())
}

async fn monitor_file_changes(
    temp_path: String,
    session_id: String,
    remote_path: String,
    app_handle: tauri::AppHandle,
) {
    let mut last_modified = SystemTime::UNIX_EPOCH;
    let mut debounce_start: Option<Instant> = None;
    let debounce_duration = Duration::from_secs(3);

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let path = Path::new(&temp_path);
        if !path.exists() {
            break;
        }

        let modified = match fs::metadata(&temp_path) {
            Ok(meta) => meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            Err(_) => break,
        };

        if modified > last_modified {
            last_modified = modified;
            debounce_start = Some(Instant::now());
            let _ = app_handle.emit(
                "webdav-sync-status",
                serde_json::json!({
                    "path": temp_path,
                    "sessionId": session_id,
                    "remotePath": remote_path,
                    "status": "pending"
                }),
            );
        }

        if let Some(start) = debounce_start {
            if start.elapsed() >= debounce_duration {
                debounce_start = None;
                let _ = app_handle.emit(
                    "webdav-sync-status",
                    serde_json::json!({
                        "path": temp_path,
                        "sessionId": session_id,
                        "remotePath": remote_path,
                        "status": "syncing"
                    }),
                );
                match upload(&session_id, &temp_path, &remote_path).await {
                    Ok(_) => {
                        let _ = app_handle.emit(
                            "webdav-sync-status",
                            serde_json::json!({
                                "path": temp_path,
                                "sessionId": session_id,
                                "remotePath": remote_path,
                                "status": "synced"
                            }),
                        );
                    }
                    Err(e) => {
                        debounce_start = Some(Instant::now());
                        let _ = app_handle.emit(
                            "webdav-sync-status",
                            serde_json::json!({
                                "path": temp_path,
                                "sessionId": session_id,
                                "remotePath": remote_path,
                                "status": "error",
                                "error": e
                            }),
                        );
                    }
                }
            }
        }
    }
}

pub async fn create_dir(session_id: &str, path: &str) -> Result<(), String> {
    let session = get_session(session_id)?;
    let url = remote_url(&session, path);

    let resp = session
        .client
        .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), &url)
        .send()
        .await
        .map_err(|e| format!("创建目录失败: {}", e))?;

    if !resp.status().is_success() && resp.status().as_u16() != 201 {
        return Err(format!("创建目录失败: HTTP {}", resp.status().as_u16()));
    }

    Ok(())
}

pub async fn remove(session_id: &str, path: &str) -> Result<(), String> {
    let session = get_session(session_id)?;
    let url = remote_url(&session, path);

    let resp = session
        .client
        .delete(&url)
        .send()
        .await
        .map_err(|e| format!("删除失败: {}", e))?;

    if !resp.status().is_success() && resp.status().as_u16() != 204 {
        return Err(format!("删除失败: HTTP {}", resp.status().as_u16()));
    }

    Ok(())
}

pub async fn rename(session_id: &str, from: &str, to: &str) -> Result<(), String> {
    let session = get_session(session_id)?;
    let from_url = remote_url(&session, from);
    let to_url = remote_url(&session, to);

    let dest_header = HeaderName::from_static("destination");
    let resp = session
        .client
        .request(reqwest::Method::from_bytes(b"MOVE").unwrap(), &from_url)
        .header(dest_header, HeaderValue::from_str(&to_url).map_err(|e| format!("无效的目标URL: {}", e))?)
        .send()
        .await
        .map_err(|e| format!("重命名失败: {}", e))?;

    if !resp.status().is_success() && resp.status().as_u16() != 201 && resp.status().as_u16() != 204 {
        return Err(format!("重命名失败: HTTP {}", resp.status().as_u16()));
    }

    Ok(())
}
