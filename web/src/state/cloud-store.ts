// Cloud transcription on this page (cloudimplementation §8.2, D-366, D-367):
// whether the server said it will try, and what this device chose. The
// choice is kept in IndexedDB by `persistence/prefs-repo.ts`; this store
// holds it for the page.

import { create } from "zustand";

/** What `/healthz` said this page load. `unknown` until it answers. */
export type CloudAvailability = "unknown" | "available" | "unavailable";
/** The card's two choices. Absent from storage means `cloud` (D-367). */
export type TranscriptionChoice = "cloud" | "device";

export type CloudState = {
  availability: CloudAvailability;
  choice: TranscriptionChoice;
};

export const useCloudStore = create<CloudState>(() => ({ availability: "unknown", choice: "cloud" }));

/** Set once per page load, from `/healthz`. */
export function setAvailability(availability: Exclude<CloudAvailability, "unknown">): void {
  useCloudStore.setState({ availability });
}

export function setChoice(choice: TranscriptionChoice): void {
  useCloudStore.setState({ choice });
}

/**
 * The availability once it is known, or `unavailable` after `waitMs`: a
 * clip waits for a waking API at most that long (D-374).
 */
export function settledAvailability(waitMs: number): Promise<Exclude<CloudAvailability, "unknown">> {
  const now = useCloudStore.getState().availability;
  if (now !== "unknown") {
    return Promise.resolve(now);
  }
  return new Promise((resolve) => {
    const timer = setTimeout(() => {
      stop();
      resolve("unavailable");
    }, waitMs);
    const stop = useCloudStore.subscribe((state) => {
      if (state.availability !== "unknown") {
        clearTimeout(timer);
        stop();
        resolve(state.availability);
      }
    });
  });
}
