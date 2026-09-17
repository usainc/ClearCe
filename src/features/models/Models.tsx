import { t, engineText, number } from "../../i18n";
import { useCallback, useEffect, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Box,
  Cpu,
  RefreshCw,
  FolderOpen,
  ShieldCheck,
  Image,
  Download,
  CircleHelp,
  CheckCircle2,
  AlertTriangle,
  X,
} from "lucide-react";
import { useProcessing } from "../../stores/ProcessingContext";
import { useApp } from "../../stores/AppContext";
import { Readiness } from "../../components/Readiness";
import { message } from "../../lib/processing";
import { Button, PageHeading, Panel, Status } from "../../components/ui";

interface CatalogItem {
  id: string;
  displayName: string;
  category: string;
  descriptionKey: string;
  installStatus: string;
  engineCompatibility: string;
  downloadable: boolean;
  availability: string;
}
interface ManagedState {
  catalog: CatalogItem[];
  gpu: {
    names: string[];
    vendors: string[];
    deviceCount: number;
    vulkanAvailable: boolean;
    dedicatedMemoryBytes: number | null;
    tier: string;
  };
  recommendation: {
    modelId: string | null;
    displayName: string | null;
    profile: string;
    reasonKey: string;
    installed: boolean;
    action: string;
  };
  activeEngineType: string;
  activeModel: string;
  activePath: string | null;
  managedInstalled: boolean;
  manualInstalled: boolean;
  packageVersion: string;
  packageSource: string;
}
interface InstallProgress {
  stage: string;
  downloadedBytes: number;
  totalBytes: number | null;
}

export function Models() {
  const { engine: e, refresh, refreshing, busy } = useProcessing();
  const { settings, saveEnginePreference, notify } = useApp();
  const [managed, setManaged] = useState<ManagedState | null>(null);
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<InstallProgress | null>(null);
  const [guide, setGuide] = useState(false);
  const load = useCallback(async () => {
    if (!isTauri()) return;
    try {
      setManaged(
        await invoke<ManagedState>("managed_engine_state", {
          category: settings.mode,
          engineMode: settings.engineMode,
          modelId: settings.modelId,
        }),
      );
    } catch (error) {
      notify(message(error));
    }
  }, [settings.mode, settings.engineMode, settings.modelId, notify]);
  useEffect(() => {
    void load();
  }, [load, e]);
  useEffect(() => {
    if (!isTauri()) return;
    const sub = listen<InstallProgress>("managed-engine-progress", (event) =>
      setProgress(event.payload),
    );
    return () => void sub.then((unlisten) => unlisten());
  }, []);
  async function choose(
    engineMode: "Auto" | "Manual",
    modelId = settings.modelId,
  ) {
    if (await saveEnginePreference(engineMode, modelId)) {
      await refresh();
      await load();
    }
  }
  async function installManaged() {
    if (!managed || installing) return;
    setInstalling(true);
    setProgress({ stage: "preparing", downloadedBytes: 0, totalBytes: null });
    try {
      await invoke("install_managed_engine", {
        sourceUrl: managed.packageSource,
      });
      await saveEnginePreference("Auto", "auto");
      await refresh();
      await load();
      notify(t("managed_install_complete"));
    } catch (error) {
      notify(message(error));
    } finally {
      setInstalling(false);
    }
  }
  const recommendation = managed?.recommendation;
  return (
    <>
      <PageHeading
        icon={<Box />}
        title={t("ai_models_engine_management")}
        subtitle={t("managed_models_subtitle")}
        actions={
          <Button onClick={() => setGuide(true)}>
            <CircleHelp size={15} /> {t("learn_about_engines")}
          </Button>
        }
      />
      <div className="workspace-layout models-workspace">
        <div className="workspace-column">
          <Panel className="engine-overview">
            <div className="panel-heading">
              <div>
                <h2>{t("current_status")}</h2>
                <small>{t("engine_status_always_visible")}</small>
              </div>
              <Button
                disabled={refreshing || installing}
                onClick={() => void refresh().then(load)}
              >
                <RefreshCw size={15} />{" "}
                {refreshing ? t("checking") : t("verify_installation")}
              </Button>
            </div>
            <div className="engine-status-grid">
              <div>
                <small>{t("engine_source")}</small>
                <strong>
                  {t(`engine_type_${managed?.activeEngineType || "none"}`)}
                </strong>
              </div>
              <div>
                <small>{t("active_model")}</small>
                <strong>
                  {settings.modelId === "auto"
                    ? recommendation?.displayName || t("auto_recommended")
                    : managed?.catalog.find((m) => m.id === settings.modelId)
                        ?.displayName}
                </strong>
              </div>
              <div>
                <small>{t("vulkan_status")}</small>
                <strong>
                  {managed?.gpu.vulkanAvailable
                    ? t("available")
                    : t("unavailable")}
                </strong>
              </div>
            </div>
            <div
              className="engine-mode-control"
              role="group"
              aria-label={t("engine_selection_mode")}
            >
              <button
                className={settings.engineMode === "Auto" ? "selected" : ""}
                onClick={() => void choose("Auto", "auto")}
              >
                {t("auto_recommended")}
              </button>
              <button
                className={settings.engineMode === "Manual" ? "selected" : ""}
                onClick={() => void choose("Manual")}
              >
                {t("manual_engine")}
              </button>
            </div>
            <label className="model-select">
              <span>{t("model_selection")}</span>
              <select
                value={settings.modelId}
                onChange={(event) =>
                  void choose(
                    settings.engineMode,
                    event.target.value as typeof settings.modelId,
                  )
                }
              >
                <option value="auto">{t("auto_recommended")}</option>
                {managed?.catalog
                  .filter((m) => m.availability === "available")
                  .map((model) => (
                    <option key={model.id} value={model.id}>
                      {model.displayName}
                      {model.installStatus !== "installed"
                        ? ` · ${t("not_installed")}`
                        : ""}
                    </option>
                  ))}
              </select>
            </label>
          </Panel>

          <Panel className="recommendation-panel">
            <div className="recommendation-icon">
              {recommendation?.modelId ? <CheckCircle2 /> : <AlertTriangle />}
            </div>
            <div>
              <small>{t("recommended_for_your_pc")}</small>
              <h2>
                {recommendation?.displayName ||
                  t("no_compatible_recommendation")}
              </h2>
              <p>
                {recommendation ? t(recommendation.reasonKey) : t("checking")}
              </p>
              {recommendation?.modelId && (
                <p>
                  {t("recommended_profile")}:{" "}
                  <strong>
                    {t(`profile_${recommendation.profile.toLowerCase()}`)}
                  </strong>
                </p>
              )}
              <div className="recommendation-actions">
                {!recommendation?.installed &&
                  recommendation?.action === "download" && (
                    <Button
                      primary
                      disabled={installing || busy}
                      onClick={() => void installManaged()}
                    >
                      <Download size={16} /> {t("download_recommended")}
                    </Button>
                  )}
                {managed?.managedInstalled && (
                  <Button
                    disabled={installing || busy}
                    onClick={() => void installManaged()}
                  >
                    {t("reinstall_managed_engine")}
                  </Button>
                )}
              </div>
            </div>
          </Panel>

          {progress && (
            <Panel className="managed-progress" aria-live="polite">
              <strong>{t(`install_stage_${progress.stage}`)}</strong>
              {progress.totalBytes ? (
                <progress
                  value={progress.downloadedBytes}
                  max={progress.totalBytes}
                />
              ) : (
                <progress />
              )}
              <small>
                {progress.downloadedBytes > 0
                  ? `${number(progress.downloadedBytes / 1048576, 1)} / ${progress.totalBytes ? number(progress.totalBytes / 1048576, 1) : "—"} ${t("mib")}`
                  : t("stage_progress_note")}
              </small>
            </Panel>
          )}

          <Panel className="table-panel models-panel">
            <div className="panel-heading padded">
              <div>
                <h2>{t("supported_catalog")}</h2>
                <small>{t("catalog_truth_note")}</small>
              </div>
            </div>
            <div className="table-scroll">
              <table>
                <thead>
                  <tr>
                    <th>{t("model_engine")}</th>
                    <th>{t("status")}</th>
                    <th>{t("integration")}</th>
                  </tr>
                </thead>
                <tbody>
                  {managed?.catalog.map((model) => (
                    <tr key={model.id}>
                      <td>
                        <div className="model-name">
                          <div className="model-icon">
                            <Image />
                          </div>
                          <div>
                            <strong>{model.displayName}</strong>
                            <small>{t(model.descriptionKey)}</small>
                          </div>
                        </div>
                      </td>
                      <td>
                        <Status
                          status={
                            model.availability === "deferred"
                              ? "Planned"
                              : model.installStatus === "installed"
                                ? "Completed"
                                : "Missing"
                          }
                        />
                        <small>
                          {t(
                            model.availability === "deferred"
                              ? "coming_later"
                              : model.installStatus,
                          )}
                        </small>
                      </td>
                      <td>
                        <small>{model.engineCompatibility}</small>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Panel>

          <Readiness details />
          <Panel className="storage-panel">
            <FolderOpen />
            <div>
              <h2>{t("engine_path")}</h2>
              <p>{engineText(e?.availability)}</p>
              <input
                aria-label={t("engine_path_2")}
                value={e?.location || e?.installDir || t("not_checked")}
                readOnly
              />
              <small>{t("engine_path_managed_manual_note")}</small>
            </div>
          </Panel>
        </div>
        <aside className="settings-column">
          <Panel className="hardware-panel">
            <div className="panel-heading">
              <h2>
                <Cpu />
                {t("gpu_information")}
              </h2>
            </div>
            <div className="hardware-device">
              <Cpu size={32} />
              <div>
                <strong>{managed?.gpu.names.join(", ") || t("no_gpu")}</strong>
                <small>{t("detected_through_the_vulkan_loader")}</small>
              </div>
            </div>
            <dl>
              <div>
                <dt>{t("vulkan_devices")}</dt>
                <dd>{managed?.gpu.deviceCount ?? 0}</dd>
              </div>
              <div>
                <dt>{t("gpu_vendor")}</dt>
                <dd>{managed?.gpu.vendors.join(", ") || t("unknown")}</dd>
              </div>
              <div>
                <dt>{t("dedicated_vram")}</dt>
                <dd>
                  {managed?.gpu.dedicatedMemoryBytes
                    ? `${number(managed.gpu.dedicatedMemoryBytes / 1073741824, 1)} GB`
                    : t("not_measured")}
                </dd>
              </div>
              <div>
                <dt>{t("hardware_tier")}</dt>
                <dd>
                  {managed ? t(`tier_${managed.gpu.tier}`) : t("checking")}
                </dd>
              </div>
            </dl>
          </Panel>
          <Panel className="info-panel">
            <ShieldCheck />
            <div>
              <h3>{t("safe_managed_install")}</h3>
              <p>{t("safe_install_note")}</p>
            </div>
          </Panel>
          <Panel className="info-panel">
            <FolderOpen />
            <div>
              <h3>{t("manual_option")}</h3>
              <p>{t("manual_option_description")}</p>
            </div>
          </Panel>
        </aside>
      </div>
      {guide && (
        <div
          className="modal-backdrop"
          role="presentation"
          onMouseDown={() => setGuide(false)}
        >
          <section
            className="panel engine-guide"
            role="dialog"
            aria-modal="true"
            aria-labelledby="engine-guide-title"
            onMouseDown={(event) => event.stopPropagation()}
          >
            <button
              className="guide-close"
              aria-label={t("close")}
              onClick={() => setGuide(false)}
            >
              <X />
            </button>
            <h2 id="engine-guide-title">{t("engine_guide_title")}</h2>
            {[
              "engine_guide_engine",
              "engine_guide_model",
              "engine_guide_managed",
              "engine_guide_manual",
              "engine_guide_sources",
              "engine_guide_recommendation",
              "engine_guide_local",
            ].map((key) => (
              <p key={key}>{t(key)}</p>
            ))}
          </section>
        </div>
      )}
    </>
  );
}
