// The WebCodecs encoder configurations (TS §21.2). This is the only file that
// defines them. V1 holds the constants: the capability check must probe
// exactly the configurations the export will encode with. Nothing here
// touches a browser API when the module loads.

import { type BitsPerSec, LIMITS, type Px } from "../../gen/domain";

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
