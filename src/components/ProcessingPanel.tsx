import { t, errorText, phaseText, time } from "../i18n";
import { useEffect, useState } from "react";
import { useProcessing } from "../stores/ProcessingContext";
import { terminal } from "../lib/processing";
import { Button, Panel, Status } from "./ui";
export function ProcessingPanel() {
  const { task, cancel, open, compare } = useProcessing();
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, []);
  if (!task) return null;
  return (
    <Panel className="processing-panel">
      <div className="panel-heading">
        <h2>
          {task.status === "completed"
            ? t("enhancement_complete")
            : t("local_processing")}
        </h2>
        <Status status={task.status} />
      </div>
      <p role="status">
        {task.error ? errorText(task.error) : phaseText(task)}
      </p>
      {!terminal(task) && (
        <progress aria-label={t("local_inference_in_progress")} />
      )}
      <small>
        {task.request.scale}
        {t("x")}
        {task.request.format} ·{" "}
        {Math.max(
          0,
          Math.round(
            ((task.completedAt || now) - (task.startedAt || task.createdAt)) /
              1000,
          ),
        )}{" "}
        {t("seconds_elapsed")} {task.model}
      </small>
      {task.output && (
        <p>
          {task.input?.width} × {task.input?.height} →{" "}
          <strong>
            {task.output.width} × {task.output.height}
          </strong>
          <br />
          <small className="file-path">{task.output.path}</small>
        </p>
      )}
      {task.warning && <p>{t("warning_metadata")}</p>}
      {task.target && !task.output && (
        <p>
          {task.input?.width} × {task.input?.height} →{" "}
          <strong>
            {task.target.width} × {task.target.height}
          </strong>
        </p>
      )}
      {task.selectedGpu && (
        <small>
          {t("gpu")} {task.request.gpuId ? task.selectedGpu : t("auto")}{" "}
          {t("tile_size")} {task.request.tileSize || t("auto")}
        </small>
      )}
      {!!task.stages?.length && (
        <ol className="pipeline-stages">
          {task.stages.map((stage, i) => (
            <li key={i}>
              <strong>
                {t("pipeline_stage", {
                  pass: i + 1,
                  resize:
                    stage.scale === 4
                      ? ""
                      : t("pipeline_resize", { scale: stage.scale }),
                })}
              </strong>
              <Status
                status={stage.status === "queued" ? "Waiting" : stage.status}
              />
            </li>
          ))}
        </ol>
      )}
      {!!task.activity?.length && (
        <details className="activity-log" open={!terminal(task)}>
          <summary>{t("activity_log")}</summary>
          <ul>
            {task.activity.map((event, i) => (
              <li key={i}>
                <time>{time(event.at)}</time>{" "}
                {t("phase_" + (event.code || "recorded"))}
              </li>
            ))}
          </ul>
        </details>
      )}
      <div className="processing-actions">
        {!terminal(task) && (
          <Button onClick={() => void cancel()}>
            {t("cancel_processing")}
          </Button>
        )}
        {task.output && (
          <>
            <Button primary onClick={() => void compare(task)}>
              {t("compare_result")}
            </Button>
            <Button onClick={() => void open(task.id)}>
              {t("open_output")}
            </Button>
            <Button onClick={() => void open(task.id, true)}>
              {t("open_folder")}
            </Button>
          </>
        )}
      </div>
    </Panel>
  );
}
