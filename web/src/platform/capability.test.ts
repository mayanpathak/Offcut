import { afterEach, describe, expect, it, vi } from "vitest";

import type { CheckResult, GpuVendor, MemoryBucket, Platform } from "../gen/api";
import { LIMITS, UNSUPPORTED_REASONS, type UnsupportedReason } from "../gen/domain";
import { AAC_ENCODE_CONFIG, VIDEO_ENCODE_LADDER } from "../workers/render/encoders";
import { PER_CHECK_TIMEOUT_MS, type PlatformProbe, runCapabilityCheck } from "./capability";
import { SIMD_PROBE_BYTES } from "./simd-probe";

/** A browser that passes every check, with the given answers replaced. */
function probe(overrides: Partial<PlatformProbe> = {}): PlatformProbe {
  return {
    mobileHint: () => false,
    platformHint: () => "Windows",
    hasWebCodecs: () => true,
    isolated: () => true,
    simdOk: () => true,
    storageOk: () => Promise.resolve(true),
    deviceMemoryGb: () => 8,
    gpuVendor: () => Promise.resolve("Intel Inc."),
    h264Decode: () => Promise.resolve(true),
    h264Encode: () => Promise.resolve(true),
    aacDecode: () => Promise.resolve(true),
    aacEncode: () => Promise.resolve(true),
    ...overrides,
  };
}

/** For each reason, the one answer that makes exactly that check fail. */
const FAILING: Record<UnsupportedReason, Partial<PlatformProbe>> = {
  UNSUPPORTED_MOBILE: { mobileHint: () => true },
  UNSUPPORTED_WEBCODECS: { hasWebCodecs: () => false },
  UNSUPPORTED_THREADS: { isolated: () => false },
  UNSUPPORTED_WASM_SIMD: { simdOk: () => false },
  UNSUPPORTED_STORAGE: { storageOk: () => Promise.resolve(false) },
  UNSUPPORTED_LOW_MEMORY: { deviceMemoryGb: () => 2 },
  UNSUPPORTED_WEBGPU: { gpuVendor: () => Promise.resolve(null) },
  UNSUPPORTED_H264_DECODE: { h264Decode: () => Promise.resolve(false) },
  UNSUPPORTED_H264_ENCODE: { h264Encode: () => Promise.resolve(false) },
  UNSUPPORTED_AAC_DECODE: { aacDecode: () => Promise.resolve(false) },
  UNSUPPORTED_AAC_ENCODE: { aacEncode: () => Promise.resolve(false) },
};

const never = <T>(): Promise<T> => new Promise<T>(() => undefined);

afterEach(() => {
  vi.useRealTimers();
});

describe("every check passes", () => {
  it("reports pass and no reason", async () => {
    const report = await runCapabilityCheck(probe());
    expect(report).toEqual({
      result: "pass",
      gpu_vendor: "intel",
      memory_bucket: "gb8plus",
      platform: "windows",
    });
    // The key is absent, not present with `undefined` or `null`.
    expect("unsupported_reason" in report).toBe(false);
  });
});

describe("one check fails", () => {
  it("covers all eleven reasons, in the order of the generated enum", () => {
    expect(Object.keys(FAILING)).toEqual([...UNSUPPORTED_REASONS]);
    expect(UNSUPPORTED_REASONS).toHaveLength(11);
  });

  it.each(UNSUPPORTED_REASONS)("%s alone gives its own reason", async (reason) => {
    const report = await runCapabilityCheck(probe(FAILING[reason]));
    expect(report.result).toBe("fail");
    expect(report.unsupported_reason).toBe(reason);
  });
});

describe("two checks fail", () => {
  it("reports the earlier one, for every pair", async () => {
    for (const [i, earlier] of UNSUPPORTED_REASONS.entries()) {
      for (const later of UNSUPPORTED_REASONS.slice(i + 1)) {
        // The later check's answer is given first, so the order of the
        // answers cannot be what decides.
        const report = await runCapabilityCheck(probe({ ...FAILING[later], ...FAILING[earlier] }));
        expect(report.unsupported_reason, `${earlier} + ${later}`).toBe(earlier);
      }
    }
  });

  it("reports the first reason when every check fails", async () => {
    const all = Object.assign({}, ...Object.values(FAILING)) as Partial<PlatformProbe>;
    const report = await runCapabilityCheck(probe(all));
    expect(report.unsupported_reason).toBe("UNSUPPORTED_MOBILE");
  });
});

describe("a check that never answers", () => {
  it("has failed after PER_CHECK_TIMEOUT_MS, and the run ends inside the 3 s budget", async () => {
    vi.useFakeTimers();
    let report: Awaited<ReturnType<typeof runCapabilityCheck>> | undefined;
    void runCapabilityCheck(probe({ storageOk: never })).then((done) => {
      report = done;
    });

    await vi.advanceTimersByTimeAsync(PER_CHECK_TIMEOUT_MS - 1);
    expect(report).toBeUndefined();
    await vi.advanceTimersByTimeAsync(1);
    expect(report?.result).toBe("fail");
    expect(report?.unsupported_reason).toBe("UNSUPPORTED_STORAGE");
    expect(PER_CHECK_TIMEOUT_MS).toBeLessThanOrEqual(LIMITS.CAPABILITY_CHECK_BUDGET);
  });

  it("still ends after one timeout when every slow check hangs, because they run together", async () => {
    vi.useFakeTimers();
    const hanging = probe({
      storageOk: never,
      gpuVendor: never,
      h264Decode: never,
      h264Encode: never,
      aacDecode: never,
      aacEncode: never,
    });
    let report: Awaited<ReturnType<typeof runCapabilityCheck>> | undefined;
    void runCapabilityCheck(hanging).then((done) => {
      report = done;
    });

    await vi.advanceTimersByTimeAsync(PER_CHECK_TIMEOUT_MS);
    // Six checks timed out, in one second and not in six.
    expect(report?.unsupported_reason).toBe("UNSUPPORTED_STORAGE");
    expect(report?.gpu_vendor).toBe("unknown");
  });

  it("starts every check before any timeout, so a slow start does not use up the wait", async () => {
    vi.useFakeTimers();
    // The first WebCodecs call of a page can hold the main thread for half a
    // second. Here it holds it for 600 ms, and the GPU adapter then answers
    // 100 ms before its timeout runs out: in time, counted from when the
    // checks had all started, and 500 ms too late, counted from the start.
    const coldStart = probe({
      h264Decode: () => {
        vi.advanceTimersByTime(600);
        return Promise.resolve(true);
      },
      gpuVendor: () =>
        new Promise((resolve) => {
          setTimeout(() => {
            resolve("Intel Inc.");
          }, 600 + PER_CHECK_TIMEOUT_MS - 100);
        }),
    });
    let report: Awaited<ReturnType<typeof runCapabilityCheck>> | undefined;
    void runCapabilityCheck(coldStart).then((done) => {
      report = done;
    });

    await vi.advanceTimersByTimeAsync(PER_CHECK_TIMEOUT_MS);
    expect(report).toEqual({ result: "pass", gpu_vendor: "intel", memory_bucket: "gb8plus", platform: "windows" });
  });

  it("does not hold the run up when the other checks answer at once", async () => {
    vi.useFakeTimers();
    const report = runCapabilityCheck(probe());
    // No timer has to fire for a browser that answers.
    await vi.advanceTimersByTimeAsync(0);
    await expect(report).resolves.toMatchObject({ result: "pass" });
    expect(vi.getTimerCount()).toBe(0);
  });
});

describe("a check that throws", () => {
  const boom = () => {
    throw new Error("the browser threw");
  };

  it("has failed, whether it throws at once or rejects", async () => {
    const thrown = await runCapabilityCheck(probe({ simdOk: boom }));
    expect(thrown.unsupported_reason).toBe("UNSUPPORTED_WASM_SIMD");

    const rejected = await runCapabilityCheck(probe({ h264Decode: () => Promise.reject(new Error("rejected")) }));
    expect(rejected.unsupported_reason).toBe("UNSUPPORTED_H264_DECODE");

    const asyncThrow = await runCapabilityCheck(probe({ storageOk: boom }));
    expect(asyncThrow.unsupported_reason).toBe("UNSUPPORTED_STORAGE");
  });

  it("leaves a complete report when every probe function throws", async () => {
    const broken: PlatformProbe = {
      mobileHint: boom,
      platformHint: boom,
      hasWebCodecs: boom,
      isolated: boom,
      simdOk: boom,
      storageOk: boom,
      deviceMemoryGb: boom,
      gpuVendor: boom,
      h264Decode: boom,
      h264Encode: boom,
      aacDecode: boom,
      aacEncode: boom,
    };
    const report = await runCapabilityCheck(broken);
    expect(report).toEqual({
      result: "fail",
      // A mobile hint that cannot be read is a failed check, like any other.
      unsupported_reason: "UNSUPPORTED_MOBILE",
      gpu_vendor: "unknown",
      memory_bucket: "unknown",
      platform: "unknown",
    });
  });

  it("causes no unhandled rejection when it rejects after its timeout", async () => {
    vi.useFakeTimers();
    const late = new Promise<boolean>((_resolve, reject) => {
      setTimeout(() => {
        reject(new Error("too late"));
      }, PER_CHECK_TIMEOUT_MS * 2);
    });
    const report = runCapabilityCheck(probe({ aacEncode: () => late }));
    await vi.advanceTimersByTimeAsync(PER_CHECK_TIMEOUT_MS * 3);
    // Vitest fails the run on an unhandled rejection; reaching here is the proof.
    await expect(report).resolves.toMatchObject({ unsupported_reason: "UNSUPPORTED_AAC_ENCODE" });
  });
});

describe("device memory", () => {
  it("passes with bucket unknown when the browser does not report it", async () => {
    const report = await runCapabilityCheck(probe({ deviceMemoryGb: () => undefined }));
    expect(report.result).toBe("pass");
    expect(report.memory_bucket).toBe("unknown");
  });

  it.each([
    [2, "lt4", "fail"],
    [3.9, "lt4", "fail"],
    [4, "gb4", "pass"],
    [7.9, "gb4", "pass"],
    [8, "gb8plus", "pass"],
    [64, "gb8plus", "pass"],
  ] as const)("%s GB is bucket %s and the check is a %s", async (gb, bucket, result) => {
    const report = await runCapabilityCheck(probe({ deviceMemoryGb: () => gb }));
    expect(report.memory_bucket).toBe(bucket);
    expect(report.result).toBe(result);
    expect(report.unsupported_reason).toBe(result === "fail" ? "UNSUPPORTED_LOW_MEMORY" : undefined);
  });

  it("takes its lower limit from the generated limits", () => {
    expect(LIMITS.MIN_DEVICE_MEMORY_GB).toBe(4);
  });
});

describe("GPU vendor", () => {
  it.each([
    ["Intel Inc.", "intel"],
    ["apple", "apple"],
    ["ARM", "other"],
    ["", "unknown"],
    ["AMD", "amd"],
    ["Advanced Micro Devices, Inc.", "amd"],
    ["NVIDIA Corporation", "nvidia"],
    ["Qualcomm", "qualcomm"],
    ["Google Inc. (Intel)", "intel"],
  ] as const)("%j is %s", async (vendor, expected) => {
    const report = await runCapabilityCheck(probe({ gpuVendor: () => Promise.resolve(vendor) }));
    expect(report.gpu_vendor).toBe(expected);
    // An adapter with an empty vendor string is still an adapter.
    expect(report.result).toBe("pass");
  });

  it("no adapter is unknown, and the WebGPU check fails", async () => {
    const report = await runCapabilityCheck(probe({ gpuVendor: () => Promise.resolve(null) }));
    expect(report.gpu_vendor).toBe("unknown");
    expect(report.unsupported_reason).toBe("UNSUPPORTED_WEBGPU");
  });
});

describe("platform", () => {
  it.each([
    ["Windows", "windows"],
    ["macOS", "macos"],
    ["Linux", "linux"],
    ["Chrome OS", "chromeos"],
    ["ChromeOS", "chromeos"],
    ["Android", "android"],
    ["Fuchsia", "other"],
    ["windows", "other"],
    ["toString", "other"],
    [undefined, "unknown"],
  ] as const)("%j is %s", async (hint, expected) => {
    const report = await runCapabilityCheck(probe({ platformHint: () => hint }));
    expect(report.platform).toBe(expected);
    // The platform is reported, never judged.
    expect(report.result).toBe("pass");
  });
});

describe("mobile", () => {
  it("is unsupported even when every other check passes", async () => {
    const report = await runCapabilityCheck(probe({ mobileHint: () => true, platformHint: () => "Android" }));
    expect(report).toEqual({
      result: "fail",
      unsupported_reason: "UNSUPPORTED_MOBILE",
      gpu_vendor: "intel",
      memory_bucket: "gb8plus",
      platform: "android",
    });
  });

  it("passes when the browser gives no hint", async () => {
    const report = await runCapabilityCheck(probe({ mobileHint: () => undefined }));
    expect(report.result).toBe("pass");
  });
});

describe("the report", () => {
  const RESULTS: readonly CheckResult[] = ["pass", "fail"];
  const VENDORS: readonly GpuVendor[] = ["intel", "amd", "nvidia", "apple", "qualcomm", "other", "unknown"];
  const BUCKETS: readonly MemoryBucket[] = ["lt4", "gb4", "gb8plus", "unknown"];
  const PLATFORMS: readonly Platform[] = ["windows", "macos", "linux", "chromeos", "android", "other", "unknown"];

  it("holds only members of the generated enums, whatever the browser says", async () => {
    const strings = ["Some Vendor 9000 <b>", "My Own OS 3.1", "", "intel", "Windows", "\u0000"];
    const memories = [undefined, 0, 0.25, 4, 8, 1024, Number.NaN];
    for (const text of strings) {
      for (const memory of memories) {
        for (const mobile of [true, false, undefined]) {
          const report = await runCapabilityCheck(
            probe({
              gpuVendor: () => Promise.resolve(text),
              platformHint: () => text,
              deviceMemoryGb: () => memory,
              mobileHint: () => mobile,
            }),
          );
          const keys = Object.keys(report).sort();
          const expected = ["gpu_vendor", "memory_bucket", "platform", "result"];
          expect(keys.filter((key) => key !== "unsupported_reason")).toEqual(expected);

          expect(RESULTS).toContain(report.result);
          expect(VENDORS).toContain(report.gpu_vendor);
          expect(BUCKETS).toContain(report.memory_bucket);
          expect(PLATFORMS).toContain(report.platform);
          if (report.result === "fail") {
            expect(UNSUPPORTED_REASONS).toContain(report.unsupported_reason);
          } else {
            expect("unsupported_reason" in report).toBe(false);
          }
          // No string the browser gave is in the report.
          const values: unknown[] = Object.values(report);
          expect(values.includes(text) && !VENDORS.includes(text as GpuVendor)).toBe(false);
        }
      }
    }
  });
});

describe("the constants the check probes with", () => {
  it("the SIMD probe is a valid module where SIMD is supported", () => {
    // Node supports WASM SIMD, so a typing mistake in the bytes fails here.
    expect(WebAssembly.validate(SIMD_PROBE_BYTES)).toBe(true);
  });

  it("the encode ladder and the AAC configuration are those of TS §21.2", () => {
    expect(VIDEO_ENCODE_LADDER).toEqual([
      { codec: "avc1.640028", hardwareAcceleration: "prefer-hardware" },
      { codec: "avc1.4d0028", hardwareAcceleration: "prefer-hardware" },
      { codec: "avc1.640028", hardwareAcceleration: "prefer-software" },
      { codec: "avc1.42e028", hardwareAcceleration: "prefer-software" },
    ]);
    expect(AAC_ENCODE_CONFIG).toEqual({
      codec: "mp4a.40.2",
      sampleRate: 48_000,
      numberOfChannels: 2,
      bitrate: 160_000,
    });
  });
});
