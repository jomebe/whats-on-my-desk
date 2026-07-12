use crate::devices::{self, DeviceSnapshot};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub fn run() {
    let output = Arc::new(Mutex::new(std::io::stdout()));
    let state = Arc::new(Mutex::new(devices::snapshot()));
    write(&output, json!({"type":"ready","version":"0.2.0"}));
    let scan_output = output.clone();
    let scan_state = state.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(2));
        let mut next = devices::snapshot();
        let mut previous = scan_state.lock().expect("native snapshot");
        if fingerprint(&next) != fingerprint(&previous) {
            next.revision = previous.revision + 1;
            *previous = next.clone();
            write(
                &scan_output,
                json!({"type":"device-snapshot-updated","payload":next}),
            );
            eprintln!("[native] snapshot updated");
        }
    });
    let mut input = std::io::stdin();
    while let Some(message) = read(&mut input) {
        match message.get("type").and_then(Value::as_str) {
            Some("ping") => write(&output, json!({"type":"ready","version":"0.2.0"})),
            Some("get-device-snapshot") => write(
                &output,
                json!({"type":"device-snapshot","payload":state.lock().expect("native snapshot").clone()}),
            ),
            Some("refresh-device-snapshot") => {
                let mut next = devices::snapshot();
                let mut previous = state.lock().expect("native snapshot");
                next.revision = previous.revision + 1;
                *previous = next.clone();
                write(&output, json!({"type":"device-snapshot","payload":next}));
            }
            Some("get-diagnostics") => write(
                &output,
                json!({"type":"diagnostics","payload":{"mode":"nativeMessaging"}}),
            ),
            _ => write(
                &output,
                json!({"type":"agent-error","payload":{"code":"INVALID_MESSAGE","message":"Unsupported message"}}),
            ),
        }
    }
}
fn read(input: &mut impl Read) -> Option<Value> {
    let mut length = [0; 4];
    input.read_exact(&mut length).ok()?;
    let mut bytes = vec![0; u32::from_le_bytes(length) as usize];
    input.read_exact(&mut bytes).ok()?;
    serde_json::from_slice(&bytes).ok()
}
fn write(output: &Arc<Mutex<std::io::Stdout>>, value: Value) {
    let bytes = serde_json::to_vec(&value).unwrap_or_default();
    let mut out = output.lock().expect("native output");
    let _ = out.write_all(&(bytes.len() as u32).to_le_bytes());
    let _ = out.write_all(&bytes);
    let _ = out.flush();
}
fn fingerprint(snapshot: &DeviceSnapshot) -> String {
    let mut devices: Vec<_> = snapshot.devices.iter().collect();
    devices.sort_by(|a, b| a.id.cmp(&b.id));
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&devices).unwrap_or_default())
    )
}
