export type Page =
  "Home" | "Preview" | "Batch Queue" | "History" | "Models" | "Settings";
export const modes = [
  "Photo",
  "Portrait",
  "Screenshot / Text",
  "Anime / Illustration",
] as const;
export type Mode = (typeof modes)[number];
export type Scale = 2 | 4 | 8 | 12;
export type Format = "PNG" | "JPG" | "WEBP";
export interface Settings {
  gpuId: string | null;
  tileSize: number;
  appearance: "Dark" | "Light" | "System";
  language: "en" | "tr";
  mode: Mode;
  scale: Scale;
  quality: "Natural" | "Balanced" | "Strong";
  denoise: boolean;
  sharpen: boolean;
  faceEnhance: boolean;
  format: Format;
  outputFolder: string;
  lowVram: boolean;
  previewQuality: "High Quality" | "Fast";
  privacyReminders: boolean;
}
export interface ImageItem {
  path?: string;
  enhancedSrc?: string;
  id: string;
  name: string;
  src: string;
  width: number;
  height: number;
  bytes: number;
  demo?: boolean;
}
export interface Job {
  id: string;
  image: ImageItem;
  mode: Mode;
  scale: Scale;
  format: Format;
  status: "Waiting" | "Processing" | "Completed" | "Failed" | "Cancelled";
  progress: number;
  seconds: number;
  date: string;
}
