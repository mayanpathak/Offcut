// The export loop with fake WebCodecs (v5implementation D-219, §17.3): which
// frames it asks for and encodes, the audio it gives, and every way a failure
// ends or restarts it. It also proves the seam of TS §40.9 S-8 that no browser
// test can reach while `set_scene` refuses a map that is not the identity.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ExportProfile } from "../../gen/domain";
import type { RenderSession } from "../../wasm/load-render";
import { WorkerFailure } from "../rpc-worker";
import { AAC_ENCODE_CONFIG, ENCODE_QUEUE_MAX, KEYFRAME_INTERVAL_FRAMES, VIDEO_ENCODE_LADDER } from "./encoders";
import { runExport } from "./export-loop";
import type { OpfsSink } from "./opfs-sink";
import { liveFrames, type VideoSource } from "./video-source";

const m = vi.hoisted(() => ({
  muxers: [] as { video: number[]; audio: number; freed: boolean; finalized: boolean }[],
  /** What the muxer throws for the frame with this number. */
  muxFailAt: undefined as number | undefined,
  muxFailure: new Error("no failure was set"),
}));

vi.mock("../../wasm/load-render", () => ({
  loadRender: () =>
    Promise.resolve({
      newMuxer: (sink: { writeAt(offset: number, data: Uint8Array): void }) => {
        const muxer = { video: [] as number[], audio: 0, freed: false, finalized: false };
        m.muxers.push(muxer);
        return {
          add_video_sample(data: Uint8Array, frame: number) {
            if (frame === m.muxFailAt) {
              m.muxFailAt = undefined;
              throw m.muxFailure;
            }
            sink.writeAt(0, data);
            muxer.video.push(frame);
          },
          add_audio_sample() {
            muxer.audio += 1;
          },
          finalize() {
            muxer.finalized = true;
            return 1234;
          },
          free() {
            muxer.freed = true;
          },
        };
      },
    }),
}));

const FPS = 30;
const SAMPLES_PER_FRAME = 1_600;
const AAC_FRAME = 1_024;
const DESCRIPTION = { description: new Uint8Array([1]) };

/** A chunk as an encoder gives it. */
function chunk(timestamp: number) {
  return {
    timestamp,
    duration: 21_333,
    byteLength: 1,
    type: "key",
    copyTo(target: Uint8Array) {
      target.fill(1);
    },
  };
}

type EncoderInit = { output: (c: unknown, metadata?: unknown) => void; error: (e: Error) => void };

class FakeVideoEncoder {
  static made: FakeVideoEncoder[] = [];
  /** The frame each encoder, by the order they are made, fails at. */
  static failAt: (number | undefined)[] = [];
  /** Takes its work one macrotask at a time, so that a queue can build up. */
  static slow = false;
  static peak = 0;
  static supported: (config: VideoEncoderConfig) => boolean | "throws" = () => true;

  static isConfigSupported(config: VideoEncoderConfig): Promise<{ supported: boolean }> {
    const answer = FakeVideoEncoder.supported(config);
    return answer === "throws" ? Promise.reject(new TypeError("bad config")) : Promise.resolve({ supported: answer });
  }

  state = "unconfigured";
  encodeQueueSize = 0;
  config: VideoEncoderConfig | undefined;
  readonly stamps: number[] = [];
  readonly keys: boolean[] = [];
  readonly #init: EncoderInit;
  readonly #index: number;
  #dequeued: (() => void) | undefined;

  constructor(init: EncoderInit) {
    this.#init = init;
    this.#index = FakeVideoEncoder.made.length;
    FakeVideoEncoder.made.push(this);
  }

  addEventListener(_name: string, listener: () => void): void {
    this.#dequeued = listener;
  }

  configure(config: VideoEncoderConfig): void {
    this.config = config;
    this.state = "configured";
  }

  encode(frame: { timestamp: number }, options: { keyFrame: boolean }): void {
    if (this.state !== "configured") {
      throw new DOMException("closed", "InvalidStateError");
    }
    if (this.stamps.length === FakeVideoEncoder.failAt[this.#index]) {
      this.state = "closed";
      this.#init.error(new DOMException("the encoder failed", "EncodingError"));
      return;
    }
    this.stamps.push(frame.timestamp);
    this.keys.push(options.keyFrame);
    this.encodeQueueSize += 1;
    FakeVideoEncoder.peak = Math.max(FakeVideoEncoder.peak, this.encodeQueueSize);
    const done = () => {
      if (this.state === "closed") {
        return;
      }
      this.encodeQueueSize -= 1;
      this.#init.output(chunk(frame.timestamp), { decoderConfig: DESCRIPTION });
      this.#dequeued?.();
    };
    if (FakeVideoEncoder.slow) {
      setTimeout(done, 0);
    } else {
      queueMicrotask(done);
    }
  }

  async flush(): Promise<void> {
    while (this.encodeQueueSize > 0 && this.state !== "closed") {
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
  }

  close(): void {
    this.state = "closed";
  }
}

class FakeAudioEncoder {
  static made: FakeAudioEncoder[] = [];
  static failAt: (number | undefined)[] = [];

  state = "unconfigured";
  encodeQueueSize = 0;
  /** The planes of each frame given. */
  readonly given: Float32Array[] = [];
  readonly #init: EncoderInit;
  readonly #index: number;
  #dequeued: (() => void) | undefined;

  constructor(init: EncoderInit) {
    this.#init = init;
    this.#index = FakeAudioEncoder.made.length;
    FakeAudioEncoder.made.push(this);
  }

  addEventListener(_name: string, listener: () => void): void {
    this.#dequeued = listener;
  }

  configure(): void {
    this.state = "configured";
  }

  encode(data: { timestamp: number; data: Float32Array }): void {
    if (this.state !== "configured") {
      throw new DOMException("closed", "InvalidStateError");
    }
    if (this.given.length === FakeAudioEncoder.failAt[this.#index]) {
      this.state = "closed";
      this.#init.error(new DOMException("the encoder failed", "EncodingError"));
      return;
    }
    this.given.push(data.data);
    this.encodeQueueSize += 1;
    queueMicrotask(() => {
      if (this.state === "closed") {
        return;
      }
      this.encodeQueueSize -= 1;
      this.#init.output(chunk(data.timestamp), { decoderConfig: DESCRIPTION });
      this.#dequeued?.();
    });
  }

  async flush(): Promise<void> {
    while (this.encodeQueueSize > 0 && this.state !== "closed") {
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
  }

  close(): void {
    this.state = "closed";
  }
}

/** A frame the loop makes from the canvas. */
class FakeVideoFrame {
  static open = 0;
  readonly timestamp: number;

  constructor(_canvas: unknown, init: { timestamp: number }) {
    this.timestamp = init.timestamp;
    FakeVideoFrame.open += 1;
  }

  close(): void {
    FakeVideoFrame.open -= 1;
  }
}

class FakeAudioData {
  readonly timestamp: number;
  readonly data: Float32Array;

  constructor(init: { timestamp: number; data: Float32Array }) {
    this.timestamp = init.timestamp;
    this.data = init.data;
  }

  close(): void {
    // Nothing is held.
  }
}

type Pos = { at: number } | { held: number } | null;

/** A session of `frames` frames, a source and a sink, all recording what they are asked. */
function setup(o: { frames?: number; sourceAt?: (t: number) => Pos; gpuLostAt?: number; prog48?: Float32Array; sinkFailsAt?: number; cancelAfter?: number } = {}) {
  const frames = o.frames ?? 90;
  const asks: number[] = [];
  const atTimes: number[] = [];
  const drawn: number[] = [];
  const progress: [number, number][] = [];
  const events = { rewinds: 0, resets: 0, encoded: 0, writes: 0 };
  // Each failure comes once: a loop that wrongly restarted would succeed, and its case fail by name.
  let gpuLostAt = o.gpuLostAt;
  const session = {
    frame_count: () => frames,
    source_at(t: number): Pos {
      atTimes.push(t);
      return o.sourceAt === undefined ? { at: t } : o.sourceAt(t);
    },
    render_frame(_frame: unknown, t: number) {
      if (drawn.length === gpuLostAt) {
        gpuLostAt = undefined;
        throw new WorkerFailure("E_GPU_LOST", "Lost");
      }
      drawn.push(t);
    },
  };
  const source = {
    rewind() {
      events.rewinds += 1;
    },
    frameAtBlocking(t: number) {
      asks.push(t);
      return Promise.resolve({ recording: t });
    },
  };
  const sink = {
    writeAt() {
      events.writes += 1;
      if (events.writes === o.sinkFailsAt) {
        throw new Error("the disk is full");
      }
    },
    reset() {
      events.resets += 1;
    },
  };
  const prog48 = o.prog48 ?? Float32Array.from({ length: frames * SAMPLES_PER_FRAME }, (_, i) => (i % 1000) / 1000);
  const args = {
    session: session as unknown as RenderSession,
    source: source as unknown as VideoSource,
    profile: { kind: "creator", width: 1080, height: 1920, watermark: false, video_bitrate: 8_000_000, audio_bitrate: 160_000 } as unknown as ExportProfile,
    prog48,
    sink: sink as unknown as OpfsSink,
    canvas: {} as OffscreenCanvas,
    onProgress: (done: number, total: number) => {
      progress.push([done, total]);
    },
    onEncoded: () => {
      events.encoded += 1;
    },
    isCancelled: () => o.cancelAfter !== undefined && drawn.length >= o.cancelAfter,
  };
  return { args, asks, atTimes, drawn, progress, events, prog48, frames };
}

const lastVideo = () => {
  const encoder = FakeVideoEncoder.made.at(-1);
  if (encoder === undefined) {
    throw new Error("no video encoder was made");
  }
  return encoder;
};

async function failureOf(run: Promise<unknown>): Promise<WorkerFailure> {
  const thrown: unknown = await run.then(
    () => undefined,
    (error: unknown) => error,
  );
  expect(thrown).toBeInstanceOf(WorkerFailure);
  return thrown as WorkerFailure;
}

const secondEntry = VIDEO_ENCODE_LADDER[1];

describe("the export loop", () => {
  beforeEach(() => {
    m.muxers = [];
    m.muxFailAt = undefined;
    FakeVideoEncoder.made = [];
    FakeVideoEncoder.failAt = [];
    FakeVideoEncoder.slow = false;
    FakeVideoEncoder.peak = 0;
    FakeVideoEncoder.supported = () => true;
    FakeAudioEncoder.made = [];
    FakeAudioEncoder.failAt = [];
    FakeVideoFrame.open = 0;
    liveFrames.count = 0;
    vi.stubGlobal("VideoEncoder", FakeVideoEncoder);
    vi.stubGlobal("AudioEncoder", FakeAudioEncoder);
    vi.stubGlobal("VideoFrame", FakeVideoFrame);
    vi.stubGlobal("AudioData", FakeAudioData);
  });

  afterEach(() => {
    // Every frame the loop made is closed, however it ended (INV-11).
    expect(liveFrames.count).toBe(0);
    expect(FakeVideoFrame.open).toBe(0);
    for (const encoder of [...FakeVideoEncoder.made, ...FakeAudioEncoder.made]) {
      expect(encoder.state).toBe("closed");
    }
    vi.unstubAllGlobals();
  });

  it("frame accounting: each output frame once, stamped by its number, a keyframe every 60", async () => {
    const t = setup({ frames: 150 });
    expect(await runExport(t.args)).toEqual({ bytes: 1234 });

    const encoder = lastVideo();
    expect(encoder.stamps).toEqual(Array.from({ length: 150 }, (_, n) => Math.floor((n * 1_000_000) / FPS)));
    expect(encoder.keys).toEqual(Array.from({ length: 150 }, (_, n) => n % KEYFRAME_INTERVAL_FRAMES === 0));
    expect(m.muxers[0]?.video).toEqual(Array.from({ length: 150 }, (_, n) => n));
    expect(t.progress.at(-1)).toEqual([150, 150]);
    expect(t.events.encoded).toBe(1);
    expect(m.muxers[0]?.finalized).toBe(true);
  });

  it("identity: the scene is asked for each output time, and the source for what it answers", async () => {
    const t = setup({ frames: 30 });
    await runExport(t.args);
    const times = Array.from({ length: 30 }, (_, n) => Math.floor((n * 1000) / FPS));
    expect(t.atTimes).toEqual(times);
    expect(t.asks).toEqual(times);
    expect(t.drawn).toEqual(times);
  });

  it("the seam (S-8): a map that jumps and holds reaches the source as it says, and the frames stay on program time", async () => {
    const answers: Pos[] = [{ at: 0 }, { at: 1_000 }, { at: 1_033 }, { held: 2_000 }, { held: 2_000 }];
    const t = setup({
      frames: 5,
      sourceAt: (time) => answers[Math.round((time * FPS) / 1000)] ?? null,
    });
    expect(await runExport(t.args)).toEqual({ bytes: 1234 });
    expect(t.asks).toEqual([0, 1_000, 1_033, 2_000, 2_000]);
    expect(t.drawn).toEqual([0, 33, 66, 100, 133]);
    expect(lastVideo().stamps).toHaveLength(5);
  });

  it("no position for a time: E_INTERNAL NoScene, and no restart", async () => {
    const t = setup({ frames: 5, sourceAt: () => null });
    const failure = await failureOf(runExport(t.args));
    expect([failure.code, failure.detail]).toEqual(["E_INTERNAL", "NoScene"]);
    expect(t.events.resets).toBe(0);
  });

  it("audio: every sample reaches the encoder, padded to the frames' end, in both channels", async () => {
    const frames = 90;
    const shorter = Float32Array.from({ length: frames * SAMPLES_PER_FRAME - 500 }, (_, i) => (i % 1000) / 1000);
    const t = setup({ frames, prog48: shorter });
    await runExport(t.args);

    const given = FakeAudioEncoder.made[0]?.given ?? [];
    const left: number[] = [];
    for (const planes of given) {
      const count = planes.length / AAC_ENCODE_CONFIG.numberOfChannels;
      expect(planes.subarray(count)).toEqual(planes.subarray(0, count));
      left.push(...planes.subarray(0, count));
    }
    expect(left).toHaveLength(frames * SAMPLES_PER_FRAME);
    expect(left.slice(0, shorter.length)).toEqual([...shorter]);
    expect(left.slice(shorter.length).every((sample) => sample === 0)).toBe(true);
  });

  it("audio longer than the frames is never trimmed", async () => {
    const frames = 30;
    const longer = new Float32Array(frames * SAMPLES_PER_FRAME + 5 * AAC_FRAME);
    const t = setup({ frames, prog48: longer });
    await runExport(t.args);
    const given = FakeAudioEncoder.made[0]?.given ?? [];
    const samples = given.reduce((sum, planes) => sum + planes.length / AAC_ENCODE_CONFIG.numberOfChannels, 0);
    expect(samples).toBe(longer.length);
  });

  it("backpressure: never more than the limit is queued at the video encoder", async () => {
    FakeVideoEncoder.slow = true;
    const t = setup({ frames: 40 });
    await runExport(t.args);
    expect(lastVideo().stamps).toHaveLength(40);
    // It reaches the limit, which shows the queue did fill.
    expect(FakeVideoEncoder.peak).toBe(ENCODE_QUEUE_MAX + 1);
  });

  it("a video encoder that fails: the next entry, from frame 0, in an emptied file, the bar back at 0", async () => {
    FakeVideoEncoder.failAt = [30];
    const t = setup();
    expect(await runExport(t.args)).toEqual({ bytes: 1234 });

    expect(FakeVideoEncoder.made).toHaveLength(2);
    expect(FakeVideoEncoder.made[1]?.config).toMatchObject({
      codec: secondEntry?.codec,
      hardwareAcceleration: secondEntry?.hardwareAcceleration,
    });
    expect(FakeAudioEncoder.made).toHaveLength(2);
    expect(t.events.resets).toBe(1);
    expect(t.events.rewinds).toBe(2);
    expect(t.progress).toContainEqual([0, 90]);
    expect(lastVideo().stamps).toEqual(Array.from({ length: 90 }, (_, n) => Math.floor((n * 1_000_000) / FPS)));
    // The partial attempt's muxer is gone; only the second made the file.
    expect(m.muxers[0]?.freed).toBe(true);
    expect(m.muxers[0]?.finalized).toBe(false);
    expect(m.muxers.at(-1)?.finalized).toBe(true);
    expect(m.muxers.at(-1)?.video).toHaveLength(90);
    expect(t.events.encoded).toBe(1);
  });

  it("a failure in the very first frames rewinds the source too: the next attempt asks for frame 0", async () => {
    FakeVideoEncoder.failAt = [2];
    const t = setup({ frames: 10 });
    await runExport(t.args);
    expect(t.events.rewinds).toBe(2);
    expect(t.asks.slice(-10)).toEqual(Array.from({ length: 10 }, (_, n) => Math.floor((n * 1000) / FPS)));
  });

  it("every entry fails: the last failure is thrown, the sink emptied each time", async () => {
    FakeVideoEncoder.failAt = VIDEO_ENCODE_LADDER.map(() => 5);
    const t = setup({ frames: 20 });
    const failure = await failureOf(runExport(t.args));
    expect(failure.code).toBe("E_ENCODE_VIDEO");
    expect(FakeVideoEncoder.made).toHaveLength(VIDEO_ENCODE_LADDER.length);
    expect(t.events.resets).toBe(VIDEO_ENCODE_LADDER.length);
    expect(t.events.encoded).toBe(0);
  });

  it("an unsupported entry is skipped, and one the browser cannot read too", async () => {
    FakeVideoEncoder.supported = (config) => {
      if (config.hardwareAcceleration === "prefer-software") {
        return true;
      }
      return config.codec === VIDEO_ENCODE_LADDER[0]?.codec ? false : "throws";
    };
    const t = setup({ frames: 5 });
    await runExport(t.args);
    expect(FakeVideoEncoder.made).toHaveLength(1);
    expect(FakeVideoEncoder.made[0]?.config?.hardwareAcceleration).toBe("prefer-software");
    expect(t.events.resets).toBe(0);
  });

  it("no entry supported: E_ENCODE_VIDEO NoSupportedConfig", async () => {
    FakeVideoEncoder.supported = () => false;
    const failure = await failureOf(runExport(setup({ frames: 5 }).args));
    expect([failure.code, failure.detail]).toEqual(["E_ENCODE_VIDEO", "NoSupportedConfig"]);
    expect(FakeVideoEncoder.made).toHaveLength(0);
  });

  it("an audio encoder that fails once: once more on the same entry", async () => {
    FakeAudioEncoder.failAt = [3];
    const t = setup({ frames: 30 });
    expect(await runExport(t.args)).toEqual({ bytes: 1234 });
    expect(FakeAudioEncoder.made).toHaveLength(2);
    expect(FakeVideoEncoder.made).toHaveLength(2);
    expect(FakeVideoEncoder.made[1]?.config?.codec).toBe(VIDEO_ENCODE_LADDER[0]?.codec);
    expect(FakeVideoEncoder.made[1]?.config?.hardwareAcceleration).toBe(VIDEO_ENCODE_LADDER[0]?.hardwareAcceleration);
    expect(t.events.resets).toBe(1);
  });

  it("an audio encoder that fails twice: thrown", async () => {
    FakeAudioEncoder.failAt = [3, 3];
    const t = setup({ frames: 30 });
    const failure = await failureOf(runExport(t.args));
    expect(failure.code).toBe("E_ENCODE_AUDIO");
    expect(FakeAudioEncoder.made).toHaveLength(2);
    expect(t.events.resets).toBe(1);
  });

  it("a write the sink refuses is not restarted, and is thrown as the sink's", async () => {
    const t = setup({ frames: 30, sinkFailsAt: 3 });
    await expect(runExport(t.args)).rejects.toThrow("the disk is full");
    expect(FakeVideoEncoder.made).toHaveLength(1);
    expect(t.events.resets).toBe(0);
  });

  it("a lost GPU is not restarted", async () => {
    const t = setup({ frames: 30, gpuLostAt: 4 });
    const failure = await failureOf(runExport(t.args));
    expect(failure.code).toBe("E_GPU_LOST");
    expect(FakeVideoEncoder.made).toHaveLength(1);
    expect(t.events.resets).toBe(0);
  });

  it("a failure of the muxer is not restarted", async () => {
    m.muxFailAt = 2;
    m.muxFailure = new WorkerFailure("E_MUX", "OutOfOrder");
    const t = setup({ frames: 30 });
    const failure = await failureOf(runExport(t.args));
    expect(failure.code).toBe("E_MUX");
    expect(FakeVideoEncoder.made).toHaveLength(1);
    expect(t.events.resets).toBe(0);
  });

  it("a cancel: `cancelled`, no restart, and nothing encoded is finished", async () => {
    const t = setup({ frames: 30, cancelAfter: 5 });
    expect(await runExport(t.args)).toEqual({ cancelled: true });
    expect(FakeVideoEncoder.made).toHaveLength(1);
    expect(t.events.resets).toBe(0);
    expect(t.events.encoded).toBe(0);
    expect(m.muxers[0]?.finalized).toBe(false);
  });
});
