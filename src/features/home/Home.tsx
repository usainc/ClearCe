import { t } from "../../i18n";
import { useState } from "react";
import { Upload, ImagePlus, ArrowUpRight } from "lucide-react";
import { useApp } from "../../stores/AppContext";
import { useImport } from "../../hooks/useImport";
import { Compare } from "../../components/Compare";
import { EnhancementSettings } from "../../components/EnhancementSettings";
import { Panel, PrivacyPanel } from "../../components/ui";
import { formatBytes } from "../../lib/demo";
import { ProcessingPanel } from "../../components/ProcessingPanel";
export function Home() {
  const { selected, images, setSelected, setImages, setPage } = useApp();
  const { picker, open, importFiles } = useImport();
  const [drag, setDrag] = useState(false);
  return (
    <div className="workspace-layout">
      <div className="workspace-column">
        {picker}
        <button
          className={`dropzone panel ${drag ? "dragging" : ""}`}
          onClick={open}
          onDragOver={(e) => {
            e.preventDefault();
            setDrag(true);
          }}
          onDragLeave={() => setDrag(false)}
          onDrop={(e) => {
            e.preventDefault();
            setDrag(false);
            void importFiles(e.dataTransfer.files);
          }}
        >
          <Upload size={31} />
          <strong>{t("drag_drop_an_image_here")}</strong>
          <span>
            {t("or")} <em>{t("browse_files")}</em>
          </span>
          <small>
            {t("jpg_png_webp")}
            <i /> {t("up_to_100_mb_6_megapixels")}
          </small>
        </button>
        <ProcessingPanel />
        <div className="image-workspace">
          <Compare image={selected} />
          {selected && (
            <div className="file-bar">
              <span>
                <ImagePlus size={15} />
                <strong>{selected.name}</strong>
                <small>
                  {selected.width} × {selected.height} <i />{" "}
                  {formatBytes(selected.bytes)}
                </small>
              </span>
              <button
                className="text-button"
                onClick={() => setPage("Preview")}
              >
                {t("open_preview")}
                <ArrowUpRight size={14} />
              </button>
            </div>
          )}
        </div>
        <Panel className="recent-panel">
          <div className="panel-heading">
            <h2>
              {t("recent_files")}
              <span className="count">{images.length}</span>
            </h2>
            <button
              className="text-button"
              onClick={() => setImages([])}
              disabled={!images.length}
            >
              {t("clear")}
            </button>
          </div>
          <div className="recent-strip">
            {images.map((image) => (
              <button
                key={image.id}
                className={`recent-image ${selected?.id === image.id ? "active" : ""}`}
                onClick={() => setSelected(image)}
                title={image.name}
              >
                <img src={image.src} alt={image.name} />
                {image.demo && <span>{t("sample")}</span>}
              </button>
            ))}
            <button className="add-image" onClick={open}>
              <ImagePlus size={23} />
              <span>{t("add_images")}</span>
            </button>
          </div>
        </Panel>
      </div>
      <aside className="settings-column">
        <PrivacyPanel />
        <EnhancementSettings />
      </aside>
    </div>
  );
}
