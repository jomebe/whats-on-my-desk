use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{
    sync::{broadcast, mpsc, Mutex},
    time::interval,
};

use crate::devices::{self, watcher::NativeWatcher, DeviceSnapshot};

static WEB: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../dist");
const ORIGIN: &str = "http://127.0.0.1:47831";

#[derive(Clone)]
pub struct AppState {
    snapshot: Arc<Mutex<DeviceSnapshot>>,
    settings: Arc<Mutex<Settings>>,
    events: broadcast::Sender<AgentEvent>,
    diagnostics: Arc<Mutex<Diagnostics>>,
    clients: Arc<AtomicUsize>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentEvent {
    #[serde(rename = "type")]
    kind: &'static str,
    payload: DeviceSnapshot,
}

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct Diagnostics {
    native_notification_registered: bool,
    notification_callback_count: u64,
    last_native_event_at: Option<u64>,
    polling_iteration_count: u64,
    last_poll_at: Option<u64>,
    last_snapshot_rebuild_at: Option<u64>,
    last_snapshot_hash: String,
    last_broadcast_at: Option<u64>,
    connected_websocket_clients: usize,
    last_client_message_at: Option<u64>,
    latest_raw_device_count: u32,
    latest_visual_device_count: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub animations: bool,
    pub show_names: bool,
    pub show_built_in: bool,
    pub show_unknown: bool,
    pub show_usb_generic: bool,
    pub show_virtual: bool,
    pub show_printers: bool,
    pub theme: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            animations: true,
            show_names: false,
            show_built_in: false,
            show_unknown: false,
            show_usb_generic: false,
            show_virtual: false,
            show_printers: false,
            theme: "system".into(),
        }
    }
}

pub async fn serve() {
    eprintln!("[agent] starting snapshot");
    let first = devices::snapshot();
    eprintln!("[agent] snapshot ready");
    let (events, _) = broadcast::channel(32);
    let (native_tx, native_rx) = mpsc::unbounded_channel();
    let native = NativeWatcher::register(native_tx);
    let state = AppState {
        snapshot: Arc::new(Mutex::new(first.clone())),
        settings: Arc::new(Mutex::new(load_settings())),
        events,
        diagnostics: Arc::new(Mutex::new(Diagnostics {
            native_notification_registered: native.is_ok(),
            last_snapshot_hash: fingerprint(&first),
            latest_raw_device_count: first.raw_device_count,
            latest_visual_device_count: first.devices.len() as u32,
            ..Default::default()
        })),
        clients: Arc::new(AtomicUsize::new(0)),
    };
    if native.is_ok() {
        eprintln!("[devices] native notification registration success");
    } else {
        eprintln!("[devices] native notification registration failed; polling fallback active");
    }
    spawn_watchers(state.clone(), native_rx);
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/device-snapshot", get(get_snapshot))
        .route("/api/refresh", post(refresh))
        .route("/api/settings", get(get_settings).put(put_settings))
        .route("/api/diagnostics", get(get_diagnostics))
        .route("/ws", get(websocket))
        .fallback(get(static_file))
        .with_state(state);
    eprintln!("[agent] binding loopback");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:47831")
        .await
        .expect("bind local agent");
    let _keep_native_registration = native.ok();
    let _ = webbrowser::open(ORIGIN);
    axum::serve(listener, app).await.expect("serve local agent");
}

fn spawn_watchers(state: AppState, mut native_rx: mpsc::UnboundedReceiver<()>) {
    let native_state = state.clone();
    tokio::spawn(async move {
        while native_rx.recv().await.is_some() {
            {
                let mut diag = native_state.diagnostics.lock().await;
                diag.notification_callback_count += 1;
                diag.last_native_event_at = Some(now());
            }
            eprintln!("[devices] event=arrival debounce=500ms");
            tokio::time::sleep(Duration::from_millis(500)).await;
            while native_rx.try_recv().is_ok() {}
            rebuild_and_broadcast(&native_state, "native").await;
        }
    });
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(2));
        loop {
            tick.tick().await;
            {
                let mut diag = state.diagnostics.lock().await;
                diag.polling_iteration_count += 1;
                diag.last_poll_at = Some(now());
            }
            rebuild_and_broadcast(&state, "poll").await;
        }
    });
}

async fn rebuild_and_broadcast(state: &AppState, _: &str) -> DeviceSnapshot {
    let next = devices::snapshot();
    let next_hash = fingerprint(&next);
    let mut previous = state.snapshot.lock().await;
    let previous_hash = fingerprint(&previous);
    let changed = previous_hash != next_hash;
    if changed {
        *previous = next.clone();
        let _ = state.events.send(AgentEvent {
            kind: "device-snapshot-updated",
            payload: next.clone(),
        });
        eprintln!(
            "[ws] broadcast clients={}",
            state.clients.load(Ordering::Relaxed)
        );
    }
    let mut diag = state.diagnostics.lock().await;
    diag.last_snapshot_rebuild_at = Some(now());
    diag.last_snapshot_hash = next_hash;
    diag.latest_raw_device_count = next.raw_device_count;
    diag.latest_visual_device_count = next.devices.len() as u32;
    if changed {
        diag.last_broadcast_at = Some(now());
    }
    eprintln!(
        "[devices] raw={} visual={} changed={changed}",
        next.raw_device_count,
        next.devices.len()
    );
    next
}

fn fingerprint(snapshot: &DeviceSnapshot) -> String {
    let mut devices: Vec<_> = snapshot.devices.iter().collect();
    devices.sort_by(|a, b| a.id.cmp(&b.id));
    let data = serde_json::to_vec(&devices).unwrap_or_default();
    format!("{:x}", Sha256::digest(data))
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

async fn health() -> Json<HashMap<&'static str, &'static str>> {
    Json(HashMap::from([("status", "ok"), ("version", "0.2.0")]))
}
async fn get_snapshot(State(state): State<AppState>) -> Json<DeviceSnapshot> {
    Json(state.snapshot.lock().await.clone())
}
async fn refresh(State(state): State<AppState>) -> Json<DeviceSnapshot> {
    Json(rebuild_and_broadcast(&state, "manual").await)
}
async fn get_settings(State(state): State<AppState>) -> Json<Settings> {
    Json(state.settings.lock().await.clone())
}
async fn put_settings(State(state): State<AppState>, Json(next): Json<Settings>) -> Json<Settings> {
    save_settings(&next);
    *state.settings.lock().await = next.clone();
    Json(next)
}
async fn get_diagnostics(State(state): State<AppState>) -> Json<Diagnostics> {
    let mut diag = state.diagnostics.lock().await.clone();
    diag.connected_websocket_clients = state.clients.load(Ordering::Relaxed);
    Json(diag)
}

async fn websocket(
    headers: HeaderMap,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    if !allowed_origin(&headers) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(ws
        .on_upgrade(move |socket| websocket_task(socket, state))
        .into_response())
}
async fn websocket_task(mut socket: WebSocket, state: AppState) {
    state.clients.fetch_add(1, Ordering::Relaxed);
    eprintln!("[ws] client connected");
    let initial = state.snapshot.lock().await.clone();
    let _ = send_event(
        &mut socket,
        AgentEvent {
            kind: "device-snapshot-updated",
            payload: initial,
        },
    )
    .await;
    let mut events = state.events.subscribe();
    loop {
        tokio::select! {
            result = events.recv() => match result { Ok(event) => if send_event(&mut socket, event).await.is_err() { break }, Err(_) => break },
            incoming = socket.recv() => match incoming { Some(Ok(_)) => { state.diagnostics.lock().await.last_client_message_at = Some(now()); }, _ => break },
        }
    }
    state.clients.fetch_sub(1, Ordering::Relaxed);
}
async fn send_event(socket: &mut WebSocket, event: AgentEvent) -> Result<(), ()> {
    socket
        .send(Message::Text(
            serde_json::to_string(&event).unwrap_or_default().into(),
        ))
        .await
        .map_err(|_| ())
}

fn allowed_origin(headers: &HeaderMap) -> bool {
    headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .map(|origin| {
            origin == ORIGIN
                || origin == "http://127.0.0.1:1420"
                || origin == "http://localhost:1420"
        })
        .unwrap_or(true)
}
async fn static_file(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let file = WEB
        .get_file(if path.is_empty() { "index.html" } else { path })
        .or_else(|| WEB.get_file("index.html"));
    match file {
        Some(file) => (
            [(
                header::CONTENT_TYPE,
                mime_guess::from_path(file.path())
                    .first_or_octet_stream()
                    .as_ref(),
            )],
            file.contents(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
fn settings_path() -> std::path::PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("WhatsOnMyDesk")
        .join("settings.json")
}
fn load_settings() -> Settings {
    std::fs::read_to_string(settings_path())
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}
fn save_settings(settings: &Settings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(
        path,
        serde_json::to_vec_pretty(settings).unwrap_or_default(),
    );
}
