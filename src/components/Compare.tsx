import { t } from "../i18n";
import { useState } from "react";
import { ChevronsLeftRight, ZoomIn } from "lucide-react";
import type { ImageItem } from "../types";
import { Empty } from "./ui";
export function Compare({
  image,
  detail = false,
}: {
  image: ImageItem | null;
  detail?: boolean;
}) {
  const [split, setSplit] = useState(50);
  const [zoom, setZoom] = useState(false);
  if (!image)
    return (
      <Empty title={t("your_workspace_is_ready")}>
        <p>{t("open_an_image_to_explore_the_enhancement_controls")}</p>
      </Empty>
    );
  return (
    <div
      className={`compare ${detail ? "compact" : ""} ${zoom ? "zoomed" : ""}`}
    >
      <img
        src={image.enhancedSrc || image.src}
        alt={image.enhancedSrc ? t("enhanced_result") : image.name}
        className="compare-image"
      />
      <div
        className="before-layer"
        style={{ clipPath: `inset(0 ${100 - split}% 0 0)` }}
      >
        <img
          src={image.src}
          alt={t("original_image")}
          className={`compare-image ${image.demo ? "demo-soft" : ""}`}
        />
      </div>
      <span className="image-label left">{t("original")}</span>
      <span className="image-label right">
        {image.enhancedSrc
          ? t("enhanced")
          : image.demo
            ? t("sample_detail")
            : t("unprocessed")}
      </span>
      <div className="split-line" style={{ left: `${split}%` }}>
        <div className="split-handle">
          <ChevronsLeftRight size={20} />
        </div>
      </div>
      <input
        className="compare-range"
        type="range"
        min="0"
        max="100"
        value={split}
        aria-label={t("before_and_after_comparison")}
        onChange={(e) => setSplit(Number(e.target.value))}
      />
      <span className="sample-label">
        {image.enhancedSrc
          ? t("original_enhanced_preview_full_size_output_preserved")
          : image.demo
            ? t("demo_comparison_simulated_softness")
            : t("original_image_no_processing_applied")}
      </span>
      <button
        className="zoom-button"
        aria-label={t("toggle_detail_zoom")}
        onClick={() => setZoom(!zoom)}
      >
        <ZoomIn size={15} />
        {zoom ? t("fit") : t("zoom")}
      </button>
    </div>
  );
}
