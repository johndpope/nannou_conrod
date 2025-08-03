//! Internationalization module for timeline UI
//! Loads translations from JSON files

use std::collections::HashMap;
use serde_json::Value;

/// Load translations from embedded JSON files
pub fn load_translations(language: &str) -> HashMap<String, String> {
    let json_content = match language {
        "en" => include_str!("en.json"),
        "es" => include_str!("es.json"),
        "ja" => include_str!("ja.json"),
        "zh" => include_str!("zh.json"),
        _ => include_str!("en.json"), // Default to English
    };
    
    let mut translations = HashMap::new();
    
    if let Ok(json) = serde_json::from_str::<Value>(json_content) {
        flatten_json(&json, String::new(), &mut translations);
    }
    
    translations
}

/// Recursively flatten JSON structure into dot-notation keys
fn flatten_json(value: &Value, prefix: String, map: &mut HashMap<String, String>) {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_json(val, new_prefix, map);
            }
        }
        Value::String(s) => {
            map.insert(prefix, s.clone());
        }
        _ => {} // Ignore other types
    }
}

/// Available languages
pub const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("es", "Español"),
    ("ja", "日本語"),
    ("zh", "中文"),
];

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_english() {
        let translations = load_translations("en");
        assert!(!translations.is_empty());
        assert_eq!(
            translations.get("timeline.toolbar.first_frame"),
            Some(&"First Frame (Home)".to_string())
        );
    }
    
    #[test]
    fn test_load_spanish() {
        let translations = load_translations("es");
        assert!(!translations.is_empty());
        assert_eq!(
            translations.get("timeline.toolbar.first_frame"),
            Some(&"Primer Fotograma (Inicio)".to_string())
        );
    }
}