use serde::{Deserialize, Serialize};

/// Per-context keybinding definition stored directly as `config.keybindings`.
///
/// `global` bindings are always visible (fallback for all contexts).
/// Other fields hold bindings active only in that specific context.
/// Presets are loaded on demand from the UI, not stored as a field.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BindingSet {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub global: Vec<KeyAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<KeyAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sidebar: Vec<KeyAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quick_access: Vec<KeyAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right_sidebar: Vec<KeyAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub search: Vec<KeyAction>,
}

/// A single key → action mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyAction {
    pub key: String,
    pub action: String,
}

impl BindingSet {
    /// Returns true if all context fields are empty.
    pub fn is_empty(&self) -> bool {
        self.global.is_empty()
            && self.content.is_empty()
            && self.sidebar.is_empty()
            && self.quick_access.is_empty()
            && self.right_sidebar.is_empty()
            && self.search.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(BindingSet::default().is_empty());
    }

    #[test]
    fn roundtrip() {
        let set = BindingSet {
            global: vec![KeyAction {
                key: "g g".to_string(),
                action: "scroll.top".to_string(),
            }],
            sidebar: vec![KeyAction {
                key: "j".to_string(),
                action: "cursor.down".to_string(),
            }],
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&set).unwrap();
        let parsed: BindingSet = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, set);
    }

    #[test]
    fn empty_json_uses_defaults() {
        let set: BindingSet = serde_json::from_str("{}").unwrap();
        assert!(set.is_empty());
    }

    #[test]
    fn skip_serializing_empty_fields() {
        let json = serde_json::to_string(&BindingSet::default()).unwrap();
        assert!(!json.contains("global"));
        assert!(!json.contains("sidebar"));
    }

    #[test]
    fn structured_json() {
        let json = r#"{
            "global": [{"key": "x", "action": "tab.close"}],
            "sidebar": [{"key": "o", "action": "cursor.enter"}]
        }"#;
        let set: BindingSet = serde_json::from_str(json).unwrap();
        assert_eq!(set.global.len(), 1);
        assert_eq!(set.sidebar.len(), 1);
        assert!(set.content.is_empty());
    }

    #[test]
    fn is_empty_check() {
        assert!(!BindingSet {
            global: vec![KeyAction {
                key: "j".to_string(),
                action: "scroll.down".to_string(),
            }],
            ..Default::default()
        }
        .is_empty());
    }
}
