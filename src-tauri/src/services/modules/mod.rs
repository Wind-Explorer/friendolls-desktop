use tauri::Manager;
use tracing::{error, info, warn};

use crate::get_app_handle;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct ModuleMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

fn get_module_metadata(path: PathBuf) -> Option<ModuleMetadata> {
    let metadata_path = path.join("metadata.json");
    if metadata_path.exists() {
        match fs::read_to_string(&metadata_path) {
            Ok(content) => match serde_json::from_str::<ModuleMetadata>(&content) {
                Ok(metadata) => {
                    info!(
                        "Loaded module metadata: {} v{} - {:?}",
                        metadata.name, metadata.version, metadata.description
                    );

                    return Some(metadata);
                }
                Err(e) => {
                    warn!("Failed to parse metadata.json in {}: {}", path.display(), e);
                    None
                }
            },
            Err(e) => {
                warn!("Failed to read metadata.json in {}: {}", path.display(), e);
                None
            }
        }
    } else {
        None
    }
}

/// Initialize installed modules
pub fn init_modules() {
    let modules_path = get_app_handle()
        .path()
        .app_data_dir()
        .expect("App data directory is unavailable.")
        .join("modules");

    if !modules_path.exists() {
        if let Err(e) = fs::create_dir_all(&modules_path) {
            error!("Failed to create modules directory: {}", e);
            return;
        }
        return;
    }

    let entries = match fs::read_dir(&modules_path) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read app data directory: {}", e);
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();
        if path.is_dir() {
            let module_metadata = match get_module_metadata(path) {
                Some(metadata) => metadata,
                None => continue,
            };
            dbg!(module_metadata);
            // TODO: Initialize the module based on metadata
        }
    }
}
