use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::models::{MidiInfo, VisualDevice};

#[cfg(windows)]
pub fn enumerate() -> Vec<VisualDevice> {
    use windows::Win32::Media::Audio::{
        midiInGetDevCapsW, midiInGetNumDevs, midiOutGetDevCapsW, midiOutGetNumDevs, MIDIINCAPSW,
        MIDIOUTCAPSW,
    };

    let mut ports: BTreeMap<String, (String, bool, bool)> = BTreeMap::new();
    unsafe {
        for index in 0..midiInGetNumDevs() {
            let mut caps = MIDIINCAPSW::default();
            if midiInGetDevCapsW(
                index as usize,
                &mut caps,
                std::mem::size_of::<MIDIINCAPSW>() as u32,
            ) == 0
            {
                let name = std::ptr::addr_of!(caps.szPname).read_unaligned();
                add_port(&mut ports, wide(&name), true);
            }
        }
        for index in 0..midiOutGetNumDevs() {
            let mut caps = MIDIOUTCAPSW::default();
            if midiOutGetDevCapsW(
                index as usize,
                &mut caps,
                std::mem::size_of::<MIDIOUTCAPSW>() as u32,
            ) == 0
            {
                let name = std::ptr::addr_of!(caps.szPname).read_unaligned();
                add_port(&mut ports, wide(&name), false);
            }
        }
    }

    ports
        .into_iter()
        .map(|(key, (name, input, output))| {
            let category = if has(
                &key,
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
                "midiKeyboard"
            } else if has(&key, &["interface", "midi in", "midi out", "scarlett"]) {
                "midiInterface"
            } else {
                "midiController"
            };
            let mut hash = Sha256::new();
            hash.update(key.as_bytes());
            VisualDevice {
                id: format!("{:x}", hash.finalize())[..16].into(),
                category: category.into(),
                display_name: Some(name),
                manufacturer: None,
                connection_type: "USB".into(),
                icon_key: category.into(),
                count: 1,
                is_external: true,
                is_virtual: false,
                present: true,
                position_hint: None,
                visual_variant: Some(category.into()),
                midi: Some(MidiInfo {
                    has_input: input,
                    has_output: output,
                    port_count: input as u32 + output as u32,
                }),
            }
        })
        .collect()
}

#[cfg(not(windows))]
pub fn enumerate() -> Vec<VisualDevice> {
    vec![]
}

fn add_port(ports: &mut BTreeMap<String, (String, bool, bool)>, name: String, input: bool) {
    if name.is_empty() || has(&name.to_lowercase(), &["mapper", "synth", "software"]) {
        return;
    }
    let key = normalize(&name);
    let entry = ports.entry(key).or_insert((name, false, false));
    if input {
        entry.1 = true;
    } else {
        entry.2 = true;
    }
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .replace(" midi in", "")
        .replace(" midi out", "")
        .replace(" input", "")
        .replace(" output", "")
        .trim()
        .into()
}

fn wide(value: &[u16]) -> String {
    String::from_utf16_lossy(&value[..value.iter().position(|x| *x == 0).unwrap_or(value.len())])
        .trim()
        .into()
}

fn has(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}
