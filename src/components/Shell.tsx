import { t, label as display } from "../i18n";
import {
  Home,
  Image,
  Layers,
  Clock3,
  Box,
  Settings,
  Cpu,
  ShieldCheck,
  Minus,
  Square,
  X,
} from "lucide-react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useApp } from "../stores/AppContext";
import { useProcessing } from "../stores/ProcessingContext";
import { useQueue } from "../stores/QueueContext";
import { Readiness } from "./Readiness";
import { ThemeBrand } from "./ThemeBrand";
import type { Page } from "../types";
import { useEffect, type ReactNode } from "react";
const nav = [
  [Home, "Home"],
  [Image, "Preview"],
  [Layers, "Batch Queue"],
  [Clock3, "History"],
  [Box, "Models"],
  [Settings, "Settings"],
] as const;
export function Shell({ children }: { children: ReactNode }) {
  const { page, setPage, notice, notify } = useApp();
  const { queue } = useQueue();
  const { engine, busy } = useProcessing();
  useEffect(() => {
    window.scrollTo({ top: 0, left: 0 });
  }, [page]);
  async function windowAction(action: "minimize" | "toggleMaximize" | "close") {
    try {
      await getCurrentWindow()[action]();
    } catch {
      notify(t("window_action_could_not_be_completed"));
    }
  }
  return (
    <div className="app-shell">
      <div className="titlebar">
        <div className="titlebar-brand" data-tauri-drag-region>
          <ThemeBrand className="titlebar-logo" symbol decorative />
          <span>ClearCe</span>
        </div>
        <div className="titlebar-drag" data-tauri-drag-region />
        <span className="titlebar-caption">Local AI Image Enhancer</span>
        {isTauri() && (
          <div className="window-controls">
            {[
              [Minus, "minimize"],
              [Square, "toggleMaximize"],
              [X, "close"],
            ].map(([Icon, action]) => {
              const I = Icon as typeof X;
              return (
                <button
                  key={action as string}
                  aria-label={display(action as string)}
                  onClick={() => void windowAction(action as "close")}
                >
                  <I size={14} />
                </button>
              );
            })}
          </div>
        )}
      </div>
      <aside className="sidebar">
        <div className="brand">
          <ThemeBrand className="brand-logo" />
          <ThemeBrand className="brand-compact" symbol />
        </div>
        <nav aria-label={t("main_navigation")}>
          {nav.map(([Icon, label]) => (
            <button
              key={label}
              aria-label={display(label)}
              className={`nav-item ${page === label ? "active" : ""}`}
              aria-current={page === label ? "page" : undefined}
              onClick={() => setPage(label as Page)}
            >
              <Icon size={22} />
              <span>{display(label)}</span>
              {label === "Batch Queue" && queue.items.length > 0 && (
                <b>{queue.items.length}</b>
              )}
            </button>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <div className="sidebar-card">
            <ShieldCheck />
            <div>
              <strong>{t("local_by_design")}</strong>
              <small>{t("no_cloud_upload")}</small>
              <small>{t("your_images_stay_yours")}</small>
            </div>
          </div>
          <button
            className="sidebar-card gpu-card"
            onClick={() => setPage("Models")}
          >
            <Cpu />
            <div>
              <strong>{t("processing_engine")}</strong>
              <small>{engine?.gpuDevices[0] || t("no_gpu")}</small>
              <span className="engine-dot">
                {busy
                  ? t("processing_2")
                  : display(engine?.availability || "Checking")}
              </span>
            </div>
          </button>
          <footer>
            ClearCe <span>v0.1.0</span>
            <small>by UsainCe.dev</small>
          </footer>
        </div>
      </aside>
      <main>
        {page === "Home" && <Readiness />}
        {children}
      </main>
      {notice && (
        <div className="toast" role="status">
          <ShieldCheck size={18} />
          <span>{notice}</span>
          <button
            aria-label={t("dismiss_notification")}
            onClick={() => notify("")}
          >
            <X size={16} />
          </button>
        </div>
      )}
    </div>
  );
}
