// The WebCodecs encoder configurations (TS §21.2). This is the only file that
// defines them. V1 holds the constants: the capability check must probe
// exactly the configurations the export will encode with. Nothing here
// touches a browser API when the module loads.

import { type BitsPerSec, type ExportProfile, LIMITS, type Px } from "../../gen/domain";
import { WorkerFailure } from "../rpc";

export type VideoLadderEntry = {
  codec: string;
  hardwareAcceleration: "prefer-hardware" | "prefer-software";
};

/** Tried in this order; the first supported entry is used. All are H.264 level 4.0. */
export const VIDEO_ENCODE_LADDER: readonly VideoLadderEntry[] = [
  { codec: "avc1.640028", hardwareAcceleration: "prefer-hardware" }, // High
  { codec: "avc1.4d0028", hardwareAcceleration: "prefer-hardware" }, // Main
  { codec: "avc1.640028", hardwareAcceleration: "prefer-software" }, // High
  { codec: "avc1.42e028", hardwareAcceleration: "prefer-software" }, // Constrained Baseline
];

/** AAC-LC, stereo: the mono voice is duplicated to both channels. */
export const AAC_ENCODE_CONFIG: AudioEncoderConfig = {
  codec: "mp4a.40.2",
  sampleRate: 48_000,
  numberOfChannels: 2,
  bitrate: 160_000,
};

/** A keyframe every 2 s at 30 fps. (assumption, TE-4) */
export const KEYFRAME_INTERVAL_FRAMES = 60;

/** The video bitrate of a Creator export, the largest the app encodes. (assumption, TE-4) */
export const CREATOR_VIDEO_BITRATE = 8_000_000 as BitsPerSec;

/** The encoder configuration of one ladder entry at a given size and bitrate. */
export function videoConfigFor(e: VideoLadderEntry, width: Px, height: Px, bitrate: BitsPerSec): VideoEncoderConfig {
  return {
    codec: e.codec,
    hardwareAcceleration: e.hardwareAcceleration,
    width,
    height,
    bitrate,
    framerate: LIMITS.OUTPUT_FPS,
    bitrateMode: "variable",
    latencyMode: "quality",
    avc: { format: "avc" },
  };
}

/** The export loop waits while the video encoder holds more frames than this (TS §21.3, §31). */
export const ENCODE_QUEUE_MAX = 4;

/**
 * The samples the AAC encoder puts before the first one it was given. The
 * muxer takes them out of the presentation with an edit list (D-33).
 * TE-4 measured none: the encoder's first chunk has the timestamp 0, and the
 * decoded sound of an export lies on the source's with no shift. That is the
 * Windows encoder on D1; no other was measured (D-65, D-69).
 */
export const AAC_PRIMING_SAMPLES: number = 0; // TE-4: measured

/**
 * The configuration an export of this profile is encoded with: the first
 * entry of the ladder the browser supports at the profile's size and
 * bitrate. The size and the bitrate come from the profile and from nowhere
 * else (D-26). No entry supported: `E_ENCODE_VIDEO`.
 */
export async function pickVideoConfig(profile: ExportProfile): Promise<VideoEncoderConfig> {
  for (const entry of VIDEO_ENCODE_LADDER) {
    const config = videoConfigFor(entry, profile.width, profile.height, profile.video_bitrate);
    let supported = false;
    try {
      supported = (await VideoEncoder.isConfigSupported(config)).supported === true;
    } catch {
      // The browser refuses a configuration it cannot even read, such as the
      // bitrate of 0 of the preview profile: not supported, like any other.
      supported = false;
    }
    if (supported) {
      return config;
    }
  }
  throw new WorkerFailure("E_ENCODE_VIDEO", "NoSupportedConfig");
}
