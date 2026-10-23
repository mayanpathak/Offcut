//! Styles are data on theme roles (D-136, TS §40.5): `theme.rs` is the only
//! home of a colour value, so no `Rgba {` literal appears in the scene's
//! source anywhere else but `display_list.rs`, which defines the type. A
//! brand kit (V6.7) then changes every colour by changing one `Theme`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The two files a colour literal may be written in.
const ALLOWED: [&str; 2] = ["theme.rs", "display_list.rs"];

/// Every `.rs` file under `dir`, in name order.
fn sources(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<PathBuf>>>()?;
    entries.sort();
    let mut files = Vec::new();
    for path in entries {
        if path.is_dir() {
            files.extend(sources(&path)?);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(files)
}

#[test]
fn no_colour_literal_outside_the_theme() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = sources(&src).unwrap();
    // The crate's files are read: `lib.rs`, `captions_draw.rs` and `events/`.
    let names: Vec<String> = files
        .iter()
        .map(|path| path.strip_prefix(&src).unwrap().display().to_string())
        .collect();
    assert!(names.len() > 15, "{names:?}");
    assert!(names.iter().any(|name| name == "captions_draw.rs"));
    let mut found = Vec::new();
    for (path, name) in files.iter().zip(&names) {
        if path.parent() == Some(src.as_path()) && ALLOWED.contains(&name.as_str()) {
            continue;
        }
        let text = fs::read_to_string(path).unwrap();
        for (number, line) in text.lines().enumerate() {
            if line.contains("Rgba {") {
                found.push(format!("{name}:{}: {}", number + 1, line.trim()));
            }
        }
    }
    assert!(
        found.is_empty(),
        "colour values belong in theme.rs (D-136):\n{}",
        found.join("\n")
    );
}
