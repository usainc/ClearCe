import { t, label as display, number } from "../../i18n";
import { useEffect, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { message } from "../../lib/processing";
import {
  Settings as SettingsIcon,
  Palette,
  Globe,
  Image,
  Expand,
  FileImage,
  FolderOpen,
  Cpu,
  Gauge,
  ScanSearch,
  ShieldCheck,
  Database,
  Save,
  RotateCcw,
  Check,
} from "lucide-react";
import { useApp } from "../../stores/AppContext";
import { useProcessing } from "../../stores/ProcessingContext";
import { defaults } from "../../lib/settings";
import { appearances, modes, type Settings as AppSettings } from "../../types";
import {
  Button,
  FolderField,
  PageHeading,
  Panel,
  Segmented,
  Toggle,
} from "../../components/ui";
import type { ReactNode } from "react";
function SettingCard({
  icon,
  title,
  hint,
  children,
}: {
  icon: ReactNode;
  title: string;
  hint: string;
  children: ReactNode;
}) {
  return (
    <Panel className="setting-card">
      <div className="setting-title">
        {icon}
        <div>
          <h2>{title}</h2>
          <small>{hint}</small>
        </div>
      </div>
      {children}
    </Panel>
  );
}
export function Settings() {
  const { engine } = useProcessing();
  const { savedSettings, saveSettings, saveLanguage, ready, notify } = useApp();
  const [cache, setCache] = useState<number | null>(null);
  async function cacheAction(clear = false) {
    try {
      setCache(await invoke<number>("processing_cache", { clear }));
    } catch (e) {
      notify(message(e));
    }
  }
  useEffect(() => {
    if (isTauri()) void cacheAction();
  }, []);
  const [draft, setDraft] = useState(savedSettings);
  const [saving, setSaving] = useState(false);
  const update = <K extends keyof AppSettings>(k: K, v: AppSettings[K]) =>
    setDraft((s) => ({ ...s, [k]: v }));
  async function save() {
    setSaving(true);
    await saveSettings(draft);
    setSaving(false);
  }
  return (
    <>
      <PageHeading
        icon={<SettingsIcon />}
        title={t("settings")}
        subtitle={t("make_enhancece_feel_at_home_on_your_device")}
      />
      <div className="settings-grid">
        <SettingCard
          icon={<Palette />}
          title={t("appearance")}
          hint={t("choose_your_workspace_theme")}
        >
          <div className="theme-options"><Segmented
            label={t("appearance")}
            values={appearances}
            value={draft.appearance}
            onChange={(v) => update("appearance", v)}
          /></div>
        </SettingCard>
        <SettingCard
          icon={<Globe />}
          title={t("language")}
          hint={t("your_preferred_app_language")}
        >
          <select
            aria-label={t("language")}
            value={draft.language}
            onChange={(e) => {
              const language = e.target.value as "en" | "tr";
              update("language", language);
              void saveLanguage(language);
            }}
          >
            <option value="en">{t("english_english")}</option>
            <option value="tr">Türkçe</option>
          </select>
        </SettingCard>
        <SettingCard
          icon={<Image />}
          title={t("default_mode")}
          hint={t("start_with_the_right_enhancement")}
        >
          <select
            aria-label={t("default_mode_2")}
            value={draft.mode}
            onChange={(e) =>
              update("mode", e.target.value as AppSettings["mode"])
            }
          >
            {modes.map((m) => (
              <option key={m} value={m} disabled={m !== "Photo"}>
                {display(m)}
              </option>
            ))}
          </select>
        </SettingCard>
        <SettingCard
          icon={<Expand />}
          title={t("default_upscale")}
          hint={t("a_little_bigger_a_lot_clearer")}
        >
          <Segmented
            label={t("default_upscale_2")}
            values={[2, 4, 8, 12] as const}
            value={draft.scale}
            onChange={(v) => update("scale", v)}
          />
        </SettingCard>
        <SettingCard
          icon={<FileImage />}
          title={t("default_output_format")}
          hint={t("choose_the_format_for_new_images")}
        >
          <select
            aria-label={t("default_output_format_2")}
            value={draft.format}
            onChange={(e) =>
              update("format", e.target.value as AppSettings["format"])
            }
          >
            {["PNG", "JPG", "WEBP"].map((f) => (
              <option key={f}>{f}</option>
            ))}
          </select>
          <small>
            {t(
              "exif_metadata_is_stripped_after_orientation_is_applied_existing_output_name",
            )}
          </small>
        </SettingCard>
        <SettingCard
          icon={<FolderOpen />}
          title={t("output_folder")}
          hint={t("where_your_finished_images_belong")}
        >
          <FolderField
            value={draft.outputFolder}
            onChange={(v) => update("outputFolder", v)}
          />
        </SettingCard>
        <SettingCard
          icon={<Cpu />}
          title={t("gpu_selection")}
          hint={t("your_local_processing_hardware")}
        >
          <select
            aria-label={t("gpu_selection_2")}
            value={draft.gpuId || ""}
            onChange={(e) => update("gpuId", e.target.value || null)}
          >
            <option value="">{t("automatic_ncnn_default")}</option>
            {engine?.devices.map((d) => (
              <option key={d.id} value={d.id}>
                GPU {d.index} · {d.name}
              </option>
            ))}
            {draft.gpuId &&
              !engine?.devices.some((d) => d.id === draft.gpuId) && (
                <option value={draft.gpuId} disabled>
                  {t("unavailable_gpu_choose_auto")}
                </option>
              )}
          </select>
          <small>{engine?.gpuDevices.join(", ") || t("no_gpu")}</small>
        </SettingCard>
        <SettingCard
          icon={<Gauge />}
          title={t("performance_options")}
          hint={t("tile_size_controls_gpu_memory_use")}
        >
          <select
            aria-label={t("inference_tile_size")}
            value={draft.tileSize}
            onChange={(e) => update("tileSize", Number(e.target.value))}
          >
            {[0, 32, 64, 128, 256].map((n) => (
              <option key={n} value={n}>
                {n
                  ? t("tile_pixels", {
                      size: number(n),
                      default: n === 128 ? t("balanced_default") : "",
                    })
                  : t("auto_engine_selected")}
              </option>
            ))}
          </select>
          <small>
            {t(
              "smaller_tiles_use_less_gpu_memory_the_image_still_needs_safe_cpu_memory_and",
            )}
          </small>
        </SettingCard>
        <SettingCard
          icon={<ScanSearch />}
          title={t("preview_quality")}
          hint={t("balance_image_detail_and_speed")}
        >
          <select
            aria-label={t("preview_quality_2")}
            disabled
            value={draft.previewQuality}
            onChange={(e) =>
              update(
                "previewQuality",
                e.target.value as AppSettings["previewQuality"],
              )
            }
          >
            <option>{t("high_quality")}</option>
            <option>{t("fast")}</option>
          </select>
        </SettingCard>
        <SettingCard
          icon={<RotateCcw />}
          title={t("app_updates")}
          hint={t("you_control_what_gets_installed")}
        >
          <Toggle
            label={t("automatic_updates")}
            hint={t("updater_not_configured")}
            value={false}
            disabled
            onChange={() => {}}
          />
        </SettingCard>
        <SettingCard
          icon={<ShieldCheck />}
          title={t("privacy_controls")}
          hint={t("a_private_workspace_by_default")}
        >
          <Toggle
            label={t("show_privacy_reminders")}
            value={draft.privacyReminders}
            onChange={(v) => update("privacyReminders", v)}
          />
          <small>{t("no_analytics_or_telemetry_are_collected")}</small>
        </SettingCard>
        <SettingCard
          icon={<Database />}
          title={t("cache_storage")}
          hint={t("keep_your_workspace_tidy")}
        >
          <div className="panel-heading">
            <span>{t("processing_cache")}</span>
            <strong>
              {cache === null
                ? t("not_checked")
                : `${number(cache / 1048576, 2)} MiB`}
            </strong>
          </div>
          <Button
            disabled={!isTauri()}
            className="full"
            onClick={() => void cacheAction(true)}
          >
            {t("clear_cache")}
          </Button>
          <small>
            {t(
              "only_temporary_job_data_images_outputs_history_settings_and_models_are_pres",
            )}
          </small>
        </SettingCard>
      </div>
      <Panel className="privacy-footer">
        <ShieldCheck size={34} />
        <div>
          <h2>{t("local_processing_privacy")}</h2>
          <p>
            {t(
              "images_are_processed_locally_enhancece_does_not_upload_enhancement_jobs_to_",
            )}
          </p>
        </div>
        <div>
          <span>
            <Check size={16} />
            {t("no_cloud_upload")}
          </span>
          <span>
            <Check size={16} />
            {t("no_telemetry")}
          </span>
        </div>
      </Panel>
      <div className="settings-footer">
        <small>
          {JSON.stringify(draft) === JSON.stringify(savedSettings)
            ? t("settings_are_up_to_date")
            : t("you_have_unsaved_changes")}
        </small>
        <Button onClick={() => setDraft({ ...defaults })}>
          <RotateCcw size={17} />
          {t("reset_to_defaults")}
        </Button>
        <Button primary disabled={!ready || saving} onClick={() => void save()}>
          <Save size={17} />
          {saving ? t("saving") : t("save_changes")}
        </Button>
      </div>
    </>
  );
}
