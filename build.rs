//! Generate public test fixtures from text source; no binary files are committed.
use std::{env, fs, path::Path};

fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320u32 & (0u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}
fn zip(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut data = Vec::new();
    let mut central = Vec::new();
    for (name, contents) in files {
        let offset = data.len() as u32;
        let size = contents.len() as u32;
        let checksum = crc32(contents);
        data.extend_from_slice(&[0x50, 0x4b, 3, 4, 20, 0, 0, 0, 0, 0, 0, 0, 0x21, 0]);
        data.extend_from_slice(&checksum.to_le_bytes());
        data.extend_from_slice(&size.to_le_bytes());
        data.extend_from_slice(&size.to_le_bytes());
        data.extend_from_slice(&(name.len() as u16).to_le_bytes());
        data.extend_from_slice(&[0, 0]);
        data.extend_from_slice(name.as_bytes());
        data.extend_from_slice(contents);
        central.extend_from_slice(&[0x50, 0x4b, 1, 2, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0x21, 0]);
        central.extend_from_slice(&checksum.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&(name.len() as u16).to_le_bytes());
        central.extend_from_slice(&[0; 12]);
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name.as_bytes());
    }
    let offset = data.len() as u32;
    let size = central.len() as u32;
    data.extend(central);
    data.extend_from_slice(&[0x50, 0x4b, 5, 6, 0, 0, 0, 0]);
    data.extend_from_slice(&(files.len() as u16).to_le_bytes());
    data.extend_from_slice(&(files.len() as u16).to_le_bytes());
    data.extend_from_slice(&size.to_le_bytes());
    data.extend_from_slice(&offset.to_le_bytes());
    data.extend_from_slice(&[0, 0]);
    data
}
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=examples/button-app");
    println!("cargo:rerun-if-changed=tests/fixtures/photo.hex");
    let out = env::var_os("OUT_DIR").unwrap();
    let out = Path::new(&out);
    let hex = fs::read_to_string("tests/fixtures/photo.hex").unwrap();
    let hex = hex.trim();
    let photo: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    fs::write(out.join("photo.png"), photo).unwrap();
    fs::write(
        out.join("minimal.xdc"),
        zip(&[(
            "index.html",
            b"<!doctype html><title>Test</title><p>Sigma test app</p>",
        )]),
    )
    .unwrap();
    let index = fs::read("examples/button-app/index.html").unwrap();
    let manifest = fs::read("examples/button-app/manifest.toml").unwrap();
    fs::write(
        out.join("button-app.xdc"),
        zip(&[("index.html", &index), ("manifest.toml", &manifest)]),
    )
    .unwrap();
    let samples: Vec<u8> = (0..16000)
        .flat_map(|i| {
            let phase = (i as f64) * 440.0 * std::f64::consts::TAU / 16000.0;
            ((phase.sin() * 4000.0) as i16).to_le_bytes()
        })
        .collect();
    let mut wav = b"RIFF".to_vec();
    wav.extend_from_slice(&(36 + samples.len() as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt \x10\0\0\0\x01\0\x01\0");
    wav.extend_from_slice(&16000u32.to_le_bytes());
    wav.extend_from_slice(&32000u32.to_le_bytes());
    wav.extend_from_slice(b"\x02\0\x10\0data");
    wav.extend_from_slice(&(samples.len() as u32).to_le_bytes());
    wav.extend(samples);
    fs::write(out.join("voice-tone.wav"), wav).unwrap();
}
