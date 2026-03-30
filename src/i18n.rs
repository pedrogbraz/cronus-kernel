#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS i18n — Internationalization engine
//!
//! HashMap-based translation system with locale detection from Accept-Language.

use std::collections::HashMap;
use std::sync::RwLock;

pub struct I18n {
    default_locale: String,
    translations: RwLock<HashMap<String, HashMap<String, String>>>,
}

impl I18n {
    pub fn new(default_locale: &str) -> Self {
        Self {
            default_locale: default_locale.to_string(),
            translations: RwLock::new(HashMap::new()),
        }
    }

    /// Add a translation: i18n.add("pt-BR", "welcome", "Bem-vindo")
    pub fn add(&self, locale: &str, key: &str, value: &str) {
        let mut map = self.translations.write().unwrap();
        map.entry(locale.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
    }

    /// Bulk add translations for a locale
    pub fn add_locale(&self, locale: &str, entries: &[(&str, &str)]) {
        let mut map = self.translations.write().unwrap();
        let locale_map = map.entry(locale.to_string()).or_insert_with(HashMap::new);
        for (key, value) in entries {
            locale_map.insert(key.to_string(), value.to_string());
        }
    }

    /// Translate a key. Falls back to default locale, then returns key itself.
    pub fn t(&self, key: &str, locale: &str) -> String {
        let map = self.translations.read().unwrap();
        // Try requested locale
        if let Some(locale_map) = map.get(locale) {
            if let Some(value) = locale_map.get(key) {
                return value.clone();
            }
        }
        // Fall back to default locale
        if locale != self.default_locale {
            if let Some(locale_map) = map.get(&self.default_locale) {
                if let Some(value) = locale_map.get(key) {
                    return value.clone();
                }
            }
        }
        // Return key as fallback
        key.to_string()
    }

    /// Detect locale from Accept-Language header
    /// e.g. "pt-BR,pt;q=0.9,en-US;q=0.8,en;q=0.7" → "pt-BR"
    pub fn detect_locale(accept_language: Option<&str>) -> String {
        if let Some(header) = accept_language {
            // Parse first locale from Accept-Language
            if let Some(first) = header.split(',').next() {
                let locale = first.split(';').next().unwrap_or("").trim();
                if !locale.is_empty() {
                    return locale.to_string();
                }
            }
        }
        "en".to_string()
    }

    /// Get default locale
    pub fn default_locale(&self) -> &str {
        &self.default_locale
    }

    /// List available locales
    pub fn locales(&self) -> Vec<String> {
        let map = self.translations.read().unwrap();
        map.keys().cloned().collect()
    }
}

/// Create default i18n with common translations
pub fn default_i18n() -> I18n {
    let i18n = I18n::new("en");
    i18n.add_locale("en", &[
        ("welcome", "Welcome"),
        ("sign_in", "Sign In"),
        ("sign_up", "Create Account"),
        ("dashboard", "Dashboard"),
        ("settings", "Settings"),
        ("save", "Save"),
        ("cancel", "Cancel"),
        ("delete", "Delete"),
        ("loading", "Loading..."),
        ("no_data", "No data yet"),
        ("error", "An error occurred"),
        ("success", "Success"),
    ]);
    i18n.add_locale("pt-BR", &[
        ("welcome", "Bem-vindo"),
        ("sign_in", "Entrar"),
        ("sign_up", "Criar Conta"),
        ("dashboard", "Painel"),
        ("settings", "Configurações"),
        ("save", "Salvar"),
        ("cancel", "Cancelar"),
        ("delete", "Excluir"),
        ("loading", "Carregando..."),
        ("no_data", "Nenhum dado ainda"),
        ("error", "Ocorreu um erro"),
        ("success", "Sucesso"),
    ]);
    i18n
}
