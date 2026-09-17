import { invoke, convertFileSrc, isTauri } from "@tauri-apps/api/core";
export { isTauri };
export const requestSettings = (
  s: import("../types").Settings,
  inputPath = "",
): Task["request"] => ({
  inputPath,
  outputDir: s.outputFolder || null,
  scale: s.scale,
  format: s.format,
  mode: s.mode,
  gpuIndex: null,
  gpuId: s.gpuId,
  tileSize: s.tileSize,
  modelId: s.modelId,
  engineMode: s.engineMode,
});
export interface QueueItem {
  id: string;
  task: Task;
  retryCount: number;
  cancelPending: boolean;
}
export interface QueueSnapshot {
  items: QueueItem[];
  running: boolean;
  activeId: string | null;
  revision: number;
  error: string | null;
}
export interface EnqueueReport {
  accepted: number;
  rejected: { name: string; error: { code: string; message: string } }[];
  snapshot: QueueSnapshot;
}
export const queueApi = {
  snapshot: () => invoke<QueueSnapshot>("queue_snapshot"),
  add: (paths: string[], settings: Task["request"]) =>
    invoke<EnqueueReport>("queue_add", { paths, settings }),
  folder: (path: string, settings: Task["request"]) =>
    invoke<EnqueueReport>("queue_add_folder", { path, settings }),
  control: (action: string, id: string | null = null) =>
    invoke<void>("queue_control", { action, id }),
  apply: (settings: Task["request"]) =>
    invoke<void>("queue_apply_settings", { settings }),
};
export interface ImageInfo {
  path: string;
  name: string;
  width: number;
  height: number;
  bytes: number;
}
export interface Task {
  stages: { label: string; scale: number; status: Task["status"] }[];
  activity: { at: number; message: string; code?: string }[];
  target: {
    width: number;
    height: number;
    pixels: number;
    workingBytes: number;
    temporaryBytes: number;
    finalBytes: number;
  } | null;
  selectedGpu: string;
  id: string;
  status:
    | "queued"
    | "preparing"
    | "processing"
    | "completed"
    | "failed"
    | "cancelled";
  phase: string;
  phaseCode?: string;
  progress: number | null;
  input: ImageInfo | null;
  output: ImageInfo | null;
  request: {
    inputPath: string;
    outputDir: string | null;
    scale: number;
    format: "PNG" | "JPG" | "WEBP";
    mode: string;
    gpuIndex: number | null;
    gpuId: string | null;
    tileSize: number;
    modelId: string;
    engineMode: string;
  };
  createdAt: number;
  startedAt: number | null;
  completedAt: number | null;
  engineId: string;
  engineVersion: string | null;
  model: string;
  error: { code: string; message: string } | null;
  warning: string | null;
}
export interface EngineStatus {
  devices: {
    id: string;
    index: number;
    name: string;
    vendor: string;
    deviceType: string;
    dedicatedMemoryBytes: number | null;
  }[];
  id: string;
  name: string;
  model: string;
  models: string[];
  version: string | null;
  availability: string;
  message: string;
  location: string | null;
  installDir: string;
  source: string | null;
  scales: number[];
  formats: string[];
  gpuDevices: string[];
  gpuSelection: string;
}
export const terminal = (t: Task) =>
  ["completed", "failed", "cancelled"].includes(t.status);
export function reconcileTask(previous: Task | null, incoming: Task): Task {
  if (
    previous &&
    ((previous.startedAt || previous.createdAt) >
      (incoming.startedAt || incoming.createdAt) ||
      (previous.id === incoming.id &&
        terminal(previous) &&
        !terminal(incoming)))
  )
    return previous;
  return incoming;
}
export { errorText as message } from "../i18n";
export const inspectImage = (path: string) =>
  invoke<ImageInfo>("inspect_image", { path });
export const imageItem = (i: ImageInfo) => ({
  ...i,
  id: crypto.randomUUID(),
  src: convertFileSrc(i.path),
});
export const api = {
  status: () => invoke<EngineStatus>("engine_status"),
  active: () => invoke<Task | null>("active_enhancement"),
  history: () => invoke<Task[]>("processing_history"),
  start: (request: Task["request"]) =>
    invoke<Task>("start_enhancement", { request }),
  cancel: (id: string) => invoke<void>("cancel_enhancement", { id }),
  remove: (id: string) => invoke<void>("delete_processing_history", { id }),
  result: (id: string) => invoke<ImageInfo>("result_image", { id }),
  open: (id: string, folder = false) =>
    invoke<void>("open_result", { id, folder }),
};
