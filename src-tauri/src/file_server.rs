use axum::{
    body::Body,
    extract::{ws::WebSocket, FromRequest, Query, State, WebSocketUpgrade},
    http::{header, HeaderMap, Method, Request, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::extract::Multipart;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tokio_util::io::ReaderStream;
use tower_http::cors::CorsLayer;

use crate::portal::{self, FileItem};

// ─── Shared Server State ──────────────────────────────────────────────

#[derive(Clone)]
pub struct ServerState {
    pub shared_path: Arc<RwLock<String>>,
    pub password_hash: Arc<RwLock<String>>,
    pub theme: Arc<RwLock<String>>,
    pub ws_tx: broadcast::Sender<String>,
}

// ─── Global server handle ─────────────────────────────────────────────

struct RunningServer {
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    _watcher: Option<RecommendedWatcher>,
}

static SERVER_HANDLE: std::sync::Mutex<Option<RunningServer>> = std::sync::Mutex::new(None);
static SERVER_STATE: std::sync::Mutex<Option<ServerState>> = std::sync::Mutex::new(None);

// ─── Public types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct StartServerResult {
    #[serde(rename = "passwordHash")]
    pub password_hash: String,
    #[serde(rename = "mdnsUrl")]
    pub mdns_url: String,
}

// ─── File listing ─────────────────────────────────────────────────────

pub fn get_shared_files(dir_path: &str) -> Vec<FileItem> {
    let path = Path::new(dir_path);
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to read directory: {}", e);
            return vec![];
        }
    };

    let mut items: Vec<FileItem> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            Some(FileItem {
                name,
                is_directory: metadata.is_dir(),
                size: if metadata.is_dir() { 0 } else { metadata.len() },
            })
        })
        .collect();

    // Sort: directories first, then alphabetical
    items.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    items
}

// ─── MIME type helper ─────────────────────────────────────────────────

fn get_mime_type(file_path: &Path) -> &'static str {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogg" => "video/ogg",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

// ─── Cookie helper ────────────────────────────────────────────────────

fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    for cookie in cookie_header.split(';') {
        let mut parts = cookie.splitn(2, '=');
        let key = parts.next()?.trim();
        let value = parts.next()?.trim();
        if key == name {
            return Some(value.to_string());
        }
    }
    None
}

// ─── Broadcast helper ─────────────────────────────────────────────────

fn broadcast_msg(state: &ServerState, msg: &serde_json::Value) {
    let json_str = serde_json::to_string(msg).unwrap_or_default();
    let _ = state.ws_tx.send(json_str);
}

// ─── Hash helper ──────────────────────────────────────────────────────

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

// ─── Request handler: main catch-all ──────────────────────────────────

#[derive(Deserialize)]
struct FileQuery {
    json: Option<String>,
    preview: Option<String>,
    token: Option<String>,
}

async fn handle_get(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Query(query): Query<FileQuery>,
    req: Request<Body>,
) -> Response {
    let shared_path = state.shared_path.read().await.clone();
    let password_hash = state.password_hash.read().await.clone();
    let theme = state.theme.read().await.clone();

    let raw_path = percent_encoding::percent_decode_str(req.uri().path())
        .decode_utf8_lossy()
        .to_string();

    // Auth check
    if !password_hash.is_empty() {
        // Auto-login via token query parameter
        if let Some(ref token) = query.token {
            if token == &password_hash {
                return Response::builder()
                    .status(StatusCode::SEE_OTHER)
                    .header(header::LOCATION, raw_path.as_str())
                    .header(
                        header::SET_COOKIE,
                        format!(
                            "thlauh_session={}; Path=/; HttpOnly; Max-Age=31536000",
                            password_hash
                        ),
                    )
                    .body(Body::empty())
                    .unwrap();
            }
        }

        let cookie_token = get_cookie(&headers, "thlauh_session");
        if cookie_token.as_deref() != Some(&password_hash) {
            return Html(portal::generate_login_html(&theme, None)).into_response();
        }
    }

    // Resolve file path safely
    let safe_suffix = raw_path.trim_start_matches('/');
    let file_path = PathBuf::from(&shared_path).join(safe_suffix);

    // Prevent path traversal
    if !file_path.starts_with(&shared_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    let metadata = match fs::metadata(&file_path) {
        Ok(m) => m,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };

    if metadata.is_dir() {
        let files = get_shared_files(&file_path.to_string_lossy());
        let is_json = query.json.as_deref() == Some("true");

        if is_json {
            let json = serde_json::to_string(&files).unwrap_or_else(|_| "[]".to_string());
            return Response::builder()
                .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
                .body(Body::from(json))
                .unwrap();
        }

        let html = portal::generate_directory_html(
            &file_path.to_string_lossy(),
            &raw_path,
            &files,
            &theme,
        );
        return Html(html).into_response();
    }

    // File serving
    let is_preview = query.preview.as_deref() == Some("true");
    let file_size = metadata.len();
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let file = match tokio::fs::File::open(&file_path).await {
        Ok(f) => f,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file").into_response(),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    if is_preview {
        // Inline preview (images/videos) — no download header, no broadcast
        return Response::builder()
            .header(header::CONTENT_TYPE, get_mime_type(&file_path))
            .header(header::CONTENT_LENGTH, file_size.to_string())
            .body(body)
            .unwrap();
    }

    // Standard file download
    broadcast_msg(
        &state,
        &serde_json::json!({
            "type": "download-start",
            "filename": file_name,
            "totalSize": file_size,
        }),
    );

    // We broadcast completion after headers are sent.
    // The client-side progress uses its own tracking.
    let state_clone = state.clone();
    let fname = file_name.to_string();
    tokio::spawn(async move {
        // Small delay to let the download start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        broadcast_msg(
            &state_clone,
            &serde_json::json!({
                "type": "download-complete",
                "filename": fname,
            }),
        );
    });

    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_name),
        )
        .body(body)
        .unwrap()
}

// ─── POST handler for login + uploads ─────────────────────────────────

async fn handle_post(
    State(state): State<ServerState>,
    headers: HeaderMap,
    req: Request<Body>,
) -> Response {
    let shared_path = state.shared_path.read().await.clone();
    let password_hash = state.password_hash.read().await.clone();
    let theme = state.theme.read().await.clone();

    let raw_path = percent_encoding::percent_decode_str(req.uri().path())
        .decode_utf8_lossy()
        .to_string();

    // Handle login form submission
    if raw_path == "/login" {
        let body_bytes = match axum::body::to_bytes(req.into_body(), 1024 * 64).await {
            Ok(b) => b,
            Err(_) => return (StatusCode::BAD_REQUEST, "Invalid request body").into_response(),
        };
        let body_str = String::from_utf8_lossy(&body_bytes);
        let params: HashMap<String, String> = form_urlencoded::parse(body_str.as_bytes())
            .into_owned()
            .collect();

        let pwd = params.get("password").cloned().unwrap_or_default();
        let hashed = sha256_hex(&pwd);

        if hashed == password_hash {
            return Response::builder()
                .status(StatusCode::SEE_OTHER)
                .header(header::LOCATION, "/")
                .header(
                    header::SET_COOKIE,
                    format!(
                        "thlauh_session={}; Path=/; HttpOnly; Max-Age=31536000",
                        password_hash
                    ),
                )
                .body(Body::empty())
                .unwrap();
        } else {
            return Html(portal::generate_login_html(&theme, Some("Invalid password")))
                .into_response();
        }
    }

    // Auth check for uploads
    if !password_hash.is_empty() {
        let cookie_token = get_cookie(&headers, "thlauh_session");
        if cookie_token.as_deref() != Some(&password_hash) {
            return Html(portal::generate_login_html(&theme, None)).into_response();
        }
    }

    // File upload
    let safe_suffix = raw_path.trim_start_matches('/');
    let upload_dir = PathBuf::from(&shared_path).join(safe_suffix);

    if !upload_dir.exists() || !upload_dir.is_dir() {
        return (StatusCode::NOT_FOUND, "Target directory not found").into_response();
    }

    // Extract multipart from the request
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !content_type.starts_with("multipart/form-data") {
        return (StatusCode::BAD_REQUEST, "Expected multipart form data").into_response();
    }

    let mut multipart = match Multipart::from_request(req, &()).await {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to parse multipart: {}", e),
            )
                .into_response();
        }
    };

    let mut uploaded_count = 0u32;
    let mut last_filename = String::from("File");

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field
            .file_name()
            .unwrap_or("uploaded_file")
            .to_string();

        last_filename = file_name.clone();

        broadcast_msg(
            &state,
            &serde_json::json!({
                "type": "upload-start",
                "filename": &file_name,
                "totalSize": 0,
            }),
        );

        let target_path = upload_dir.join(&file_name);

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read upload: {}", e),
                )
                    .into_response();
            }
        };

        if let Err(e) = tokio::fs::write(&target_path, &data).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to save file: {}", e),
            )
                .into_response();
        }

        uploaded_count += 1;
    }

    if uploaded_count > 0 {
        broadcast_msg(
            &state,
            &serde_json::json!({
                "type": "upload-complete",
                "filename": last_filename,
            }),
        );
    }

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, raw_path.as_str())
        .body(Body::empty())
        .unwrap()
}

async fn handle_logo() -> Response {
    let bytes = include_bytes!("../icons/128x128.png");
    Response::builder()
        .header(header::CONTENT_TYPE, "image/png")
        .header(header::CONTENT_LENGTH, bytes.len().to_string())
        .body(Body::from(bytes.as_slice()))
        .unwrap()
}

// ─── WebSocket handler ────────────────────────────────────────────────

async fn handle_ws(
    State(state): State<ServerState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let password_hash = state.password_hash.read().await.clone();

    // Auth check for WebSocket connections
    if !password_hash.is_empty() {
        let cookie_token = get_cookie(&headers, "thlauh_session");
        if cookie_token.as_deref() != Some(&password_hash) {
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }

    ws.on_upgrade(move |socket| handle_ws_connection(socket, state))
}

async fn handle_ws_connection(mut socket: WebSocket, state: ServerState) {
    let mut rx = state.ws_tx.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(text) => {
                        if socket.send(axum::extract::ws::Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            ws_msg = socket.recv() => {
                match ws_msg {
                    Some(Ok(_)) => {
                        // Client messages are ignored (same as Electron version)
                    }
                    _ => break,
                }
            }
        }
    }
}

// ─── Start / Stop server ──────────────────────────────────────────────

pub fn start_sharing_server(
    dir_path: &str,
    host_address: &str,
    port: u16,
    password: Option<&str>,
    custom_mdns_host: Option<&str>,
) -> StartServerResult {
    // Stop any existing server first
    stop_sharing_server();

    let password_hash = match password {
        Some(pwd) if !pwd.is_empty() => sha256_hex(pwd),
        _ => String::new(),
    };

    // Publish mDNS
    let mdns_hostname = custom_mdns_host.unwrap_or("thlauh");
    let mdns_host = crate::mdns::publish_service(mdns_hostname, host_address, port);
    let mdns_url = format!("http://{}:{}", mdns_host, port);

    let (ws_tx, _) = broadcast::channel::<String>(256);

    let state = ServerState {
        shared_path: Arc::new(RwLock::new(dir_path.to_string())),
        password_hash: Arc::new(RwLock::new(password_hash.clone())),
        theme: Arc::new(RwLock::new("dark".to_string())),
        ws_tx: ws_tx.clone(),
    };

    // Store state globally for theme updates
    if let Ok(mut global_state) = SERVER_STATE.lock() {
        *global_state = Some(state.clone());
    }

    // Build axum router
    let app = Router::new()
        .route("/ws", get(handle_ws))
        .route("/logo.png", get(handle_logo))
        .fallback(|state: State<ServerState>, headers: HeaderMap, query: Query<FileQuery>, req: Request<Body>| async move {
            match req.method().clone() {
                Method::GET => handle_get(State(state.0), headers, query, req).await,
                Method::POST => handle_post(State(state.0), headers, req).await,
                _ => (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed").into_response(),
            }
        })
        .with_state(state.clone())
        .layer(CorsLayer::permissive());

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let addr: SocketAddr = format!("{}:{}", host_address, port)
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], port)));

    // Spawn the HTTP server
    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind server to {}: {}", addr, e);
                return;
            }
        };
        println!("Sharing server running at http://{}", addr);

        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();
    });

    // Set up file watcher
    let ws_tx_watcher = ws_tx.clone();
    let dir_path_owned = dir_path.to_string();
    let watcher = setup_file_watcher(&dir_path_owned, ws_tx_watcher);

    // Store the server handle
    if let Ok(mut handle) = SERVER_HANDLE.lock() {
        *handle = Some(RunningServer {
            shutdown_tx,
            _watcher: watcher,
        });
    }

    StartServerResult {
        password_hash,
        mdns_url,
    }
}

pub fn stop_sharing_server() {
    // Stop mDNS
    crate::mdns::unpublish_service();

    // Broadcast logout before shutdown
    if let Ok(global_state) = SERVER_STATE.lock() {
        if let Some(ref state) = *global_state {
            broadcast_msg(state, &serde_json::json!({"type": "logout"}));
        }
    }

    // Shutdown the HTTP server
    if let Ok(mut handle) = SERVER_HANDLE.lock() {
        if let Some(server) = handle.take() {
            let _ = server.shutdown_tx.send(());
            // Watcher is dropped automatically
        }
    }

    // Clear global state
    if let Ok(mut global_state) = SERVER_STATE.lock() {
        *global_state = None;
    }
}

pub async fn set_theme(theme: &str) {
    let state = {
        if let Ok(global_state) = SERVER_STATE.lock() {
            global_state.clone()
        } else {
            None
        }
    };

    if let Some(state) = state {
        *state.theme.write().await = theme.to_string();
        broadcast_msg(
            &state,
            &serde_json::json!({"type": "theme-changed", "theme": theme}),
        );
    }
}

// ─── File watcher ─────────────────────────────────────────────────────

fn setup_file_watcher(
    dir_path: &str,
    ws_tx: broadcast::Sender<String>,
) -> Option<RecommendedWatcher> {
    let ws_tx = ws_tx.clone();
    let mut watcher = match notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        if let Ok(event) = res {
            let filename = event
                .paths
                .first()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let msg = serde_json::json!({
                "type": "directory-changed",
                "filename": filename,
                "eventType": format!("{:?}", event.kind),
            });

            let _ = ws_tx.send(serde_json::to_string(&msg).unwrap_or_default());
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to start file watcher: {}", e);
            return None;
        }
    };

    if let Err(e) = watcher.watch(Path::new(dir_path), RecursiveMode::Recursive) {
        eprintln!("Failed to watch directory: {}", e);
        return None;
    }

    Some(watcher)
}
