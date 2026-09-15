import { t } from "../../i18n";
import { FolderOpen, ScanSearch } from "lucide-react";
import { useApp } from "../../stores/AppContext";
import { useImport } from "../../hooks/useImport";
import { Compare } from "../../components/Compare";
import { EnhancementSettings } from "../../components/EnhancementSettings";
import { Button, Panel, PrivacyPanel } from "../../components/ui";
import { formatBytes } from "../../lib/demo";
export function Preview() {
  const { selected } = useApp();
  const { picker, open } = useImport();
  return (
    <div className="workspace-layout">
      <div className="workspace-column">
        {picker}
        <Panel className="preview-main">
          <div className="panel-heading">
            <div>
              <h2>{selected?.name || t("preview")}</h2>
              <small>
                {selected
                  ? `${selected.width} × ${selected.height}  ·  ${formatBytes(selected.bytes)}`
                  : t("open_an_image_to_get_started")}
              </small>
            </div>
            <Button onClick={open}>
              <FolderOpen size={17} />
              {t("open_image")}
            </Button>
          </div>
          <Compare image={selected} detail />
        </Panel>
        <Panel className="detail-panel">
          <div className="panel-heading">
            <h2>
              <ScanSearch size={17} />
              {t("detail_comparison")}
            </h2>
            <small>
              {selected?.enhancedSrc
                ? t("actual_original_and_generated_output")
                : t("no_enhanced_result_selected")}
            </small>
          </div>
          <div className="detail-grid">
            {[t("original"), t("sample_detail")].map((label, i) => (
              <div className="detail-crop" key={label}>
                {selected && (
                  <img
                    src={
                      i === 1 && selected.enhancedSrc
                        ? selected.enhancedSrc
                        : selected.src
                    }
                    alt={label}
                    className={i === 0 && selected.demo ? "demo-soft" : ""}
                  />
                )}
                <span className="image-label left">
                  {i === 1 && selected?.enhancedSrc
                    ? t("enhanced")
                    : !selected?.demo && i === 1
                      ? t("unprocessed")
                      : label}
                </span>
              </div>
            ))}
          </div>
        </Panel>
      </div>
      <aside className="settings-column">
        <PrivacyPanel />
        <EnhancementSettings preview />
        <Panel className="info-panel">
          <ScanSearch />
          <div>
            <h3>{t("preview_before_you_commit")}</h3>
            <p>
              {t(
                "compare_an_enhanced_result_from_home_or_history_independent_preview_patch_g",
              )}
            </p>
          </div>
        </Panel>
      </aside>
    </div>
  );
}
