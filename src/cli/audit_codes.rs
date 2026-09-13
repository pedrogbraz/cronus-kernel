//! Stable Cronus Audit error codes. Byte-compared with `@cronus-ui/audit`.

pub const HTML_IN_SOURCE: &str = "CRONUS_AUDIT_HTML_IN_SOURCE";
pub const JSX: &str = "CRONUS_AUDIT_JSX";
pub const TEMPLATE: &str = "CRONUS_AUDIT_TEMPLATE";
pub const SIDECAR_SOURCE: &str = "CRONUS_AUDIT_SIDECAR_SOURCE";
pub const SIDECAR_RENDERER: &str = "CRONUS_AUDIT_SIDECAR_RENDERER";
pub const STACK_REACT: &str = "CRONUS_AUDIT_STACK_REACT";
pub const STACK_VOODOO: &str = "CRONUS_AUDIT_STACK_VOODOO";
pub const TW_CSS: &str = "CRONUS_AUDIT_TW_CSS";
pub const VOODOO: &str = "CRONUS_AUDIT_VOODOO";
pub const STUB_RENDERER: &str = "CRONUS_AUDIT_STUB_RENDERER";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditFinding {
    pub axis: &'static str,
    pub ok: bool,
    pub code: &'static str,
    pub message: String,
    pub family: Option<String>,
    pub fixture: Option<String>,
}

impl AuditFinding {
    pub fn fail(axis: &'static str, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            axis,
            ok: false,
            code,
            message: message.into(),
            family: None,
            fixture: None,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "axis": self.axis,
            "ok": self.ok,
            "code": self.code,
            "message": self.message,
            "family": self.family,
            "fixture": self.fixture,
            "ratio": serde_json::Value::Null,
        })
    }
}
