<script lang="ts">
  import { onMount } from "svelte";
  import {
    commands,
    type AppConfig,
    type SceneInteractivityHotkey,
    type SceneInteractivityKey,
    type SceneInteractivityModifier,
  } from "$lib/bindings";
  import { appData } from "../../../events/app-data";
  import Power from "../../../assets/icons/power.svelte";

  const DEFAULT_HOTKEY: SceneInteractivityHotkey = {
    modifiers: ["alt"],
    key: null,
  };

  const MODIFIER_LABELS: Record<SceneInteractivityModifier, string> = {
    cmd: "Cmd",
    alt: "Alt",
    ctrl: "Ctrl",
    shift: "Shift",
  };

  const KEY_LABELS: Record<SceneInteractivityKey, string> = {
    a: "A",
    b: "B",
    c: "C",
    d: "D",
    e: "E",
    f: "F",
    g: "G",
    h: "H",
    i: "I",
    j: "J",
    k: "K",
    l: "L",
    m: "M",
    n: "N",
    o: "O",
    p: "P",
    q: "Q",
    r: "R",
    s: "S",
    t: "T",
    u: "U",
    v: "V",
    w: "W",
    x: "X",
    y: "Y",
    z: "Z",
    num_0: "0",
    num_1: "1",
    num_2: "2",
    num_3: "3",
    num_4: "4",
    num_5: "5",
    num_6: "6",
    num_7: "7",
    num_8: "8",
    num_9: "9",
    f1: "F1",
    f2: "F2",
    f3: "F3",
    f4: "F4",
    f5: "F5",
    f6: "F6",
    f7: "F7",
    f8: "F8",
    f9: "F9",
    f10: "F10",
    f11: "F11",
    f12: "F12",
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

  const CODE_TO_KEY: Partial<Record<string, SceneInteractivityKey>> = {
    KeyA: "a",
    KeyB: "b",
    KeyC: "c",
    KeyD: "d",
    KeyE: "e",
    KeyF: "f",
    KeyG: "g",
    KeyH: "h",
    KeyI: "i",
    KeyJ: "j",
    KeyK: "k",
    KeyL: "l",
    KeyM: "m",
    KeyN: "n",
    KeyO: "o",
    KeyP: "p",
    KeyQ: "q",
    KeyR: "r",
    KeyS: "s",
    KeyT: "t",
    KeyU: "u",
    KeyV: "v",
    KeyW: "w",
    KeyX: "x",
    KeyY: "y",
    KeyZ: "z",
    Digit0: "num_0",
    Digit1: "num_1",
    Digit2: "num_2",
    Digit3: "num_3",
    Digit4: "num_4",
    Digit5: "num_5",
    Digit6: "num_6",
    Digit7: "num_7",
    Digit8: "num_8",
    Digit9: "num_9",
    F1: "f1",
    F2: "f2",
    F3: "f3",
    F4: "f4",
    F5: "f5",
    F6: "f6",
    F7: "f7",
    F8: "f8",
    F9: "f9",
    F10: "f10",
    F11: "f11",
    F12: "f12",
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

  const MODIFIER_CODES = new Set([
    "MetaLeft",
    "MetaRight",
    "AltLeft",
    "AltRight",
    "ControlLeft",
    "ControlRight",
    "ShiftLeft",
    "ShiftRight",
  ]);

  const normalizeHotkey = (
    hotkey: SceneInteractivityHotkey | null | undefined,
  ): SceneInteractivityHotkey => {
    if (!hotkey) return { ...DEFAULT_HOTKEY };

    const uniqueModifiers = [...new Set(hotkey.modifiers ?? [])].sort();
    if (uniqueModifiers.length === 0 && !hotkey.key) {
      return { ...DEFAULT_HOTKEY };
    }

    return {
      modifiers: uniqueModifiers,
      key: hotkey.key ?? null,
    };
  };

  const formatHotkeyLabel = (hotkey: SceneInteractivityHotkey): string => {
    const modifiers = (hotkey.modifiers ?? []).map(
      (modifier) => MODIFIER_LABELS[modifier],
    );
    const key = hotkey.key ? KEY_LABELS[hotkey.key] : null;
    const parts = key ? [...modifiers, key] : modifiers;
    return parts.join(" + ");
  };

  const getModifiersFromEvent = (
    event: KeyboardEvent,
  ): SceneInteractivityModifier[] => {
    const modifiers: SceneInteractivityModifier[] = [];
    if (event.metaKey) modifiers.push("cmd");
    if (event.altKey) modifiers.push("alt");
    if (event.ctrlKey) modifiers.push("ctrl");
    if (event.shiftKey) modifiers.push("shift");
    return modifiers;
  };

  const hotkeysEqual = (
    a: SceneInteractivityHotkey | null | undefined,
    b: SceneInteractivityHotkey | null | undefined,
  ): boolean => {
    const left = normalizeHotkey(a);
    const right = normalizeHotkey(b);

    if (left.key !== right.key) return false;
    if ((left.modifiers?.length ?? 0) !== (right.modifiers?.length ?? 0)) {
      return false;
    }

    return (left.modifiers ?? []).every(
      (modifier, index) => modifier === (right.modifiers ?? [])[index],
    );
  };

  let signingOut = false;
  let appConfig: AppConfig | null = null;
  let hotkey = { ...DEFAULT_HOTKEY };
  let captureMode = false;
  let capturePreviewLabel = "";
  let hotkeyInput: HTMLInputElement | null = null;
  let hotkeyLabel = "";
  let hotkeyError = "";
  let hotkeySuccess = "";
  let hotkeyDirty = false;
  let hotkeySaving = false;

  const loadConfig = async () => {
    try {
      const config = await commands.getClientConfig();
      appConfig = config;
      hotkey = normalizeHotkey(config.scene_interactivity_hotkey);
    } catch (error) {
      console.error("Failed to load client config", error);
      hotkeyError = "Failed to load current hotkey settings.";
    }
  };

  async function handleSignOut() {
    if (signingOut) return;
    signingOut = true;
    try {
      await commands.logoutAndRestart();
    } catch (error) {
      console.error("Failed to sign out", error);
      signingOut = false;
    }
  }

  const openClientConfig = async () => {
    try {
      await commands.openClientConfig();
    } catch (error) {
      console.error("Failed to open client config", error);
    }
  };

  const beginCapture = () => {
    captureMode = true;
    capturePreviewLabel = "";
    hotkeyError = "";
    hotkeySuccess = "";
    setTimeout(() => hotkeyInput?.focus(), 0);
  };

  const stopCapture = () => {
    captureMode = false;
    capturePreviewLabel = "";
    saveHotkey();
  };

  const saveHotkey = async () => {
    if (!appConfig || hotkeySaving) return;
    hotkeySaving = true;
    hotkeyError = "";
    hotkeySuccess = "";

    try {
      const nextConfig: AppConfig = {
        ...appConfig,
        scene_interactivity_hotkey: hotkey,
      };
      await commands.saveClientConfig(nextConfig);
      appConfig = nextConfig;
      hotkeySuccess = "Hotkey saved.";
    } catch (error) {
      console.error("Failed to save hotkey", error);
      hotkeyError = "Failed to save hotkey.";
    } finally {
      hotkeySaving = false;
    }
  };

  const onHotkeyCapture = async (event: KeyboardEvent) => {
    if (!captureMode || hotkeySaving) return;

    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      stopCapture();
      return;
    }

    const modifiers = getModifiersFromEvent(event);
    const key = CODE_TO_KEY[event.code] ?? null;
    const modifierOnlyPress = MODIFIER_CODES.has(event.code);

    if (modifiers.length === 0) {
      capturePreviewLabel = "";
      hotkeyError = "Hotkey must include at least one modifier key.";
      return;
    }

    if (modifierOnlyPress) {
      const nextHotkey = normalizeHotkey({
        modifiers,
        key: null,
      });
      hotkey = nextHotkey;
      capturePreviewLabel = formatHotkeyLabel(nextHotkey);
      hotkeySuccess = "";
      return;
    }

    if (!key) {
      hotkeyError = "That key is not supported for hotkey combos yet.";
      return;
    }

    const nextHotkey = normalizeHotkey({ modifiers, key });
    hotkey = nextHotkey;
    capturePreviewLabel = formatHotkeyLabel(nextHotkey);
    hotkeySuccess = "";
  };

  $: hotkeyLabel = formatHotkeyLabel(hotkey);
  $: hotkeyDirty = appConfig
    ? !hotkeysEqual(hotkey, appConfig.scene_interactivity_hotkey)
    : false;

  onMount(() => {
    loadConfig();

    const handleKeydown = (event: KeyboardEvent) => {
      void onHotkeyCapture(event);
    };

    window.addEventListener("keydown", handleKeydown, true);

    return () => {
      window.removeEventListener("keydown", handleKeydown, true);
    };
  });
</script>

<div class="size-full flex flex-col justify-between">
  <div class="flex flex-col gap-4 max-w-md">
    <p>{$appData?.user?.name}'s preferences</p>
    <label class="flex flex-col gap-2">
      <span class="text-sm">Scene Interactivity Hotkey</span>
      <div class="flex flex-row items-center gap-2">
        <input
          class="input input-bordered flex-1"
          readonly
          bind:this={hotkeyInput}
          value={captureMode
            ? capturePreviewLabel || "Press your shortcut..."
            : hotkeyLabel}
        />
        {#if captureMode}
          <button
            class="btn btn-outline"
            type="button"
            disabled={hotkeySaving}
            onclick={stopCapture}
          >
            {hotkeySaving ? "Saving..." : "Stop Record"}
          </button>
        {:else}
          <div class="flex flex-row gap-2">
            <button
              class="btn btn-outline"
              type="button"
              disabled={!appConfig || hotkeySaving}
              onclick={beginCapture}
            >
              Record
            </button>
          </div>
        {/if}
      </div>
      <span class="text-xs opacity-70">
        Requires at least one modifier (Cmd, Alt, Ctrl, Shift). Press Escape to
        cancel recording.
      </span>
      {#if hotkeyError}
        <span class="text-xs text-error">{hotkeyError}</span>
      {/if}
      {#if hotkeySuccess}
        <span class="text-xs text-success">{hotkeySuccess}</span>
      {/if}
    </label>
    <div class="flex flex-row gap-2">
      <button
        class="btn"
        class:btn-disabled={signingOut}
        onclick={handleSignOut}
      >
        {signingOut ? "Signing out..." : "Sign out"}
      </button>
      <button class="btn btn-outline" onclick={openClientConfig}>
        Advanced options
      </button>
    </div>
  </div>
  <div class="w-full flex flex-row justify-between">
    <div></div>
    <div>
      <button
        class="btn btn-error btn-square btn-soft"
        onclick={async () => {
          await commands.quitApp();
        }}
      >
        <div class="scale-50">
          <Power />
        </div>
      </button>
    </div>
  </div>
</div>
