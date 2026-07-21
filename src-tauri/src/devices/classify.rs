use super::{
    models::{RawDevice, VisualDevice},
    raw_input::InputPresence,
};
use sha2::{Digest, Sha256};

fn has(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}

pub fn classify(raw: RawDevice, raw_input: &InputPresence) -> Option<VisualDevice> {
    let leaf_text = format!(
        "{} {} {} {} {}",
        raw.name, raw.physical_name, raw.manufacturer, raw.class_name, raw.instance_id
    )
    .to_lowercase();
    let all_text = format!("{} {}", leaf_text, raw.metadata_text.to_lowercase());
    if ignored_midi_endpoint(&all_text) || ignored_component(&leaf_text) {
        return None;
    }
    let category = category(&all_text)?;
    if category == "usbGeneric" && !raw.is_external {
        return None;
    }
    let is_virtual = has(
        &leaf_text,
        &["virtual", "remote desktop", "software device", "root\\"],
    );
    if is_virtual {
        return None;
    }
    let internal = !raw.is_external
        || has(
            &leaf_text,
            &[
                "acpi",
                "i8042",
                "ps/2",
                "touchpad",
                "precision touchpad",
                "i2c hid",
                "elan",
                "synaptics",
                "internal pointing",
            ],
        );
    let raw_input_present = (category == "keyboard" || category == "mouse")
        && raw_input.contains(category, &raw.instance_id);
    if (category == "keyboard" || category == "mouse") && (!raw_input_present || internal) {
        eprintln!("[input] category={category} raw_input_present={raw_input_present} internal={internal} selected=false name={}", raw.physical_name);
        return None;
    }
    if category == "keyboard" || category == "mouse" {
        eprintln!("[input] category={category} raw_input_present={raw_input_present} internal={internal} selected=true name={}", raw.physical_name);
    }
    let mut hash = Sha256::new();
    hash.update(raw.stable_key.as_bytes());
    let display_name = if raw.physical_name.is_empty() {
        raw.name
    } else {
        raw.physical_name
    };
    Some(VisualDevice {
        id: format!("{:x}", hash.finalize())[..16].into(),
        category: category.into(),
        display_name: (!display_name.is_empty()).then_some(display_name),
        manufacturer: (!raw.manufacturer.is_empty()).then_some(raw.manufacturer),
        connection_type: raw.connection_type,
        icon_key: category.into(),
        count: 1,
        is_external: raw.is_external,
        is_virtual,
        present: true,
        position_hint: None,
        visual_variant: category.starts_with("midi").then_some(category.into()),
        midi: None,
    })
}

fn category(text: &str) -> Option<&'static str> {
    if has(text, &["midi", "musical instrument", "ump endpoint"]) {
        if has(
            text,
            &[
                "piano",
                "keystation",
                "keylab",
                "launchkey",
                "komplete",
                "oxygen",
                "keyboard",
                "yamaha",
                "roland",
                "korg",
                "casio",
            ],
        ) {
            Some("midiKeyboard")
        } else if has(
            text,
            &["interface", "midi in", "midi out", "scarlett", "din midi"],
        ) {
            Some("midiInterface")
        } else {
            Some("midiController")
        }
    } else if has(text, &["keyboard", "키보드"]) {
        Some("keyboard")
    } else if has(text, &["mouse", "pointing device", "마우스"]) {
        Some("mouse")
    } else if has(
        text,
        &[
            "diskdrive",
            "disk drive",
            "mass storage",
            "external ssd",
            "usb storage",
            "external hdd",
            "scsi disk",
            "storage",
            "디스크",
            "저장 장치",
        ],
    ) {
        Some("storage")
    } else if has(text, &["camera", "webcam", "image device", "카메라"]) {
        Some("camera")
    } else if has(
        text,
        &["game controller", "gamepad", "xinput", "xbox controller"],
    ) {
        Some("gameController")
    } else if has(text, &["printer", "printqueue", "프린터"]) {
        Some("printer")
    } else if has(
        text,
        &["headset", "headphone", "earbud", "buds", "헤드폰", "이어폰"],
    ) {
        Some("headset")
    } else if has(text, &["speaker", "audio output", "스피커"]) {
        Some("speaker")
    } else if has(text, &["microphone", "audio input", "마이크"]) {
        Some("microphone")
    } else if has(text, &["phone", "galaxy s", "iphone", "android", "휴대폰"]) {
        Some("phone")
    } else if text.contains("usb") {
        Some("usbGeneric")
    } else {
        None
    }
}

fn ignored_component(text: &str) -> bool {
    has(
        text,
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
            "hancom pdf",
            "root print queue",
            "printenum\\printqueues",
            "fax",
        ],
    )
}

fn ignored_midi_endpoint(text: &str) -> bool {
    has(
        text,
        &[
            "midiu_diag",
            "midiu_loop",
            "midi 2.0 service tests",
            "service test loopback",
            "diagnostic loopback",
            "midi 2.0 virtual devices",
            "microsoft gs wavetable synth",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(name: &str, metadata: &str, connection_type: &str) -> RawDevice {
        RawDevice {
            stable_key: name.into(),
            name: name.into(),
            physical_name: name.into(),
            manufacturer: String::new(),
            class_name: String::new(),
            instance_id: metadata.into(),
            metadata_text: metadata.into(),
            connection_type: connection_type.into(),
            is_external: true,
        }
    }

    #[test]
    fn midi_keyboard_wins_before_generic_usb_keyboard() {
        let device = classify(
            raw("Launchkey MIDI Keyboard", "USB\\VID_1234&PID_5678", "USB"),
            &InputPresence::default(),
        )
        .expect("MIDI device");
        assert_eq!(device.category, "midiKeyboard");
        assert_eq!(device.connection_type, "USB");
    }

    #[test]
    fn usb_parent_keeps_scsi_storage_external() {
        let device = classify(
            raw(
                "Seagate One Touch SCSI Disk Device",
                "SCSI\\Disk parent USB\\VID_0BC2&PID_AB75",
                "USB",
            ),
            &InputPresence::default(),
        )
        .expect("external storage");
        assert_eq!(device.category, "storage");
        assert!(device.is_external);
    }

    #[test]
    fn diagnostic_midi_endpoints_are_not_visual_devices() {
        assert!(classify(
            raw(
                "Service Test Loopback A",
                "SWD\\MIDISRV\\MIDIU_DIAG_LOOPBACK_A",
                "Unknown",
            ),
            &InputPresence::default(),
        )
        .is_none());
    }

    #[test]
    fn external_keyboard_survives_root_and_acpi_ancestry() {
        let instance_id = "HID\\VID_046D&PID_C54D&MI_01&COL01\\7&ABC&0&0000";
        let mut device = raw("HID Keyboard Device", instance_id, "USB");
        device.metadata_text =
            format!("{instance_id} USB\\VID_046D&PID_C54D ACPI\\PNP0A08 HTREE\\ROOT\\0");
        let input = InputPresence::with_keyboard(instance_id);
        let visual = classify(device, &input).expect("external keyboard");
        assert_eq!(visual.category, "keyboard");
    }

    #[test]
    fn root_virtual_camera_is_not_visualized() {
        assert!(classify(
            raw(
                "Mirametrix Virtual Camera",
                "ROOT\\MMXVIRTUALCAMERA\\0000",
                "Unknown",
            ),
            &InputPresence::default(),
        )
        .is_none());
    }

    #[test]
    fn software_printer_is_not_visualized_even_when_marked_external() {
        for name in ["Microsoft Print to PDF", "Hancom PDF", "Root Print Queue"] {
            assert!(classify(
                raw(name, "SWD\\PRINTENUM\\PRINTQUEUES", "Network"),
                &InputPresence::default(),
            )
            .is_none());
        }
    }
}
