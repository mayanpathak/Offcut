// The clip's sound for cloud transcription, and the guard on what comes back
// (cloudimplementation §8.3, D-370, D-382). `encodeOpus` turns the 16 kHz
// PCM the device recognizer hears into an Ogg Opus file with the browser's
// own encoder; `dropSilentWords` removes the words a cloud recognizer
// invents over silence. Both run in `asr.worker`.

import type { CloudWord } from "../../gen/api";
import { DEFAULT_PRE_SKIP, writeOggOpus } from "./ogg-opus";

/** The rate of `pcm16`, and of the encoder's input. */
const SAMPLE_RATE = 16_000;
/** Opus counts in 48 kHz samples whatever the input rate. */
const OPUS_RATE = 48_000;
const US_PER_SECOND = 1_000_000;
const MS_PER_SECOND = 1_000;
/** 20 ms of audio at 48 kHz: the packet the encoder is asked for. */
const PACKET_SAMPLES = 960;

/** TE-C1: supported in Chrome and Edge 156; 32 kbps chosen by TE-C2. */
export const CLOUD_OPUS_CONFIG: AudioEncoderConfig = {
  codec: "opus",
  sampleRate: SAMPLE_RATE,
  numberOfChannels: 1,
  bitrate: 32_000,
  opus: { frameDuration: 20_000 },
};

/** The browser's encoder, handed in so that the tests can fake it. */
export type EncoderDeps = {
  isConfigSupported: (config: AudioEncoderConfig) => Promise<AudioEncoderSupport>;
  createEncoder: (init: AudioEncoderInit) => Pick<AudioEncoder, "configure" | "encode" | "flush" | "close">;
  createData: (init: AudioDataInit) => AudioData;
  serial: () => number;
};

export const browserEncoder: EncoderDeps = {
  isConfigSupported: (config) => AudioEncoder.isConfigSupported(config),
  createEncoder: (init) => new AudioEncoder(init),
  createData: (init) => new AudioData(init),
  serial: () => crypto.getRandomValues(new Uint32Array(1))[0] ?? 1,
};

/** The pre-skip of an `OpusHead` description, or `undefined` for anything else. */
export function preSkipOf(description: AllowSharedBufferSource | undefined): number | undefined {
  if (description === undefined) {
    return undefined;
  }
  const bytes = ArrayBuffer.isView(description)
    ? new Uint8Array(description.buffer, description.byteOffset, description.byteLength)
    : new Uint8Array(description);
  const magic = String.fromCharCode(...bytes.subarray(0, 8));
  if (magic !== "OpusHead" || bytes.length < 12) {
    return undefined;
  }
  return new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getUint16(10, true);
}

/** `pcm16` as an Ogg Opus file, or `unsupported` if this browser cannot encode it. */
export async function encodeOpus(
  pcm16: Float32Array,
  deps: EncoderDeps = browserEncoder,
): Promise<{ ogg: Uint8Array<ArrayBuffer>; durationMs: number } | { unsupported: true }> {
  const support = await deps.isConfigSupported(CLOUD_OPUS_CONFIG).catch(() => ({ supported: false }));
  if (support.supported !== true) {
    return { unsupported: true };
  }
  const packets: Uint8Array[] = [];
  const packetSamples: number[] = [];
  let preSkip: number | undefined;
  let failure: unknown;
  const encoder = deps.createEncoder({
    output: (chunk, metadata) => {
      const bytes = new Uint8Array(chunk.byteLength);
      chunk.copyTo(bytes);
      packets.push(bytes);
      const samples = chunk.duration === null ? PACKET_SAMPLES : Math.round((chunk.duration * OPUS_RATE) / US_PER_SECOND);
      packetSamples.push(samples);
      preSkip ??= preSkipOf(metadata?.decoderConfig?.description);
    },
    error: (error) => {
      failure = error;
    },
  });
  try {
    encoder.configure(CLOUD_OPUS_CONFIG);
    for (let start = 0; start < pcm16.length; start += SAMPLE_RATE) {
      const data = pcm16.slice(start, start + SAMPLE_RATE);
      const audio = deps.createData({
        format: "f32",
        sampleRate: SAMPLE_RATE,
        numberOfFrames: data.length,
        numberOfChannels: 1,
        timestamp: Math.round((start / SAMPLE_RATE) * US_PER_SECOND),
        data,
      });
      encoder.encode(audio);
      audio.close();
    }
    await encoder.flush();
  } finally {
    encoder.close();
  }
  if (failure !== undefined) {
    throw failure instanceof Error ? failure : new Error("the audio encoder failed");
  }
  const ogg = writeOggOpus({
    packets,
    packetSamples,
    preSkip: preSkip ?? DEFAULT_PRE_SKIP,
    inputSampleRate: SAMPLE_RATE,
    totalSamples: pcm16.length * (OPUS_RATE / SAMPLE_RATE),
    serial: deps.serial(),
  });
  return { ogg: new Uint8Array(ogg), durationMs: Math.round((pcm16.length / SAMPLE_RATE) * MS_PER_SECOND) };
}

/** How far under the clip's loud level a word's level may fall before it counts as silence (D-382). */
export const SILENCE_BELOW_DB = 30;
/** The share of samples quieter than the clip's "loud" level. */
const LOUD_PERCENTILE = 0.95;
/** A word too short to measure is measured over this much from its start (`word-timestamps.ts`'s minimum). */
const MIN_WINDOW_MS = 40;
const SAMPLES_PER_MS = SAMPLE_RATE / MS_PER_SECOND;
/** Below this the clip is digital silence. */
const FLOOR_DB = -120;

function db(level: number): number {
  return level > 0 ? 20 * Math.log10(level) : Number.NEGATIVE_INFINITY;
}

/** The level of `pcm16` that 95% of its samples are under, in dBFS. */
function loudLevelDb(pcm16: Float32Array): number {
  if (pcm16.length === 0) {
    return Number.NEGATIVE_INFINITY;
  }
  const magnitudes = Float32Array.from(pcm16, Math.abs).sort();
  return db(magnitudes[Math.min(magnitudes.length - 1, Math.floor(magnitudes.length * LOUD_PERCENTILE))] ?? 0);
}

function rmsDb(pcm16: Float32Array, from: number, to: number): number {
  let sum = 0;
  for (let i = from; i < to; i += 1) {
    const s = pcm16[i] ?? 0;
    sum += s * s;
  }
  return to > from ? db(Math.sqrt(sum / (to - from))) : Number.NEGATIVE_INFINITY;
}

/**
 * The words that were spoken over sound (D-382). A word is dropped when it
 * starts at or after the end of the audio, or when the level under it is
 * more than `SILENCE_BELOW_DB` below the clip's loud level: Whisper invents
 * a closing "Thank you." in the silence after the last sentence (TE-C2).
 */
export function dropSilentWords(words: readonly CloudWord[], pcm16: Float32Array, refLevel?: number): CloudWord[] {
  // V6.1 (D-407): a long source's window is judged against the source's level, not its own.
  const loud = refLevel ?? loudLevelDb(pcm16);
  if (loud <= FLOOR_DB) {
    return [];
  }
  const floor = loud - SILENCE_BELOW_DB;
  return words.filter((word) => {
    const from = Math.floor(word.start_ms * SAMPLES_PER_MS);
    if (from >= pcm16.length) {
      return false;
    }
    const end = Math.max(word.end_ms, word.start_ms + MIN_WINDOW_MS);
    const to = Math.min(pcm16.length, Math.ceil(end * SAMPLES_PER_MS));
    return rmsDb(pcm16, from, to) >= floor;
  });
}
