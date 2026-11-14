import { describe, expect, it } from "vitest";

import { DEFAULTS, FEATURES, featureFlags } from "./features";

// v6.1implementation D-393: each flag has a default, and the V6.x flags default off.
describe("feature flags", () => {
  it("default to cloud on and the V6.x flags off, as the production build has them", () => {
    // V6.2 (D-443, D-440 (18)): the two new flags join, off.
    expect(DEFAULTS).toEqual({ cloud_transcribe: true, long_video: false, high_res: false, ai_plan: false, motion: false });
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
      ai_plan: false,
      motion: false,
    });
    expect(featureFlags(" high_res ", "cloud_transcribe")).toEqual({
      cloud_transcribe: false,
      long_video: false,
      high_res: true,
      ai_plan: false,
      motion: false,
    });
    expect(featureFlags(undefined, "cloud_transcribe").cloud_transcribe).toBe(false);
  });

  it("V6.2: ai_plan and motion are names a build can turn on, each alone (D-443, D-444)", () => {
    expect(featureFlags("long_video,high_res,ai_plan,motion", undefined)).toEqual({
      cloud_transcribe: true,
      long_video: true,
      high_res: true,
      ai_plan: true,
      motion: true,
    });
    expect(featureFlags("ai_plan", undefined)).toEqual({ ...DEFAULTS, ai_plan: true });
    expect(featureFlags("motion", undefined)).toEqual({ ...DEFAULTS, motion: true });
    expect(() => featureFlags("ai_plan", "ai_plan")).toThrow("both name ai_plan");
  });

  it("stops the build on an unknown name or a name in both lists", () => {
    expect(() => featureFlags("long_videos", undefined)).toThrow("OFFCUT_FEATURES_ON names an unknown flag: long_videos");
    expect(() => featureFlags(undefined, "hdr")).toThrow("OFFCUT_FEATURES_OFF names an unknown flag: hdr");
    expect(() => featureFlags("high_res", "high_res")).toThrow("both name high_res");
  });
});
