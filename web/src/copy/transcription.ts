// The words of cloud transcription (cloudimplementation §9.2, D-380): the
// drop zone's card, the processing card's line, and the two toasts. Kept out
// of messages.ts, which is near its line limit; messages.ts holds them as
// `messages.transcription`. The founder approves them at G 6.6 (F-4).

export const transcriptionCopy = {
  /** The card's name, for assistive technology. */
  legend: "How captions are written",
  cloud: "Cloud · fast",
  device: "On device · private",
  cloudLine: "Captions in seconds. Only your clip's sound is sent to write them, and it is not kept. Your video stays on your computer.",
  deviceLine: "Nothing of your clip leaves this computer. It takes a little longer, and short clips are quick either way.",
  signedOutLine: "Cloud captions need a free account. Until then, everything runs on this computer.",
  signIn: "Sign in to use cloud transcription",
  processingCloud: "Your captions are being written in the cloud. Your video stays on your computer.",
  fallbackToast: "Cloud transcription did not answer, so this clip is being transcribed on your computer.",
  limitToast: "You have used today's cloud transcriptions, so this clip is being transcribed on your computer.",
};
