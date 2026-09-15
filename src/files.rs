//! `file` field storage: data-URL uploads become files next to the database,
//! served at `/_files/<id>.<ext>`. External `http(s):` URLs are stored as-is.

use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use rand::RngCore;
use serde_json::{Map, Value};

use crate::parser::{EntityNode, FieldType};
use crate::validation::FieldErrors;

/// Decoded payload ceiling (JSON bodies are also capped by `CRONUS_MAX_BODY_BYTES`).
pub const MAX_BYTES: usize = 512 * 1024;

/// Directory next to the sqlite file: `./data.db` → `./files`.
pub fn dir_for(db_path: &str) -> PathBuf {
    let p = Path::new(db_path);
    match p.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join("files"),
        _ => PathBuf::from("files"),
    }
}

/// Rewrite `file` fields in `body`: data URLs are written under `files_dir`
/// and replaced with `/_files/<id>.<ext>`. Empty values are left alone
/// (update without a new file). Other values are validated later.
pub fn persist_uploads(
    entity: &EntityNode,
    body: &mut Map<String, Value>,
    files_dir: &Path,
) -> Result<(), FieldErrors> {
    let mut errors = FieldErrors::new();
    for field in entity
        .fields
        .iter()
        .filter(|f| f.field_type == FieldType::File)
    {
        let Some(Value::String(raw)) = body.get(&field.name) else {
            continue;
        };
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        if raw.starts_with("http://") || raw.starts_with("https://") {
            continue;
        }
        if raw.starts_with("/_files/") {
            if !safe_stored_name(&raw["/_files/".len()..]) {
                errors.insert(
                    field.name.clone(),
                    vec!["must be a file upload or URL".into()],
                );
            }
            continue;
        }
        match store_data_url(raw, files_dir) {
            Ok(stored) => {
                body.insert(field.name.clone(), Value::String(stored));
            }
            Err(msg) => {
                errors.insert(field.name.clone(), vec![msg]);
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn store_data_url(raw: &str, files_dir: &Path) -> Result<String, String> {
    let rest = raw
        .strip_prefix("data:")
        .ok_or_else(|| "must be a file upload or URL".to_string())?;
    let (meta, b64) = rest
        .split_once(',')
        .ok_or_else(|| "must be a file upload or URL".to_string())?;
    if !meta.contains("base64") {
        return Err("must be a file upload or URL".into());
    }
    let mime = meta.split(';').next().unwrap_or("").trim();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64.trim())
        .map_err(|_| "must be a file upload or URL".to_string())?;
    if bytes.is_empty() {
        return Err("must be a file upload or URL".into());
    }
    if bytes.len() > MAX_BYTES {
        return Err(format!("must be at most {MAX_BYTES} bytes"));
    }
    let ext = ext_for_mime(mime);
    let name = format!("{}{ext}", opaque_id());
    fs::create_dir_all(files_dir).map_err(|_| "could not store file".to_string())?;
    fs::write(files_dir.join(&name), &bytes).map_err(|_| "could not store file".to_string())?;
    Ok(format!("/_files/{name}"))
}

fn ext_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/png" => ".png",
        "image/jpeg" | "image/jpg" => ".jpg",
        "image/gif" => ".gif",
        "image/webp" => ".webp",
        "application/pdf" => ".pdf",
        "text/plain" => ".txt",
        _ => ".bin",
    }
}

fn opaque_id() -> String {
    let mut b = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn safe_stored_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 80
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
}

/// Read a stored file. `None` when the name is unsafe or the file is missing.
pub fn read_stored(files_dir: &Path, name: &str) -> Option<(String, Vec<u8>)> {
    if !safe_stored_name(name) {
        return None;
    }
    let bytes = fs::read(files_dir.join(name)).ok()?;
    let mime = match Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };
    Some((mime.to_string(), bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{parse, AstNode};

    fn entity() -> EntityNode {
        let src = "entity Doc { title string  avatar file }\n";
        match parse(src).unwrap().into_iter().next() {
            Some(AstNode::Entity(e)) => e,
            _ => panic!("entity"),
        }
    }

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cronus-files-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn data_url_is_written_and_path_is_safe() {
        let dir = tmp();
        let e = entity();
        let png = base64::engine::general_purpose::STANDARD.encode([0x89, b'P', b'N', b'G']);
        let mut body = Map::new();
        body.insert(
            "avatar".into(),
            Value::String(format!("data:image/png;base64,{png}")),
        );
        persist_uploads(&e, &mut body, &dir).unwrap();
        let stored = body["avatar"].as_str().unwrap();
        assert!(stored.starts_with("/_files/"), "{stored}");
        assert!(stored.ends_with(".png"), "{stored}");
        let name = &stored["/_files/".len()..];
        let (mime, bytes) = read_stored(&dir, name).expect("read");
        assert_eq!(mime, "image/png");
        assert_eq!(bytes, vec![0x89, b'P', b'N', b'G']);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn http_url_is_left_alone() {
        let dir = tmp();
        let e = entity();
        let mut body = Map::new();
        body.insert(
            "avatar".into(),
            Value::String("https://cdn.example/a.png".into()),
        );
        persist_uploads(&e, &mut body, &dir).unwrap();
        assert_eq!(body["avatar"], "https://cdn.example/a.png");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn traversal_name_is_rejected() {
        assert!(!safe_stored_name("../x"));
        assert!(!safe_stored_name("a/b"));
        assert!(safe_stored_name("ab12.png"));
        assert!(read_stored(Path::new("."), "../x").is_none());
    }

    #[test]
    fn oversized_data_url_fails() {
        let dir = tmp();
        let e = entity();
        let big = base64::engine::general_purpose::STANDARD.encode(vec![0u8; MAX_BYTES + 1]);
        let mut body = Map::new();
        body.insert(
            "avatar".into(),
            Value::String(format!("data:application/octet-stream;base64,{big}")),
        );
        let err = persist_uploads(&e, &mut body, &dir).unwrap_err();
        assert!(err["avatar"][0].contains("at most"), "{err:?}");
        let _ = fs::remove_dir_all(&dir);
    }
}
