import { beforeEach, expect, it } from "vitest";
import i18n, {
  changeLanguage,
  t,
  resolveLanguage,
  errorText,
  number,
} from "./index";
import en from "./locales/en.json";
import tr from "./locales/tr.json";
import rustErrors from "../../src-tauri/src/core/processing/errors.rs?raw";
beforeEach(async () => {
  await changeLanguage("en");
});
it("covers every serialized Rust error code", () => {
  const codes = rustErrors.split("pub enum ErrorCode {")[1].split("}")[0].match(/\w+/g)!;
  for (const code of codes) {
    expect(en, code).toHaveProperty("error_" + code);
    expect(tr, code).toHaveProperty("error_" + code);
  }
});
it("loads both complete catalogs with matching interpolation parameters", () => {
  expect(Object.keys(tr).sort()).toEqual(Object.keys(en).sort());
  for (const key of Object.keys(en) as (keyof typeof en)[]) {
    expect(tr[key].trim(), key).not.toBe("");
    expect((tr[key].match(/{{\w+}}/g) || []).sort(), key).toEqual(
      (en[key].match(/{{\w+}}/g) || []).sort(),
    );
  }
});
it("resolves system language while preserving explicit choice", () => {
  expect(resolveLanguage(undefined, "tr-TR")).toBe("tr");
  expect(resolveLanguage(null, "TR")).toBe("tr");
  expect(resolveLanguage(undefined, "de-DE")).toBe("en");
  expect(resolveLanguage("en", "tr-TR")).toBe("en");
  expect(resolveLanguage("tr", "en-US")).toBe("tr");
});
it("switches translations and number formatting live", async () => {
  expect(t("home")).toBe("Home");
  await changeLanguage("tr");
  expect(t("home")).toBe("Ana Sayfa");
  expect(number(1.5, 1)).toBe("1,5");
});
it("falls back to English for a missing Turkish key and safely handles unknown keys", async () => {
  await changeLanguage("tr");
  const bundle = i18n.getResourceBundle("tr", "translation");
  const previous = bundle.home;
  delete bundle.home;
  expect(t("home")).toBe("Home");
  bundle.home = previous;
  expect(t("not_a_real_key")).toBe(en.generic_error);
});
it("maps important typed errors in both languages without rendering backend prose", async () => {
  const codes = [
    "ENGINE_NOT_FOUND",
    "NO_COMPATIBLE_GPU",
    "INVALID_IMAGE",
    "UNSAFE_TARGET_RESOLUTION",
    "INSUFFICIENT_DISK_SPACE",
    "PROCESSING_FAILED",
    "JOB_INTERRUPTED",
  ];
  for (const language of ["en", "tr"] as const) {
    await changeLanguage(language);
    for (const code of codes) {
      const result = errorText({ code, message: "RAW RUST SECRET DEBUG" });
      expect(result).toBe(
        (language === "en" ? en : tr)[("error_" + code) as keyof typeof en],
      );
      expect(result).not.toContain("RAW");
    }
  }
});
