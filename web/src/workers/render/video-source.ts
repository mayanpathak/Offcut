// The decoded frames of a clip, by time: the demuxer's samples through one
// `VideoDecoder` into a short queue (TS §20.1). The preview asks for the
// frame of a time without waiting; the export waits for it. The decoder
// itself is managed by `frame-decoder.ts` (v4implementation D-150).
//
// Who closes a frame: this file, and nothing else. A frame that was handed
// out stays open until a newer one takes its place, so the loop that holds
// it may draw it again, and must not close it (TS §21.4).

import type { ClipInfo, TimeMs } from "../../gen/domain";
import { WorkerFailure } from "../rpc-worker";
import { decodeFailure, FrameDecoder } from "./frame-decoder";

/** What the decoder is fed through. `RenderSession` of the render bundle is one (D-30). */
export interface DemuxerHandle {
  /** The raw `avcC` payload. */
  video_description(): Uint8Array | undefined;
  video_sample_count(): number;
  /** Sample `index` in decoding order, with its place on the clip's timeline in microseconds. */
  read_video_sample(index: number): { data: Uint8Array; ptsUs: number; durationUs: number; isKeyframe: boolean };
  /** The number of the last keyframe at or before `tMs`. */
  keyframe_at_or_before(tMs: number): number;
  /** Whether the clip's video says it uses the whole brightness range, 0 to 255. */
  video_full_range(): boolean;
}

/**
 * The `VideoFrame`s that are open. Every code path that obtains one adds 1,
 * here for a decoder's output, and every `close()` takes 1 away. 0 at the end
 * of an export and after a preview stops, or a frame leaked (INV-11, D-45).
 */
export const liveFrames: { count: number } = { count: 0 };

/** The preview keeps up to this many decoded frames ahead of the time it shows. */
export const PREVIEW_QUEUE = 6;
/** The export keeps up to this many (TS §31). */
const EXPORT_QUEUE = 8;
/** What was thrown, as the failure of this file. The demuxer throws `{ code, detail }`. */
function asFailure(thrown: unknown): WorkerFailure {
  if (thrown instanceof WorkerFailure) {
    return thrown;
  }
  if (typeof thrown === "object" && thrown !== null && "detail" in thrown && typeof thrown.detail === "string") {
    return decodeFailure(thrown.detail);
  }
  return decodeFailure(thrown instanceof Error ? thrown.name : "Error");
}

export class VideoSource {
  readonly #frames: FrameDecoder;
  /** Decoded frames that were not handed out yet, in the order of their time. */
  #queue: VideoFrame[] = [];
  /** The frame that was handed out last. */
  #shown: VideoFrame | null = null;
  /** `#shown` is from before a jump: still open for the loop that holds it, and not handed out again. */
  #shownIsOld = false;

  /** The time last asked for and how many frames to keep ahead of it: what feeding goes by between two calls. */
  #wantUs = 0;
  #ahead = PREVIEW_QUEUE;

  #failed: WorkerFailure | undefined;
  #closed = false;
  #waiters: (() => void)[] = [];

  constructor(demuxer: DemuxerHandle, info: ClipInfo) {
    this.#frames = new FrameDecoder(demuxer, info, {
      opened: () => {
        liveFrames.count += 1;
      },
      keep: (frame) => {
        this.#keep(frame);
      },
      release: (frame) => {
        this.#release(frame);
      },
      wake: () => {
        this.#wake();
      },
      fail: (thrown) => {
        this.#fail(thrown);
      },
      failed: () => this.#failed !== undefined,
      wantsMore: () => this.#framesAhead() < this.#ahead,
      startAgain: () => {
        try {
          this.#restart(this.#wantUs);
          this.#frames.feed();
        } catch (thrown) {
          this.#fail(thrown);
        }
      },
    });
  }

  /**
   * The latest decoded frame at or before `t`, or `null` while it is still
   * being decoded. Never waits. The frame stays this object's: do not close it.
   */
  frameAt(t: TimeMs): VideoFrame | null {
    const tUs = t * 1000;
    this.#ask(tUs, PREVIEW_QUEUE);
    return this.#take(tUs);
  }

  /**
   * The frame of `t`: the latest at or before it, known to be the latest
   * because the one after it is decoded too, or the clip has ended. Waits for
   * the decoder; never returns the frame of an earlier time in its place. For
   * the export. The frame stays this object's: do not close it.
   */
  async frameAtBlocking(t: TimeMs): Promise<VideoFrame> {
    const tUs = t * 1000;
    for (;;) {
      this.#ask(tUs, EXPORT_QUEUE);
      if (this.#frames.drained || this.#queue.some((frame) => frame.timestamp > tUs)) {
        // A clip whose first frame is after `t` starts with that frame.
        const frame = this.#take(tUs) ?? this.#takeFirst();
        if (frame === null) {
          throw decodeFailure("NoFrame");
        }
        return frame;
      }
      await new Promise<void>((resolve) => {
        this.#waiters.push(resolve);
      });
    }
  }

  /** V4: starts decoding at `from` before a frame of it is asked for. */
  prefetch(from: TimeMs): void {
    void from;
  }

  /** Closes every frame this object holds, the one handed out last too, and the decoder. */
  close(): void {
    if (this.#closed) {
      return;
    }
    this.#closed = true;
    this.#dropQueue();
    if (this.#shown !== null) {
      this.#release(this.#shown);
      this.#shown = null;
    }
    this.#frames.close();
    this.#wake();
  }

  #release(frame: VideoFrame): void {
    frame.close();
    liveFrames.count -= 1;
  }

  #dropQueue(): void {
    for (const frame of this.#queue) {
      this.#release(frame);
    }
    this.#queue = [];
  }

  #wake(): void {
    const waiters = this.#waiters;
    this.#waiters = [];
    for (const resolve of waiters) {
      resolve();
    }
  }

  #fail(thrown: unknown): void {
    this.#failed ??= asFailure(thrown);
    this.#wake();
  }

  /** Notes what is asked for, starts decoding somewhere else when the time jumped, and feeds the decoder. */
  #ask(tUs: number, ahead: number): void {
    if (this.#closed) {
      throw decodeFailure("Closed");
    }
    if (this.#failed !== undefined) {
      throw this.#failed;
    }
    this.#wantUs = tUs;
    this.#ahead = ahead;
    try {
      // The time of the oldest frame that is still held, if any: a time
      // before it is a jump back (`FrameDecoder.jumped`).
      const held = this.#shown !== null && !this.#shownIsOld ? this.#shown.timestamp : this.#queue[0]?.timestamp;
      if (this.#frames.jumped(tUs, held)) {
        this.#restart(tUs);
      }
      this.#frames.feed();
    } catch (thrown) {
      this.#failed ??= asFailure(thrown);
      throw this.#failed;
    }
  }

  /** Decoding starts again at the keyframe at or before `tUs`. What was decoded for another place is closed. */
  #restart(tUs: number): void {
    this.#frames.stop();
    this.#dropQueue();
    this.#shownIsOld = this.#shown !== null;
    this.#frames.start(tUs);
  }

  #framesAhead(): number {
    return this.#queue.reduce((count, frame) => count + (frame.timestamp > this.#wantUs ? 1 : 0), 0);
  }

  /** Puts a decoded frame in its place. Of the frames at or before the time asked for, only the latest is kept. */
  #keep(frame: VideoFrame): void {
    const after = this.#queue.findIndex((queued) => queued.timestamp > frame.timestamp);
    this.#queue.splice(after === -1 ? this.#queue.length : after, 0, frame);
    const due = this.#queue.filter((queued) => queued.timestamp <= this.#wantUs).length;
    for (const old of this.#queue.splice(0, Math.max(0, due - 1))) {
      this.#release(old);
    }
  }

  /** Hands out the latest frame at or before `tUs`, closing the ones before it; or the one handed out last, if it still is that frame. */
  #take(tUs: number): VideoFrame | null {
    const due = this.#queue.filter((frame) => frame.timestamp <= tUs).length;
    if (due > 0) {
      const taken = this.#queue.splice(0, due);
      const chosen = taken.pop();
      for (const old of taken) {
        this.#release(old);
      }
      if (chosen !== undefined) {
        return this.#show(chosen);
      }
    }
    return this.#shown !== null && !this.#shownIsOld && this.#shown.timestamp <= tUs ? this.#shown : null;
  }

  /** Hands out the earliest decoded frame, when none is at or before the time asked for. */
  #takeFirst(): VideoFrame | null {
    const first = this.#queue.shift();
    return first === undefined ? null : this.#show(first);
  }

  #show(frame: VideoFrame): VideoFrame {
    if (this.#shown !== null) {
      this.#release(this.#shown);
    }
    this.#shown = frame;
    this.#shownIsOld = false;
    return frame;
  }
}
