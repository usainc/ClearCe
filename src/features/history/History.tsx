import { t, label as display, errorText, number, date } from "../../i18n";
import { useState } from "react";
import {
  Clock3,
  FileImage,
  CheckCircle2,
  Database,
  FolderOpen,
  Trash2,
  Search,
} from "lucide-react";
import { useProcessing } from "../../stores/ProcessingContext";
import { useApp } from "../../stores/AppContext";
import { inspectImage, imageItem, message } from "../../lib/processing";
import { terminal, type Task } from "../../lib/processing";
import { formatBytes } from "../../lib/demo";
import { Button, Empty, PageHeading, Panel, Status } from "../../components/ui";
const duration = (t: Task) =>
  Math.max(
    0,
    ((t.completedAt || t.createdAt) - (t.startedAt || t.createdAt)) / 1000,
  );
export function History() {
  const p = useProcessing();
  const app = useApp();
  async function rerun(t: Task) {
    try {
      const info = await inspectImage(t.request.inputPath);
      app.setSelected(imageItem(info));
      app.setSettings({
        ...app.settings,
        scale: t.request.scale as 2 | 4 | 8 | 12,
        format: t.request.format,
        outputFolder: t.request.outputDir || "",
        gpuId: t.request.gpuId,
        tileSize: t.request.tileSize,
      });
      app.setPage("Home");
    } catch (e) {
      app.notify(message(e));
    }
  }
  const [id, setId] = useState("");
  const [filter, setFilter] = useState("all");
  const [query, setQuery] = useState("");
  const selected = p.jobs.find((j) => j.id === id) || p.jobs[0];
  const completed = p.jobs.filter((j) => j.status === "completed");
  const shown = p.jobs.filter(
    (j) =>
      (filter === "all" || j.status === filter) &&
      `${j.input?.name || ""} ${j.request.inputPath}`
        .toLowerCase()
        .includes(query.toLowerCase()),
  );
  return (
    <>
      <PageHeading
        icon={<Clock3 />}
        title={t("history")}
        subtitle={t("your_actual_enhancement_jobs_saved_on_this_device")}
      />
      <div className="history-stats">
        {[
          [FileImage, t("total_jobs"), String(p.jobs.length)],
          [CheckCircle2, t("completed"), String(completed.length)],
          [
            Clock3,
            t("average_time"),
            t("seconds", {
              value: number(
                completed.length
                  ? completed.reduce((s, j) => s + duration(j), 0) /
                      completed.length
                  : 0,
                1,
              ),
            }),
          ],
          [
            Database,
            t("output_size"),
            formatBytes(
              completed.reduce((s, j) => s + (j.output?.bytes || 0), 0),
            ),
          ],
        ].map(([Icon, label, value]) => {
          const I = Icon as typeof Clock3;
          return (
            <Panel key={String(label)}>
              <I />
              <div>
                <small>{String(label)}</small>
                <strong>{String(value)}</strong>
                <small>{t("recorded_jobs_up_to_500_shown")}</small>
              </div>
            </Panel>
          );
        })}
      </div>
      <div className="toolbar">
        {["all", "completed", "failed", "cancelled"].map((f) => (
          <Button key={f} primary={filter === f} onClick={() => setFilter(f)}>
            {display(f)}
          </Button>
        ))}
        <div className="search-field">
          <Search size={16} />
          <input
            aria-label={t("search_history")}
            placeholder={t("search_files_or_paths")}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
      </div>
      <div className="workspace-layout">
        <Panel className="table-panel">
          <div className="table-scroll">
            <table>
              <thead>
                <tr>
                  <th>{t("file_name_2")}</th>
                  <th>{t("date_time")}</th>
                  <th>{t("scale")}</th>
                  <th>{t("format")}</th>
                  <th>{t("duration")}</th>
                  <th>{t("status")}</th>
                </tr>
              </thead>
              <tbody>
                {shown.map((j) => (
                  <tr
                    key={j.id}
                    className={selected?.id === j.id ? "selected-row" : ""}
                    onClick={() => setId(j.id)}
                  >
                    <td>
                      <button
                        className="text-button"
                        onClick={() => setId(j.id)}
                      >
                        {j.input?.name ||
                          j.request.inputPath.split(/[\\/]/).pop()}
                      </button>
                    </td>
                    <td>{date(j.createdAt)}</td>
                    <td>
                      {j.request.scale}
                      {t("x_2")}
                    </td>
                    <td>{j.request.format}</td>
                    <td>
                      {terminal(j)
                        ? t("seconds", { value: number(duration(j), 1) })
                        : "—"}
                    </td>
                    <td>
                      <Status status={j.status} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {!shown.length && (
            <Empty title={t("no_recorded_jobs")}>
              <p>{t("enhance_a_local_image_to_start_your_history")}</p>
            </Empty>
          )}
        </Panel>
        <aside className="settings-column">
          <Panel className="hardware-panel">
            <h2>{t("job_details")}</h2>
            {selected ? (
              <>
                <p className="file-path">
                  {selected.input?.name || selected.request.inputPath}
                </p>
                <dl>
                  {[
                    [t("mode"), display(selected.request.mode)],
                    [t("engine"), selected.engineId || t("unknown")],
                    [t("model"), selected.model || "—"],
                    [
                      t("original"),
                      selected.input
                        ? `${selected.input.width} × ${selected.input.height}`
                        : "—",
                    ],
                    [
                      t("output"),
                      selected.output
                        ? `${selected.output.width} × ${selected.output.height}`
                        : "—",
                    ],
                    [t("status"), display(selected.status)],
                  ].map(([k, v]) => (
                    <div key={k}>
                      <dt>{k}</dt>
                      <dd>{v}</dd>
                    </div>
                  ))}
                </dl>
                <p className="file-path">
                  <small>{selected.output?.path}</small>
                </p>
                {selected.error && (
                  <p role="status">{errorText(selected.error)}</p>
                )}
                {selected.warning && <p>{t("warning_metadata")}</p>}
                <div className="processing-actions">
                  <Button
                    disabled={p.busy}
                    onClick={() => void rerun(selected)}
                  >
                    {t("re_run_settings")}
                  </Button>
                  <Button
                    primary
                    disabled={!selected.output}
                    onClick={() => void p.compare(selected)}
                  >
                    {t("compare_actual_result")}
                  </Button>
                  <Button
                    disabled={!selected.output}
                    onClick={() => void p.open(selected.id, true)}
                  >
                    <FolderOpen size={16} />
                    {t("open_folder")}
                  </Button>
                  <Button
                    disabled={!selected.output}
                    onClick={() => void p.open(selected.id)}
                  >
                    {t("open_output")}
                  </Button>
                  <Button
                    disabled={!terminal(selected)}
                    onClick={() => void p.remove(selected.id)}
                  >
                    <Trash2 size={16} />
                    {t("remove_record")}
                  </Button>
                </div>
                <small>{t("removing_a_record_keeps_your_output_file")}</small>
              </>
            ) : (
              <p>{t("select_a_job_to_view_its_details")}</p>
            )}
          </Panel>
        </aside>
      </div>
    </>
  );
}
