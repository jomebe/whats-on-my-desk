use std::{collections::HashMap, sync::Arc, time::Duration};

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
use tokio::{
    sync::{broadcast, Mutex},
    time::interval,
};

use crate::devices::{self, DeviceSnapshot};

static WEB: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../dist");
const ORIGIN: &str = "http://127.0.0.1:47831";

#[derive(Clone)]
pub struct AppState {
    snapshot: Arc<Mutex<DeviceSnapshot>>,
    settings: Arc<Mutex<Settings>>,
    events: broadcast::Sender<AgentEvent>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentEvent {
    #[serde(rename = "type")]
    kind: &'static str,
    payload: DeviceSnapshot,
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
    let (events, _) = broadcast::channel(16);
    let state = AppState {
        snapshot: Arc::new(Mutex::new(devices::snapshot())),
        settings: Arc::new(Mutex::new(load_settings())),
        events,
    };
    spawn_watcher(state.clone());
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/device-snapshot", get(get_snapshot))
        .route("/api/refresh", post(refresh))
        .route("/api/settings", get(get_settings).put(put_settings))
        .route("/ws", get(websocket))
        .fallback(get(static_file))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:47831")
        .await
        .expect("bind local agent");
    let _ = webbrowser::open(ORIGIN);
    axum::serve(listener, app).await.expect("serve local agent");
}

fn spawn_watcher(state: AppState) {
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(1));
        loop {
            tick.tick().await;
            let next = devices::snapshot();
            let mut previous = state.snapshot.lock().await;
            let before = serde_json::to_string(&previous.devices).unwrap_or_default();
            let after = serde_json::to_string(&next.devices).unwrap_or_default();
            if before != after {
                *previous = next.clone();
                let _ = state.events.send(AgentEvent {
                    kind: "device-snapshot-updated",
                    payload: next,
                });
            }
        }
    });
}

async fn health() -> Json<HashMap<&'static str, &'static str>> {
    Json(HashMap::from([("status", "ok"), ("version", "0.1.0")]))
}
async fn get_snapshot(State(state): State<AppState>) -> Json<DeviceSnapshot> {
    Json(state.snapshot.lock().await.clone())
}
async fn refresh(State(state): State<AppState>) -> Json<DeviceSnapshot> {
    let next = devices::snapshot();
    *state.snapshot.lock().await = next.clone();
    let _ = state.events.send(AgentEvent {
        kind: "device-snapshot-updated",
        payload: next.clone(),
    });
    Json(next)
}
async fn get_settings(State(state): State<AppState>) -> Json<Settings> {
    Json(state.settings.lock().await.clone())
}
async fn put_settings(State(state): State<AppState>, Json(next): Json<Settings>) -> Json<Settings> {
    save_settings(&next);
    *state.settings.lock().await = next.clone();
    Json(next)
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
    let initial = state.snapshot.lock().await.clone();
    let _ = socket
        .send(Message::Text(
            serde_json::to_string(&AgentEvent {
                kind: "device-snapshot-updated",
                payload: initial,
            })
            .unwrap_or_default()
            .into(),
        ))
        .await;
    let mut events = state.events.subscribe();
    while let Ok(event) = events.recv().await {
        if socket
            .send(Message::Text(
                serde_json::to_string(&event).unwrap_or_default().into(),
            ))
            .await
            .is_err()
        {
            break;
        }
    }
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
