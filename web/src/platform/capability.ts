// The startup capability check (TS §13.2): can this browser run Offcut, and
// if not, what is the first thing it lacks? The report holds enums only. No
// device attribute is read beyond the ones named here (PS §18).

import type { AnalyticsEvent, GpuVendor, MemoryBucket, Platform } from "../gen/api";
import { LIMITS, UNSUPPORTED_REASONS, type UnsupportedReason } from "../gen/domain";
import {
  AAC_ENCODE_CONFIG,
  CREATOR_VIDEO_BITRATE,
  VIDEO_ENCODE_LADDER,
  videoConfigFor,
} from "../workers/render/encoders";
import { SIMD_PROBE_BYTES } from "./simd-probe";

/** The props of the `capability_check` event; `state/capability-store.ts` names the same type. */
type CapabilityReport = Extract<AnalyticsEvent, { name: "capability_check" }>["props"];

export const PER_CHECK_TIMEOUT_MS = 1_000;

/** H.264 High, level 4.0, at the largest input size (PS §9.4). */
export const H264_DECODE_PROBE: VideoDecoderConfig = {
  codec: "avc1.640028",
  codedWidth: 1920,
  codedHeight: 1080,
};

export const AAC_DECODE_PROBE: AudioDecoderConfig = {
  codec: "mp4a.40.2",
  sampleRate: 48_000,
  numberOfChannels: 2,
};

/** Everything the check asks of the browser. Tests pass a fake. */
export type PlatformProbe = {
  /** `navigator.userAgentData?.mobile` */
  mobileHint(): boolean | undefined;
  /** `navigator.userAgentData?.platform` */
  platformHint(): string | undefined;
  hasWebCodecs(): boolean;
  /** Cross-origin isolated, with `SharedArrayBuffer`: what WASM threads need. */
  isolated(): boolean;
  simdOk(): boolean;
  /** OPFS and IndexedDB both open. */
  storageOk(): Promise<boolean>;
  deviceMemoryGb(): number | undefined;
  /** The adapter's vendor string; `null` when there is no adapter. */
  gpuVendor(): Promise<string | null>;
  h264Decode(): Promise<boolean>;
  /** Some entry of the encode ladder is supported at 1080x1920, 30 fps. */
  h264Encode(): Promise<boolean>;
  aacDecode(): Promise<boolean>;
  aacEncode(): Promise<boolean>;
};

// The two navigator fields that TypeScript's DOM types do not declare.
type NavigatorHints = Navigator & {
  userAgentData?: { mobile?: boolean; platform?: string };
  deviceMemory?: number;
};
const hints = (): NavigatorHints => navigator;

const STORAGE_PROBE_DB = "offcut-capability-probe";

/** Opens and removes a database of its own. Opening the app's database here would create it empty. */
function openProbeDatabase(): Promise<void> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(STORAGE_PROBE_DB);
    request.onerror = () => {
      reject(request.error ?? new Error("indexedDB.open failed"));
    };
    request.onsuccess = () => {
      request.result.close();
      indexedDB.deleteDatabase(STORAGE_PROBE_DB);
      resolve();
    };
  });
}

export const browserProbe: PlatformProbe = {
  mobileHint: () => hints().userAgentData?.mobile,
  platformHint: () => hints().userAgentData?.platform,
  hasWebCodecs: () => "VideoEncoder" in globalThis && "AudioEncoder" in globalThis,
  isolated: () => crossOriginIsolated && typeof SharedArrayBuffer !== "undefined",
  simdOk: () => WebAssembly.validate(SIMD_PROBE_BYTES),
  storageOk: async () => {
    await navigator.storage.getDirectory();
    await openProbeDatabase();
    return true;
  },
  deviceMemoryGb: () => hints().deviceMemory,
  gpuVendor: async () => {
    if (!("gpu" in navigator)) {
      return null;
    }
    const adapter = await navigator.gpu.requestAdapter();
    return adapter === null ? null : adapter.info.vendor;
  },
  h264Decode: async () => (await VideoDecoder.isConfigSupported(H264_DECODE_PROBE)).supported === true,
  h264Encode: async () => {
    const supports = VIDEO_ENCODE_LADDER.map(async (entry) => {
      const config = videoConfigFor(entry, LIMITS.CREATOR_WIDTH, LIMITS.CREATOR_HEIGHT, CREATOR_VIDEO_BITRATE);
      return (await VideoEncoder.isConfigSupported(config)).supported === true;
    });
    return (await Promise.all(supports)).includes(true);
  },
  aacDecode: async () => (await AudioDecoder.isConfigSupported(AAC_DECODE_PROBE)).supported === true,
  aacEncode: async () => (await AudioEncoder.isConfigSupported(AAC_ENCODE_CONFIG)).supported === true,
};

/**
 * The value of `check`, or `otherwise` if it throws, rejects, or has not
 * answered within `PER_CHECK_TIMEOUT_MS`.
 */
function guarded<T>(check: () => T | Promise<T>, otherwise: T): Promise<T> {
  return new Promise((resolve) => {
    const timer = setTimeout(() => {
      resolve(otherwise);
    }, PER_CHECK_TIMEOUT_MS);
    const settle = (value: T) => {
      clearTimeout(timer);
      resolve(value);
    };
    // Called inside `then`, so a synchronous throw is a rejection too.
    void Promise.resolve()
      .then(check)
      .then(settle, () => {
        settle(otherwise);
      });
  });
}

function gpuVendorOf(vendor: string | null): GpuVendor {
  const name = (vendor ?? "").toLowerCase();
  if (name === "") {
    return "unknown";
  }
  if (name.includes("intel")) {
    return "intel";
  }
  if (name.includes("amd") || name.includes("advanced micro")) {
    return "amd";
  }
  if (name.includes("nvidia")) {
    return "nvidia";
  }
  if (name.includes("apple")) {
    return "apple";
  }
  if (name.includes("qualcomm")) {
    return "qualcomm";
  }
  return "other";
}

/** Where the top memory bucket starts. */
const LARGE_MEMORY_GB = 8;

function memoryBucketOf(gb: number | undefined): MemoryBucket {
  if (gb === undefined) {
    return "unknown";
  }
  if (gb < LIMITS.MIN_DEVICE_MEMORY_GB) {
    return "lt4";
  }
  return gb < LARGE_MEMORY_GB ? "gb4" : "gb8plus";
}

/** The values of `navigator.userAgentData.platform` that have a bucket of their own. */
const PLATFORMS = new Map<string, Platform>([
  ["Windows", "windows"],
  ["macOS", "macos"],
  ["Linux", "linux"],
  ["Chrome OS", "chromeos"],
  ["ChromeOS", "chromeos"],
  ["Android", "android"],
]);

function platformOf(hint: string | undefined): Platform {
  if (hint === undefined) {
    return "unknown";
  }
  return PLATFORMS.get(hint) ?? "other";
}

/** Stands for a memory size that could not be read, which is different from one the browser does not report. */
const MEMORY_UNREADABLE = "unreadable";

/**
 * Runs all eleven checks at once, each with its own timeout, so the whole
 * check takes about one second at most, inside `LIMITS.CAPABILITY_CHECK_BUDGET`.
 * A check that throws or times out has failed. The reported reason is the
 * first failed check in the order of `UnsupportedReason`; the later checks
 * still run, so the report is complete.
 */
export async function runCapabilityCheck(probe: PlatformProbe = browserProbe): Promise<CapabilityReport> {
  const [
    notMobile,
    webCodecs,
    isolated,
    simd,
    storage,
    memoryGb,
    vendor,
    h264Decode,
    h264Encode,
    aacDecode,
    aacEncode,
    platform,
  ] = await Promise.all([
    guarded(() => probe.mobileHint() !== true, false),
    guarded(() => probe.hasWebCodecs(), false),
    guarded(() => probe.isolated(), false),
    guarded(() => probe.simdOk(), false),
    guarded(() => probe.storageOk(), false),
    guarded<number | undefined | typeof MEMORY_UNREADABLE>(() => probe.deviceMemoryGb(), MEMORY_UNREADABLE),
    guarded(() => probe.gpuVendor(), null),
    guarded(() => probe.h264Decode(), false),
    guarded(() => probe.h264Encode(), false),
    guarded(() => probe.aacDecode(), false),
    guarded(() => probe.aacEncode(), false),
    // Not a check: it only fills the report.
    guarded(() => probe.platformHint(), undefined),
  ]);

  // A browser that does not report its memory passes (TS §13.2).
  const memoryOk =
    memoryGb !== MEMORY_UNREADABLE && (memoryGb === undefined || memoryGb >= LIMITS.MIN_DEVICE_MEMORY_GB);

  const passed: Record<UnsupportedReason, boolean> = {
    UNSUPPORTED_MOBILE: notMobile,
    UNSUPPORTED_WEBCODECS: webCodecs,
    UNSUPPORTED_THREADS: isolated,
    UNSUPPORTED_WASM_SIMD: simd,
    UNSUPPORTED_STORAGE: storage,
    UNSUPPORTED_LOW_MEMORY: memoryOk,
    UNSUPPORTED_WEBGPU: vendor !== null,
    UNSUPPORTED_H264_DECODE: h264Decode,
    UNSUPPORTED_H264_ENCODE: h264Encode,
    UNSUPPORTED_AAC_DECODE: aacDecode,
    UNSUPPORTED_AAC_ENCODE: aacEncode,
  };
  // The generated list is in the order of the enum, which is the order of TS §13.2.
  const reason = UNSUPPORTED_REASONS.find((candidate) => !passed[candidate]);

  const report: CapabilityReport = {
    result: reason === undefined ? "pass" : "fail",
    gpu_vendor: gpuVendorOf(vendor),
    memory_bucket: memoryBucketOf(memoryGb === MEMORY_UNREADABLE ? undefined : memoryGb),
    platform: platformOf(platform),
  };
  if (reason !== undefined) {
    report.unsupported_reason = reason;
  }
  return report;
}
