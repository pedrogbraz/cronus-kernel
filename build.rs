//! Packs `src/cronus_ui_css/*.css` (the per-family stylesheets listed in the
//! MANIFEST) into `$OUT_DIR/cronus_ui_css.pack` so the release binary carries
//! them compressed. `cronus_ui_css::lz::unpack` inflates the pack once at
//! runtime; the format is a tiny LZ77 (no crates on either side):
//!
//! ```text
//! pack   := entry*
//! entry  := u16 name_len, name bytes, u32 raw_len, u32 packed_len, packed bytes
//! packed := token*
//! token  := 0x00..=0x7F  n+1 literal bytes follow
//!         | 0x80..=0xFF  match: length (byte & 0x7F) + 3, then u16 LE offset (1..=65535)
//! ```
//!
//! `tokens.css`, `fallback.css`, `base.css` and `audit.css` stay `include_str!`
//! (they are referenced as constants); everything else in the directory is packed.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

const UNPACKED: &[&str] = &["tokens", "fallback", "base", "audit"];
const WINDOW: usize = 65_535;
const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 130;

fn compress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() / 2);
    let mut heads: HashMap<[u8; 3], Vec<usize>> = HashMap::new();
    let mut literals: Vec<u8> = Vec::new();
    let flush = |literals: &mut Vec<u8>, out: &mut Vec<u8>| {
        for chunk in literals.chunks(128) {
            out.push((chunk.len() - 1) as u8);
            out.extend_from_slice(chunk);
        }
        literals.clear();
    };
    let mut i = 0;
    while i < input.len() {
        let mut best_len = 0;
        let mut best_off = 0;
        if i + MIN_MATCH <= input.len() {
            let key = [input[i], input[i + 1], input[i + 2]];
            if let Some(positions) = heads.get(&key) {
                for &p in positions.iter().rev().take(64) {
                    if i - p > WINDOW {
                        break;
                    }
                    let mut len = 0;
                    while len < MAX_MATCH
                        && i + len < input.len()
                        && input[p + len] == input[i + len]
                    {
                        len += 1;
                    }
                    if len > best_len {
                        best_len = len;
                        best_off = i - p;
                        if len == MAX_MATCH {
                            break;
                        }
                    }
                }
            }
        }
        let advance = if best_len >= MIN_MATCH {
            flush(&mut literals, &mut out);
            out.push(0x80 | (best_len - MIN_MATCH) as u8);
            out.extend_from_slice(&(best_off as u16).to_le_bytes());
            best_len
        } else {
            literals.push(input[i]);
            1
        };
        for k in i..i + advance {
            if k + MIN_MATCH <= input.len() {
                heads
                    .entry([input[k], input[k + 1], input[k + 2]])
                    .or_default()
                    .push(k);
            }
        }
        i += advance;
    }
    flush(&mut literals, &mut out);
    out
}

fn main() {
    let dir = Path::new("src/cronus_ui_css");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("src/cronus_ui_css")
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.strip_suffix(".css").map(str::to_string)
        })
        .filter(|n| !UNPACKED.contains(&n.as_str()))
        .collect();
    names.sort();
    let mut pack = Vec::new();
    for name in &names {
        let path = dir.join(format!("{name}.css"));
        println!("cargo:rerun-if-changed={}", path.display());
        let raw = fs::read(&path).expect("css");
        let packed = compress(&raw);
        pack.extend_from_slice(&(name.len() as u16).to_le_bytes());
        pack.extend_from_slice(name.as_bytes());
        pack.extend_from_slice(&(raw.len() as u32).to_le_bytes());
        pack.extend_from_slice(&(packed.len() as u32).to_le_bytes());
        pack.extend_from_slice(&packed);
    }
    let out = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("cronus_ui_css.pack");
    fs::write(out, pack).expect("write pack");
}
