// The screen wake lock while a long source works (v6.1implementation D-418,
// TS §43.7). Taken when a stage starts, taken again when the tab is shown, and
// let go on pause, cancel, failure and the end. A lock that is lost and cannot
// be taken again is reported once: the job pauses ("Your computer went to sleep").

/** What the job holds while it works. */
export type WakeHold = { release(): void };

type Sentinel = { released: boolean; release(): Promise<void>; addEventListener(type: "release", cb: () => void): void };
type WakeApi = { request(type: "screen"): Promise<Sentinel> };

function wakeApi(): WakeApi | undefined {
  const nav = navigator as Navigator & { wakeLock?: WakeApi };
  return nav.wakeLock;
}

/**
 * Holds the screen awake. `onLost` is called once if the lock goes while the
 * tab is shown and cannot be taken again. A browser without the API gets a
 * hold that does nothing: the job works on, as it would.
 */
export function holdAwake(onLost: () => void): WakeHold {
  const api = wakeApi();
  let sentinel: Sentinel | undefined;
  let done = false;

  const take = async (): Promise<void> => {
    if (api === undefined || done) {
      return;
    }
    try {
      sentinel = await api.request("screen");
      sentinel.addEventListener("release", () => {
        // A hidden tab loses its lock; it is taken again when the tab is shown.
        if (!done && document.visibilityState === "visible") {
          void take().then(() => {
            if (!done && sentinel?.released !== false) {
              done = true;
              onLost();
            }
          });
        }
      });
    } catch {
      // The browser refused (no user activation, low battery): the work goes on without it.
      sentinel = undefined;
    }
  };

  const onVisible = (): void => {
    if (document.visibilityState === "visible" && (sentinel === undefined || sentinel.released)) {
      void take();
    }
  };

  void take();
  document.addEventListener("visibilitychange", onVisible);
  return {
    release() {
      done = true;
      document.removeEventListener("visibilitychange", onVisible);
      void sentinel?.release().catch(() => undefined);
    },
  };
}

const TITLE_SUFFIX = " - Offcut";

/** The tab's title while a source works: "42% - Offcut"; `null` gives the page's own back (D-418). */
export function showProgressInTitle(percent: number | null, pageTitle: string): void {
  document.title = percent === null ? pageTitle : `${Math.max(0, Math.min(100, Math.round(percent)))}%${TITLE_SUFFIX}`;
}
