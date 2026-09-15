import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  api,
  imageItem,
  inspectImage,
  isTauri,
  message,
  terminal,
  reconcileTask,
  requestSettings,
  type Task,
  type EngineStatus,
} from "../lib/processing";
import { useApp } from "./AppContext";
interface State {
  task: Task | null;
  engine: EngineStatus | null;
  jobs: Task[];
  busy: boolean;
  refreshing: boolean;
  refresh: () => Promise<void>;
  start: () => Promise<void>;
  cancel: () => Promise<void>;
  compare: (t: Task) => Promise<void>;
  remove: (id: string) => Promise<void>;
  open: (id: string, folder?: boolean) => Promise<void>;
}
const Context = createContext<State | null>(null);
export function ProcessingProvider({ children }: { children: ReactNode }) {
  const app = useApp();
  const latest = useRef(app);
  latest.current = app;
  const [task, setTask] = useState<Task | null>(null);
  const [engine, setEngine] = useState<EngineStatus | null>(null);
  const [jobs, setJobs] = useState<Task[]>([]);
  const [starting, setStarting] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const handled = useRef(new Set<string>());
  const mounted = useRef(true);
  async function refresh() {
    if (!isTauri()) return;
    setRefreshing(true);
    try {
      const [e, h] = await Promise.all([api.status(), api.history()]);
      if (mounted.current) {
        setEngine(e);
        setJobs(h);
      }
    } catch (e) {
      latest.current.notify(message(e));
    } finally {
      if (mounted.current) setRefreshing(false);
    }
  }
  async function receive(t: Task | null) {
    if (!mounted.current || !t) return;
    setTask((prev) => reconcileTask(prev, t));
    if (!terminal(t) || handled.current.has(t.id)) return;
    handled.current.add(t.id);
    try {
      setJobs(await api.history());
      if (t.output) {
        const output = await api.result(t.id);
        const enhancedSrc = convertFileSrc(output.path);
        const a = latest.current;
        a.setImages((items) =>
          items.map((i) =>
            i.path === t.request.inputPath ? { ...i, enhancedSrc } : i,
          ),
        );
        if (a.selected?.path === t.request.inputPath)
          a.setSelected({ ...a.selected, enhancedSrc });
      }
    } catch (e) {
      latest.current.notify(message(e));
    }
  }
  useEffect(() => {
    mounted.current = true;
    if (!isTauri())
      return () => {
        mounted.current = false;
      };
    let stopped = false;
    const subscription = listen<Task>("enhancement-task", (e) => {
      if (!stopped) void receive(e.payload);
    });
    void refresh();
    void api
      .active()
      .then((t) => {
        if (!stopped) void receive(t);
      })
      .catch((e) => latest.current.notify(message(e)));
    // Recover missed event deliveries, including completion after navigation.
    const timer = setInterval(() => {
      void api
        .active()
        .then((t) => {
          if (!stopped) void receive(t);
        })
        .catch(() => {});
    }, 1500);
    return () => {
      stopped = true;
      mounted.current = false;
      clearInterval(timer);
      void subscription.then((unlisten) => unlisten());
    };
  }, []);
  const busy = starting || !!(task && !terminal(task));
  async function start() {
    const a = latest.current;
    if (!a.selected?.path || busy) return;
    setStarting(true);
    try {
      await api.start(requestSettings(a.settings, a.selected.path));
      await receive(await api.active());
    } catch (e) {
      a.notify(message(e));
    } finally {
      setStarting(false);
    }
  }
  async function compare(t: Task) {
    try {
      const input = await inspectImage(t.request.inputPath);
      const enhancedSrc = t.output
        ? convertFileSrc((await api.result(t.id)).path)
        : undefined;
      latest.current.setSelected({ ...imageItem(input), enhancedSrc });
      latest.current.setPage("Home");
    } catch (e) {
      latest.current.notify(message(e));
    }
  }
  async function safe(action: () => Promise<unknown>) {
    try {
      await action();
    } catch (e) {
      latest.current.notify(message(e));
    }
  }
  return (
    <Context.Provider
      value={{
        task,
        engine,
        jobs,
        busy,
        refreshing,
        refresh,
        start,
        compare,
        cancel: () =>
          safe(() => (task ? api.cancel(task.id) : Promise.resolve())),
        remove: (id) =>
          safe(async () => {
            await api.remove(id);
            setJobs(await api.history());
          }),
        open: (id, folder) => safe(() => api.open(id, folder)),
      }}
    >
      {children}
    </Context.Provider>
  );
}
export function useProcessing() {
  const c = useContext(Context);
  if (!c) throw new Error("ProcessingProvider is missing");
  return c;
}
