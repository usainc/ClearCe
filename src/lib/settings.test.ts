import { describe, expect, it } from "vitest";
import { defaults, validateSettings } from "./settings";
describe("persisted settings validation", () => {
  it("validates GPU identity and tile size independently", () => {
    expect(validateSettings({ gpuId: "device-id", tileSize: 0 })).toMatchObject(
      { gpuId: "device-id", tileSize: 0 },
    );
    expect(validateSettings({ gpuId: 7, tileSize: 999 })).toMatchObject({
      gpuId: null,
      tileSize: 128,
    });
  });
  it("recovers defaults from malformed or missing storage", () => {
    for (const value of [null, undefined, 42, "bad"])
      expect(validateSettings(value)).toEqual(defaults);
  });
  it("rejects invalid enums, booleans, scales, and paths while preserving valid preferences", () => {
    const s = validateSettings({
      appearance: "Light",
      scale: 999,
      mode: "untrusted",
      denoise: "true",
      outputFolder: "x".repeat(5000),
      format: "WEBP",
    });
    expect(s.appearance).toBe("Light");
    expect(s.scale).toBe(4);
    expect(s.mode).toBe("Photo");
    expect(s.denoise).toBe(true);
    expect(s.outputFolder).toBe("");
    expect(s.format).toBe("WEBP");
  });
  it("roundtrips supported preferences without retaining unknown keys", () => {
    const value = {
      ...defaults,
      scale: 12,
      lowVram: true,
      outputFolder: "C:\\Pictures",
      unknown: "ignored",
    };
    expect(validateSettings(JSON.parse(JSON.stringify(value)))).toEqual({
      ...defaults,
      scale: 12,
      lowVram: true,
      outputFolder: "C:\\Pictures",
    });
  });
  it("persists safe auto/manual engine and model choices", () => {
    expect(
      validateSettings({
        engineMode: "Manual",
        modelId: "realesrgan-x4plus-anime",
      }),
    ).toMatchObject({
      engineMode: "Manual",
      modelId: "realesrgan-x4plus-anime",
    });
    expect(
      validateSettings({ engineMode: "Remote", modelId: "random-url" }),
    ).toMatchObject({ engineMode: "Auto", modelId: "auto" });
  });
});
