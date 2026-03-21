import type {
  AcceleratorAction,
  AcceleratorKey,
  AcceleratorModifier,
  AppConfig,
  KeyboardAccelerator,
} from "$lib/bindings";

export const SCENE_INTERACTIVITY_ACTION: AcceleratorAction =
  "scene_interactivity";

const MODIFIER_PRIORITY: Record<AcceleratorModifier, number> = {
  cmd: 0,
  alt: 1,
  ctrl: 2,
  shift: 3,
};

export const MODIFIER_LABELS: Record<AcceleratorModifier, string> = {
  cmd: "Cmd",
  alt: "Alt",
  ctrl: "Ctrl",
  shift: "Shift",
};

const SPECIAL_KEY_LABELS: Partial<Record<AcceleratorKey, string>> = {
  enter: "Enter",
  space: "Space",
  escape: "Escape",
  tab: "Tab",
  backspace: "Backspace",
  delete: "Delete",
  insert: "Insert",
  home: "Home",
  end: "End",
  page_up: "Page Up",
  page_down: "Page Down",
  arrow_up: "Arrow Up",
  arrow_down: "Arrow Down",
  arrow_left: "Arrow Left",
  arrow_right: "Arrow Right",
  minus: "-",
  equal: "=",
  left_bracket: "[",
  right_bracket: "]",
  back_slash: "\\",
  semicolon: ";",
  apostrophe: "'",
  comma: ",",
  dot: ".",
  slash: "/",
  grave: "`",
};

const SPECIAL_CODE_TO_KEY: Record<string, AcceleratorKey> = {
  Enter: "enter",
  Space: "space",
  Escape: "escape",
  Tab: "tab",
  Backspace: "backspace",
  Delete: "delete",
  Insert: "insert",
  Home: "home",
  End: "end",
  PageUp: "page_up",
  PageDown: "page_down",
  ArrowUp: "arrow_up",
  ArrowDown: "arrow_down",
  ArrowLeft: "arrow_left",
  ArrowRight: "arrow_right",
  Minus: "minus",
  Equal: "equal",
  BracketLeft: "left_bracket",
  BracketRight: "right_bracket",
  Backslash: "back_slash",
  Semicolon: "semicolon",
  Quote: "apostrophe",
  Comma: "comma",
  Period: "dot",
  Slash: "slash",
  Backquote: "grave",
};

const FUNCTION_KEY_MIN = 1;
const FUNCTION_KEY_MAX = 12;

const toLetterKey = (code: string): AcceleratorKey | null => {
  if (!code.startsWith("Key") || code.length !== 4) return null;

  const letter = code[3].toLowerCase();
  if (letter < "a" || letter > "z") return null;

  return letter as AcceleratorKey;
};

const toDigitKey = (code: string): AcceleratorKey | null => {
  if (!code.startsWith("Digit") || code.length !== 6) return null;

  const digit = code[5];
  if (digit < "0" || digit > "9") return null;

  return `num_${digit}` as AcceleratorKey;
};

const toFunctionKey = (code: string): AcceleratorKey | null => {
  if (!code.startsWith("F")) return null;

  const parsed = Number.parseInt(code.slice(1), 10);
  if (Number.isNaN(parsed)) return null;
  if (parsed < FUNCTION_KEY_MIN || parsed > FUNCTION_KEY_MAX) return null;

  return `f${parsed}` as AcceleratorKey;
};

const toPatternKey = (code: string): AcceleratorKey | null => {
  return toLetterKey(code) ?? toDigitKey(code) ?? toFunctionKey(code);
};

const toPatternLabel = (key: AcceleratorKey): string | null => {
  if (key.length === 1) {
    return key.toUpperCase();
  }

  if (key.startsWith("num_")) {
    return key.slice(4);
  }

  if (key.startsWith("f")) {
    const parsed = Number.parseInt(key.slice(1), 10);
    if (Number.isNaN(parsed)) return null;
    if (parsed < FUNCTION_KEY_MIN || parsed > FUNCTION_KEY_MAX) return null;
    return `F${parsed}`;
  }

  return null;
};

export const keyFromKeyboardCode = (code: string): AcceleratorKey | null => {
  return toPatternKey(code) ?? SPECIAL_CODE_TO_KEY[code] ?? null;
};

export const labelForAcceleratorKey = (key: AcceleratorKey): string => {
  return toPatternLabel(key) ?? SPECIAL_KEY_LABELS[key] ?? key;
};

export const MODIFIER_CODES = new Set([
  "MetaLeft",
  "MetaRight",
  "AltLeft",
  "AltRight",
  "ControlLeft",
  "ControlRight",
  "ShiftLeft",
  "ShiftRight",
]);

export const normalizeAccelerator = (
  accelerator: KeyboardAccelerator,
): KeyboardAccelerator => {
  const uniqueModifiers = [...new Set(accelerator.modifiers ?? [])].sort(
    (a, b) => MODIFIER_PRIORITY[a] - MODIFIER_PRIORITY[b],
  );

  return {
    modifiers: uniqueModifiers,
    key: accelerator.key ?? null,
  };
};

export const formatAcceleratorLabel = (accelerator: KeyboardAccelerator): string => {
  const modifiers = (accelerator.modifiers ?? []).map(
    (modifier) => MODIFIER_LABELS[modifier],
  );
  const key = accelerator.key ? labelForAcceleratorKey(accelerator.key) : null;
  return key ? [...modifiers, key].join(" + ") : modifiers.join(" + ");
};

export const getModifiersFromEvent = (
  event: KeyboardEvent,
): AcceleratorModifier[] => {
  const modifiers: AcceleratorModifier[] = [];
  if (event.metaKey) modifiers.push("cmd");
  if (event.altKey) modifiers.push("alt");
  if (event.ctrlKey) modifiers.push("ctrl");
  if (event.shiftKey) modifiers.push("shift");
  return modifiers;
};

export const acceleratorsEqual = (
  left: KeyboardAccelerator,
  right: KeyboardAccelerator,
): boolean => {
  const normalizedLeft = normalizeAccelerator(left);
  const normalizedRight = normalizeAccelerator(right);

  if (normalizedLeft.key !== normalizedRight.key) return false;

  const leftModifiers = normalizedLeft.modifiers ?? [];
  const rightModifiers = normalizedRight.modifiers ?? [];

  if (leftModifiers.length !== rightModifiers.length) return false;

  return leftModifiers.every((modifier, index) => modifier === rightModifiers[index]);
};

export const getAcceleratorForAction = (
  config: AppConfig,
  action: AcceleratorAction,
): KeyboardAccelerator | null => {
  const accelerator = config.accelerators?.[action];
  return accelerator ? normalizeAccelerator(accelerator) : null;
};
