use image::{Rgba, RgbaImage};
use std::io::Cursor;

const INPUT_GIF_BASE64: &str = include_str!("./neko.gif.txt");

pub fn recolor_gif_base64(
    white_color_hex: &str,
    black_color_hex: &str,
    apply_texture: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let white_color = parse_hex_color(white_color_hex)?;
    let black_color = parse_hex_color(black_color_hex)?;

    // Decode base64 input
    let input_bytes = base64::decode(INPUT_GIF_BASE64.trim())?;

    // Process GIF
    let output_bytes = recolor_gif_bytes(&input_bytes, white_color, black_color, apply_texture)?;

    // Encode output to base64
    Ok(base64::encode(&output_bytes))
}

fn parse_hex_color(hex: &str) -> Result<Rgba<u8>, Box<dyn std::error::Error>> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("Hex color must be 6 characters".into());
    }

    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok(Rgba([r, g, b, 255]))
}

fn recolor_gif_bytes(
    input_bytes: &[u8],
    white_color: Rgba<u8>,
    black_color: Rgba<u8>,
    apply_texture: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Decode input GIF
    let reader = Cursor::new(input_bytes);
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = decoder.read_info(reader)?;

    // Get GIF properties
    let width = decoder.width();
    let height = decoder.height();

    // Prepare output encoder with in-memory buffer
    let output_buffer = Vec::new();
    let writer = Cursor::new(output_buffer);
    let mut encoder = gif::Encoder::new(writer, width, height, &[])?;
    encoder.set_repeat(gif::Repeat::Infinite)?;

    // Process each frame
    while let Some(frame) = decoder.read_next_frame()? {
        // Convert frame buffer to RgbaImage
        let mut img = RgbaImage::from_raw(width as u32, height as u32, frame.buffer.to_vec())
            .ok_or("Failed to create image from frame")?;

        // Recolor the image
        recolor_image(&mut img, white_color, black_color, apply_texture);

        // Create output frame with same timing
        let mut pixel_data = img.into_raw();
        let mut output_frame = gif::Frame::from_rgba_speed(width, height, &mut pixel_data, 10);
        output_frame.delay = frame.delay;
        output_frame.dispose = frame.dispose;

        encoder.write_frame(&output_frame)?;
    }

    // Extract the bytes from the encoder
    let writer = encoder.into_inner()?;
    Ok(writer.into_inner())
}

fn recolor_image(
    img: &mut RgbaImage,
    white_color: Rgba<u8>,
    black_color: Rgba<u8>,
    apply_texture: bool,
) {
    // Generate color palettes with variations if texture is enabled
    let white_palette = if apply_texture {
        generate_color_palette(white_color, 7)
    } else {
        vec![white_color]
    };

    let black_palette = if apply_texture {
        generate_color_palette(black_color, 7)
    } else {
        vec![black_color]
    };

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let Rgba([r, g, b, a]) = *pixel;

        // Skip fully transparent pixels
        if a == 0 {
            continue;
        }

        // Calculate brightness (0-255)
        let brightness = (r as u16 + g as u16 + b as u16) / 3;

        // Determine pixel type based on brightness
        if brightness < 64 {
            // Very dark pixels (outlines) - apply black target color
            let chosen_color = if apply_texture {
                pick_color_from_palette(&black_palette, x, y)
            } else {
                black_color
            };
            *pixel = chosen_color;
        } else if brightness > 200 {
            // White/light pixels (body) - apply white target color
            let chosen_color = if apply_texture {
                pick_color_from_palette(&white_palette, x, y)
            } else {
                white_color
            };
            *pixel = chosen_color;
        } else {
            // Gray pixels (anti-aliasing/shadows) - blend with target color
            // Blend factor based on how gray it is (0.0 = keep original, 1.0 = full target)
            let blend_factor = (brightness as f32 - 64.0) / (200.0 - 64.0);
            let blend_factor = blend_factor.clamp(0.0, 1.0);

            let chosen_color = if apply_texture {
                pick_color_from_palette(&white_palette, x, y)
            } else {
                white_color
            };

            let new_r =
                (r as f32 * (1.0 - blend_factor) + chosen_color[0] as f32 * blend_factor) as u8;
            let new_g =
                (g as f32 * (1.0 - blend_factor) + chosen_color[1] as f32 * blend_factor) as u8;
            let new_b =
                (b as f32 * (1.0 - blend_factor) + chosen_color[2] as f32 * blend_factor) as u8;

            *pixel = Rgba([new_r, new_g, new_b, a]);
        }
    }
}

fn generate_color_palette(base_color: Rgba<u8>, count: usize) -> Vec<Rgba<u8>> {
    let mut palette = Vec::with_capacity(count);
    let Rgba([r, g, b, a]) = base_color;

    // Generate darker and brighter variations
    let variation_range = 25; // How much to vary (+/- this value)
    let step = (variation_range * 2) / (count - 1).max(1);

    for i in 0..count {
        let offset = -(variation_range as i32) + (i as i32 * step as i32);

        let new_r = (r as i32 + offset).clamp(0, 255) as u8;
        let new_g = (g as i32 + offset).clamp(0, 255) as u8;
        let new_b = (b as i32 + offset).clamp(0, 255) as u8;

        palette.push(Rgba([new_r, new_g, new_b, a]));
    }

    palette
}

fn pick_color_from_palette(palette: &[Rgba<u8>], x: u32, y: u32) -> Rgba<u8> {
    // Use a deterministic "random" selection based on pixel position
    // This creates consistent noise pattern across frames
    let hash = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263));
    let index = (hash % palette.len() as u32) as usize;
    palette[index]
}
