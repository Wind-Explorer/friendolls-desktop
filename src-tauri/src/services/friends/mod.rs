mod active_doll_sprites;
mod cursor_positions;

use crate::{models::dolls::DollDto, services::cursor::CursorPositions};

pub use active_doll_sprites::FriendActiveDollSpritesDto;
pub use cursor_positions::FriendCursorPositionsDto;

pub fn sync_from_app_data() {
    active_doll_sprites::sync_from_app_data();
    cursor_positions::sync_from_app_data();
}

pub fn clear() {
    active_doll_sprites::clear();
    cursor_positions::clear();
}

pub fn remove_friend(user_id: &str) {
    active_doll_sprites::remove_friend(user_id);
    cursor_positions::remove_friend(user_id);
}

pub fn set_active_doll(user_id: &str, doll: Option<&DollDto>) {
    active_doll_sprites::set_active_doll(user_id, doll);
    cursor_positions::set_active_doll(user_id, doll.is_some());
}

pub fn update_cursor_position(user_id: String, position: CursorPositions) {
    cursor_positions::update_position(user_id, position);
}

pub fn sync_active_doll_sprites_from_app_data() {
    active_doll_sprites::sync_from_app_data();
}

pub fn get_active_doll_sprites_snapshot() -> FriendActiveDollSpritesDto {
    active_doll_sprites::get_snapshot()
}
