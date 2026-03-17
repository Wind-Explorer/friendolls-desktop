use tauri_plugin_updater::UpdaterExt;
use tracing::{error, info};

use crate::get_app_handle;

pub async fn update_app() {
    let app = get_app_handle();
    if let Some(update) = match match app.updater() {
        Ok(it) => it,
        Err(err) => {
            error!("failed to get updater: {err:?}");
            return;
        }
    }
    .check()
    .await
    {
        Ok(it) => it,
        Err(err) => {
            error!("failed to check for update: {err:?}");
            return;
        }
    } {
        let mut downloaded = 0;

        match update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    info!("download finished");
                },
            )
            .await
        {
            Ok(it) => it,
            Err(err) => {
                error!("failed to install update: {err:?}");
                return;
            }
        };

        info!("update installed");
        app.restart();
    }
}
