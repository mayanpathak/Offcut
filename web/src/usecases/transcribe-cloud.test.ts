// Cloud transcription of one clip (cloudimplementation §8.4, §11.2), with a
// fake pool and a fake API client: each answer of §6.3 to its outcome, the
// wait for a waking API, a cancel, and analytics only for a server that said yes.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { Transcript } from "../gen/domain";
import { useCloudStore } from "../state/cloud-store";
import { CLOUD_WAKE_WAIT_MS, transcribeInCloud } from "./transcribe-cloud";

const m = vi.hoisted(() => ({
  events: [] as { name: string; props?: unknown }[],
  encode: vi.fn(),
  fromCloud: vi.fn(),
  send: vi.fn(),
}));

vi.mock("../analytics/client", () => ({ track: (event: { name: string; props?: unknown }) => m.events.push(event) }));
vi.mock("../workers/pool", () => ({ pool: { asr: { encodeForCloud: m.encode, fromCloud: m.fromCloud } } }));
vi.mock("../net/api-client", () => ({ transcribeAudio: m.send }));

const TRANSCRIPT = { words: [], sentences: [], numbers: [], model_version: "groq:whisper-large-v3-turbo" } as unknown as Transcript;
const OGG = new Uint8Array([0x4f, 0x67, 0x67, 0x53]);
const WORDS = [{ text: "Hello.", start_ms: 0, end_ms: 400 }];

function pcm(): Float32Array<ArrayBuffer> {
  return new Float32Array(16_000);
}

function sentOk(): void {
  m.send.mockResolvedValue({ ok: true, status: 200, data: { words: WORDS, duration_ms: 1_000, model: "groq:whisper-large-v3-turbo" } });
}

function sentFailing(code: string, apiCode?: string): void {
  m.send.mockResolvedValue({ ok: false, failure: { code, stage: "api", retryable: true }, ...(apiCode === undefined ? {} : { apiError: { code: apiCode, retry_after_secs: null } }) });
}

const outcomes = () => m.events.filter((e) => e.name === "asr_cloud").map((e) => (e.props as { outcome: string }).outcome);

beforeEach(() => {
  m.events.length = 0;
  m.encode.mockReset().mockResolvedValue({ ogg: OGG, durationMs: 1_000 });
  m.fromCloud.mockReset().mockResolvedValue({ ok: TRANSCRIPT });
  m.send.mockReset();
  sentOk();
  useCloudStore.setState({ availability: "available", choice: "cloud" });
});

afterEach(() => {
  vi.useRealTimers();
});

describe("transcribeInCloud", () => {
  it("encodes a copy, sends it once, builds the transcript from the words and the same audio, and says ok", async () => {
    const audio = pcm();
    audio[5] = 0.25;
    const signal = new AbortController().signal;
    expect(await transcribeInCloud(audio, signal)).toEqual({ ok: TRANSCRIPT });

    const encoded = m.encode.mock.calls[0]?.[0] as { pcm16: Float32Array };
    expect(encoded.pcm16).not.toBe(audio);
    expect(encoded.pcm16[5]).toBeCloseTo(0.25);
    expect(m.send).toHaveBeenCalledTimes(1);
    expect(m.send.mock.calls[0]?.[0]).toEqual(OGG);
    expect(m.send.mock.calls[0]?.[1]).toBe(signal);
    const built = m.fromCloud.mock.calls[0]?.[0] as { words: unknown; pcm16: Float32Array; modelVersion: string };
    expect(built.words).toEqual(WORDS);
    expect(built.modelVersion).toBe("groq:whisper-large-v3-turbo");
    expect(built.pcm16).not.toBe(audio);
    expect(audio.length).toBe(16_000);
    expect(outcomes()).toEqual(["ok"]);
  });

  it.each([
    ["503 transcribe_unavailable", "E_API_5XX", "transcribe_unavailable", "unavailable"],
    ["429 rate_limited", "E_API_RATE_LIMITED", "rate_limited", "limited"],
    ["no answer in 30 s", "E_NET_TIMEOUT", undefined, "timeout"],
    ["offline", "E_NET_OFFLINE", undefined, "offline"],
    ["413", "E_INTERNAL", "payload_too_large", "failed"],
    ["a lost session", "E_AUTH_SESSION_EXPIRED", "session_expired", "failed"],
  ])("%s falls back as %s and is reported", async (_, code, apiCode, outcome) => {
    sentFailing(code, apiCode);
    expect(await transcribeInCloud(pcm(), new AbortController().signal)).toEqual({ fallback: outcome });
    expect(m.fromCloud).not.toHaveBeenCalled();
    expect(outcomes()).toEqual([outcome]);
  });

  it("a browser that cannot encode falls back as unsupported, and nothing is sent", async () => {
    m.encode.mockResolvedValue({ unsupported: true });
    expect(await transcribeInCloud(pcm(), new AbortController().signal)).toEqual({ fallback: "unsupported" });
    expect(m.send).not.toHaveBeenCalled();
    expect(outcomes()).toEqual(["unsupported"]);
  });

  it("a worker that fails falls back as failed", async () => {
    m.fromCloud.mockRejectedValue(new Error("crashed"));
    expect(await transcribeInCloud(pcm(), new AbortController().signal)).toEqual({ fallback: "failed" });
    expect(outcomes()).toEqual(["failed"]);
  });

  it("no speech in the words is the transcript's answer, not a fallback", async () => {
    m.fromCloud.mockResolvedValue({ rejected: "NoSpeech" });
    expect(await transcribeInCloud(pcm(), new AbortController().signal)).toEqual({ rejected: "NoSpeech" });
  });

  it("a server that said no is never sent anything, and no analytics value is sent", async () => {
    useCloudStore.setState({ availability: "unavailable" });
    expect(await transcribeInCloud(pcm(), new AbortController().signal)).toEqual({ fallback: "unavailable" });
    expect(m.encode).not.toHaveBeenCalled();
    expect(m.send).not.toHaveBeenCalled();
    expect(m.events).toEqual([]);
  });

  it("waits for a waking API, and goes on when it says yes", async () => {
    vi.useFakeTimers();
    useCloudStore.setState({ availability: "unknown" });
    const run = transcribeInCloud(pcm(), new AbortController().signal);
    await vi.advanceTimersByTimeAsync(20_000);
    expect(m.encode).not.toHaveBeenCalled();
    useCloudStore.setState({ availability: "available" });
    await expect(run).resolves.toEqual({ ok: TRANSCRIPT });
  });

  it("gives up on a waking API after 45 s, sending nothing", async () => {
    vi.useFakeTimers();
    useCloudStore.setState({ availability: "unknown" });
    const run = transcribeInCloud(pcm(), new AbortController().signal);
    await vi.advanceTimersByTimeAsync(CLOUD_WAKE_WAIT_MS);
    await expect(run).resolves.toEqual({ fallback: "unavailable" });
    expect(m.encode).not.toHaveBeenCalled();
    expect(m.events).toEqual([]);
  });

  it("a cancel while the request is out stops there, and reports nothing", async () => {
    const controller = new AbortController();
    m.send.mockImplementation(() => {
      controller.abort();
      return Promise.resolve({ ok: false, failure: { code: "E_INTERNAL", stage: "api", retryable: false } });
    });
    expect(await transcribeInCloud(pcm(), controller.signal)).toEqual({ fallback: "failed" });
    expect(m.fromCloud).not.toHaveBeenCalled();
    expect(m.events).toEqual([]);
  });

  it("a worker call answered as cancelled stops there", async () => {
    m.encode.mockResolvedValue({ cancelled: true });
    expect(await transcribeInCloud(pcm(), new AbortController().signal)).toEqual({ fallback: "failed" });
    expect(m.send).not.toHaveBeenCalled();
  });
});
