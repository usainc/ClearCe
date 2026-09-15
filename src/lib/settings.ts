import { modes, type Settings } from "../types";
export const defaults: Settings = {
  gpuId: null,
  tileSize: 128,
  appearance: "Dark",
  language: "en",
  mode: "Photo",
  scale: 4,
  quality: "Balanced",
  denoise: true,
  sharpen: true,
  faceEnhance: false,
  format: "PNG",
  outputFolder: "",
  lowVram: false,
  previewQuality: "High Quality",
  privacyReminders: true,
};
export function validateSettings(value: unknown): Settings {
  if (!value || typeof value !== "object") return { ...defaults };
  const v = value as Record<string, unknown>;
  const out = { ...defaults };
  const choices = {
    appearance: ["Dark", "Light", "System"],
    language: ["en", "tr"],
    mode: modes,
    scale: [2, 4, 8, 12],
    quality: ["Natural", "Balanced", "Strong"],
    format: ["PNG", "JPG", "WEBP"],
    previewQuality: ["High Quality", "Fast"],
    tileSize: [0, 32, 64, 128, 256],
  };
  for (const [key, allowed] of Object.entries(choices))
    if ((allowed as readonly unknown[]).includes(v[key]))
      Object.assign(out, { [key]: v[key] });
  for (const key of [
    "denoise",
    "sharpen",
    "faceEnhance",
    "lowVram",
    "privacyReminders",
  ] as const)
    if (typeof v[key] === "boolean") out[key] = v[key];
  if (typeof v.outputFolder === "string" && v.outputFolder.length < 4096)
    out.outputFolder = v.outputFolder;
  if (typeof v.gpuId === "string" && v.gpuId.length < 160) out.gpuId = v.gpuId;
  return out;
}
