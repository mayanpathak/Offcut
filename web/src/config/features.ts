// The release's feature flags (TS §41.3.1). Built by the cloud transcription
// release with one flag; V6.1 adds its two (v6.1implementation D-393). A flag
// comes from a build-time define that `vite.config.ts` sets, never from the
// network. **Every flag takes its default unless the build names it:**
// `OFFCUT_FEATURES_ON` (a comma list) turns flags on, `OFFCUT_FEATURES_OFF`
// turns them off. `cloud_transcribe` defaults on, as since the cloud release
// (cloudimplementation D-365): a redeploy with
// `OFFCUT_FEATURES_OFF=cloud_transcribe` removes the cloud card and path. The
// V6.x flags default off, so a merge ships them dark (BP §8A.0 step 5).

export type FeatureFlags = {
  /** The transcription card in the drop zone, and the cloud path of `run-pipeline.ts`. */
  readonly cloud_transcribe: boolean;
  /** V6.1: the long page `/app/long`, its link on `/app`, and "Make shorts from it" (DC-1). */
  readonly long_video: boolean;
  /** V6.1: 4K and decodable HEVC on the clip page, and its 16 GB file limit (DC-2, DC-3). */
  readonly high_res: boolean;
};

export type FeatureName = keyof FeatureFlags;

/** Each flag's value when the build names it in neither list. */
export const DEFAULTS: FeatureFlags = Object.freeze({
  cloud_transcribe: true,
  long_video: false,
  high_res: false,
});

const NAMES = Object.keys(DEFAULTS) as readonly FeatureName[];

function isName(name: string): name is FeatureName {
  return (NAMES as readonly string[]).includes(name);
}

function listed(list: string | undefined, variable: string): Set<FeatureName> {
  const names = new Set<FeatureName>();
  for (const raw of (list ?? "").split(",")) {
    const name = raw.trim();
    if (name === "") {
      continue;
    }
    if (!isName(name)) {
      throw new Error(`${variable} names an unknown flag: ${name}`);
    }
    names.add(name);
  }
  return names;
}

/**
 * The build's flags: the defaults, with the names in `on` turned on and those
 * in `off` turned off. An unknown name, or a name in both lists, stops the build.
 */
export function featureFlags(on: string | undefined, off: string | undefined): FeatureFlags {
  const turnOn = listed(on, "OFFCUT_FEATURES_ON");
  const turnOff = listed(off, "OFFCUT_FEATURES_OFF");
  for (const name of turnOn) {
    if (turnOff.has(name)) {
      throw new Error(`OFFCUT_FEATURES_ON and OFFCUT_FEATURES_OFF both name ${name}`);
    }
  }
  const value = (name: FeatureName): boolean => (turnOn.has(name) ? true : turnOff.has(name) ? false : DEFAULTS[name]);
  return { cloud_transcribe: value("cloud_transcribe"), long_video: value("long_video"), high_res: value("high_res") };
}

// Absent where no bundler ran (Vitest, and the e2e suites importing files in Node).
declare const __OFFCUT_FEATURES__: FeatureFlags | undefined;

export const FEATURES: FeatureFlags = Object.freeze(
  typeof __OFFCUT_FEATURES__ === "undefined" ? DEFAULTS : { ...DEFAULTS, ...__OFFCUT_FEATURES__ },
);
