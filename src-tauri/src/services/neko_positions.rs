use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event as _;

use crate::{
    get_app_handle, lock_r, lock_w,
    models::app_state::NekoPosition,
    services::{
        app_events::NekoPositionsUpdated,
        app_state,
        cursor::{CursorPosition, CursorPositions},
    },
    state::FDOLL,
};

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NekoPositionDto {
    pub user_id: String,
    pub is_self: bool,
    pub cursor: CursorPositions,
    pub target: CursorPosition,
    pub override_applied: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Type)]
#[serde(transparent)]
pub struct NekoPositionsDto(pub HashMap<String, NekoPositionDto>);

#[derive(Default)]
struct NekoPositionsProjection {
    self_cursor: Option<CursorPositions>,
    friend_cursors: HashMap<String, CursorPositions>,
    friend_active_dolls: HashMap<String, bool>,
}

static NEKO_POSITIONS: LazyLock<Arc<RwLock<NekoPositionsProjection>>> =
    LazyLock::new(|| Arc::new(RwLock::new(NekoPositionsProjection::default())));

pub fn sync_from_app_data() {
    let friends = {
        let guard = lock_r!(FDOLL);
        guard.user_data.friends.clone().unwrap_or_default()
    };

    let mut projection = lock_w!(NEKO_POSITIONS);

    projection.friend_active_dolls = friends
        .into_iter()
        .filter_map(|friendship| {
            friendship.friend.map(|friend| {
                let has_active_doll = friend.active_doll.is_some();
                (friend.id, has_active_doll)
            })
        })
        .collect();

    let active_dolls = projection.friend_active_dolls.clone();
    projection
        .friend_cursors
        .retain(|user_id, _| active_dolls.get(user_id) == Some(&true));

    emit_snapshot(&projection);
}

pub fn clear() {
    let mut projection = lock_w!(NEKO_POSITIONS);

    projection.self_cursor = None;
    projection.friend_cursors.clear();
    projection.friend_active_dolls.clear();

    emit_snapshot(&projection);
}

pub fn update_self_cursor(position: CursorPositions) {
    let mut projection = lock_w!(NEKO_POSITIONS);

    projection.self_cursor = Some(position);
    emit_snapshot(&projection);
}

pub fn update_friend_cursor(user_id: String, position: CursorPositions) {
    let mut projection = lock_w!(NEKO_POSITIONS);

    if !has_friend_active_doll(&mut projection, &user_id) {
        if projection.friend_cursors.remove(&user_id).is_some() {
            emit_snapshot(&projection);
        }
        return;
    }

    projection.friend_cursors.insert(user_id, position);
    emit_snapshot(&projection);
}

pub fn remove_friend(user_id: &str) {
    let mut projection = lock_w!(NEKO_POSITIONS);

    let removed_active_doll = projection.friend_active_dolls.remove(user_id).is_some();
    let removed_position = projection.friend_cursors.remove(user_id).is_some();

    if removed_active_doll || removed_position {
        emit_snapshot(&projection);
    }
}

pub fn set_friend_active_doll(user_id: &str, has_active_doll: bool) {
    let mut projection = lock_w!(NEKO_POSITIONS);

    projection
        .friend_active_dolls
        .insert(user_id.to_string(), has_active_doll);

    if !has_active_doll && projection.friend_cursors.remove(user_id).is_some() {
        emit_snapshot(&projection);
    }
}

pub fn refresh_from_scene_setup() {
    let projection = lock_r!(NEKO_POSITIONS);
    emit_snapshot(&projection);
}

pub fn get_snapshot() -> NekoPositionsDto {
    let projection = lock_r!(NEKO_POSITIONS);
    build_snapshot(&projection)
}

fn has_friend_active_doll(projection: &mut NekoPositionsProjection, user_id: &str) -> bool {
    if let Some(has_active_doll) = projection.friend_active_dolls.get(user_id) {
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
        .friend_active_dolls
        .insert(user_id.to_string(), has_active_doll);

    has_active_doll
}

fn has_self_active_doll() -> bool {
    let guard = lock_r!(FDOLL);
    guard
        .user_data
        .user
        .as_ref()
        .and_then(|user| user.active_doll_id.as_ref())
        .is_some()
}

fn get_self_user_id() -> Option<String> {
    let guard = lock_r!(FDOLL);
    guard.user_data.user.as_ref().map(|user| user.id.clone())
}

fn get_display_size() -> (f64, f64) {
    let guard = lock_r!(FDOLL);
    (
        guard.user_data.scene.display.screen_width as f64,
        guard.user_data.scene.display.screen_height as f64,
    )
}

fn emit_snapshot(projection: &NekoPositionsProjection) {
    let payload = build_snapshot(projection);

    if let Err(err) = NekoPositionsUpdated(payload).emit(get_app_handle()) {
        tracing::warn!("Failed to emit neko positions update: {}", err);
    }
}

fn build_snapshot(projection: &NekoPositionsProjection) -> NekoPositionsDto {
    let mut entries: Vec<(String, bool, CursorPositions)> = Vec::new();

    if has_self_active_doll() {
        if let (Some(self_user_id), Some(self_cursor)) =
            (get_self_user_id(), projection.self_cursor.clone())
        {
            entries.push((self_user_id, true, self_cursor));
        }
    }

    for (user_id, cursor) in &projection.friend_cursors {
        if projection.friend_active_dolls.get(user_id) == Some(&true) {
            entries.push((user_id.clone(), false, cursor.clone()));
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let app_state = app_state::get_snapshot();
    let override_anchor = app_state.scene_setup.nekos_position;
    let (screen_width, screen_height) = get_display_size();

    let total = entries.len();

    NekoPositionsDto(
        entries
            .into_iter()
            .enumerate()
            .map(|(index, (user_id, is_self, cursor))| {
                let (target, override_applied) = match &override_anchor {
                    Some(anchor) => (
                        get_cluster_target(
                            anchor.clone(),
                            index,
                            total,
                            screen_width,
                            screen_height,
                        ),
                        true,
                    ),
                    None => (cursor.raw.clone(), false),
                };

                (
                    user_id.clone(),
                    NekoPositionDto {
                        user_id,
                        is_self,
                        cursor,
                        target,
                        override_applied,
                    },
                )
            })
            .collect(),
    )
}

fn get_cluster_target(
    anchor: NekoPosition,
    index: usize,
    count: usize,
    screen_width: f64,
    screen_height: f64,
) -> CursorPosition {
    let spacing = 36.0;
    let margin = 28.0;

    let columns = (count as f64).sqrt().ceil().max(1.0) as usize;
    let rows = count.div_ceil(columns).max(1);
    let col = index % columns;
    let row = index / columns;

    let block_width = (columns.saturating_sub(1)) as f64 * spacing;
    let block_height = (rows.saturating_sub(1)) as f64 * spacing;

    let start_x = match anchor {
        NekoPosition::TopLeft | NekoPosition::Left | NekoPosition::BottomLeft => margin,
        NekoPosition::Top | NekoPosition::Bottom => (screen_width - block_width) / 2.0,
        NekoPosition::TopRight | NekoPosition::Right | NekoPosition::BottomRight => {
            screen_width - margin - block_width
        }
    };

    let start_y = match anchor {
        NekoPosition::TopLeft | NekoPosition::Top | NekoPosition::TopRight => margin,
        NekoPosition::Left | NekoPosition::Right => (screen_height - block_height) / 2.0,
        NekoPosition::BottomLeft | NekoPosition::Bottom | NekoPosition::BottomRight => {
            screen_height - margin - block_height
        }
    };

    CursorPosition {
        x: (start_x + col as f64 * spacing).clamp(0.0, screen_width),
        y: (start_y + row as f64 * spacing).clamp(0.0, screen_height),
    }
}
