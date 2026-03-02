use crate::models::dolls::DollDto;
use crate::services::sprite_recolor;
use image::{imageops::FilterType, RgbaImage};
use petpet::{encode_gif, generate};
use rand::prelude::IndexedRandom;

const SPRITE_SIZE: u32 = 32;
const SPRITE_FRAMES: &[(u32, u32)] = &[
    (160, 125),
    (32, 125),
    (192, 0),
    (224, 0),
    (32, 0),
    (0, 0),
    (0, 127),
    (32, 127),
    (224, 126),
    (224, 96),
    (224, 64),
    (160, 0),
    (160, 127),
    (160, 126),
    (224, 0),
    (224, 127),
    (32, 126),
    (32, 125),
    (0, 126),
    (0, 125),
    (160, 0),
    (160, 127),
    (192, 127),
    (192, 126),
    (224, 125),
    (32, 126),
    (192, 125),
    (224, 127),
    (160, 126),
    (160, 125),
    (224, 0),
    (224, 127),
];

fn crop_random_sprite(img: &RgbaImage) -> RgbaImage {
    let mut rng = rand::rng();
    let &(coord_x, coord_y) = SPRITE_FRAMES
        .choose(&mut rng)
        .expect("SPRITE_FRAMES must not be empty");
    image::imageops::crop_imm(img, coord_x, coord_y, SPRITE_SIZE, SPRITE_SIZE).to_image()
}

pub fn encode_pet_doll_gif_base64(doll: DollDto) -> Result<String, String> {
    let body_color = &doll.configuration.color_scheme.body;
    let outline_color = &doll.configuration.color_scheme.outline;

    let img: RgbaImage = sprite_recolor::get_recolored_image(body_color, outline_color)
        .map_err(|e| format!("Failed to recolor image: {}", e))?;

    let random_sprite = crop_random_sprite(&img);

    let frames = generate(random_sprite, FilterType::Lanczos3, None)
        .map_err(|e| format!("Failed to generate petpet frames: {}", e))?;

    let mut output = Vec::new();
    encode_gif(frames, &mut output, 10).map_err(|e| format!("Failed to encode GIF: {}", e))?;

    use base64::{engine::general_purpose, Engine as _};
    Ok(general_purpose::STANDARD.encode(&output))
}
