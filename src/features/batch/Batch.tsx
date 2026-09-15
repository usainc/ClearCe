import { t, errorText, phaseText } from "../../i18n";
import { useState } from "react";
import {
  Layers,
  Plus,
  FolderOpen,
  Play,
  Pause,
  Trash2,
  RotateCcw,
  X,
  FileImage,
} from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useQueue } from "../../stores/QueueContext";
import { useProcessing } from "../../stores/ProcessingContext";
import { useApp } from "../../stores/AppContext";
import { useImport } from "../../hooks/useImport";
import { isTauri, message, terminal } from "../../lib/processing";
import { EnhancementSettings } from "../../components/EnhancementSettings";
import { ProcessingPanel } from "../../components/ProcessingPanel";
import { Button, Empty, PageHeading, Panel, Status } from "../../components/ui";
export function Batch() {
  const { queue, pending, folder, control } = useQueue();
  const processing = useProcessing();
  const { notify } = useApp();
  const { picker, open } = useImport(true);
  const [page, setPage] = useState(0);
  const [selectedId, setSelectedId] = useState("");
  const waiting = queue.items.filter((i) => i.task.status === "queued").length;
  const completed = queue.items.filter(
    (i) => i.task.status === "completed",
  ).length;
  const finished = queue.items.filter((i) => terminal(i.task)).length;
  const pageCount = Math.max(1, Math.ceil(queue.items.length / 25));
  const currentPage = Math.min(page, pageCount - 1);
  const selected = queue.items.find((i) => i.id === selectedId);
  async function addFolder() {
    try {
      const path = await openDialog({ directory: true, multiple: false });
      if (typeof path === "string") await folder(path);
    } catch (e) {
      notify(message(e));
    }
  }
  return (
    <>
      <PageHeading
        icon={<Layers />}
        title={t("batch_queue")}
        subtitle={t(
          "local_enhancement_one_image_at_a_time_your_queue_is_saved_on_this_device",
        )}
      />
      <div className="workspace-layout batch-layout">
        <div className="workspace-column">
          {picker}
          <div className="toolbar">
            <Button primary disabled={pending || !isTauri()} onClick={open}>
              <Plus size={17} />
              {t("add_files")}
            </Button>
            <Button
              disabled={pending || !isTauri()}
              onClick={() => void addFolder()}
            >
              <FolderOpen size={17} />
              {t("add_folder")}
            </Button>
            <Button
              disabled={pending || !queue.items.length}
              onClick={() => void control("clear")}
            >
              <Trash2 size={16} />
              {t("clear_inactive")}
            </Button>
          </div>
          <div className="toolbar">
            <Button
              primary
              disabled={pending || queue.running || !waiting}
              onClick={() => void control("start")}
            >
              <Play size={16} />
              {t("start_processing")}
            </Button>
            <Button
              disabled={pending || !queue.running}
              onClick={() => void control("pause")}
            >
              <Pause size={16} />
              {t("pause_after_current")}
            </Button>
            <small>
              {queue.running
                ? t("running_one_gpu_job_at_a_time")
                : queue.activeId
                  ? t("pausing_after_current_job")
                  : t("queue_stopped")}
            </small>
          </div>
          {queue.error && <p role="alert">{t("queue_storage_error")}</p>}
          <Panel className="table-panel queue-panel">
            <div className="table-scroll">
              <table>
                <thead>
                  <tr>
                    <th>{t("file_name")}</th>
                    <th>{t("size_scale")}</th>
                    <th>{t("status_stage")}</th>
                    <th>{t("format")}</th>
                    <th>{t("actions")}</th>
                  </tr>
                </thead>
                <tbody>
                  {queue.items
                    .slice(currentPage * 25, currentPage * 25 + 25)
                    .map((i) => (
                      <tr
                        key={i.id}
                        className={selectedId === i.id ? "selected-row" : ""}
                      >
                        <td>
                          <button
                            className="text-button"
                            onClick={() => setSelectedId(i.id)}
                          >
                            <FileImage size={16} />
                            {i.task.input?.name ||
                              i.task.request.inputPath.split(/[\\/]/).pop()}
                          </button>
                          <small>
                            {t("attempt")}
                            {i.retryCount + 1}
                          </small>
                        </td>
                        <td>
                          {i.task.input
                            ? `${i.task.input.width} × ${i.task.input.height}`
                            : "—"}
                          <small>
                            {i.task.request.scale}
                            {t("x_2")}
                          </small>
                        </td>
                        <td>
                          <Status
                            status={
                              i.task.status === "queued"
                                ? "Waiting"
                                : i.task.status
                            }
                          />
                          <small>{phaseText(i.task)}</small>
                        </td>
                        <td>{i.task.request.format}</td>
                        <td>
                          <div className="row-actions">
                            {i.task.output && (
                              <button
                                aria-label={t("compare_named", {
                                  name: i.task.input?.name,
                                })}
                                onClick={() => void processing.compare(i.task)}
                              >
                                <FolderOpen size={16} />
                              </button>
                            )}
                            {["failed", "cancelled"].includes(
                              i.task.status,
                            ) && (
                              <button
                                disabled={pending || queue.activeId === i.id}
                                aria-label={t("retry_named", {
                                  name: i.task.input?.name,
                                })}
                                onClick={() => void control("retry", i.id)}
                              >
                                <RotateCcw size={16} />
                              </button>
                            )}
                            {!terminal(i.task) && (
                              <button
                                disabled={pending || i.cancelPending}
                                aria-label={t("cancel_named", {
                                  name: i.task.input?.name,
                                })}
                                onClick={() => void control("cancel", i.id)}
                              >
                                <X size={16} />
                              </button>
                            )}
                            {queue.activeId !== i.id && (
                              <button
                                disabled={pending}
                                aria-label={t("remove_named", {
                                  name: i.task.input?.name,
                                })}
                                onClick={() => void control("remove", i.id)}
                              >
                                <Trash2 size={16} />
                              </button>
                            )}
                          </div>
                        </td>
                      </tr>
                    ))}
                </tbody>
              </table>
            </div>
            {!queue.items.length && (
              <Empty title={t("your_batch_starts_here")}>
                <p>
                  {t("add_files_or_a_folder_folder_imports_are_non_recursive")}
                </p>
              </Empty>
            )}
            <div className="table-footer">
              <span>
                {queue.items.length} {t("200_images_persisted_locally")}
              </span>
              <Button
                disabled={!currentPage}
                onClick={() => setPage(currentPage - 1)}
              >
                {t("previous")}
              </Button>
              <span>
                {currentPage + 1} / {pageCount}
              </span>
              <Button
                disabled={currentPage + 1 >= pageCount}
                onClick={() => setPage(currentPage + 1)}
              >
                {t("next")}
              </Button>
            </div>
          </Panel>
          {selected && (
            <Panel className="processing-panel">
              <h2>{selected.task.input?.name}</h2>
              <p>
                {selected.task.error
                  ? errorText(selected.task.error)
                  : phaseText(selected.task)}
              </p>
              <small className="file-path">
                {selected.task.output?.path || selected.task.request.inputPath}
              </small>
              {selected.task.output && (
                <div className="processing-actions">
                  <Button
                    primary
                    onClick={() => void processing.compare(selected.task)}
                  >
                    {t("compare_actual_result")}
                  </Button>
                  <Button
                    onClick={() => void processing.open(selected.task.id, true)}
                  >
                    {t("open_output_folder")}
                  </Button>
                </div>
              )}
            </Panel>
          )}
          <ProcessingPanel />
          <Panel className="overall-progress">
            <div className="panel-heading">
              <h2>{t("finished_jobs")}</h2>
              <span>
                {finished} / {queue.items.length}
              </span>
            </div>
            <div className="progress-track">
              <i
                style={{
                  width: `${queue.items.length ? (finished / queue.items.length) * 100 : 0}%`,
                }}
              />
            </div>
            <small>
              {t(
                "counts_finished_jobs_including_failed_or_cancelled_items_inference_progress",
              )}
            </small>
          </Panel>
        </div>
        <aside className="settings-column">
          <EnhancementSettings batch />
          <Panel className="summary-panel">
            <h2>{t("queue_summary")}</h2>
            <div className="summary-grid">
              {[
                [t("total"), queue.items.length],
                [t("completed"), completed],
                [t("active"), queue.activeId ? 1 : 0],
                [t("waiting"), waiting],
              ].map(([label, value]) => (
                <div key={label}>
                  <small>{label}</small>
                  <strong>{value}</strong>
                </div>
              ))}
            </div>
            <p>
              {t(
                "outputs_always_get_a_new_filename_original_and_existing_files_are_preserved",
              )}
            </p>
          </Panel>
        </aside>
      </div>
    </>
  );
}
