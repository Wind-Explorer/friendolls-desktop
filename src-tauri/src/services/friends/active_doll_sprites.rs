use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event as _;

use crate::{
    get_app_handle, lock_r,
    models::{dolls::DollDto, friends::FriendshipResponseDto},
    services::{app_events::FriendActiveDollSpritesUpdated, sprite},
    state::FDOLL,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Type)]
#[serde(transparent)]
pub struct FriendActiveDollSpritesDto(pub HashMap<String, String>);

static FRIEND_ACTIVE_DOLL_SPRITES: LazyLock<Arc<RwLock<HashMap<String, String>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

pub fn sync_from_app_data() {
    let friends = {
        let guard = lock_r!(FDOLL);
        guard.user_data.friends.clone().unwrap_or_default()
    };

    let next = build_sprites(&friends);

    let mut projection = FRIEND_ACTIVE_DOLL_SPRITES
        .write()
        .expect("friend active doll sprite projection lock poisoned");
    *projection = next;

    emit_snapshot(&projection);
}

pub fn clear() {
    let mut projection = FRIEND_ACTIVE_DOLL_SPRITES
        .write()
        .expect("friend active doll sprite projection lock poisoned");
    projection.clear();

    emit_snapshot(&projection);
}

pub fn remove_friend(user_id: &str) {
    let mut projection = FRIEND_ACTIVE_DOLL_SPRITES
        .write()
        .expect("friend active doll sprite projection lock poisoned");

    if projection.remove(user_id).is_some() {
        emit_snapshot(&projection);
    }
}

pub fn set_active_doll(user_id: &str, doll: Option<&DollDto>) {
    let mut projection = FRIEND_ACTIVE_DOLL_SPRITES
        .write()
        .expect("friend active doll sprite projection lock poisoned");

    match doll {
        Some(doll) => match sprite::encode_doll_sprite_base64(doll) {
            Ok(sprite_b64) => {
                projection.insert(user_id.to_string(), sprite_b64);
                emit_snapshot(&projection);
            }
            Err(err) => {
                tracing::warn!(
                    "Failed to generate active doll sprite for friend {}: {}",
                    user_id,
                    err
                );

                if projection.remove(user_id).is_some() {
                    emit_snapshot(&projection);
                }
            }
        },
        None => {
            if projection.remove(user_id).is_some() {
                emit_snapshot(&projection);
            }
        }
    }
}

pub fn get_snapshot() -> FriendActiveDollSpritesDto {
    let projection = FRIEND_ACTIVE_DOLL_SPRITES
        .read()
        .expect("friend active doll sprite projection lock poisoned");

    FriendActiveDollSpritesDto(projection.clone())
}

fn build_sprites(friends: &[FriendshipResponseDto]) -> HashMap<String, String> {
    friends
        .iter()
        .filter_map(|friendship| {
            let friend = friendship.friend.as_ref()?;
            let doll = friend.active_doll.as_ref()?;

            match sprite::encode_doll_sprite_base64(doll) {
                Ok(sprite_b64) => Some((friend.id.clone(), sprite_b64)),
                Err(err) => {
                    tracing::warn!(
                        "Failed to generate active doll sprite for friend {}: {}",
                        friend.id,
                        err
                    );
                    None
                }
            }
        })
        .collect()
}

fn emit_snapshot(sprites: &HashMap<String, String>) {
    let payload = FriendActiveDollSpritesDto(sprites.clone());

    if let Err(err) = FriendActiveDollSpritesUpdated(payload).emit(get_app_handle()) {
        tracing::warn!("Failed to emit friend active doll sprites update: {}", err);
    }
}
