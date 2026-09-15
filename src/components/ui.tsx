import { t, label as display } from "../i18n";
import type { ReactNode, ButtonHTMLAttributes } from "react";
import {
  ShieldCheck,
  Cpu,
  LockKeyhole,
  RotateCcw,
  FolderOpen,
} from "lucide-react";
import { isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useApp } from "../stores/AppContext";
export function Button({
  children,
  primary,
  className = "",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { primary?: boolean }) {
  return (
    <button
      className={`button ${primary ? "primary" : ""} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
}
export function Panel({
  children,
  className = "",
}: {
  children: ReactNode;
  className?: string;
}) {
  return <section className={`panel ${className}`}>{children}</section>;
}
export function PageHeading({
  icon,
  title,
  subtitle,
  actions,
}: {
  icon: ReactNode;
  title: string;
  subtitle: string;
  actions?: ReactNode;
}) {
  return (
    <header className="page-heading">
      <div className="heading-icon">{icon}</div>
      <div>
        <h1>{title}</h1>
        <p>{subtitle}</p>
      </div>
      {actions && <div className="heading-actions">{actions}</div>}
    </header>
  );
}
export function Segmented<T extends string | number>({
  values,
  value,
  onChange,
  label,
  disabledValues = [],
}: {
  values: readonly T[];
  disabledValues?: readonly T[];
  value: T;
  onChange: (v: T) => void;
  label: string;
}) {
  return (
    <div className="segmented" role="group" aria-label={label}>
      {values.map((v) => (
        <button
          key={v}
          disabled={disabledValues.includes(v)}
          aria-pressed={v === value}
          className={v === value ? "selected" : ""}
          onClick={() => onChange(v)}
        >
          {typeof v === "number" ? `${v}x` : display(v)}
        </button>
      ))}
    </div>
  );
}
export function Toggle({
  label,
  hint,
  value,
  onChange,
  disabled = false,
}: {
  label: string;
  hint?: string;
  value: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <label className={`toggle-row ${disabled ? "disabled" : ""}`}>
      <button
        type="button"
        role="switch"
        aria-checked={value}
        aria-label={label}
        disabled={disabled}
        className={`switch ${value ? "on" : ""}`}
        onClick={() => onChange(!value)}
      >
        <span />
      </button>
      <span>
        {label}
        {hint && <small>{hint}</small>}
      </span>
    </label>
  );
}
export function PrivacyPanel() {
  return (
    <Panel className="privacy-panel">
      {[
        [Cpu, t("local_gpu_processing"), "Real-ESRGAN · Vulkan"],
        [
          ShieldCheck,
          t("no_cloud_upload"),
          t("your_images_stay_on_this_device"),
        ],
        [LockKeyhole, t("offline_capable"), t("a_private_local_workflow")],
      ].map(([Icon, title, description]) => {
        const I = Icon as typeof Cpu;
        return (
          <div className="privacy-row" key={title as string}>
            <I />
            <div>
              <strong>{title as string}</strong>
              <small>{description as string}</small>
            </div>
          </div>
        );
      })}
    </Panel>
  );
}
export function Reset({ onClick }: { onClick: () => void }) {
  return (
    <button className="text-button" onClick={onClick}>
      <RotateCcw size={14} />
      {t("reset")}
    </button>
  );
}
export function FolderField({
  value,
  onChange,
}: {
  value: string;
  onChange: (v: string) => void;
}) {
  const { notify } = useApp();
  async function choose() {
    if (!isTauri()) {
      notify(
        t(
          "folder_browsing_is_available_in_the_desktop_app_you_can_enter_a_path_here",
        ),
      );
      return;
    }
    try {
      const path = await open({ directory: true, multiple: false });
      if (typeof path === "string") onChange(path);
    } catch {
      notify(t("could_not_open_the_folder_picker"));
    }
  }
  return (
    <div className="folder-field">
      <input
        aria-label={t("output_folder_2")}
        placeholder={t("choose_an_output_folder")}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
      <button
        title={t("choose_output_folder")}
        aria-label={t("choose_output_folder")}
        onClick={() => void choose()}
      >
        <FolderOpen size={17} />
      </button>
    </div>
  );
}
export function Empty({
  title,
  children,
}: {
  title: string;
  children?: ReactNode;
}) {
  return (
    <div className="empty">
      <FolderOpen size={36} />
      <h3>{title}</h3>
      {children}
    </div>
  );
}
export function Status({ status }: { status: string }) {
  return (
    <span className={`status ${status.toLowerCase()}`}>
      <i />
      {display(status)}
    </span>
  );
}
