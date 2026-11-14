// V6.2: the Enhance panel's switches (v6.2implementation §11, D-485 to D-488;
// TS §47.5). A switch is a review change in mode `scene`, one step of the
// history: Motion builds the scene again; Voice also swaps the program's
// audio (review-update.ts). Once the clip shows it, the panel's state is
// remembered on this device (`meta.enhanceDefault`) as the start of the next
// clip; settings' "Back to recommended" forgets it. Nothing is sent. With the
// `motion` flag off there is no panel, and no memory is read or written.

import { FEATURES } from "../config/features";
import type { EditState, Enhance } from "../gen/domain";
import { clearEnhanceDefault, getEnhanceDefault, setEnhanceDefault } from "../persistence/prefs-repo";
import { useClipStore } from "../state/clip-store";
import { update } from "./review-update";

/** The names of the `performance` measures of a switch, from the call to the change shown. */
export const ENHANCE_MEASURE = { voice: "offcut:enhance:voice", motion: "offcut:enhance:motion" } as const;

/** V5's output: the full voice chain, no camera motion (`Enhance::v5()`). */
export const ENHANCE_V5: Enhance = { voice: "clean", motion: false };

/** Whether `enhance` is V5's output (`Enhance::is_v5`). */
export function isV5(enhance: Enhance): boolean {
  return enhance.voice === "clean" && !enhance.motion;
}

/**
 * `edit` with `enhance`, left out when it is V5's (DC-16, P209): the edit is
 * then written, stored and hashed for the render cache as V6.1 wrote it, as
 * Rust's `EditState` leaves it out of its JSON.
 */
export function withEnhance(edit: EditState, enhance: Enhance): EditState {
  if (!isV5(enhance)) {
    return { ...edit, enhance };
  }
  const plain = { ...edit };
  delete plain.enhance;
  return plain;
}

/** A new clip's settings with nothing remembered: every shipped enhancement on (`Enhance::defaults`). */
export const ENHANCE_RECOMMENDED: Enhance = { voice: "clean", motion: true };

/**
 * What a new clip starts with: the remembered state, else the recommended
 * one. `undefined` with the flag off: the clip keeps `Enhance::v5()`, and its
 * edit is written as V6.1 wrote it.
 */
export async function startEnhance(): Promise<Enhance | undefined> {
  if (!FEATURES.motion) {
    return undefined;
  }
  return (await getEnhanceDefault()) ?? ENHANCE_RECOMMENDED;
}

/**
 * Turns one switch of the panel. The value the clip has already changes
 * nothing. Remembered once the clip shows it; a change refused or failed is
 * not.
 */
export async function setEnhance(change: Partial<Enhance>): Promise<void> {
  if (!FEATURES.motion) {
    return;
  }
  const current = useClipStore.getState().edit.enhance ?? ENHANCE_V5;
  const next: Enhance = { ...current, ...change };
  if (next.voice === current.voice && next.motion === current.motion) {
    return;
  }
  const startedAt = performance.now();
  await update((edit) => withEnhance(edit, next), "scene", null);
  // D-488's budget, read on this device only (P208, P209): how long the switch took to show.
  performance.measure(next.voice === current.voice ? ENHANCE_MEASURE.motion : ENHANCE_MEASURE.voice, { start: startedAt });
  const shown = useClipStore.getState().edit.enhance ?? ENHANCE_V5;
  if (shown.voice === next.voice && shown.motion === next.motion) {
    await setEnhanceDefault(next);
  }
}

/** Settings' "Back to recommended": the next clip starts from the recommended state. The clip open now keeps its own. */
export async function resetEnhance(): Promise<void> {
  if (!FEATURES.motion) {
    return;
  }
  await clearEnhanceDefault();
}
