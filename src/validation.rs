//! Field validation for every write surface: REST (`api_crud.rs`), `/_form`
//! and `/_action` (`actions.rs`) and GraphQL (`graphql.rs`). One rule set,
//! reported per field so clients can show the message next to the input:
//!
//! ```json
//! {"error":{"code":"VALIDATION_FAILED","message":"title must be at least 3 characters",
//!           "fields":{"title":["must be at least 3 characters"]}}}
//! ```
//!
//! Messages name the violated rule. They never echo the submitted value and
//! never carry database text. Grammar and messages: LANGUAGE.md §3.6.

use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use crate::database::CronusDB;
use crate::parser::{EntityNode, FieldNode, FieldType};

/// Field name → messages, in field-name order (stable JSON).
pub type FieldErrors = BTreeMap<String, Vec<String>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Every declared field is checked; missing required fields fail.
    Create,
    /// Only fields present in the body are checked.
    Update,
}

/// Most ids a many-to-many field accepts in one write.
pub const MAX_LINKS: usize = 1000;

/// JSON value as the text the database stores.
pub fn text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn is_blank(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => true,
        Some(Value::String(s)) => s.trim().is_empty(),
        _ => false,
    }
}

fn number_text(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

fn is_date(s: &str) -> bool {
    parse_ymd(s).is_some()
}

fn parse_ymd(s: &str) -> Option<(u32, u32, u32)> {
    let mut parts = s.split('-');
    let y: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || s.len() != 10 {
        return None;
    }
    if !(1..=12).contains(&m) || d == 0 {
        return None;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let dim = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if d > dim[(m - 1) as usize] {
        return None;
    }
    Some((y, m, d))
}

fn is_datetime(s: &str) -> bool {
    let Some((date, time)) = s.split_once('T').or_else(|| s.split_once(' ')) else {
        return false;
    };
    parse_ymd(date).is_some() && parse_hms_offset(time)
}

fn two_digits(s: &str, max: u32) -> Option<u32> {
    if s.len() != 2 {
        return None;
    }
    let n: u32 = s.parse().ok()?;
    (n <= max).then_some(n)
}

fn parse_hms_offset(time: &str) -> bool {
    let time = time
        .strip_suffix('Z')
        .or_else(|| time.strip_suffix('z'))
        .unwrap_or(time);
    let time = if let Some(cut) = time.rfind('+').or_else(|| {
        time.match_indices('-')
            .map(|(i, _)| i)
            .find(|&i| i > 0 && time.len() - i == 6)
    }) {
        let off = &time[cut..];
        if off.len() != 6 || off.as_bytes().get(3) != Some(&b':') {
            return false;
        }
        if two_digits(&off[1..3], 14).is_none() || two_digits(&off[4..6], 59).is_none() {
            return false;
        }
        &time[..cut]
    } else {
        time
    };
    let mut segs = time.split(':');
    let Some(hh) = segs.next() else {
        return false;
    };
    let Some(mm) = segs.next() else {
        return false;
    };
    if two_digits(hh, 23).is_none() || two_digits(mm, 59).is_none() {
        return false;
    }
    match segs.next() {
        None => true,
        Some(ss) => {
            let (ss, frac) = ss.split_once('.').unwrap_or((ss, ""));
            two_digits(ss, 60).is_some()
                && frac.chars().all(|c| c.is_ascii_digit())
                && segs.next().is_none()
        }
    }
}

fn is_file_ref(s: &str) -> bool {
    if s.starts_with("http://") || s.starts_with("https://") {
        return !s.contains(' ');
    }
    s.strip_prefix("/_files/")
        .is_some_and(crate::files::safe_stored_name)
}

fn is_email(s: &str) -> bool {
    !s.contains(char::is_whitespace)
        && s.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !domain.contains('@')
        })
}

/// Type and constraint messages for one non-empty scalar value.
pub fn value_errors(field: &FieldNode, value: &Value) -> Vec<String> {
    let s = text(value);
    let mut out = Vec::new();
    match field.field_type {
        FieldType::Number | FieldType::Money | FieldType::Percentage => {
            let Some(n) = s.trim().parse::<f64>().ok().filter(|n| n.is_finite()) else {
                return vec!["must be a number".into()];
            };
            if let Some(min) = field.min.filter(|min| n < *min) {
                out.push(format!("must be at least {}", number_text(min)));
            }
            if let Some(max) = field.max.filter(|max| n > *max) {
                out.push(format!("must be at most {}", number_text(max)));
            }
        }
        FieldType::Boolean => {
            if !matches!(
                s.to_ascii_lowercase().as_str(),
                "true" | "false" | "1" | "0"
            ) {
                return vec!["must be true or false".into()];
            }
        }
        FieldType::Email if !is_email(&s) => out.push("must be a valid email".into()),
        FieldType::Url if !(s.starts_with("http://") || s.starts_with("https://")) => {
            out.push("must be a valid URL".into())
        }
        FieldType::Date if !is_date(&s) => out.push("must be a date (YYYY-MM-DD)".into()),
        FieldType::DateTime if !is_datetime(&s) => {
            out.push("must be a datetime (YYYY-MM-DDTHH:MM)".into())
        }
        FieldType::File if !is_file_ref(&s) => out.push("must be a file upload or URL".into()),
        FieldType::Enum => {
            if let Some(allowed) = field.enum_values.as_ref().filter(|a| !a.contains(&s)) {
                out.push(format!("must be one of: {}", allowed.join(", ")));
            }
        }
        _ => {}
    }
    let chars = s.chars().count();
    if let Some(min) = field.min_length.filter(|min| chars < *min) {
        out.push(format!("must be at least {min} characters"));
    }
    if let Some(max) = field.max_length.filter(|max| chars > *max) {
        out.push(format!("must be at most {max} characters"));
    }
    if let Some(pattern) = &field.pattern {
        match regex::Regex::new(pattern) {
            Ok(re) if re.is_match(&s) => {}
            Ok(_) => out.push(format!("must match the pattern {pattern}")),
            // `build` rejects invalid patterns (FIELD_001); an AST built in
            // code can still carry one. Fail closed without echoing it.
            Err(_) => {
                eprintln!(
                    "  \x1b[31m✗\x1b[0m field '{}' has an invalid match: pattern",
                    field.name
                );
                out.push("cannot be validated".into());
            }
        }
    }
    out
}

fn link_shape_errors(field: &FieldNode, value: Option<&Value>) -> Vec<String> {
    match value {
        None | Some(Value::Null) if field.required => vec!["is required".into()],
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(ids)) => {
            if ids
                .iter()
                .any(|v| !v.as_str().is_some_and(|s| !s.is_empty()))
            {
                vec!["must be an array of ids".into()]
            } else if ids.len() > MAX_LINKS {
                vec![format!("must have at most {MAX_LINKS} ids")]
            } else if ids.is_empty() && field.required {
                vec!["is required".into()]
            } else {
                Vec::new()
            }
        }
        Some(_) => vec!["must be an array of ids".into()],
    }
}

/// Validates a write body that already went through `authz::writable_body`.
/// Many-to-many fields (`-> X[]`) are checked for shape here; whether the ids
/// exist and are readable is `relations::access_errors`.
pub fn field_errors(entity: &EntityNode, body: &Map<String, Value>, mode: Mode) -> FieldErrors {
    let mut errors = FieldErrors::new();
    for field in &entity.fields {
        if crate::authz::SYSTEM_FIELDS.contains(&field.name.as_str()) {
            continue;
        }
        let value = body.get(&field.name);
        if mode == Mode::Update && value.is_none() {
            continue;
        }
        let messages = if field.is_many() {
            link_shape_errors(field, value)
        } else {
            match value {
                v if is_blank(v) && field.required => vec!["is required".into()],
                v if is_blank(v) => Vec::new(),
                Some(v) => value_errors(field, v),
                None => Vec::new(),
            }
        };
        if !messages.is_empty() {
            errors.insert(field.name.clone(), messages);
        }
    }
    errors
}

/// `unique` fields whose value is already taken by another row (`409`).
/// `exclude_id` is the row being updated.
pub fn unique_errors(
    db: &CronusDB,
    entity: &EntityNode,
    body: &Map<String, Value>,
    exclude_id: Option<&str>,
) -> FieldErrors {
    let mut errors = FieldErrors::new();
    if !crate::security::is_safe_identifier(&entity.name) {
        return errors;
    }
    for field in entity.fields.iter().filter(|f| f.unique && !f.is_many()) {
        let Some(value) = body.get(&field.name).filter(|v| !is_blank(Some(v))) else {
            continue;
        };
        if !crate::security::is_safe_identifier(&field.name) {
            continue;
        }
        let mut sql = format!(
            "SELECT 1 AS hit FROM \"{}\" WHERE \"{}\" = ?",
            entity.name, field.name
        );
        let mut params = vec![text(value)];
        if let Some(id) = exclude_id {
            sql.push_str(" AND \"id\" != ?");
            params.push(id.to_string());
        }
        sql.push_str(" LIMIT 1");
        match db.query_raw_params(&sql, &params) {
            Ok(rows) if !rows.is_empty() => {
                errors.insert(field.name.clone(), vec!["already exists".into()]);
            }
            Ok(_) => {}
            Err(e) => eprintln!(
                "  \x1b[31m✗\x1b[0m unique check {}.{} failed: {}",
                entity.name, field.name, e
            ),
        }
    }
    errors
}

/// Error `message`: the single failure, or how many fields failed.
pub fn message(errors: &FieldErrors) -> String {
    match errors.iter().next() {
        Some((field, messages)) if errors.len() == 1 => format!(
            "{} {}",
            field,
            messages.first().map_or("is invalid", String::as_str)
        ),
        _ => format!("{} fields are invalid", errors.len()),
    }
}

/// `error.fields` value.
pub fn fields_json(errors: &FieldErrors) -> Value {
    json!(errors)
}

/// First message per field — the `/_form` response's legacy `errors` map,
/// which the client runtime renders under each input.
pub fn first_messages(errors: &FieldErrors) -> BTreeMap<String, String> {
    errors
        .iter()
        .filter_map(|(k, v)| v.first().map(|m| (k.clone(), m.clone())))
        .collect()
}

/// `authz::error_body_with_fields` for these errors.
pub fn error_body(errors: &FieldErrors) -> Value {
    crate::authz::error_body_with_fields("VALIDATION_FAILED", &message(errors), fields_json(errors))
}

/// The write check shared by REST, `/_form` and GraphQL, in order: field
/// rules (`422`), many-to-many ids that exist and the caller may read
/// (`422`), then `unique` (`409`). `columns` and `many` come from
/// `relations::split` of an `authz::writable_body`. `Ok` carries the link
/// sets to store.
pub fn check_write(
    db: &CronusDB,
    entities: &[EntityNode],
    entity: &EntityNode,
    columns: &Map<String, Value>,
    many: &Map<String, Value>,
    mode: Mode,
    exclude_id: Option<&str>,
    access: &crate::access::Access,
) -> Result<Vec<crate::relations::LinkSet>, (u16, FieldErrors)> {
    let mut checked = columns.clone();
    checked.extend(many.clone());
    let mut errors = field_errors(entity, &checked, mode);
    let links = crate::relations::link_sets(many);
    if errors.is_empty() {
        errors = crate::relations::access_errors(db, entities, entity, &links, access);
    }
    if !errors.is_empty() {
        return Err((422, errors));
    }
    let conflicts = unique_errors(db, entity, columns, exclude_id);
    if !conflicts.is_empty() {
        return Err((409, conflicts));
    }
    Ok(links)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{parse, AstNode};

    fn entity(src: &str) -> EntityNode {
        parse(src)
            .expect("parse")
            .into_iter()
            .find_map(|n| match n {
                AstNode::Entity(e) => Some(e),
                _ => None,
            })
            .expect("entity")
    }

    fn body(v: Value) -> Map<String, Value> {
        v.as_object().cloned().expect("object")
    }

    const SRC: &str = "entity Item {\n  title string! min:3 max:5\n  qty number min:1 max:9\n  price money min:0\n  slug slug match:\"^[a-z]+$\"\n  mail email\n  site url\n  on_sale boolean\n  kind enum [a, b]\n  tags -> Item[] required\n}\n";

    #[test]
    fn each_constraint_has_one_stable_message() {
        let e = entity(SRC);
        let errors = field_errors(
            &e,
            &body(json!({
                "title": "ab", "qty": 10, "price": -1, "slug": "A1", "mail": "x@y",
                "site": "ftp://x", "on_sale": "maybe", "kind": "c", "tags": "t"
            })),
            Mode::Create,
        );
        let expect = json!({
            "title": ["must be at least 3 characters"],
            "qty": ["must be at most 9"],
            "price": ["must be at least 0"],
            "slug": ["must match the pattern ^[a-z]+$"],
            "mail": ["must be a valid email"],
            "site": ["must be a valid URL"],
            "on_sale": ["must be true or false"],
            "kind": ["must be one of: a, b"],
            "tags": ["must be an array of ids"],
        });
        assert_eq!(fields_json(&errors), expect);
        assert_eq!(message(&errors), "9 fields are invalid");
    }

    #[test]
    fn create_requires_and_update_checks_only_sent_fields() {
        let e = entity(SRC);
        let created = field_errors(&e, &body(json!({"title": "   "})), Mode::Create);
        assert_eq!(created["title"], vec!["is required"]);
        assert_eq!(created["tags"], vec!["is required"]);
        let updated = field_errors(&e, &body(json!({"qty": "x"})), Mode::Update);
        assert_eq!(fields_json(&updated), json!({"qty": ["must be a number"]}));
        assert_eq!(message(&updated), "qty must be a number");
        let ok = field_errors(
            &e,
            &body(json!({"title": "abcde", "qty": 9, "tags": ["i1", "i2"], "on_sale": true})),
            Mode::Create,
        );
        assert!(ok.is_empty(), "{ok:?}");
    }

    #[test]
    fn length_counts_characters_not_bytes_and_messages_never_echo_values() {
        let e = entity(SRC);
        assert!(field_errors(&e, &body(json!({"title": "ééééé"})), Mode::Update).is_empty());
        let too_long = field_errors(&e, &body(json!({"title": "secret-value"})), Mode::Update);
        assert_eq!(too_long["title"], vec!["must be at most 5 characters"]);
        assert!(!fields_json(&too_long).to_string().contains("secret-value"));
    }

    #[test]
    fn many_to_many_shape_limits() {
        let e = entity(SRC);
        let ids: Vec<String> = (0..=MAX_LINKS).map(|i| format!("id{i}")).collect();
        let errors = field_errors(&e, &body(json!({"tags": ids})), Mode::Update);
        assert_eq!(
            errors["tags"],
            vec![format!("must have at most {MAX_LINKS} ids")]
        );
        let errors = field_errors(&e, &body(json!({"tags": ["a", 1]})), Mode::Update);
        assert_eq!(errors["tags"], vec!["must be an array of ids"]);
    }

    #[test]
    fn unique_errors_ignore_the_row_being_updated() {
        let e = entity("entity Acct {\n  handle string unique\n}\n");
        let db = CronusDB::open_memory().unwrap();
        db.migrate(std::slice::from_ref(&e)).unwrap();
        let row = db.insert("Acct", &json!({"handle": "neo"})).unwrap();
        let id = row["id"].as_str().unwrap();
        let taken = unique_errors(&db, &e, &body(json!({"handle": "neo"})), None);
        assert_eq!(taken["handle"], vec!["already exists"]);
        assert!(unique_errors(&db, &e, &body(json!({"handle": "neo"})), Some(id)).is_empty());
        assert!(unique_errors(&db, &e, &body(json!({"handle": "trinity"})), None).is_empty());
    }

    #[test]
    fn date_datetime_and_file_have_distinct_rules() {
        let e = entity("entity Event {\n  day date!\n  at datetime!\n  poster file\n}\n");
        let bad = field_errors(
            &e,
            &body(json!({
                "day": "2026-13-01",
                "at": "2026-09-15",
                "poster": "not-a-file"
            })),
            Mode::Create,
        );
        assert_eq!(bad["day"], vec!["must be a date (YYYY-MM-DD)"]);
        assert_eq!(bad["at"], vec!["must be a datetime (YYYY-MM-DDTHH:MM)"]);
        assert_eq!(bad["poster"], vec!["must be a file upload or URL"]);
        let ok = field_errors(
            &e,
            &body(json!({
                "day": "2026-09-15",
                "at": "2026-09-15T14:30",
                "poster": "https://cdn.example/a.png"
            })),
            Mode::Create,
        );
        assert!(ok.is_empty(), "{ok:?}");
        assert!(is_datetime("2026-09-15T14:30:00Z"));
        assert!(is_datetime("2026-09-15 14:30:00+00:00"));
        assert!(!is_date("2026-09-15T14:30"));
        assert!(!is_datetime("2026-09-15"));
    }
}
