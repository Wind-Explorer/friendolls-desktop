use crate::services::sprite_recolor;

#[tauri::command]
pub fn recolor_gif_base64(
    white_color_hex: String,
    black_color_hex: String,
    apply_texture: bool,
) -> Result<String, String> {
    sprite_recolor::recolor_gif_base64(
        white_color_hex.as_str(),
        black_color_hex.as_str(),
        apply_texture,
    )
    .map_err(|e: Box<dyn std::error::Error>| e.to_string())
}