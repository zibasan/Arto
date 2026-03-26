use crate::config::CONFIG;
use crate::keybindings::KeyContext;

/// Return a formatted shortcut hint for the given action.
///
/// Context bindings are preferred over global when both exist.
pub fn shortcut_hint_for_action(action: &str, context: Option<KeyContext>) -> Option<String> {
    let config = CONFIG.read();
    let bindings = &config.keybindings;

    let key = context
        .and_then(|ctx| find_key_for_action(bindings_for_context(bindings, ctx), action))
        .or_else(|| find_key_for_action(&bindings.global, action))?;
    Some(format_shortcut_hint(key))
}

/// Return a formatted shortcut hint from global keybindings.
pub fn shortcut_hint_for_global_action(action: &str) -> Option<String> {
    shortcut_hint_for_action(action, None)
}

/// Return a formatted shortcut hint in the given context.
pub fn shortcut_hint_for_context_action(context: KeyContext, action: &str) -> Option<String> {
    shortcut_hint_for_action(action, Some(context))
}

fn find_key_for_action<'a>(
    bindings: &'a [crate::config::KeyAction],
    action: &str,
) -> Option<&'a str> {
    bindings
        .iter()
        .find(|ka| ka.action == action)
        .map(|ka| ka.key.as_str())
}

fn bindings_for_context(
    bindings: &crate::config::BindingSet,
    context: KeyContext,
) -> &[crate::config::KeyAction] {
    match context {
        KeyContext::Content => &bindings.content,
        KeyContext::Sidebar => &bindings.sidebar,
        KeyContext::QuickAccess => &bindings.quick_access,
        KeyContext::RightSidebar => &bindings.right_sidebar,
        KeyContext::Search => &bindings.search,
    }
}

/// Switch the context menu shortcut hint display depending on the OS
///
/// Examples:
/// -(macOS) `Cmd+Shift+o` -> `⌘⇧O`
/// -(macOS) `Ctrl+w h` -> `⌃W H`
/// -(Windows) `Ctrl+w h` -> `Ctrl+W H`
fn format_shortcut_hint(key: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        let mut out = String::new();
        let parts: Vec<&str> = key.split('+').collect();
        for part in parts {
            match part {
                "Cmd" | "Meta" => out.push('⌘'),
                "Shift" => out.push('⇧'),
                "Alt" | "Option" => out.push('⌥'),
                "Ctrl" | "Control" => out.push('⌃'),
                _ => out.push_str(&format_key_name_hint(part)),
            }
        }
        out
    }

    #[cfg(not(target_os = "macos"))]
    {
        let parts: Vec<&str> = key.split('+').collect();
        let mut formatted_parts = Vec::new();
        for part in parts {
            let formatted = match part {
                "Cmd" | "Meta" => "Ctrl".to_string(),
                "Shift" => "Shift".to_string(),
                "Alt" | "Option" => "Alt".to_string(),
                "Ctrl" | "Control" => "Ctrl".to_string(),
                _ => format_key_name_hint(part),
            };
            formatted_parts.push(formatted);
        }
        // On Windows/Linux, connect with "+" like "Ctrl+Shift+O"
        formatted_parts.join("+")
    }
}

fn format_key_name_hint(key: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        match key {
            "ArrowUp" => "↑".to_string(),
            "ArrowDown" => "↓".to_string(),
            "ArrowLeft" => "←".to_string(),
            "ArrowRight" => "→".to_string(),
            "Backspace" => "⌫".to_string(),
            "Enter" => "↩".to_string(),
            "Escape" => "⎋".to_string(),
            "Tab" => "⇥".to_string(),
            "Space" => "␠".to_string(),
            "PageUp" => "⇞".to_string(),
            "PageDown" => "⇟".to_string(),
            "Home" => "↖".to_string(),
            "End" => "↘".to_string(),
            _ => key.to_uppercase(),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        match key {
            "ArrowUp" => "Up".to_string(),
            "ArrowDown" => "Down".to_string(),
            "ArrowLeft" => "Left".to_string(),
            "ArrowRight" => "Right".to_string(),
            "Backspace" => "Backspace".to_string(),
            "Enter" => "Enter".to_string(),
            "Escape" => "Esc".to_string(),
            "Tab" => "Tab".to_string(),
            "Space" => "Space".to_string(),
            "PageUp" => "PageUp".to_string(),
            "PageDown" => "PageDown".to_string(),
            "Home" => "Home".to_string(),
            "End" => "End".to_string(),
            "BracketLeft" => "[".to_string(),
            "BracketRight" => "]".to_string(),
            "Equal" => "=".to_string(),
            "Minus" => "-".to_string(),
            "Comma" => ",".to_string(),
            "Slash" => "/".to_string(),
            _ => {
                // One character (a, b, c, etc.) is capitalized, other characters (BracketLeft, etc.) are displayed as is.
                if key.len() == 1 {
                    key.to_uppercase()
                } else {
                    key.to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn format_shortcut_hint_uses_menu_style_symbols() {
        assert_eq!(format_shortcut_hint("Cmd+Shift+o"), "⌘⇧O");
        assert_eq!(format_shortcut_hint("Ctrl+w"), "⌃W");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn format_shortcut_hint_uses_windows_style() {
        assert_eq!(format_shortcut_hint("Cmd+Shift+o"), "Ctrl+Shift+O");
        assert_eq!(format_shortcut_hint("Ctrl+w"), "Ctrl+W");
        assert_eq!(format_shortcut_hint("Alt+ArrowDown"), "Alt+Down");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn format_shortcut_hint_formats_arrow_keys() {
        assert_eq!(format_shortcut_hint("ArrowDown"), "↓");
        assert_eq!(format_shortcut_hint("Cmd+ArrowLeft"), "⌘←");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn format_shortcut_hint_formats_arrow_keys() {
        assert_eq!(format_shortcut_hint("ArrowDown"), "Down");
        assert_eq!(format_shortcut_hint("Cmd+ArrowLeft"), "Ctrl+Left");
    }
}
