import { t } from "../i18n";
import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { listen } from "@tauri-apps/api/event";
import {
  queueApi,
  requestSettings,
  isTauri,
  message,
  type QueueSnapshot,
  type EnqueueReport,
} from "../lib/processing";
import { useApp } from "./AppContext";
const empty: QueueSnapshot = {
  items: [],
  running: false,
  activeId: null,
  revision: 0,
  error: null,
};
interface State {
  queue: QueueSnapshot;
  pending: boolean;
  add: (paths: string[]) => Promise<void>;
  folder: (path: string) => Promise<void>;
  control: (action: string, id?: string) => Promise<void>;
  apply: () => Promise<void>;
}
const Context = createContext<State | null>(null);
export function QueueProvider({ children }: { children: ReactNode }) {
  const app = useApp();
  const latest = useRef(app);
  latest.current = app;
  const [queue, setQueue] = useState(empty);
  const [pending, setPending] = useState(false);
  function receive(next: QueueSnapshot) {
    setQueue((prev) => (next.revision >= prev.revision ? next : prev));
  }
  useEffect(() => {
    if (!isTauri()) return;
    let stopped = false;
    const sub = listen<QueueSnapshot>("batch-queue", (e) => {
      if (!stopped) receive(e.payload);
    });
    void queueApi
      .snapshot()
      .then((q) => {
        if (!stopped) receive(q);
      })
      .catch((e) => latest.current.notify(message(e)));
    return () => {
      stopped = true;
      void sub.then((fn) => fn());
    };
  }, []);
  async function perform(action: () => Promise<void | EnqueueReport>) {
    if (!isTauri()) {
      latest.current.notify(
        t("batch_processing_requires_the_windows_application"),
      );
      return;
    }
    setPending(true);
    try {
      const report = await action();
      if (report) {
        receive(report.snapshot);
        latest.current.notify(
          t("queue_import", {
            accepted: report.accepted,
            rejected: report.rejected.length,
            details: report.rejected
              .slice(0, 2)
              .map((r) => message(r.error))
              .join(" · "),
          }),
        );
      } else receive(await queueApi.snapshot());
    } catch (e) {
      latest.current.notify(message(e));
    } finally {
      setPending(false);
    }
  }
  return (
    <Context.Provider
      value={{
        queue,
        pending,
        add: (paths) =>
          perform(() =>
            queueApi.add(paths, requestSettings(latest.current.settings)),
          ),
        folder: (path) =>
          perform(() =>
            queueApi.folder(path, requestSettings(latest.current.settings)),
          ),
        control: (action, id) => perform(() => queueApi.control(action, id)),
        apply: () =>
          perform(() =>
            queueApi.apply(requestSettings(latest.current.settings)),
          ),
      }}
    >
      {children}
    </Context.Provider>
  );
}
export function useQueue() {
  const c = useContext(Context);
  if (!c) throw new Error("QueueProvider missing");
  return c;
}
