import { isTauri, invoke } from "@tauri-apps/api/core";
import { load } from "@tauri-apps/plugin-store";
import { validateSettings } from "./settings";
import type { Settings } from "../types";
import { resolveLanguage, type Language } from "../i18n";
const key = "enhancece.settings.v1";
const nativeStore = () =>
  load("settings.json", { autoSave: false, defaults: {} });
export async function readSettings(): Promise<Settings> {
  const value = isTauri()
    ? await (await nativeStore()).get(key)
    : JSON.parse(localStorage.getItem(key) || "null");
  const safe = validateSettings(value);
  const explicit =
    value && typeof value === "object" && "language" in value
      ? value.language
      : undefined;
  // Read once, persist through the existing store, then acknowledge. A failed
  // save retains the handoff for the next launch. Saved user choices always win.
  if (isTauri()) {
    const seed = await invoke<unknown>("installer_language").catch(() => null);
    if (
      explicit !== "en" &&
      explicit !== "tr" &&
      (seed === "en" || seed === "tr")
    ) {
      safe.language = seed;
      const store = await nativeStore();
      await store.set(key, safe);
      await store.save();
      await invoke("acknowledge_installer_language").catch(() => undefined);
      return safe;
    }
    await invoke("acknowledge_installer_language").catch(() => undefined);
  }
  const system =
    explicit === "en" || explicit === "tr"
      ? "en"
      : isTauri()
        ? await invoke<string>("system_locale").catch(() => navigator.language)
        : navigator.language;
  safe.language = resolveLanguage(explicit, system || "en");
  return safe;
}
export async function writeLanguage(language: Language) {
  const safe = { ...(await readSettings()), language };
  if (isTauri()) {
    const store = await nativeStore();
    await store.set(key, safe);
    await store.save();
  } else localStorage.setItem(key, JSON.stringify(safe));
}
export async function writeEnginePreference(
  engineMode: Settings["engineMode"],
  modelId: Settings["modelId"],
) {
  const safe = validateSettings({
    ...(await readSettings()),
    engineMode,
    modelId,
  });
  if (isTauri()) {
    const store = await nativeStore();
    await store.set(key, safe);
    await store.save();
  } else localStorage.setItem(key, JSON.stringify(safe));
  return safe;
}
export async function writeSettings(settings: Settings) {
  const safe = validateSettings(settings);
  if (isTauri()) {
    await invoke("validate_preferences", {
      outputDir: safe.outputFolder || null,
      gpuId: safe.gpuId,
    });
    const store = await nativeStore();
    await store.set(key, safe);
    await store.save();
  } else localStorage.setItem(key, JSON.stringify(safe));
}
