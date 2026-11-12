// The page's development hooks (v4implementation D-155, D-191): ways for the
// screen captures and the end-to-end tests to reach states whose flows do not
// exist yet, or that a test cannot cause from outside. `EditorPage` installs
// them only when `DEV_HOOKS` holds, so a production build has none: neither
// this module nor the name `__offcutDev` reaches `web/dist` (checked by
// Prompt 124). `crash`, `failNextSetScene` and `hangWorker` cause the
// failures of G 14.5 on purpose; their worker halves are in
// `workers/rpc-worker.ts`, behind the same condition.

import { type FeatureFlags, FEATURES } from "../config/features";
import type { PlanOutcome } from "../gen/api";
import type { EditPlan, ErrorCode, EventKind, PlanItem } from "../gen/domain";
import { useClipStore } from "../state/clip-store";
import { useExportStore } from "../state/export-store";
import { useUiStore } from "../state/ui-store";
import { crashWorker, failEncoderAt, failNextSetScene, hangWorker } from "../usecases/dev-faults";
import { wordAt } from "../usecases/replace-all";

type WorkerKind = Parameters<typeof crashWorker>[0];

/** Frames an export of a fake clip "renders"; only their ratio is shown. */
const FAKE_EXPORT_FRAMES = 1000;
/** How long the fake export has been running: its start, as a real one keeps it. */
const FAKE_EXPORT_ELAPSED_MS = 12_000;
/** The fake done state's summary: enough for every row to show, in the plural. */
const FAKE_EMPHASIZED = 3;
const FAKE_VISUALS = 2;

export type DevHooks = {
  /** Opens word `i` of the transcript in its field, on the Captions tab. */
  openWordField(i: number): void;
  /** Ends the clip in `failed` with `code`, from any state, as a failure of the preview. */
  failClip(code: ErrorCode): void;
  /** Shows an export rendering, `fraction` (0 to 1) of the way through. Nothing is rendered. */
  fakeExport(fraction: number): void;
  /** Shows the done state with a summary. Nothing was exported (v5implementation D-235). */
  fakeExportDone(): void;
  /** Shows the upgrade prompt as after a watermarked export, or at the limit (v6implementation §13). Nothing was exported. */
  fakeUpgrade(placement: "after_export" | "limit_reached"): void;
  /** Worker `kind` crashes, as a worker killed from DevTools would (D-153). */
  crash(kind: WorkerKind): void;
  /** The next scene the render worker builds fails: the next review change fails (D-190). */
  failNextSetScene(): void;
  /** Worker `kind` stops answering for good: a cancel restarts it after 2 s (D-152). */
  hangWorker(kind: WorkerKind): void;
  /** The video encoder fails at `frame`, the next `times` attempts (v5implementation D-218). */
  failEncoderAt(frame: number, times: number): void;
  /** V6.2 (P194): the build's feature flags, for a capture that needs one on. */
  flags(): FeatureFlags;
  /**
   * V6.2 (P194): the clip shows a model's plan in the visuals list, one item of
   * each of the five kinds on the transcript's words after its own items, and
   * its text extras. Only the list and the panel: the scene is not rebuilt.
   */
  fakePlan(): void;
  /** V6.2 (P194): the run fell back to the rules plan for `why`; its toast shows. */
  fakePlanFallback(why: Exclude<PlanOutcome, "ok" | "off">): void;
};

/** The five kinds of V6.2, each on the word at that share of the transcript. */
const FAKE_PLAN_KINDS: readonly [EventKind, number][] = [
  ["hook_title", 0],
  ["emphasis", 0.2],
  ["punch_in", 0.4],
  ["emoji", 0.6],
  ["icon_pop", 0.8],
];
const FAKE_EXTRAS = {
  hook: "The one thing nobody tells you",
  post_caption: "Three habits that changed how I edit, in under a minute. Which one would you try first?",
  hashtags: ["#editing", "#creator", "#shorts"],
};

function fakePlan(): void {
  const { plan, transcript } = useClipStore.getState();
  const words = transcript?.words.length ?? 0;
  if (plan === undefined || words === 0) {
    return;
  }
  const items = FAKE_PLAN_KINDS.map(([kind, at], i) => {
    const word = Math.min(words - 1, Math.floor(at * words));
    // The list reads the kind, the id and the words; the params are a stand-in.
    return {
      id: String(9_000 + i),
      kind,
      anchors: { start: word, end: word + 1 },
      params: { kind, params: { word } },
      confidence: null,
    } as unknown as PlanItem;
  });
  const model: EditPlan = {
    ...plan,
    source: { kind: "model", provider: "deepseek", model: "deepseek-flash" },
    items: [...plan.items, ...items],
  };
  useClipStore.setState({ plan: model, planExtras: FAKE_EXTRAS });
}

declare global {
  interface Window {
    __offcutDev?: DevHooks;
  }
}

export function installDevHooks(): void {
  window.__offcutDev = {
    openWordField: (i) => {
      if (i < 0 || i >= (useClipStore.getState().transcript?.words.length ?? 0)) {
        return;
      }
      const ui = useUiStore.getState();
      ui.selectTab("captions");
      ui.editWord(wordAt(i));
    },
    failClip: (code) => {
      // Past the clip machine on purpose: `ready` has no `fail` edge (TS §12.1).
      useClipStore.setState({ status: "failed", failure: { code, stage: "preview", retryable: true } });
    },
    fakeExport: (fraction) => {
      const done = Math.round(Math.min(Math.max(fraction, 0), 1) * FAKE_EXPORT_FRAMES);
      useExportStore.setState({
        status: "rendering",
        done,
        total: FAKE_EXPORT_FRAMES,
        startedAt: Date.now() - FAKE_EXPORT_ELAPSED_MS,
      });
    },
    fakeExportDone: () => {
      useExportStore.setState({
        status: "done",
        summary: { voice_cleaned: true, captions_emphasized: FAKE_EMPHASIZED, visual_moments: FAKE_VISUALS },
        answered: false,
      });
    },
    fakeUpgrade: (placement) => {
      useExportStore.setState(placement === "after_export" ? { upgradeOffer: true, gate: null } : { gate: "upgrade" });
    },
    crash: crashWorker,
    failNextSetScene,
    hangWorker,
    failEncoderAt,
    flags: () => FEATURES,
    fakePlan,
    fakePlanFallback: (why) => {
      useClipStore.setState({ planFallback: why });
    },
  };
}
