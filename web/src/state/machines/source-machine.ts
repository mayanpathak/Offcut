// The long source being worked on (v6.1implementation §9.4, D-417, TS §43.8):
// idle → probing → extracting_audio → transcribing → transcribed. A drop on the
// long page may be gated (signed out, cloud unknown or off), rejected, or
// handed to the clip page (it goes back to idle). Either working stage pauses
// with a reason and continues where it left. After a reload, a source still
// extracting waits for its file; one transcribing continues without it. V6.4
// adds the states after `transcribed`.

import type { MachineDef } from "./transition";

export type SourceStatus =
  | "idle"
  | "probing"
  | "gated"
  | "rejected"
  | "extracting_audio"
  | "transcribing"
  | "paused"
  | "awaiting_file"
  | "transcribed"
  | "cancelled"
  | "failed";

export type SourceEvent =
  | "drop"
  | "gate"
  | "ungate"
  | "reject"
  | "hand_over"
  | "accept"
  | "audio_done"
  | "transcribed"
  | "pause"
  | "continue_audio"
  | "continue_transcribe"
  | "restore_audio"
  | "restore_transcribe"
  | "restore_transcribed"
  | "file_again"
  | "cancel"
  | "fail"
  | "retry_audio"
  | "retry_transcribe"
  | "reset";

// Absent on purpose: nothing is transcribed before its sound is written; a
// rejected or handed-over file starts no work; a paused job continues only to
// the stage it left (the use-case says which); `transcribed` ends V6.1.
export const SOURCE_MACHINE: MachineDef<SourceStatus, SourceEvent> = {
  idle: {
    drop: "probing",
    restore_audio: "awaiting_file",
    restore_transcribe: "transcribing",
    restore_transcribed: "transcribed",
  },
  probing: {
    gate: "gated",
    reject: "rejected",
    hand_over: "idle",
    accept: "extracting_audio",
    fail: "failed",
  },
  gated: { ungate: "probing", reset: "idle" },
  rejected: { reset: "idle" },
  extracting_audio: { audio_done: "transcribing", pause: "paused", cancel: "cancelled", fail: "failed" },
  transcribing: { transcribed: "transcribed", pause: "paused", cancel: "cancelled", fail: "failed" },
  paused: {
    continue_audio: "extracting_audio",
    continue_transcribe: "transcribing",
    cancel: "cancelled",
    fail: "failed",
  },
  awaiting_file: { file_again: "extracting_audio", cancel: "cancelled", reset: "idle" },
  transcribed: { reset: "idle" },
  cancelled: { reset: "idle" },
  failed: { retry_audio: "awaiting_file", retry_transcribe: "transcribing", cancel: "cancelled", reset: "idle" },
};
