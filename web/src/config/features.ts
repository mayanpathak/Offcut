// The release's feature flags (TS §41.3.1). Built by the cloud transcription
// release with one flag; V6.1 adds its own here. A flag comes from a
// build-time define that `vite.config.ts` sets, never from the network. Every
// flag is on unless the build names it in `OFFCUT_FEATURES_OFF`, a comma
// list: a redeploy with `OFFCUT_FEATURES_OFF=cloud_transcribe` removes the
// cloud transcription card and path entirely (cloudimplementation D-365).

export type FeatureFlags = {
  /** The transcription card in the drop zone, and the cloud path of `run-pipeline.ts`. */
  readonly cloud_transcribe: boolean;
};

// Absent where no bundler ran (Vitest, and the e2e suites importing files in Node).
declare const __OFFCUT_FEATURES__: FeatureFlags | undefined;

const ALL_ON: FeatureFlags = { cloud_transcribe: true };

export const FEATURES: FeatureFlags = Object.freeze(
  typeof __OFFCUT_FEATURES__ === "undefined" ? ALL_ON : { ...ALL_ON, ...__OFFCUT_FEATURES__ },
);
