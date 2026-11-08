import { describe, expect, it } from "vitest";

import { DEFAULTS, FEATURES, featureFlags } from "./features";

// v6.1implementation D-393: each flag has a default, and the V6.x flags default off.
describe("feature flags", () => {
  it("default to cloud on and the V6.x flags off, as the production build has them", () => {
    expect(DEFAULTS).toEqual({ cloud_transcribe: true, long_video: false, high_res: false });
    expect(featureFlags(undefined, undefined)).toEqual(DEFAULTS);
    expect(featureFlags("", " , ")).toEqual(DEFAULTS);
    // No bundler ran: the defaults.
    expect(FEATURES).toEqual(DEFAULTS);
  });

  it("turns flags on and off by name", () => {
    expect(featureFlags("long_video,high_res", undefined)).toEqual({
      cloud_transcribe: true,
      long_video: true,
      high_res: true,
    });
    expect(featureFlags(" high_res ", "cloud_transcribe")).toEqual({
      cloud_transcribe: false,
      long_video: false,
      high_res: true,
    });
    expect(featureFlags(undefined, "cloud_transcribe").cloud_transcribe).toBe(false);
  });

  it("stops the build on an unknown name or a name in both lists", () => {
    expect(() => featureFlags("long_videos", undefined)).toThrow("OFFCUT_FEATURES_ON names an unknown flag: long_videos");
    expect(() => featureFlags(undefined, "hdr")).toThrow("OFFCUT_FEATURES_OFF names an unknown flag: hdr");
    expect(() => featureFlags("high_res", "high_res")).toThrow("both name high_res");
  });
});
