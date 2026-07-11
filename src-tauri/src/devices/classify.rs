use super::models::{RawDevice, VisualDevice};
use sha2::{Digest, Sha256};

fn has(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}

pub fn classify(raw: RawDevice) -> Option<VisualDevice> {
    let text = format!("{} {} {}", raw.name, raw.class_name, raw.instance_id).to_lowercase();
    if has(
        &text,
        &[
            "enumerator",
            "rfcomm",
            "a2dp",
            "avrcp",
            "hands-free",
            "headset gateway",
            "audio profile",
            "usb input device",
            "consumer control",
            "system controller",
            "software device",
            "root\\",
            "composite device",
            "studio effects",
            "spitcameragroup",
            "obs virtual",
            "print to pdf",
            "xps document writer",
            "onenote",
            "root print queue",
            "fax",
        ],
    ) {
        return None;
    }
    let category = if has(&text, &["keyboard", "키보드"]) {
        "keyboard"
    } else if has(&text, &["mouse", "pointing device", "마우스"]) {
        "mouse"
    } else if has(
        &text,
        &[
            "diskdrive",
            "disk drive",
            "mass storage",
            "external ssd",
            "usb storage",
            "external hdd",
        ],
    ) {
        "storage"
    } else if has(&text, &["camera", "webcam", "image device"]) {
        "camera"
    } else if has(
        &text,
        &["game controller", "gamepad", "xinput", "xbox controller"],
    ) {
        "gameController"
    } else if has(&text, &["printer", "printqueue"]) {
        "printer"
    } else if has(&text, &["headset", "headphone", "buds"]) {
        "headset"
    } else if has(&text, &["speaker", "audio output"]) {
        "speaker"
    } else if has(&text, &["microphone", "audio input"]) {
        "microphone"
    } else if has(&text, &["phone", "galaxy s", "iphone", "android"]) {
        "phone"
    } else if text.contains("usb") {
        "usbGeneric"
    } else {
        return None;
    };
    let connection = if text.contains("bluetooth") {
        "Bluetooth"
    } else if text.contains("usb") {
        "USB"
    } else {
        "Unknown"
    };
    let is_virtual = has(&text, &["virtual", "remote desktop", "software device"]);
    if is_virtual {
        return None;
    }
    let is_external = connection == "USB"
        || connection == "Bluetooth"
        || ((category == "keyboard" || category == "mouse")
            && !has(
                &text,
                &["acpi", "i8042", "ps/2", "touchpad", "precision touchpad"],
            ));
    let mut hash = Sha256::new();
    hash.update(raw.stable_key.as_bytes());
    Some(VisualDevice {
        id: format!("{:x}", hash.finalize())[..16].into(),
        category: category.into(),
        display_name: (!raw.name.is_empty()).then_some(raw.name),
        manufacturer: (!raw.manufacturer.is_empty()).then_some(raw.manufacturer),
        connection_type: connection.into(),
        icon_key: category.into(),
        count: 1,
        is_external,
        is_virtual,
        present: true,
        position_hint: None,
    })
}
