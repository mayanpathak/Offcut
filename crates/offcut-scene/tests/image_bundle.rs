//! The image bundle equals its manifest and the closed sets (v6.2implementation
//! D-447, D-470, D-471, D-495): `assets/images/manifest.json`, written by
//! `scripts/gen-image-bundle.mjs`, names every `EmojiId` and every `IconId` in
//! code order; each file it lists is a 160 x 160 RGBA PNG with the SHA-256 it
//! records, and is the file `ImageSet::embedded()` gives for that name's key.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use offcut_scene::images::ICON_BASE;
use offcut_scene::{ImageKey, ImageSet};
use offcut_types::{EmojiId, IconId};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn bundle_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/images")
}

fn manifest() -> Value {
    let text = std::fs::read_to_string(bundle_dir().join("manifest.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn images() -> Vec<Value> {
    manifest()["images"].as_array().unwrap().clone()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The manifest's ids of one set, in its order.
fn ids_of(set: &str) -> Vec<String> {
    images()
        .iter()
        .filter(|i| i["set"] == set)
        .map(|i| i["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn the_manifest_names_equal_the_closed_sets_in_code_order() {
    let all = images();
    assert_eq!(all.len(), 72);
    assert_eq!(ids_of("emoji"), EmojiId::NAMES.to_vec());
    assert_eq!(ids_of("icons"), IconId::NAMES.to_vec());
    // The emoji first, then the icons: no other set, nothing between.
    let sets: Vec<&str> = all.iter().map(|i| i["set"].as_str().unwrap()).collect();
    assert_eq!(sets[..48], ["emoji"; 48]);
    assert_eq!(sets[48..], ["icons"; 24]);
    // Each entry's code is its place in its set, and its key is ImageKey's.
    for (code, id) in EmojiId::ALL.iter().enumerate() {
        let entry = &all[code];
        assert_eq!(entry["code"], code);
        assert_eq!(entry["key"], ImageKey::of_emoji(*id).0);
    }
    for (code, id) in IconId::ALL.iter().enumerate() {
        let entry = &all[48 + code];
        assert_eq!(entry["code"], code);
        assert_eq!(entry["key"], ImageKey::of_icon(*id).0);
        assert_eq!(
            ImageKey::of_icon(*id).0,
            ICON_BASE + u16::try_from(code).unwrap()
        );
    }
    assert_eq!(manifest()["size"], ImageSet::SIZE);
}

#[test]
fn every_file_is_a_160_px_rgba_png_with_the_recorded_sha256() {
    for entry in images() {
        let file = entry["file"].as_str().unwrap();
        let bytes = std::fs::read(bundle_dir().join(file)).unwrap();
        assert_eq!(hex(&Sha256::digest(&bytes)), entry["sha256"], "{file}");
        assert_eq!(bytes.len(), entry["bytes"], "{file}");
        let mut reader = png::Decoder::new(std::io::Cursor::new(&bytes))
            .read_info()
            .unwrap();
        let mut data = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut data).unwrap();
        assert_eq!(
            (info.width, info.height, info.color_type, info.bit_depth),
            (
                ImageSet::SIZE,
                ImageSet::SIZE,
                png::ColorType::Rgba,
                png::BitDepth::Eight
            ),
            "{file}"
        );
        // Not blank: some pixel is opaque, and some is fully transparent
        // (every image is a shape on transparent, not a filled square).
        let alpha: Vec<u8> = data.iter().skip(3).step_by(4).copied().collect();
        assert!(alpha.contains(&255), "{file}: no opaque pixel");
        assert!(alpha.contains(&0), "{file}: no transparent pixel");
    }
}

#[test]
fn every_key_is_distinct_and_the_embedded_set_holds_the_manifest_files() {
    let all = images();
    let keys: BTreeSet<u64> = all.iter().map(|i| i["key"].as_u64().unwrap()).collect();
    assert_eq!(keys.len(), all.len(), "a key appears twice");
    let files: BTreeSet<&str> = all.iter().map(|i| i["file"].as_str().unwrap()).collect();
    assert_eq!(files.len(), all.len(), "a file appears twice");

    let set = ImageSet::embedded();
    assert_eq!(set.len(), all.len());
    for entry in &all {
        let key = ImageKey(u16::try_from(entry["key"].as_u64().unwrap()).unwrap());
        let png = set.get(key).unwrap();
        assert_eq!(
            hex(&Sha256::digest(png)),
            entry["sha256"],
            "{}",
            entry["id"]
        );
    }
}

#[test]
fn every_image_names_a_pinned_source_and_a_licence_file_that_exists() {
    for entry in images() {
        let source = entry["source"].as_str().unwrap();
        let pinned = source.starts_with(
            "https://raw.githubusercontent.com/microsoft/fluentui-emoji/1ffb34c752ecf5d402f04cfb4b392c77f57c54bc/assets/",
        ) || source.starts_with("https://unpkg.com/lucide-static@1.61.0/icons/");
        assert!(pinned, "{source}");
        assert_eq!(entry["upstream_sha256"].as_str().unwrap().len(), 64);
        let (licence, file) = match entry["set"].as_str().unwrap() {
            "emoji" => ("MIT", "LICENSES/fluent-emoji-MIT.txt"),
            _ => ("ISC", "LICENSES/lucide-ISC.txt"),
        };
        assert_eq!(entry["licence"], licence);
        assert_eq!(entry["licence_file"], file);
    }
    let fluent =
        std::fs::read_to_string(bundle_dir().join("LICENSES/fluent-emoji-MIT.txt")).unwrap();
    assert!(
        fluent.contains("MIT License") && fluent.contains("Copyright (c) Microsoft Corporation.")
    );
    let lucide = std::fs::read_to_string(bundle_dir().join("LICENSES/lucide-ISC.txt")).unwrap();
    assert!(lucide.contains("ISC License") && lucide.contains("Lucide Icons and Contributors"));
    assert!(lucide.contains("Cole Bemis"), "Feather's MIT notice");
}
