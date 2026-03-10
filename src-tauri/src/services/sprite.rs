use crate::{lock_r, models::dolls::DollDto, state::FDOLL};

const APPLY_TEXTURE: bool = true;

pub fn encode_doll_sprite_base64(doll: &DollDto) -> Result<String, String> {
    let color_scheme = &doll.configuration.color_scheme;

    super::sprite_recolor::recolor_gif_base64(
        &color_scheme.body,
        &color_scheme.outline,
        APPLY_TEXTURE,
    )
    .map_err(|err| err.to_string())
}

pub fn get_active_doll() -> Option<DollDto> {
    let guard = lock_r!(FDOLL);
    let active_doll_id = guard
        .user_data
        .user
        .as_ref()
        .and_then(|user| user.active_doll_id.as_deref())?;

    guard
        .user_data
        .dolls
        .as_ref()
        .and_then(|dolls| dolls.iter().find(|doll| doll.id == active_doll_id))
        .cloned()
}

pub fn get_active_doll_sprite_base64() -> Result<Option<String>, String> {
    get_active_doll()
        .as_ref()
        .map(encode_doll_sprite_base64)
        .transpose()
}
