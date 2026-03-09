import { commands, type DollColorSchemeDto } from "$lib/bindings";
import onekoGif from "../../assets/oneko/oneko.gif";

export async function getSpriteSheetUrl(
  options?: DollColorSchemeDto,
): Promise<string> {
  if (!options) {
    return onekoGif;
  }

  try {
    const result = await commands.recolorGifBase64(
      options.body,
      options.outline,
      true, // TODO: default true for now, will add customization in the future
    );
    return `data:image/gif;base64,${result}`;
  } catch (e) {
    console.error("Failed to recolor sprite:", e);
    return onekoGif;
  }
}
