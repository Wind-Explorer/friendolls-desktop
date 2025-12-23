use base64::{engine::GeneralPurpose, Engine};
use image::{Rgba, RgbaImage};
use std::io::Cursor;
use std::sync::OnceLock;

const INPUT_GIF_BYTES: &[u8] = include_bytes!("./neko.gif");
const B64_ENGINE: GeneralPurpose = base64::engine::general_purpose::STANDARD;

struct GifFrame {
    pixels: Vec<u8>,
    delay: u16,
    dispose: gif::DisposalMethod,
}

struct DecodedGif {
    width: u16,
    height: u16,
    frames: Vec<GifFrame>,
}

static DECODED_GIF: OnceLock<DecodedGif> = OnceLock::new();

fn get_decoded_gif() -> &'static DecodedGif {
    DECODED_GIF
        .get_or_init(|| decode_gif_once(INPUT_GIF_BYTES).expect("Failed to decode embedded GIF"))
}

fn decode_gif_once(input_bytes: &[u8]) -> Result<DecodedGif, Box<dyn std::error::Error>> {
    let reader = Cursor::new(input_bytes);
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = decoder.read_info(reader)?;

    let width = decoder.width();
    let height = decoder.height();
    let mut frames = Vec::new();

    while let Some(frame) = decoder.read_next_frame()? {
        frames.push(GifFrame {
            pixels: frame.buffer.to_vec(),
            delay: frame.delay,
            dispose: frame.dispose,
        });
    }

    Ok(DecodedGif {
        width,
        height,
        frames,
    })
}

pub fn recolor_gif_base64(
    white_color_hex: &str,
    black_color_hex: &str,
    apply_texture: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let white_color = parse_hex_color(white_color_hex)?;
    let black_color = parse_hex_color(black_color_hex)?;

    // Get pre-decoded GIF data
    let decoded_gif = get_decoded_gif();

    // Process GIF with cached decoded frames
    let output_bytes = recolor_gif_frames(decoded_gif, white_color, black_color, apply_texture)?;

    // Encode output to base64
    Ok(B64_ENGINE.encode(&output_bytes))
}

#[inline]
fn parse_hex_color(hex: &str) -> Result<Rgba<u8>, Box<dyn std::error::Error>> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return Err("Hex color must be 6 characters".into());
    }

    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok(Rgba([r, g, b, 255]))
}

fn recolor_gif_frames(
    decoded_gif: &DecodedGif,
    white_color: Rgba<u8>,
    black_color: Rgba<u8>,
    apply_texture: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let width = decoded_gif.width;
    let height = decoded_gif.height;

    // Pre-allocate output buffer
    let estimated_capacity = INPUT_GIF_BYTES.len() * 2;
    let output_buffer = Vec::with_capacity(estimated_capacity);
    let writer = Cursor::new(output_buffer);
    let mut encoder = gif::Encoder::new(writer, width, height, &[])?;
    encoder.set_repeat(gif::Repeat::Infinite)?;

    // Pre-generate palettes once
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

    // Process each pre-decoded frame
    for frame in &decoded_gif.frames {
        // Create image from cached pixel data
        let mut img = RgbaImage::from_raw(width as u32, height as u32, frame.pixels.clone())
            .ok_or("Failed to create image from frame")?;

        // Recolor the image
        recolor_image(&mut img, &white_palette, &black_palette, apply_texture);

        // Create output frame
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
    white_palette: &[Rgba<u8>],
    black_palette: &[Rgba<u8>],
    apply_texture: bool,
) {
    // Pre-compute values for hot path
    let white_single = white_palette[0];
    let black_single = black_palette[0];

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let Rgba([r, g, b, a]) = *pixel;

        // Skip fully transparent pixels
        if a == 0 {
            continue;
        }

        // Calculate brightness using faster bit shift approximation
        let brightness = ((r as u16 + g as u16 + b as u16) * 85) >> 8;

        // Determine pixel type based on brightness
        if brightness < 64 {
            // Very dark pixels (outlines) - apply black target color
            let chosen_color = if apply_texture {
                pick_color_from_palette(black_palette, x, y)
            } else {
                black_single
            };
            *pixel = chosen_color;
        } else if brightness > 200 {
            // White/light pixels (body) - apply white target color
            let chosen_color = if apply_texture {
                pick_color_from_palette(white_palette, x, y)
            } else {
                white_single
            };
            *pixel = chosen_color;
        } else {
            // Gray pixels (anti-aliasing/shadows) - blend with target color
            let chosen_color = if apply_texture {
                pick_color_from_palette(white_palette, x, y)
            } else {
                white_single
            };

            // Use integer arithmetic instead of floating point
            let blend_scaled = ((brightness as u32 - 64) * 256) / 136;
            let blend_scaled = blend_scaled.min(256) as u16;
            let inv_blend = 256 - blend_scaled;

            let new_r = ((r as u16 * inv_blend + chosen_color[0] as u16 * blend_scaled) >> 8) as u8;
            let new_g = ((g as u16 * inv_blend + chosen_color[1] as u16 * blend_scaled) >> 8) as u8;
            let new_b = ((b as u16 * inv_blend + chosen_color[2] as u16 * blend_scaled) >> 8) as u8;

            *pixel = Rgba([new_r, new_g, new_b, a]);
        }
    }
}

fn generate_color_palette(base_color: Rgba<u8>, count: usize) -> Vec<Rgba<u8>> {
    let mut palette = Vec::with_capacity(count);
    let Rgba([r, g, b, a]) = base_color;

    // Generate darker and brighter variations
    let variation_range = 25;
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

#[inline(always)]
fn pick_color_from_palette(palette: &[Rgba<u8>], x: u32, y: u32) -> Rgba<u8> {
    // Use a deterministic "random" selection based on pixel position
    let hash = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263));
    let index = (hash as usize) % palette.len();

    // SAFETY: index is guaranteed to be < palette.len() due to modulo operation
    unsafe { *palette.get_unchecked(index) }
}
