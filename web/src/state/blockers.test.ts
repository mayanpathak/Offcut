// Why an action cannot start (TS §12.5; v5implementation §12.1, §17.3): for each
// status of the two machines, the order of the codes, and the room on the device
// to the byte.

import { beforeEach, describe, expect, it } from "vitest";

import { useAuthStore } from "./auth-store";
import { BLOCKER_CODES, disablesExport, forCheckout, forExport, forImport } from "./blockers";
import { useCapabilityStore } from "./capability-store";
import { useClipStore } from "./clip-store";
import { setOutboxFree, setToken, useEntitlementStore } from "./entitlement-store";
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
const SUB = "me";

/** A token as the server signs it, good for a day unless told otherwise. */
function tokenWith(over: Record<string, unknown> = {}): string {
  const now = Math.floor(Date.now() / 1000);
  const claims = { v: 1, sub: SUB, plan: "creator", free_exports_remaining: 3, period_end: now + 86_400, iat: now, exp: now + 86_400, ...over };
  return `${btoa(JSON.stringify(claims))}.signature`;
}

const HOUR_AGO = () => Math.floor(Date.now() / 1000) - 3_600;

beforeEach(() => {
  useCapabilityStore.setState({ status: "supported" });
  given("ready", "idle");
  // What the cases about the clip and the export need out of their way.
  useAuthStore.setState({ status: "anonymous" });
  useEntitlementStore.setState({ outboxFree: {} });
  setToken(tokenWith());
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

describe("forExport: the account and the plan (v6implementation §12)", () => {
  it("signed out with no token: not signed in", () => {
    setToken(null);
    expect(forExport()).toBe("B_NOT_SIGNED_IN");
    useAuthStore.setState({ status: "unknown" });
    expect(forExport()).toBe("B_NOT_SIGNED_IN");
  });

  it("signed out with an unexpired token: an offline export goes on (C-15)", () => {
    expect(forExport()).toBeNull();
    useAuthStore.setState({ status: "anonymous_offline" });
    expect(forExport()).toBeNull();
  });

  it("signed out with only an expired token: not signed in, as nothing could renew it", () => {
    setToken(tokenWith({ exp: HOUR_AGO() }));
    expect(forExport()).toBe("B_NOT_SIGNED_IN");
  });

  it("signed in with an expired token: expired, which the gate can mend", () => {
    useAuthStore.setState({ status: "signed_in" });
    setToken(tokenWith({ exp: HOUR_AGO() }));
    expect(forExport()).toBe("B_ENTITLEMENT_EXPIRED");
  });

  it("signed in with no token: the same, as a new one can be fetched", () => {
    useAuthStore.setState({ status: "signed_in" });
    setToken(null);
    expect(forExport()).toBe("B_ENTITLEMENT_EXPIRED");
  });

  it("Free with none left: no free exports; with one left, none in the way", () => {
    setToken(tokenWith({ plan: "free", free_exports_remaining: 0 }));
    expect(forExport()).toBe("B_NO_FREE_EXPORTS");
    setToken(tokenWith({ plan: "free", free_exports_remaining: 1 }));
    expect(forExport()).toBeNull();
  });

  it("D-267: this account's unsent Free rows count, and another's do not", () => {
    setToken(tokenWith({ plan: "free", free_exports_remaining: 1 }));
    setOutboxFree({ someone_else: 5 });
    expect(forExport()).toBeNull();
    setOutboxFree({ [SUB]: 1 });
    expect(forExport()).toBe("B_NO_FREE_EXPORTS");
  });

  it("Creator with no count left: nothing is in the way", () => {
    setToken(tokenWith({ plan: "creator", free_exports_remaining: 0 }));
    setOutboxFree({ [SUB]: 9 });
    expect(forExport()).toBeNull();
  });

  it("V5's order comes first: a running export and an unready clip before the account", () => {
    setToken(null);
    given("ready", "rendering");
    expect(forExport()).toBe("B_EXPORT_IN_PROGRESS");
    given("processing", "idle");
    expect(forExport()).toBe("B_PIPELINE_NOT_READY");
  });

  it("D-309: the room is asked of an export past the gate, and the account is not asked again", () => {
    setToken(null);
    given("ready", "gating");
    expect(forExport({ freeBytes: ENOUGH, neededBytes: NEEDED })).toBeNull();
  });

  it("D-290: only the two a click could not change disable the button", () => {
    const disabling = BLOCKER_CODES.filter(disablesExport);
    expect(disabling).toEqual(["B_EXPORT_IN_PROGRESS", "B_PIPELINE_NOT_READY"]);
  });
});

describe("forCheckout", () => {
  it("asks for a session: a cached token is none", () => {
    expect(forCheckout()).toBe("B_NOT_SIGNED_IN");
    useAuthStore.setState({ status: "signed_in" });
    expect(forCheckout()).toBeNull();
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
