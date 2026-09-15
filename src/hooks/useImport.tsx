import { t } from "../i18n";
import { useRef, useEffect } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { inspectImage, imageItem, isTauri, message } from "../lib/processing";
import { useApp } from "../stores/AppContext";
import type { ImageItem, Job } from "../types";
import { useQueue } from "../stores/QueueContext";
export function useImport(toQueue = false) {
  const input = useRef<HTMLInputElement>(null);
  const app = useApp();
  const queue = useQueue();
  const currentQueue = useRef(queue);
  currentQueue.current = queue;
  const current = useRef(app);
  current.current = app;
  function accept(accepted: ImageItem[]) {
    if (!accepted.length) return;
    const a = current.current;
    a.setImages((prev) => [...accepted, ...prev]);
    a.setSelected(accepted[0]);
    if (toQueue)
      a.setQueue((prev) => [
        ...prev,
        ...accepted.map((image) => ({
          id: crypto.randomUUID(),
          image,
          mode: a.settings.mode,
          scale: a.settings.scale,
          format: a.settings.format,
          status: "Waiting" as const,
          progress: 0,
          seconds: 0,
          date: new Date().toISOString(),
        })),
      ]);
  }
  async function importPaths(paths: string[]) {
    if (toQueue) {
      await currentQueue.current.add(paths);
      return;
    }
    const accepted: ImageItem[] = [];
    for (const path of paths.slice(0, 100)) {
      try {
        accepted.push(imageItem(await inspectImage(path)));
      } catch (e) {
        current.current.notify(message(e));
      }
    }
    accept(accepted);
  }
  useEffect(() => {
    if (!isTauri()) return;
    let stopped = false;
    const listener = getCurrentWebviewWindow().onDragDropEvent((e) => {
      if (!stopped && e.payload.type === "drop")
        void importPaths(e.payload.paths);
    });
    return () => {
      stopped = true;
      void listener.then((unlisten) => unlisten());
    };
  }, [toQueue]);
  async function open() {
    if (!isTauri()) {
      input.current?.click();
      return;
    }
    try {
      const paths = await openDialog({
        multiple: true,
        filters: [
          { name: t("images"), extensions: ["jpg", "jpeg", "png", "webp"] },
        ],
      });
      if (paths) await importPaths(Array.isArray(paths) ? paths : [paths]);
    } catch (e) {
      current.current.notify(message(e));
    }
  }
  async function importFiles(files: FileList | File[]) {
    if (isTauri()) return; // Native drops supply canonical paths through Tauri.
    const accepted: ImageItem[] = [];
    let rejected = 0;
    for (const file of Array.from(files).slice(0, 100)) {
      if (
        !/\.(png|jpe?g|webp)$/i.test(file.name) ||
        file.size > 100 * 1024 * 1024
      ) {
        rejected++;
        continue;
      }
      const src = URL.createObjectURL(file);
      try {
        const image = new Image();
        image.src = src;
        await image.decode();
        accepted.push({
          id: crypto.randomUUID(),
          name: file.name,
          src,
          width: image.naturalWidth,
          height: image.naturalHeight,
          bytes: file.size,
        });
      } catch {
        URL.revokeObjectURL(src);
        rejected++;
      }
    }
    if (accepted.length) {
      app.setImages((prev) => [...accepted, ...prev]);
      app.setSelected(accepted[0]);
      if (toQueue)
        app.setQueue((prev) => [
          ...prev,
          ...accepted.map(
            (image) =>
              ({
                id: crypto.randomUUID(),
                image,
                mode: app.settings.mode,
                scale: app.settings.scale,
                format: app.settings.format,
                status: "Waiting",
                progress: 0,
                seconds: 0,
                date: new Date().toISOString(),
              }) as Job,
          ),
        ]);
    }
    if (rejected || files.length > 100)
      app.notify(t("import_summary", { count: accepted.length }));
  }
  const picker = (
    <input
      ref={input}
      type="file"
      accept=".png,.jpg,.jpeg,.webp"
      multiple
      hidden
      onChange={(e) => {
        if (e.target.files) void importFiles(Array.from(e.target.files));
        e.target.value = "";
      }}
    />
  );
  return { picker, open, importFiles };
}
