import { expect, it } from "vitest";
import { reconcileTask, type Task } from "./processing";
const task = (id: string, createdAt: number, status: Task["status"]) =>
  ({ id, createdAt, status }) as Task;
it("late polling cannot replace a newer enhancement", () => {
  const current = task("new", 20, "processing");
  expect(reconcileTask(current, task("old", 10, "completed"))).toBe(current);
});
it("a queued command response cannot undo a completed event", () => {
  const done = task("a", 10, "completed");
  expect(reconcileTask(done, task("a", 10, "queued"))).toBe(done);
  expect(reconcileTask(done, task("b", 20, "queued")).id).toBe("b");
});
it("a previously queued job takes focus when it actually starts", () => {
  const current = { ...task("single", 20, "completed"), startedAt: 21 };
  const queued = { ...task("batch", 10, "processing"), startedAt: 30 };
  expect(reconcileTask(current, queued)).toBe(queued);
  expect(reconcileTask(queued, current)).toBe(queued);
});
