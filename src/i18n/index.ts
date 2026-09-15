import i18next from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./locales/en.json";
import tr from "./locales/tr.json";
export type Language = "en" | "tr";
export function resolveLanguage(explicit: unknown, system: string): Language {
  return explicit === "en" || explicit === "tr"
    ? explicit
    : system.toLowerCase().startsWith("tr")
      ? "tr"
      : "en";
}
void i18next.use(initReactI18next).init({
  resources: { en: { translation: en }, tr: { translation: tr } },
  lng: "en",
  fallbackLng: "en",
  initAsync: false,
  interpolation: { escapeValue: false },
  returnNull: false,
});
export const t = (key: string, params: Record<string, unknown> = {}) =>
  String(i18next.t(key, { ...params, defaultValue: en.generic_error }));
export const language = (): Language =>
  i18next.resolvedLanguage === "tr" ? "tr" : "en";
export const locale = () => (language() === "tr" ? "tr-TR" : "en-US");
export async function changeLanguage(value: Language) {
  await i18next.changeLanguage(value);
  if (typeof document !== "undefined") document.documentElement.lang = value;
}
export const number = (value: number, digits = 0) =>
  new Intl.NumberFormat(locale(), {
    maximumFractionDigits: digits,
    minimumFractionDigits: digits,
  }).format(value);
export const date = (value: number | string) =>
  new Date(value).toLocaleString(locale());
export const time = (value: number) =>
  new Date(value).toLocaleTimeString(locale());
const labels: Record<string, string> = {
  Home: "home",
  Preview: "preview",
  "Batch Queue": "batch_queue",
  History: "history",
  Models: "models",
  Settings: "settings",
  Photo: "photo",
  Portrait: "portrait",
  "Screenshot / Text": "screenshot_text",
  "Anime / Illustration": "anime_illustration",
  Dark: "dark",
  Light: "light",
  System: "system",
  Natural: "natural",
  Balanced: "balanced",
  Strong: "strong",
  "High Quality": "high_quality",
  Fast: "fast",
  Auto: "auto",
  Automatic: "auto",
  Waiting: "waiting",
  queued: "waiting",
  waiting: "waiting",
  preparing: "preparing",
  processing: "processing_2",
  Processing: "processing_2",
  completed: "completed",
  Completed: "completed",
  failed: "failed",
  Failed: "failed",
  cancelled: "cancelled",
  Cancelled: "cancelled",
  interrupted: "interrupted",
  available: "ready",
  missing: "missing",
  invalid: "invalid",
  unsupported: "unsupported",
  unhealthy: "unhealthy",
  ready: "ready",
  active: "active",
  Planned: "deferred",
  Checking: "checking",
  all: "all",
  minimize: "minimize",
  toggleMaximize: "togglemaximize",
  close: "close",
  installed: "installed",
  resource: "resource",
};
export const label = (value: string) =>
  labels[value] ? t(labels[value]) : value;
export function engineText(state?: string) {
  return state === "available"
    ? t("engine_ready")
    : state === "missing"
      ? t("error_EngineNotFound")
      : state === "invalid"
        ? t("error_EngineInvalid")
        : state === "unsupported"
          ? t("error_GpuUnavailable")
          : t("engine_checking");
}
export function errorText(error: unknown): string {
  const code =
    typeof error === "object" && error && "code" in error
      ? String(error.code)
      : "IoError";
  return t("error_" + code);
}
export const phaseText = (task: { status: string; phaseCode?: string }) =>
  task.phaseCode ? t("phase_" + task.phaseCode) : label(task.status);
export default i18next;
