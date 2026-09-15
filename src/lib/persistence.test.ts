import { beforeEach, expect, it, vi } from "vitest";
const mock = vi.hoisted(() => ({
  invoke: vi.fn(),
  get: vi.fn(),
  set: vi.fn(),
  save: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({
  isTauri: () => true,
  invoke: mock.invoke,
}));
vi.mock("@tauri-apps/plugin-store", () => ({ load: async () => mock }));
import { readSettings, writeSettings, writeLanguage } from "./persistence";
import { defaults } from "./settings";
beforeEach(() => {
  vi.clearAllMocks();
  mock.invoke.mockResolvedValue(undefined);
});
it("persists an explicit language independently of invalid processing preferences", async () => {
  mock.get.mockResolvedValue({ ...defaults, outputFolder: "missing folder" });
  await writeLanguage("tr");
  expect(mock.set).toHaveBeenCalledWith(
    "enhancece.settings.v1",
    expect.objectContaining({ language: "tr", outputFolder: "missing folder" }),
  );
  expect(mock.invoke).not.toHaveBeenCalled();
  mock.get.mockResolvedValue({ ...defaults, language: "tr" });
  expect((await readSettings()).language).toBe("tr");
});
it("uses Windows locale only when no language choice was saved", async () => {
  mock.get.mockResolvedValue(undefined);
  mock.invoke.mockResolvedValue("tr-TR");
  expect((await readSettings()).language).toBe("tr");
  mock.invoke.mockResolvedValue("fr-FR");
  expect((await readSettings()).language).toBe("en");
});
it("validates native output and GPU before persisting defaults", async () => {
  const value = { ...defaults, scale: 12 as const, format: "WEBP" as const };
  await writeSettings(value);
  expect(mock.invoke).toHaveBeenCalledWith("validate_preferences", {
    outputDir: null,
    gpuId: null,
  });
  expect(mock.set).toHaveBeenCalledWith("enhancece.settings.v1", value);
  expect(mock.save).toHaveBeenCalledOnce();
  mock.get.mockResolvedValue(value);
  expect(await readSettings()).toEqual(value);
});
it("invalid output or stale GPU never overwrites saved preferences", async () => {
  mock.invoke.mockRejectedValue({
    message: "Choose an accessible folder or available GPU",
  });
  await expect(writeSettings(defaults)).rejects.toBeDefined();
  expect(mock.set).not.toHaveBeenCalled();
  expect(mock.save).not.toHaveBeenCalled();
});
