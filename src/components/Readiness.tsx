import { t, number } from "../i18n";
import { useEffect, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useApp } from "../stores/AppContext";
import { useProcessing } from "../stores/ProcessingContext";
import { message } from "../lib/processing";
import { Button, Panel } from "./ui";
export interface Health {
  state: string;
  engineState: string;
  checks: { code: string; name: string; ok: boolean; message: string }[];
  files: [string, number][];
  cacheBytes: number | null;
}
export function Readiness({ details = false }: { details?: boolean }) {
  const { savedSettings, notify } = useApp();
  const { refresh, busy } = useProcessing();
  const [health, setHealth] = useState<Health | null>(null);
  const [checking, setChecking] = useState(false);
  async function verify() {
    if (!isTauri()) return;
    setChecking(true);
    try {
      const result = await invoke<Health>("engine_health", {
        outputDir: savedSettings.outputFolder || null,
        gpuId: savedSettings.gpuId,
        engineMode: savedSettings.engineMode,
      });
      setHealth(result);
      return result;
    } catch (e) {
      notify(message(e));
    } finally {
      setChecking(false);
    }
  }
  useEffect(() => {
    void verify();
  }, [
    savedSettings.outputFolder,
    savedSettings.gpuId,
    savedSettings.engineMode,
  ]);
  async function install() {
    try {
      const source = await open({
        directory: true,
        multiple: false,
        title: t("setup_folder"),
      });
      if (!source) return;
      setChecking(true);
      await invoke("install_local_engine", { source });
      await refresh();
      const result = await verify();
      notify(
        result?.state === "ready" ? t("setup_ready") : t("setup_attention"),
      );
    } catch (e) {
      notify(message(e));
    } finally {
      setChecking(false);
    }
  }
  return (
    <Panel className="readiness-panel">
      <div className="panel-heading">
        <h2>
          {checking
            ? t("checking_local_readiness")
            : health?.state === "ready"
              ? t("local_ai_ready")
              : t("setup_requires_attention")}
        </h2>
        <Button disabled={checking || busy} onClick={() => void verify()}>
          {t("verify_readiness")}
        </Button>
      </div>
      {(details || health?.state !== "ready") && (
        <>
          {health?.engineState === "missing" && (
            <p>
              {t("setup_success")} {t("setup_intro")}
            </p>
          )}
          <dl>
            {health?.checks.map((c) => (
              <div key={t("check_" + c.code)}>
                <dt>
                  {c.ok ? "✓" : "!"} {t("check_" + c.code)}
                </dt>
                <dd>{t(c.ok ? "check_ok" : "check_" + c.code + "_fail")}</dd>
              </div>
            ))}
          </dl>
          <p>{t("setup_expected")}</p>
          <Button disabled={checking || busy} onClick={() => void install()}>
            {t("setup_select")}
          </Button>
        </>
      )}
      {details && health && (
        <ul>
          {health.files.map(([name, bytes]) => (
            <li key={name}>
              {name} · {number(bytes / 1048576, 2)} {t("mib")}
            </li>
          ))}
        </ul>
      )}
    </Panel>
  );
}
