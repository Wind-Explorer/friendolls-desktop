use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event as _;

use crate::{
    get_app_handle, lock_r,
    services::{app_events::FriendCursorPositionsUpdated, cursor::CursorPositions},
    state::FDOLL,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Type)]
#[serde(transparent)]
pub struct FriendCursorPositionsDto(pub HashMap<String, CursorPositions>);

#[derive(Default)]
struct FriendCursorProjection {
    active_dolls: HashMap<String, bool>,
    positions: HashMap<String, CursorPositions>,
}

static FRIEND_CURSOR_PROJECTION: LazyLock<Arc<RwLock<FriendCursorProjection>>> =
    LazyLock::new(|| Arc::new(RwLock::new(FriendCursorProjection::default())));

pub fn sync_from_app_data() {
    let friends = {
        let guard = lock_r!(FDOLL);
        guard.user_data.friends.clone().unwrap_or_default()
    };

    let mut projection = FRIEND_CURSOR_PROJECTION
        .write()
        .expect("friend cursor projection lock poisoned");

    projection.active_dolls = friends
        .into_iter()
        .filter_map(|friendship| {
            friendship.friend.map(|friend| {
                let has_active_doll = friend.active_doll.is_some();
                (friend.id, has_active_doll)
            })
        })
        .collect();

    let active_dolls = projection.active_dolls.clone();
    projection
        .positions
        .retain(|user_id, _| active_dolls.get(user_id) == Some(&true));

    emit_snapshot(&projection.positions);
}

pub fn clear() {
    let mut projection = FRIEND_CURSOR_PROJECTION
        .write()
        .expect("friend cursor projection lock poisoned");

    projection.active_dolls.clear();
    projection.positions.clear();

    emit_snapshot(&projection.positions);
}

pub fn update_position(user_id: String, position: CursorPositions) {
    let mut projection = FRIEND_CURSOR_PROJECTION
        .write()
        .expect("friend cursor projection lock poisoned");

    if !has_active_doll(&mut projection, &user_id) {
        if projection.positions.remove(&user_id).is_some() {
            emit_snapshot(&projection.positions);
        }
        return;
    }

    projection.positions.insert(user_id, position);
    emit_snapshot(&projection.positions);
}

pub fn remove_friend(user_id: &str) {
    let mut projection = FRIEND_CURSOR_PROJECTION
        .write()
        .expect("friend cursor projection lock poisoned");

    let removed_active_doll = projection.active_dolls.remove(user_id).is_some();
    let removed_position = projection.positions.remove(user_id).is_some();

    if removed_active_doll || removed_position {
        emit_snapshot(&projection.positions);
    }
}

pub fn set_active_doll(user_id: &str, has_active_doll: bool) {
    let mut projection = FRIEND_CURSOR_PROJECTION
        .write()
        .expect("friend cursor projection lock poisoned");

    projection
        .active_dolls
        .insert(user_id.to_string(), has_active_doll);

    if !has_active_doll && projection.positions.remove(user_id).is_some() {
        emit_snapshot(&projection.positions);
    }
}

fn has_active_doll(projection: &mut FriendCursorProjection, user_id: &str) -> bool {
    if let Some(has_active_doll) = projection.active_dolls.get(user_id) {
        return *has_active_doll;
    }

    let has_active_doll = {
        let guard = lock_r!(FDOLL);
        guard
            .user_data
            .friends
            .as_ref()
            .and_then(|friends| {
                friends.iter().find_map(|friendship| {
                    let friend = friendship.friend.as_ref()?;
                    (friend.id == user_id).then_some(friend)
                })
            })
            .and_then(|friend| friend.active_doll.as_ref())
            .is_some()
    };

    projection
        .active_dolls
        .insert(user_id.to_string(), has_active_doll);

    has_active_doll
}

fn emit_snapshot(positions: &HashMap<String, CursorPositions>) {
    let payload = FriendCursorPositionsDto(positions.clone());

    if let Err(err) = FriendCursorPositionsUpdated(payload).emit(get_app_handle()) {
        tracing::warn!("Failed to emit friend cursor positions update: {}", err);
    }
}
