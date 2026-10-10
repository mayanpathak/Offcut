// The export: every output frame of the clip drawn and encoded once, and the
// clip's audio beside it, written as one MP4 through the sink (TS §21.3).
// The loop goes over the clip's own timeline: output frame n shows the source
// at n / 30 s, whatever the audio holds. No frame is left out or shown twice
// and no sample of the audio is removed (INV-5, INV-8, INV-10).
//
// The frames are captured with method A of TS §21.4, `new VideoFrame(canvas)`
// straight after the draw. TE-3 chose it over a readback of the pixels, which
// took 2.8 times as long and wrote full-range brightness into a stream that
// does not say so. The choice is this file's alone.

import { type Bytes, type ExportProfile, LIMITS, type TimeMs } from "../../gen/domain";
import { loadRender, type Mp4MuxerHandle, type RenderSession } from "../../wasm/load-render";
import { toAppFailure, WorkerFailure } from "../rpc";
import {
  AAC_ENCODE_CONFIG,
  AAC_PRIMING_SAMPLES,
  ENCODE_QUEUE_MAX,
  KEYFRAME_INTERVAL_FRAMES,
  pickVideoConfig,
} from "./encoders";
import type { OpfsSink } from "./opfs-sink";
import { liveFrames, type VideoSource } from "./video-source";

const MICROS_PER_SEC = 1_000_000;
const AUDIO_RATE = AAC_ENCODE_CONFIG.sampleRate;
const AUDIO_CHANNELS = AAC_ENCODE_CONFIG.numberOfChannels;
/** The audio that goes with one output frame: 1,600 samples. */
const SAMPLES_PER_FRAME = AUDIO_RATE / LIMITS.OUTPUT_FPS;
/** An AAC frame holds this many samples of each channel. */
const AAC_FRAME_SAMPLES = 1_024;
const AAC_FRAME_US = Math.round((AAC_FRAME_SAMPLES * MICROS_PER_SEC) / AUDIO_RATE);
/** How long an output frame is shown, as WebCodecs wants it: 33,333. */
const FRAME_DURATION_US = Math.floor(MICROS_PER_SEC / LIMITS.OUTPUT_FPS);
/** What the encoder puts in front of the audio, as a time (D-33). */
const PRIMING_US = (AAC_PRIMING_SAMPLES * MICROS_PER_SEC) / AUDIO_RATE;
/**
 * The audio frames encoded before the first video frame, about one second.
 * The muxer cannot be made without the audio encoder's description, which
 * comes with that encoder's first chunk. The rest of the audio follows the
 * video, as TS §21.3 has it.
 */
const AUDIO_HEAD_FRAMES = 47;
/** The audio encoder is given more only while it holds fewer frames than this. */
const AUDIO_QUEUE_MAX = 32;

type EncodeCode = "E_ENCODE_VIDEO" | "E_ENCODE_AUDIO";

/** What was thrown, as an error. The render bundle throws the plain object `{ code, detail }`. */
function named(thrown: unknown): Error {
  if (thrown instanceof Error) {
    return thrown;
  }
  const failure = toAppFailure(thrown, "render_encode");
  return new WorkerFailure(failure.code, failure.detail ?? "");
}

/** Runs one step of an encoder. Whatever it throws is that encoder's failure. */
function encoding<T>(code: EncodeCode, step: () => T): T {
  try {
    return step();
  } catch (thrown) {
    const detail = thrown instanceof Error ? thrown.name : "Error";
    throw thrown instanceof WorkerFailure ? thrown : new WorkerFailure(code, detail);
  }
}

/** Waits until an encoder has given out everything it was given. */
async function flushed(code: EncodeCode, encoder: VideoEncoder | AudioEncoder): Promise<void> {
  try {
    await encoder.flush();
  } catch (thrown) {
    throw new WorkerFailure(code, thrown instanceof Error ? thrown.name : "Error");
  }
}

function bytesOf(chunk: EncodedVideoChunk | EncodedAudioChunk): Uint8Array {
  const data = new Uint8Array(chunk.byteLength);
  chunk.copyTo(data);
  return data;
}

/** The description an encoder gives with its first chunk: the `avcC`, or the `AudioSpecificConfig`. */
function describedBy(config: { description?: AllowSharedBufferSource } | undefined): Uint8Array | undefined {
  const description = config?.description;
  if (description === undefined) {
    return undefined;
  }
  const view = ArrayBuffer.isView(description) ? description : new Uint8Array(description);
  return new Uint8Array(view.buffer, view.byteOffset, view.byteLength).slice();
}

/** Closes a frame this loop made, and takes it off the count of open frames (D-45). */
function closeFrame(frame: VideoFrame): void {
  frame.close();
  liveFrames.count -= 1;
}

/**
 * The first thing that went wrong in a callback, and the loop that waits to
 * hear of it: an encoder reports a failure, and gives out its chunks, between
 * two steps of the loop.
 */
class Watch {
  failure: Error | undefined;
  #waiters: (() => void)[] = [];

  fail(thrown: unknown): void {
    this.failure ??= named(thrown);
    this.wake();
  }

  /** A callback that reports what `step` throws, in place of throwing it into the browser. */
  guard<A extends unknown[]>(step: (...args: A) => void): (...args: A) => void {
    return (...args) => {
      try {
        step(...args);
      } catch (thrown) {
        this.fail(thrown);
      }
    };
  }

  /** Throws what a callback reported, if one did. */
  check(): void {
    if (this.failure !== undefined) {
      throw this.failure;
    }
  }

  wake = (): void => {
    const waiters = this.#waiters;
    this.#waiters = [];
    for (const resolve of waiters) {
      resolve();
    }
  };

  /** Resolves when an encoder has taken work off its queue, or something failed. */
  woken(): Promise<void> {
    return new Promise((resolve) => {
      this.#waiters.push(resolve);
    });
  }
}

/**
 * The file. The muxer is made when the first chunk of each encoder has come,
 * because it needs the description of both; the few chunks that come before
 * that wait here. After it, every chunk goes through the sink at once and is
 * not kept (TS §31).
 */
class Writer {
  readonly #makeMuxer: (avcc: Uint8Array, asc: Uint8Array) => Mp4MuxerHandle;
  readonly #frames: number;
  #muxer: Mp4MuxerHandle | undefined;
  #avcc: Uint8Array | undefined;
  #asc: Uint8Array | undefined;
  /** The samples that came before the muxer was made, in the order they came. */
  #waiting: ((muxer: Mp4MuxerHandle) => void)[] = [];
  /** The video frames that are in the file. */
  #written = 0;

  constructor(makeMuxer: (avcc: Uint8Array, asc: Uint8Array) => Mp4MuxerHandle, frames: number) {
    this.#makeMuxer = makeMuxer;
    this.#frames = frames;
  }

  video(chunk: EncodedVideoChunk, metadata: EncodedVideoChunkMetadata | undefined): void {
    this.#avcc ??= describedBy(metadata?.decoderConfig);
    const data = bytesOf(chunk);
    // The frame's number is read from its timestamp: a chunk that comes out
    // of its turn is refused by the muxer (`E_MUX`), not written in the wrong place.
    const frame = Math.round((chunk.timestamp * LIMITS.OUTPUT_FPS) / MICROS_PER_SEC);
    const key = chunk.type === "key";
    this.#write((muxer) => {
      muxer.add_video_sample(data, frame, key);
      this.#written += 1;
    });
  }

  audio(chunk: EncodedAudioChunk, metadata: EncodedAudioChunkMetadata | undefined): void {
    this.#asc ??= describedBy(metadata?.decoderConfig);
    const data = bytesOf(chunk);
    const ptsUs = chunk.timestamp - PRIMING_US;
    const durationUs = chunk.duration ?? AAC_FRAME_US;
    this.#write((muxer) => {
      muxer.add_audio_sample(data, ptsUs, durationUs);
    });
  }

  /** Throws unless every output frame is in the file, once (TS §21.4). For when both encoders are flushed. */
  assertComplete(): void {
    if (this.#avcc === undefined || this.#asc === undefined) {
      throw new WorkerFailure(this.#avcc === undefined ? "E_ENCODE_VIDEO" : "E_ENCODE_AUDIO", "NoDescription");
    }
    if (this.#written !== this.#frames) {
      throw new WorkerFailure("E_ENCODE_VIDEO", "FrameCount");
    }
  }

  /** Writes what makes the file whole, and returns its size. The muxer is used up. */
  finalize(): Bytes {
    const muxer = this.#muxer;
    this.#muxer = undefined;
    if (muxer === undefined) {
      throw new WorkerFailure("E_MUX", "NoMuxer");
    }
    return muxer.finalize() as Bytes;
  }

  /** Gives back the muxer's memory, if `finalize` did not. */
  free(): void {
    this.#muxer?.free();
    this.#muxer = undefined;
  }

  #write(sample: (muxer: Mp4MuxerHandle) => void): void {
    this.#waiting.push(sample);
    if (this.#avcc === undefined || this.#asc === undefined) {
      return;
    }
    this.#muxer ??= this.#makeMuxer(this.#avcc, this.#asc);
    for (const write of this.#waiting.splice(0)) {
      write(this.#muxer);
    }
  }
}

/**
 * Renders, encodes and writes the whole clip. `canvas` is the one the session
 * draws to. `onEncoded` is called once, when everything is encoded and before
 * the file is made whole: what comes after it is the `mux` stage. The sink and
 * the frames of `source` stay the caller's. A failure is thrown as it was
 * reported: a `WorkerFailure`, or the sink's own error.
 */
export async function runExport(a: {
  session: RenderSession;
  source: VideoSource;
  profile: ExportProfile;
  out48: Float32Array;
  sink: OpfsSink;
  canvas: OffscreenCanvas;
  onProgress: (done: number, total: number) => void;
  onEncoded: () => void;
  isCancelled: () => boolean;
}): Promise<{ bytes: Bytes } | { cancelled: true }> {
  const render = await loadRender();
  const config = await pickVideoConfig(a.profile);
  const total = a.session.frame_count();
  const watch = new Watch();
  // Why the sink refused a write is kept: the muxer reports such a write as
  // a failure of its own, without the reason.
  const sink = {
    writeAt: (offset: number, data: Uint8Array): void => {
      try {
        a.sink.writeAt(offset as Bytes, data);
      } catch (thrown) {
        watch.fail(thrown);
        throw named(thrown);
      }
    },
  };
  const { width, height } = a.profile;
  const writer = new Writer(
    (avcc, asc) => render.newMuxer(sink, { width, height, avcc, frameCountHint: total }, asc),
    total,
  );

  const reported = (code: EncodeCode) => (error: DOMException) => {
    watch.fail(new WorkerFailure(code, error.name));
  };
  const venc = new VideoEncoder({
    output: watch.guard((chunk: EncodedVideoChunk, metadata?: EncodedVideoChunkMetadata) => {
      writer.video(chunk, metadata);
    }),
    error: reported("E_ENCODE_VIDEO"),
  });
  const aenc = new AudioEncoder({
    output: watch.guard((chunk: EncodedAudioChunk, metadata?: EncodedAudioChunkMetadata) => {
      writer.audio(chunk, metadata);
    }),
    error: reported("E_ENCODE_AUDIO"),
  });
  venc.addEventListener("dequeue", watch.wake);
  aenc.addEventListener("dequeue", watch.wake);

  // The audio is padded with silence to the end of the last video frame,
  // which is less than one frame of it. No sample is ever removed.
  const samples = Math.max(a.out48.length, total * SAMPLES_PER_FRAME);
  const audioFrames = Math.ceil(samples / AAC_FRAME_SAMPLES);
  let fed = 0;
  /** Gives the audio encoder its frames up to number `until`. `false` when the job was cancelled. */
  const feedAudio = async (until: number): Promise<boolean> => {
    while (fed < until) {
      if (a.isCancelled()) {
        return false;
      }
      watch.check();
      if (aenc.encodeQueueSize >= AUDIO_QUEUE_MAX) {
        await watch.woken();
        continue;
      }
      const start = fed * AAC_FRAME_SAMPLES;
      const count = Math.min(AAC_FRAME_SAMPLES, samples - start);
      // One plane for each channel, the mono voice in every one.
      const planes = new Float32Array(count * AUDIO_CHANNELS);
      planes.set(a.out48.subarray(start, start + count));
      for (let channel = 1; channel < AUDIO_CHANNELS; channel += 1) {
        planes.copyWithin(channel * count, 0, count);
      }
      encoding("E_ENCODE_AUDIO", () => {
        const data = new AudioData({
          format: "f32-planar",
          sampleRate: AUDIO_RATE,
          numberOfFrames: count,
          numberOfChannels: AUDIO_CHANNELS,
          timestamp: Math.round((start * MICROS_PER_SEC) / AUDIO_RATE),
          data: planes,
        });
        try {
          aenc.encode(data);
        } finally {
          data.close();
        }
      });
      fed += 1;
    }
    return true;
  };

  try {
    encoding("E_ENCODE_VIDEO", () => {
      venc.configure(config);
    });
    encoding("E_ENCODE_AUDIO", () => {
      aenc.configure(AAC_ENCODE_CONFIG);
    });
    if (!(await feedAudio(Math.min(AUDIO_HEAD_FRAMES, audioFrames)))) {
      return { cancelled: true };
    }

    for (let n = 0; n < total; n += 1) {
      if (a.isCancelled()) {
        return { cancelled: true };
      }
      watch.check();
      // Output frame n shows the clip at n x 1000 / 30 ms, in whole
      // milliseconds; its own timestamp is the same in microseconds.
      const t = Math.floor((n * 1000) / LIMITS.OUTPUT_FPS) as TimeMs;
      const timestamp = Math.floor((n * MICROS_PER_SEC) / LIMITS.OUTPUT_FPS);
      // The frame stays the source's: it is not closed here.
      const frame = await a.source.frameAtBlocking(t);
      a.session.render_frame(frame, t);
      const vf = encoding("E_ENCODE_VIDEO", () => new VideoFrame(a.canvas, { timestamp, duration: FRAME_DURATION_US }));
      liveFrames.count += 1;
      try {
        while (venc.encodeQueueSize > ENCODE_QUEUE_MAX) {
          await watch.woken();
          watch.check();
        }
        encoding("E_ENCODE_VIDEO", () => {
          venc.encode(vf, { keyFrame: n % KEYFRAME_INTERVAL_FRAMES === 0 });
        });
      } finally {
        closeFrame(vf);
      }
      a.onProgress(n + 1, total);
    }

    await flushed("E_ENCODE_VIDEO", venc);
    watch.check();
    if (!(await feedAudio(audioFrames))) {
      return { cancelled: true };
    }
    await flushed("E_ENCODE_AUDIO", aenc);
    watch.check();
    writer.assertComplete();

    a.onEncoded();
    return { bytes: writer.finalize() };
  } catch (thrown) {
    // The reason is what happened first: a write the sink refused, before the
    // muxer's word for it; a failure an encoder reported, before the call
    // that then found the encoder closed.
    throw watch.failure ?? named(thrown);
  } finally {
    for (const encoder of [venc, aenc]) {
      if (encoder.state !== "closed") {
        encoder.close();
      }
    }
    writer.free();
    watch.wake();
  }
}
