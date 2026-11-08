// The words of the long page (v6.1implementation §10.5, approved by the
// founder as drafted: F-5, 2026-11-08). Kept out of messages.ts, which is
// near its line limit; messages.ts holds them as `messages.long`, and the
// three V6.1 codes' copy in its lists (D-426). No string says "never leaves"
// (P-12): the sound is sent, the video stays.

/** A rejection or an error as a card shows it. */
type Card = { title: string; body: string; action: string };

export const longCopy = {
  linkFromClip: "Got a long video? Make shorts from it",
  makeShortsInstead: "Make shorts from it",
  dropTitle: "Drop a video (up to 45 minutes, English, one speaker).",
  dropHint: "The video stays on your computer. Its sound is sent to write the transcript.",
  shortHandOver: "This one is short, so it opens as a clip.",
  gate: {
    signedOut: "A long video needs a free account, because its sound is transcribed in the cloud.",
    /** The gate's button (§1.3: the gate "with Sign in"). */
    signIn: "Sign in",
    checking: "Checking that transcription is available…",
    unavailable: "Transcription for long videos is off right now. Try again later.",
    sends: "Its sound is sent to be transcribed and is not kept. The video stays on your computer.",
    start: "Start",
  },
  stage: {
    audio: "Reading the sound",
    transcribe: (done: number, total: number): string => `Transcribing ${String(done)} of ${String(total)}`,
  },
  keepOpen: "You can use your computer. Keep this tab open.",
  paused: {
    unavailable: "Transcription paused: it is off right now. Continue when it is back.",
    busy: "Transcription paused: it is busy. Continue in a few minutes.",
    signed_out: "Transcription paused: you were signed out. Sign in to continue.",
    offline: "Transcription paused: you are offline. Continue when you are back online.",
    wake_lost: "Paused while your computer slept.",
  },
  continue: "Continue",
  continueVideo: "Continue your video",
  awaitingFile: "Drop the same video to continue. It was read where it is on your computer, so this page needs it again.",
  differentFile: "This is a different file. Drop the video you started with.",
  done: {
    title: "Your video is transcribed",
    next: "Shorts from it come next.",
  },
} as const;

/** The two V6.1 error codes (TS §43.9). */
export const longErrors = {
  E_SOURCE_READ: {
    title: "The video could not be read",
    body: "The video could not be read. It may have been moved or deleted.",
    action: "Drop it again to continue.",
  },
  E_SOURCE_AUDIO: {
    title: "The sound of this video could not be read",
    body: "The sound of this video could not be read.",
    action: "Export it again with AAC audio, then drop it.",
  },
} satisfies Record<"E_SOURCE_READ" | "E_SOURCE_AUDIO", Card>;

/** The V6.1 rejection (TS §43.1), in `messages.rejections` with V5's. */
export const longRejections = {
  REJECT_HDR: {
    title: "This video is HDR",
    body: "Export it as SDR, then drop it again.",
    action: "",
  },
} satisfies Record<"REJECT_HDR", Card>;

/** `REJECT_DURATION` on the long page: its own words, above 45 minutes (§10.5). */
export const longPageDuration: Card = {
  title: "This video is longer than 45 minutes",
  body: "This video is longer than 45 minutes.",
  action: "Trim it, then drop it again.",
};
