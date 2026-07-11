mod devices;
use std::{sync::Mutex, time::Duration};
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
fn get_device_snapshot() -> devices::DeviceSnapshot { devices::snapshot() }

#[tauri::command]
fn refresh_device_snapshot() -> devices::DeviceSnapshot { devices::snapshot() }

struct LastSnapshot(Mutex<String>);

fn start_device_watcher(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        let snapshot = devices::snapshot();
        let encoded = serde_json::to_string(&snapshot).unwrap_or_default();
        let state = app.state::<LastSnapshot>();
        let mut previous = state.0.lock().expect("device watcher state");
        if *previous != encoded {
            *previous = encoded;
            let _ = app.emit("device-snapshot-updated", snapshot);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(LastSnapshot(Mutex::new(String::new())))
        .invoke_handler(tauri::generate_handler![get_device_snapshot, refresh_device_snapshot])
        .setup(|app| { start_device_watcher(app.handle().clone()); Ok(()) })
        .run(tauri::generate_context!())
        .expect("error while running What’s on My Desk?");
}
