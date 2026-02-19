use crate::models::dolls::DollDto;
use crate::services::sprite_recolor;
use image::{imageops::FilterType, RgbaImage};
use petpet::{encode_gif, generate};

pub fn encode_pet_doll_gif_base64(doll: DollDto) -> Result<String, String> {
    let body_color = &doll.configuration.color_scheme.body;
    let outline_color = &doll.configuration.color_scheme.outline;

    // Get recolored image
    let img: RgbaImage = sprite_recolor::get_recolored_image(body_color, outline_color)
        .map_err(|e| format!("Failed to recolor image: {}", e))?;

    // Generate petpet frames
    let frames = generate(img, FilterType::Lanczos3, None)
        .map_err(|e| format!("Failed to generate petpet frames: {}", e))?;

    // Encode to GIF
    let mut output = Vec::new();
    encode_gif(frames, &mut output, 10).map_err(|e| format!("Failed to encode GIF: {}", e))?;

    // Base64
    use base64::{engine::general_purpose, Engine as _};
    Ok(general_purpose::STANDARD.encode(&output))
}
