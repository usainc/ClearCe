import { t, label as display } from "../i18n";
import { Sparkles, Info } from "lucide-react";
import { useApp } from "../stores/AppContext";
import { defaults } from "../lib/settings";
import { useProcessing } from "../stores/ProcessingContext";
import { useQueue } from "../stores/QueueContext";
import { modes, type Settings } from "../types";
import { Button, Panel, Segmented, Toggle, Reset, FolderField } from "./ui";
export function EnhancementSettings({
  preview = false,
  batch = false,
  onAction,
}: {
  preview?: boolean;
  batch?: boolean;
  onAction?: () => void;
}) {
  const { settings: s, setSettings, selected } = useApp();
  const processing = useProcessing();
  const batchQueue = useQueue();
  const update = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setSettings({ ...s, [key]: value });
  return (
    <Panel className="enhancement-settings">
      <div className="panel-heading">
        <h2>
          {batch
            ? t("bulk_settings")
            : preview
              ? t("preview_settings")
              : t("enhancement_settings")}
        </h2>
        <Reset
          onClick={() =>
            setSettings({
              ...s,
              mode: defaults.mode,
              scale: defaults.scale,
              quality: defaults.quality,
              denoise: defaults.denoise,
              sharpen: defaults.sharpen,
              faceEnhance: defaults.faceEnhance,
              format: defaults.format,
            })
          }
        />
      </div>
      {batch && (
        <div className="field">
          <Button
            className="full"
            disabled={
              batchQueue.pending ||
              !batchQueue.queue.items.some((i) => i.task.status === "queued")
            }
            onClick={() => void batchQueue.apply()}
          >
            {t("apply_to_waiting_jobs")}
          </Button>
          <small>{t("settings_also_apply_to_newly_added_images")}</small>
        </div>
      )}
      <div className="field">
        <label htmlFor="enhance-mode">{t("mode")}</label>
        <select
          id="enhance-mode"
          value={s.mode}
          onChange={(e) => update("mode", e.target.value as Settings["mode"])}
        >
          {modes.map((m) => (
            <option key={m} value={m} disabled={m !== "Photo"}>
              {display(m)}
              {m !== "Photo" ? t("planned") : ""}
            </option>
          ))}
        </select>
        <small>
          {s.mode === "Photo"
            ? t("natural_enhancement_for_real_world_photos")
            : t("deferred")}
        </small>
      </div>
      <div className="field">
        <label>{t("upscale")}</label>
        <Segmented
          label={t("upscale")}
          values={[2, 4, 8, 12] as const}
          value={s.scale}
          onChange={(v) => update("scale", v)}
        />
        <small>
          {s.scale > 4
            ? t("two_ai_passes_target_and_intermediate_safety_limits_apply")
            : t("2x_ai_4x_then_downsample_4x_native_ai_output")}
        </small>
      </div>
      {!batch && (
        <>
          <div className="field">
            <label>{t("quality_profile")}</label>
            <Segmented
              label={t("quality_profile_2")}
              values={["Natural", "Balanced", "Strong"] as const}
              disabledValues={["Natural", "Balanced", "Strong"]}
              value={s.quality}
              onChange={(v) => update("quality", v)}
            />
            <small>
              {t(
                "fixed_realesrgan_x4plus_model_adjustable_profiles_are_planned",
              )}
            </small>
          </div>
          <div className="enhance-toggles">
            <Toggle
              label={t("denoise")}
              disabled
              hint={t("model_controlled_no_separate_adjustment")}
              value={s.denoise}
              onChange={(v) => update("denoise", v)}
            />
            <Toggle
              label={t("sharpen")}
              disabled
              hint={t("model_controlled_no_separate_adjustment")}
              value={s.sharpen}
              onChange={(v) => update("sharpen", v)}
            />
            <Toggle
              label={t("face_enhance")}
              disabled
              hint={t("not_available_in_this_phase")}
              value={false}
              onChange={(v) => update("faceEnhance", v)}
            />
          </div>
        </>
      )}
      {!preview && (
        <>
          <div className="field">
            <label>{t("output_format")}</label>
            <Segmented
              label={t("output_format_2")}
              values={["PNG", "JPG", "WEBP"] as const}
              value={s.format}
              onChange={(v) => update("format", v)}
            />
            <small>{t("png_for_quality_jpg_for_smaller_files")}</small>
          </div>
          <div className="field">
            <label>{t("output_folder")}</label>
            <FolderField
              value={s.outputFolder}
              onChange={(v) => update("outputFolder", v)}
            />
          </div>
          <div className="field">
            <label>{t("overwrite_policy")}</label>
            <select aria-label={t("overwrite_policy")} disabled>
              <option>{t("keep_both_never_overwrite")}</option>
            </select>
            <small>{t("existing_outputs_receive_a_numeric_suffix")}</small>
          </div>
        </>
      )}
      {!batch && (
        <>
          <Button
            primary
            className="full enhance-cta"
            disabled={
              preview ||
              processing.busy ||
              batchQueue.queue.running ||
              !selected?.path ||
              s.mode !== "Photo" ||
              processing.engine?.availability !== "available"
            }
            onClick={() => (onAction ? onAction() : void processing.start())}
          >
            <Sparkles size={20} />
            {preview
              ? t("preview_generation_planned")
              : processing.busy
                ? t("processing")
                : t("enhance_image")}
          </Button>
          <div className="local-note">
            <Info size={13} />
            {preview
              ? t("compare_completed_results_here_patch_generation_comes_later")
              : !selected?.path
                ? t("open_a_local_image_in_the_desktop_app_to_begin")
                : processing.engine?.availability !== "available"
                  ? t("check_engine_setup_in_models")
                  : t("local_ai_original_file_stays_untouched")}
          </div>
        </>
      )}
    </Panel>
  );
}
