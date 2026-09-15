import { t } from "../i18n";
import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { defaults } from "../lib/settings";
import { message } from "../lib/processing";
import { readSettings, writeSettings, writeLanguage } from "../lib/persistence";
import { changeLanguage, type Language } from "../i18n";
import { samples, demoJobs } from "../lib/demo";
import type { ImageItem, Job, Page, Settings } from "../types";
interface State {
  page: Page;
  setPage: (p: Page) => void;
  settings: Settings;
  setSettings: (s: Settings) => void;
  savedSettings: Settings;
  saveSettings: (s: Settings) => Promise<boolean>;
  saveLanguage: (language: Language) => Promise<void>;
  ready: boolean;
  images: ImageItem[];
  setImages: React.Dispatch<React.SetStateAction<ImageItem[]>>;
  selected: ImageItem | null;
  setSelected: (i: ImageItem | null) => void;
  queue: Job[];
  setQueue: React.Dispatch<React.SetStateAction<Job[]>>;
  history: Job[];
  setHistory: React.Dispatch<React.SetStateAction<Job[]>>;
  notice: string;
  notify: (message: string) => void;
}
const Context = createContext<State | null>(null);
export function AppProvider({ children }: { children: ReactNode }) {
  const [page, setPage] = useState<Page>("Home");
  const [settings, setSettings] = useState(defaults);
  const [savedSettings, setSavedSettings] = useState(defaults);
  const [ready, setReady] = useState(false);
  const [images, setImages] = useState(samples);
  const [selected, setSelected] = useState<ImageItem | null>(samples[0]);
  const [queue, setQueue] = useState<Job[]>([]);
  const [history, setHistory] = useState(demoJobs);
  const [notice, notify] = useState("");
  useEffect(() => {
    readSettings()
      .then(async (s) => {
        await changeLanguage(s.language);
        setSettings(s);
        setSavedSettings(s);
      })
      .catch(() =>
        notify(t("saved_settings_could_not_be_read_defaults_loaded")),
      )
      .finally(() => setReady(true));
  }, []);
  useEffect(() => {
    if (!notice) return;
    const t = setTimeout(() => notify(""), 6000);
    return () => clearTimeout(t);
  }, [notice]);
  useEffect(() => {
    const media = matchMedia("(prefers-color-scheme: light)");
    const apply = () => {
      document.documentElement.dataset.theme =
        settings.appearance === "Light" ||
        (settings.appearance === "System" && media.matches)
          ? "light"
          : "dark";
    };
    apply();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [settings.appearance]);
  async function saveSettings(s: Settings) {
    try {
      await writeSettings(s);
      await changeLanguage(s.language);
      setSettings(s);
      setSavedSettings(s);
      notify(t("settings_saved_on_this_device"));
      return true;
    } catch (error) {
      notify(message(error));
      return false;
    }
  }
  async function saveLanguage(language: Language) {
    try {
      await writeLanguage(language);
      await changeLanguage(language);
      setSettings((s) => ({ ...s, language }));
      setSavedSettings((s) => ({ ...s, language }));
      notify(t("settings_saved_on_this_device"));
    } catch (e) {
      notify(message(e));
    }
  }
  return (
    <Context.Provider
      value={{
        page,
        setPage,
        settings,
        setSettings,
        savedSettings,
        saveSettings,
        saveLanguage,
        ready,
        images,
        setImages,
        selected,
        setSelected,
        queue,
        setQueue,
        history,
        setHistory,
        notice,
        notify,
      }}
    >
      {children}
    </Context.Provider>
  );
}
export function useApp() {
  const value = useContext(Context);
  if (!value) throw new Error("AppProvider is missing");
  return value;
}
