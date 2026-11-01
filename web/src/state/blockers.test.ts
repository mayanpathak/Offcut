// Why an action cannot start (TS §12.5; v5implementation §12.1, §17.3): for each
// status of the two machines, the order of the codes, and the room on the device
// to the byte.

import { beforeEach, describe, expect, it } from "vitest";

import { BLOCKER_CODES, forExport, forImport } from "./blockers";
import { useCapabilityStore } from "./capability-store";
import { useClipStore } from "./clip-store";
import { useExportStore } from "./export-store";
import type { ClipStatus } from "./machines/clip-machine";
import type { ExportStatus } from "./machines/export-machine";

const CLIP_STATES: readonly ClipStatus[] = ["idle", "importing", "accepted", "processing", "ready", "updating", "rejected", "failed"];
const EXPORT_STATES: readonly ExportStatus[] = ["idle", "gating", "rendering", "muxing", "saving", "done", "cancelled", "failed"];
const RUNNING: readonly ExportStatus[] = ["gating", "rendering", "muxing", "saving"];
const BUSY: readonly ClipStatus[] = ["importing", "processing", "updating"];

function given(clip: ClipStatus, exported: ExportStatus): void {
  useClipStore.setState({ status: clip });
  useExportStore.setState({ status: exported });
}

const NEEDED = 1000;
const ENOUGH = 1200;

beforeEach(() => {
  useCapabilityStore.setState({ status: "supported" });
  given("ready", "idle");
});

describe("forExport", () => {
  it.each(EXPORT_STATES)("export %s", (status) => {
    given("ready", status);
    expect(forExport()).toBe(RUNNING.includes(status) ? "B_EXPORT_IN_PROGRESS" : null);
  });

  it.each(CLIP_STATES)("clip %s", (status) => {
    given(status, "idle");
    expect(forExport()).toBe(status === "ready" ? null : "B_PIPELINE_NOT_READY");
  });

  it("says a running export before a clip that is not ready, and that before the room", () => {
    given("processing", "rendering");
    expect(forExport()).toBe("B_EXPORT_IN_PROGRESS");
    given("processing", "idle");
    expect(forExport({ freeBytes: 0, neededBytes: NEEDED })).toBe("B_PIPELINE_NOT_READY");
  });

  it("has room at exactly the margin, and none one byte under", () => {
    expect(forExport({ freeBytes: ENOUGH, neededBytes: NEEDED })).toBeNull();
    expect(forExport({ freeBytes: ENOUGH - 1, neededBytes: NEEDED })).toBe("B_STORAGE_LOW");
  });

  it("without room asks nothing of the device", () => {
    expect(forExport()).toBeNull();
  });

  it("with room in gating, the gate is this export's own", () => {
    given("ready", "gating");
    expect(forExport()).toBe("B_EXPORT_IN_PROGRESS");
    expect(forExport({ freeBytes: ENOUGH, neededBytes: NEEDED })).toBeNull();
    expect(forExport({ freeBytes: 0, neededBytes: NEEDED })).toBe("B_STORAGE_LOW");
  });

  it.each<ExportStatus>(["rendering", "muxing", "saving"])("with room in %s, another export runs", (status) => {
    given("ready", status);
    expect(forExport({ freeBytes: ENOUGH, neededBytes: NEEDED })).toBe("B_EXPORT_IN_PROGRESS");
  });

  it("changes nothing", () => {
    const clip = useClipStore.getState();
    const exported = useExportStore.getState();
    forExport({ freeBytes: 0, neededBytes: NEEDED });
    expect(useClipStore.getState()).toBe(clip);
    expect(useExportStore.getState()).toBe(exported);
  });
});

describe("forImport", () => {
  it("an unsupported browser comes first", () => {
    useCapabilityStore.setState({ status: "unsupported" } as never);
    given("processing", "rendering");
    expect(forImport()).toBe("B_UNSUPPORTED");
  });

  it.each(CLIP_STATES)("clip %s", (status) => {
    given(status, "idle");
    expect(forImport()).toBe(BUSY.includes(status) ? "B_PIPELINE_BUSY" : null);
  });

  it.each(EXPORT_STATES)("export %s", (status) => {
    given("ready", status);
    expect(forImport()).toBe(RUNNING.includes(status) ? "B_EXPORT_IN_PROGRESS" : null);
  });
});

describe("BLOCKER_CODES", () => {
  it("holds the eight of TS §12.5, in its order", () => {
    expect(BLOCKER_CODES).toEqual([
      "B_UNSUPPORTED",
      "B_PIPELINE_BUSY",
      "B_EXPORT_IN_PROGRESS",
      "B_PIPELINE_NOT_READY",
      "B_STORAGE_LOW",
      "B_NOT_SIGNED_IN",
      "B_ENTITLEMENT_EXPIRED",
      "B_NO_FREE_EXPORTS",
    ]);
  });
});
