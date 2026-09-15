import { t, label as display, engineText } from "../../i18n";
import {
  Box,
  Cpu,
  RefreshCw,
  FolderOpen,
  ShieldCheck,
  Image,
  UserRound,
  Type,
  Palette,
  History,
} from "lucide-react";
import { useProcessing } from "../../stores/ProcessingContext";
import { Readiness } from "../../components/Readiness";
import { Button, PageHeading, Panel, Status } from "../../components/ui";
const future = [
  [UserRound, "portrait_model"],
  [Type, "text_model"],
  [Palette, "anime_model"],
  [History, "restore_model"],
] as const;
export function Models() {
  const { engine: e, refresh, refreshing } = useProcessing();
  return (
    <>
      <PageHeading
        icon={<Box />}
        title={t("ai_models_engine_management")}
        subtitle={t("local_intelligence_exceptional_detail_complete_control")}
      />
      <div className="workspace-layout">
        <div className="workspace-column">
          <Readiness details />
          <Panel className="table-panel models-panel">
            <div className="panel-heading padded">
              <div>
                <h2>{t("available_models")}</h2>
                <small>
                  {t("installed_engine_discovery_and_integrity_checks")}
                </small>
              </div>
              <Button disabled={refreshing} onClick={() => void refresh()}>
                <RefreshCw size={15} />
                {refreshing ? t("checking") : t("refresh")}
              </Button>
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
                  <tr>
                    <td>
                      <div className="model-name">
                        <div className="model-icon">
                          <Image />
                        </div>
                        <div>
                          <strong>Real-ESRGAN Photo</strong>
                          <small>realesrgan-x4plus · 2x / 4x / 8x / 12x</small>
                        </div>
                      </div>
                    </td>
                    <td>
                      <Status status={e?.availability || "Checking"} />
                    </td>
                    <td>
                      <small>NCNN Vulkan</small>
                    </td>
                  </tr>
                  {future.map(([Icon, name]) => (
                    <tr key={name}>
                      <td>
                        <div className="model-name">
                          <div className="model-icon">
                            <Icon />
                          </div>
                          <strong>{t(name)}</strong>
                        </div>
                      </td>
                      <td>
                        <Status status="Planned" />
                      </td>
                      <td>
                        <small>{t("future_adapter")}</small>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Panel>
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
              <small>
                {t("source")}{" "}
                {e?.source ? display(e.source) : t("not_installed")}{" "}
                {t("version")}{" "}
                {e?.version && /^v?\d/.test(e.version)
                  ? e.version
                  : t("version_unknown")}
              </small>
              <p>
                {t(
                  "use_install_local_engine_above_for_first_run_setup_sha_256_checks_detect_ac",
                )}
              </p>
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
                <strong>{e?.gpuDevices.join(", ") || t("no_gpu")}</strong>
                <small>{t("detected_through_the_vulkan_loader")}</small>
              </div>
            </div>
            <dl>
              {[
                [t("selection"), t("auto")],
                [t("vulkan_devices"), String(e?.gpuDevices.length || 0)],
                [t("vram_driver_version"), t("not_measured")],
                [
                  t("engine"),
                  e?.availability ? display(e.availability) : t("not_checked"),
                ],
              ].map(([k, v]) => (
                <div key={k}>
                  <dt>{k}</dt>
                  <dd>{v}</dd>
                </div>
              ))}
            </dl>
          </Panel>
          <Panel className="info-panel">
            <ShieldCheck />
            <div>
              <h3>{t("private_by_design")}</h3>
              <p>
                {t(
                  "images_are_processed_on_your_hardware_no_account_uploads_or_cloud_processin",
                )}
              </p>
            </div>
          </Panel>
        </aside>
      </div>
    </>
  );
}
