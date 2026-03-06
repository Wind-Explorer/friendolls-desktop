use serde::Serialize;
#[allow(unused_imports)]
use std::{fs, path::Path};
use strum::{AsRefStr, EnumIter};
use ts_rs::TS;

#[derive(Serialize, TS, EnumIter, AsRefStr)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum AppEvents {
    CursorPosition,
    SceneInteractive,
    AppDataRefreshed,
    SetInteractionOverlay,
    EditDoll,
    CreateDoll,
    UserStatusChanged,
}

impl AppEvents {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppEvents::CursorPosition => "cursor-position",
            AppEvents::SceneInteractive => "scene-interactive",
            AppEvents::AppDataRefreshed => "app-data-refreshed",
            AppEvents::SetInteractionOverlay => "set-interaction-overlay",
            AppEvents::EditDoll => "edit-doll",
            AppEvents::CreateDoll => "create-doll",
            AppEvents::UserStatusChanged => "user-status-changed",
        }
    }
}

#[test]
fn export_bindings_appeventsconsts() {
    use strum::IntoEnumIterator;

    let some_export_dir = std::env::var("TS_RS_EXPORT_DIR")
        .ok()
        .map(|s| Path::new(&s).to_owned());

    let Some(export_dir) = some_export_dir else {
        eprintln!("TS_RS_EXPORT_DIR not set, skipping constants export");
        return;
    };

    let to_kebab_case = |s: &str| -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('-');
                }
                result.push(c.to_lowercase().next().unwrap());
            } else {
                result.push(c);
            }
        }
        result
    };

    let mut lines = vec![
        r#"// Auto-generated constants - DO NOT EDIT"#.to_string(),
        r#"// Generated from Rust AppEvents enum"#.to_string(),
        "".to_string(),
        "export const AppEvents = {".to_string(),
    ];

    for variant in AppEvents::iter() {
        let name = variant.as_ref();
        let kebab = to_kebab_case(name);
        lines.push(format!("  {}: \"{}\",", name, kebab));
    }

    lines.push("} as const;".to_string());
    lines.push("".to_string());
    lines.push("export type AppEvents = typeof AppEvents[keyof typeof AppEvents];".to_string());

    let constants_content = lines.join("\n");

    let constants_path = export_dir.join("AppEventsConstants.ts");
    if let Err(e) = fs::write(&constants_path, constants_content) {
        eprintln!("Failed to write {}: {}", constants_path.display(), e);
    }
}
