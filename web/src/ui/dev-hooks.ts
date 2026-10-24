// The page's development hooks (v4implementation D-155, D-191): ways for the
// screen captures and the end-to-end tests to reach states whose flows do not
// exist yet, or that a test cannot cause from outside. `EditorPage` installs
// them only when `DEV_HOOKS` holds, so a production build has none: neither
// this module nor the name `__offcutDev` reaches `web/dist` (checked by
// Prompt 124). Prompts 119 and 124 add `crash`, `failNextSetScene` and
// `hangWorker` here, their worker halves behind the same condition.

import type { ErrorCode } from "../gen/domain";
import { useClipStore } from "../state/clip-store";
import { useExportStore } from "../state/export-store";
import { useUiStore } from "../state/ui-store";
import { wordAt } from "../usecases/replace-all";

/** Frames an export of a fake clip "renders"; only their ratio is shown. */
const FAKE_EXPORT_FRAMES = 1000;
/** How long the fake export has been running, for its "so far" line. */
const FAKE_EXPORT_ELAPSED_MS = 12_000;

export type DevHooks = {
  /** Opens word `i` of the transcript in its field, on the Captions tab. */
  openWordField(i: number): void;
  /** Ends the clip in `failed` with `code`, from any state, as a failure of the preview. */
  failClip(code: ErrorCode): void;
  /** Shows an export rendering, `fraction` (0 to 1) of the way through. Nothing is rendered. */
  fakeExport(fraction: number): void;
};

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
  };
}
