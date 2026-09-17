import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Image as NativeImage } from "@tauri-apps/api/image";

let revision = 0;
export async function updateThemeIcon(theme: string) {
  const current = ++revision;
  const src = `/brand/themes/${theme}-symbol.png`;
  const favicon = document.querySelector<HTMLLinkElement>('link[rel="icon"]');
  if (favicon) favicon.href = src;
  if (!isTauri()) return;
  let icon: NativeImage | undefined;
  try {
    const image = new Image();
    image.src = src;
    await image.decode();
    if (current !== revision) return;
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = 128;
    const context = canvas.getContext("2d");
    if (!context) return;
    context.drawImage(image, 0, 0, 128, 128);
    icon = await NativeImage.new(new Uint8Array(context.getImageData(0, 0, 128, 128).data), 128, 128);
    if (current === revision) await getCurrentWindow().setIcon(icon);
  } catch (error) {
    console.warn("Theme window icon could not be updated", error);
  } finally {
    await icon?.close();
  }
}
