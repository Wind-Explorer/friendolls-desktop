import { commands } from "$lib/bindings";
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
    const result = await commands.recolorGifBase64(
      options.bodyColor,
      options.outlineColor,
      options.applyTexture ?? true,
    );
    return `data:image/gif;base64,${result}`;
  } catch (e) {
    console.error("Failed to recolor sprite:", e);
    return onekoGif;
  }
}
