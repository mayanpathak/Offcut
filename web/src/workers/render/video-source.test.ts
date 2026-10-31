// The session may not be read while a call holds it (V5 changelog, known issue
// 7): `attach_canvas` holds it across an await, and the decoder runs by itself
// meanwhile. The session here throws as wasm-bindgen does, and the decoder is
// scripted, so that a dequeue or an output comes exactly while it is held.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ClipInfo, TimeMs } from "../../gen/domain";
import { WorkerFailure } from "../rpc-worker";
import { liveFrames, VideoSource } from "./video-source";

const RECURSIVE_USE = "recursive use of an object detected which would lead to unsafe aliasing in rust";
const FRAME_US = 33_333;
const SAMPLES = 40;

type Colours = { fullRange: boolean | null; matrix: string | null; primaries: string | null; transfer: string | null };

/** A frame as the decoder gives it out. */
class FakeFrame {
  readonly timestamp: number;
  readonly colorSpace: Colours;
  closed = false;

  constructor(timestamp: number, fullRange: boolean | null = null) {
    this.timestamp = timestamp;
    this.colorSpace = { fullRange, matrix: "bt709", primaries: "bt709", transfer: "bt709" };
  }

  close(): void {
    this.closed = true;
  }
}

/** A `VideoDecoder` that does nothing by itself: the test says when a sample leaves its queue and a frame comes out. */
class FakeDecoder {
  static made: FakeDecoder[] = [];

  state = "unconfigured";
  decodeQueueSize = 0;
  config: { colorSpace?: unknown } | undefined;
  readonly #output: (frame: FakeFrame) => void;
  #dequeued: (() => void) | undefined;

  constructor(init: { output: (frame: FakeFrame) => void; error: (error: Error) => void }) {
    this.#output = init.output;
    FakeDecoder.made.push(this);
  }

  addEventListener(_name: string, listener: () => void): void {
    this.#dequeued = listener;
  }

  configure(config: { colorSpace?: unknown }): void {
    this.config = config;
    this.state = "configured";
  }

  decode(): void {
    this.decodeQueueSize += 1;
  }

  flush(): Promise<void> {
    return Promise.resolve();
  }

  close(): void {
    this.state = "closed";
  }

  /** A sample left the queue: the browser's `dequeue` event. */
  dequeue(): void {
    this.decodeQueueSize = Math.max(0, this.decodeQueueSize - 1);
    this.#dequeued?.();
  }

  emit(frame: FakeFrame): void {
    this.#output(frame);
  }
}

/** The render session as the decoder reads it. While `held`, every call throws as wasm-bindgen does. */
function fakeSession(o: { fullRange?: boolean } = {}) {
  const session = {
    held: false,
    reads: 0,
    keyframeAsks: 0,
    video_description: () => undefined,
    video_sample_count: () => SAMPLES,
    read_video_sample(index: number) {
      if (session.held) {
        throw new Error(RECURSIVE_USE);
      }
      session.reads += 1;
      return { data: new Uint8Array(1), ptsUs: index * FRAME_US, durationUs: FRAME_US, isKeyframe: index % 30 === 0 };
    },
    keyframe_at_or_before() {
      if (session.held) {
        throw new Error(RECURSIVE_USE);
      }
      session.keyframeAsks += 1;
      return 0;
    },
    video_full_range: () => o.fullRange === true,
  };
  return session;
}

const INFO = { rotation: "r0", display_width: 1280, display_height: 720, video_codec_string: "avc1.640028" } as unknown as ClipInfo;
const at = (ms: number) => ms as TimeMs;

function source(session: ReturnType<typeof fakeSession>): VideoSource {
  return new VideoSource(session, INFO);
}

function decoder(index = -1): FakeDecoder {
  const found = FakeDecoder.made.at(index);
  if (found === undefined) {
    throw new Error("no decoder was made");
  }
  return found;
}

/** What `frameAt` throws, or `undefined`. */
function failureOf(video: VideoSource): WorkerFailure | undefined {
  try {
    video.frameAt(at(0));
    return undefined;
  } catch (thrown) {
    return thrown as WorkerFailure;
  }
}

describe("a source while the session is held", () => {
  beforeEach(() => {
    FakeDecoder.made = [];
    liveFrames.count = 0;
    vi.stubGlobal("VideoDecoder", FakeDecoder);
    vi.stubGlobal("EncodedVideoChunk", class {});
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("the bug: a dequeue while the session is held, with nothing to hold it off, ends the preview as a decode failure", () => {
    const session = fakeSession();
    const video = source(session);
    video.prefetch(at(0));

    session.held = true;
    decoder().dequeue();
    session.held = false;

    const failure = failureOf(video);
    expect(failure?.code).toBe("E_DECODE_VIDEO");
    expect(failure?.message).toBeDefined();
  });

  it("a dequeue during `whileSessionHeld` reads nothing, and feeding goes on after it", async () => {
    const session = fakeSession();
    const video = source(session);
    video.prefetch(at(0));
    const before = session.reads;
    expect(before).toBeGreaterThan(0);

    await video.whileSessionHeld(async () => {
      session.held = true;
      decoder().dequeue();
      decoder().dequeue();
      await Promise.resolve();
      expect(session.reads).toBe(before);
      session.held = false;
    });

    expect(session.reads).toBeGreaterThan(before);
    expect(failureOf(video)).toBeUndefined();
    decoder().emit(new FakeFrame(0));
    expect(video.frameAt(at(0))).not.toBeNull();
    video.close();
    expect(liveFrames.count).toBe(0);
  });

  it("gives `step`'s answer", async () => {
    await expect(source(fakeSession()).whileSessionHeld(() => Promise.resolve(7))).resolves.toBe(7);
  });

  it("a full-range clip asking to start again during it waits, and starts again after it", async () => {
    const session = fakeSession({ fullRange: true });
    const video = source(session);
    video.prefetch(at(0));
    const first = decoder();
    expect(FakeDecoder.made).toHaveLength(1);
    const frame = new FakeFrame(0, false);

    await video.whileSessionHeld(async () => {
      session.held = true;
      first.emit(frame);
      // A second frame of the old decoder, in the old colours, before the restart can happen.
      first.emit(new FakeFrame(FRAME_US, false));
      await Promise.resolve();
      expect(FakeDecoder.made).toHaveLength(1);
      session.held = false;
    });

    expect(frame.closed).toBe(true);
    expect(failureOf(video)).toBeUndefined();
    expect(FakeDecoder.made).toHaveLength(2);
    expect(first.state).toBe("closed");
    expect(decoder().config?.colorSpace).toMatchObject({ fullRange: true });
    expect(video.frameAt(at(0))).toBeNull();
    video.close();
    expect(liveFrames.count).toBe(0);
  });

  it("a `step` that fails is that call's failure, and the source goes on", async () => {
    const session = fakeSession();
    const video = source(session);
    video.prefetch(at(0));
    const before = session.reads;

    await expect(
      video.whileSessionHeld(() => {
        decoder().dequeue();
        return Promise.reject(new Error("no adapter"));
      }),
    ).rejects.toThrow("no adapter");

    expect(session.reads).toBeGreaterThan(before);
    expect(failureOf(video)).toBeUndefined();
    video.close();
  });

  it("a source closed during it feeds nothing after it", async () => {
    const session = fakeSession({ fullRange: true });
    const video = source(session);
    video.prefetch(at(0));
    const before = session.reads;

    await video.whileSessionHeld(() => {
      decoder().emit(new FakeFrame(0, false));
      video.close();
      return Promise.resolve();
    });

    expect(session.reads).toBe(before);
    expect(FakeDecoder.made).toHaveLength(1);
    expect(liveFrames.count).toBe(0);
  });

  it("`rewind` starts at the first frame, though the time last shown is under 100 ms back", async () => {
    const video = source(fakeSession());
    const first = video.frameAtBlocking(at(66));
    decoder().emit(new FakeFrame(0));
    decoder().emit(new FakeFrame(FRAME_US));
    decoder().emit(new FakeFrame(2 * FRAME_US));
    decoder().emit(new FakeFrame(3 * FRAME_US));
    await first;

    video.rewind();
    expect(FakeDecoder.made).toHaveLength(2);
    const again = video.frameAtBlocking(at(0));
    const frame = new FakeFrame(0);
    decoder().emit(frame);
    decoder().emit(new FakeFrame(FRAME_US));
    expect(await again).toBe(frame);
    video.close();
    expect(liveFrames.count).toBe(0);
  });

  it("a read that really fails after it is kept for the next ask", async () => {
    const session = fakeSession();
    const video = source(session);
    video.prefetch(at(0));

    await video.whileSessionHeld(() => {
      decoder().dequeue();
      // Still held as far as the session goes: what the resume reads now throws.
      session.held = true;
      return Promise.resolve();
    });

    session.held = false;
    expect(failureOf(video)?.code).toBe("E_DECODE_VIDEO");
    video.close();
  });
});
