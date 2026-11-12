// What this device chose in the drop zone's transcription card
// (cloudimplementation D-367), in the `meta` store under `transcription`.
// Nothing here leaves the device. Absent or unreadable means "cloud".

import { metaGet, metaSet } from "./db";
import { META_KEYS } from "./schema";

/** The same two values as `TranscriptionChoice` of `state/cloud-store.ts`. */
export type StoredChoice = "cloud" | "device";

export async function getTranscriptionChoice(): Promise<StoredChoice> {
  try {
    const stored = await metaGet<unknown>(META_KEYS.transcription);
    return stored === "device" ? "device" : "cloud";
  } catch {
    // A storage that cannot be read: the default, as for a first visit.
    return "cloud";
  }
}

/** Kept for the next page load. A failed write keeps the choice for this page only. */
export async function setTranscriptionChoice(choice: StoredChoice): Promise<void> {
  await metaSet(META_KEYS.transcription, choice).catch(() => undefined);
}

// V6.2 (v6.2implementation §8, P193): the settings switch "AI edit plan", under
// `aiPlan`. Absent or unreadable means on (TS §40.8): only `false` turns it off.

export async function getAiPlanSwitch(): Promise<boolean> {
  try {
    return (await metaGet<unknown>(META_KEYS.aiPlan)) !== false;
  } catch {
    // A storage that cannot be read: the default, as for a first visit.
    return true;
  }
}

/** Kept for the next page load. A failed write keeps the choice for this page only. */
export async function setAiPlanSwitch(on: boolean): Promise<void> {
  await metaSet(META_KEYS.aiPlan, on).catch(() => undefined);
}
