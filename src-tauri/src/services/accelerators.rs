use device_query::Keycode;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Type)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorAction {
    SceneInteractivity,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Type)]
pub struct KeyboardAccelerator {
    #[serde(default)]
    pub modifiers: Vec<AcceleratorModifier>,
    #[serde(default)]
    pub key: Option<AcceleratorKey>,
}

impl KeyboardAccelerator {
    pub fn normalized(mut self) -> Self {
        self.modifiers.sort_unstable();
        self.modifiers.dedup();

        if self.modifiers.is_empty() {
            return Self::default();
        }

        self
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Type)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorModifier {
    Cmd,
    Alt,
    Ctrl,
    Shift,
}

impl Default for KeyboardAccelerator {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self {
                modifiers: vec![AcceleratorModifier::Cmd],
                key: None,
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Self {
                modifiers: vec![AcceleratorModifier::Alt],
                key: None,
            }
        }
    }
}

pub fn default_accelerator_for_action(action: AcceleratorAction) -> KeyboardAccelerator {
    match action {
        AcceleratorAction::SceneInteractivity => KeyboardAccelerator::default(),
    }
}

pub fn default_accelerators() -> std::collections::BTreeMap<AcceleratorAction, KeyboardAccelerator>
{
    let mut map = std::collections::BTreeMap::new();
    map.insert(
        AcceleratorAction::SceneInteractivity,
        default_accelerator_for_action(AcceleratorAction::SceneInteractivity),
    );
    map
}

pub fn normalize_accelerators(
    mut accelerators: std::collections::BTreeMap<AcceleratorAction, KeyboardAccelerator>,
) -> std::collections::BTreeMap<AcceleratorAction, KeyboardAccelerator> {
    for value in accelerators.values_mut() {
        *value = value.clone().normalized();
    }

    for action in [AcceleratorAction::SceneInteractivity] {
        accelerators
            .entry(action)
            .or_insert_with(|| default_accelerator_for_action(action));
    }

    accelerators
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorKey {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Enter,
    Space,
    Escape,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    BackSlash,
    Semicolon,
    Apostrophe,
    Comma,
    Dot,
    Slash,
    Grave,
}

fn contains_any(keys: &[Keycode], candidates: &[Keycode]) -> bool {
    candidates.iter().any(|candidate| keys.contains(candidate))
}

fn has_modifier(keys: &[Keycode], modifier: AcceleratorModifier) -> bool {
    match modifier {
        AcceleratorModifier::Cmd => contains_any(
            keys,
            &[
                Keycode::Command,
                Keycode::RCommand,
                Keycode::LMeta,
                Keycode::RMeta,
            ],
        ),
        AcceleratorModifier::Alt => contains_any(
            keys,
            &[
                Keycode::LAlt,
                Keycode::RAlt,
                Keycode::LOption,
                Keycode::ROption,
            ],
        ),
        AcceleratorModifier::Ctrl => contains_any(keys, &[Keycode::LControl, Keycode::RControl]),
        AcceleratorModifier::Shift => contains_any(keys, &[Keycode::LShift, Keycode::RShift]),
    }
}

fn keycodes_for(key: AcceleratorKey) -> &'static [Keycode] {
    match key {
        AcceleratorKey::A => &[Keycode::A],
        AcceleratorKey::B => &[Keycode::B],
        AcceleratorKey::C => &[Keycode::C],
        AcceleratorKey::D => &[Keycode::D],
        AcceleratorKey::E => &[Keycode::E],
        AcceleratorKey::F => &[Keycode::F],
        AcceleratorKey::G => &[Keycode::G],
        AcceleratorKey::H => &[Keycode::H],
        AcceleratorKey::I => &[Keycode::I],
        AcceleratorKey::J => &[Keycode::J],
        AcceleratorKey::K => &[Keycode::K],
        AcceleratorKey::L => &[Keycode::L],
        AcceleratorKey::M => &[Keycode::M],
        AcceleratorKey::N => &[Keycode::N],
        AcceleratorKey::O => &[Keycode::O],
        AcceleratorKey::P => &[Keycode::P],
        AcceleratorKey::Q => &[Keycode::Q],
        AcceleratorKey::R => &[Keycode::R],
        AcceleratorKey::S => &[Keycode::S],
        AcceleratorKey::T => &[Keycode::T],
        AcceleratorKey::U => &[Keycode::U],
        AcceleratorKey::V => &[Keycode::V],
        AcceleratorKey::W => &[Keycode::W],
        AcceleratorKey::X => &[Keycode::X],
        AcceleratorKey::Y => &[Keycode::Y],
        AcceleratorKey::Z => &[Keycode::Z],
        AcceleratorKey::Num0 => &[Keycode::Key0],
        AcceleratorKey::Num1 => &[Keycode::Key1],
        AcceleratorKey::Num2 => &[Keycode::Key2],
        AcceleratorKey::Num3 => &[Keycode::Key3],
        AcceleratorKey::Num4 => &[Keycode::Key4],
        AcceleratorKey::Num5 => &[Keycode::Key5],
        AcceleratorKey::Num6 => &[Keycode::Key6],
        AcceleratorKey::Num7 => &[Keycode::Key7],
        AcceleratorKey::Num8 => &[Keycode::Key8],
        AcceleratorKey::Num9 => &[Keycode::Key9],
        AcceleratorKey::F1 => &[Keycode::F1],
        AcceleratorKey::F2 => &[Keycode::F2],
        AcceleratorKey::F3 => &[Keycode::F3],
        AcceleratorKey::F4 => &[Keycode::F4],
        AcceleratorKey::F5 => &[Keycode::F5],
        AcceleratorKey::F6 => &[Keycode::F6],
        AcceleratorKey::F7 => &[Keycode::F7],
        AcceleratorKey::F8 => &[Keycode::F8],
        AcceleratorKey::F9 => &[Keycode::F9],
        AcceleratorKey::F10 => &[Keycode::F10],
        AcceleratorKey::F11 => &[Keycode::F11],
        AcceleratorKey::F12 => &[Keycode::F12],
        AcceleratorKey::Enter => &[Keycode::Enter, Keycode::NumpadEnter],
        AcceleratorKey::Space => &[Keycode::Space],
        AcceleratorKey::Escape => &[Keycode::Escape],
        AcceleratorKey::Tab => &[Keycode::Tab],
        AcceleratorKey::Backspace => &[Keycode::Backspace],
        AcceleratorKey::Delete => &[Keycode::Delete],
        AcceleratorKey::Insert => &[Keycode::Insert],
        AcceleratorKey::Home => &[Keycode::Home],
        AcceleratorKey::End => &[Keycode::End],
        AcceleratorKey::PageUp => &[Keycode::PageUp],
        AcceleratorKey::PageDown => &[Keycode::PageDown],
        AcceleratorKey::ArrowUp => &[Keycode::Up],
        AcceleratorKey::ArrowDown => &[Keycode::Down],
        AcceleratorKey::ArrowLeft => &[Keycode::Left],
        AcceleratorKey::ArrowRight => &[Keycode::Right],
        AcceleratorKey::Minus => &[Keycode::Minus],
        AcceleratorKey::Equal => &[Keycode::Equal],
        AcceleratorKey::LeftBracket => &[Keycode::LeftBracket],
        AcceleratorKey::RightBracket => &[Keycode::RightBracket],
        AcceleratorKey::BackSlash => &[Keycode::BackSlash],
        AcceleratorKey::Semicolon => &[Keycode::Semicolon],
        AcceleratorKey::Apostrophe => &[Keycode::Apostrophe],
        AcceleratorKey::Comma => &[Keycode::Comma],
        AcceleratorKey::Dot => &[Keycode::Dot],
        AcceleratorKey::Slash => &[Keycode::Slash],
        AcceleratorKey::Grave => &[Keycode::Grave],
    }
}

fn has_key(keys: &[Keycode], key: AcceleratorKey) -> bool {
    contains_any(keys, keycodes_for(key))
}

fn pressed_modifiers(keys: &[Keycode]) -> Vec<AcceleratorModifier> {
    let mut modifiers = Vec::new();

    for modifier in [
        AcceleratorModifier::Cmd,
        AcceleratorModifier::Alt,
        AcceleratorModifier::Ctrl,
        AcceleratorModifier::Shift,
    ] {
        if has_modifier(keys, modifier) {
            modifiers.push(modifier);
        }
    }

    modifiers
}

pub fn is_accelerator_active(keys: &[Keycode], accelerator: &KeyboardAccelerator) -> bool {
    if pressed_modifiers(keys) != accelerator.modifiers {
        return false;
    }

    accelerator.key.map_or(true, |key| has_key(keys, key))
}
