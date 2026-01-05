import { invoke } from "@tauri-apps/api/core";
import onekoGif from "../../assets/oneko/oneko.gif";

export interface RecolorOptions {
  bodyColor: string;
  outlineColor: string;
  applyTexture?: boolean;
}

export async function getSpriteSheetUrl(
  options?: RecolorOptions,
): Promise<string> {
  if (!options || !options.bodyColor || !options.outlineColor) {
    return onekoGif;
  }

  try {
    const result = await invoke<string>("recolor_gif_base64", {
      whiteColorHex: options.bodyColor,
      blackColorHex: options.outlineColor,
      applyTexture: options.applyTexture ?? true,
    });
    return `data:image/gif;base64,${result}`;
  } catch (e) {
    console.error("Failed to recolor sprite:", e);
    return onekoGif;
  }
}
