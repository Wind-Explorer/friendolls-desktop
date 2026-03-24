mod active_doll_sprites;

use crate::{models::dolls::DollDto, services::neko_positions};

pub use active_doll_sprites::FriendActiveDollSpritesDto;

pub fn sync_from_app_data() {
    active_doll_sprites::sync_from_app_data();
    neko_positions::sync_from_app_data();
}

pub fn clear() {
    active_doll_sprites::clear();
    neko_positions::clear();
}

pub fn remove_friend(user_id: &str) {
    active_doll_sprites::remove_friend(user_id);
    neko_positions::remove_friend(user_id);
}

pub fn set_active_doll(user_id: &str, doll: Option<&DollDto>) {
    active_doll_sprites::set_active_doll(user_id, doll);
    neko_positions::set_friend_active_doll(user_id, doll.is_some());
}

pub fn sync_active_doll_sprites_from_app_data() {
    active_doll_sprites::sync_from_app_data();
}

pub fn get_active_doll_sprites_snapshot() -> FriendActiveDollSpritesDto {
    active_doll_sprites::get_snapshot()
}
