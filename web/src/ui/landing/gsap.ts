// GSAP, loaded once the landing page has rendered (designimplementation D-318).
// `import()` of `gsap-runtime.ts` puts GSAP and its scroll plugin in a chunk
// of their own: it never blocks the first paint, and the editor's route never
// downloads it (case L-8). Only files under ui/landing/ may import GSAP (D-319).
//
// Every animation runs inside `gsap.matchMedia()` under
// `(prefers-reduced-motion: no-preference)`, so with reduced motion nothing
// is registered, and `revert()` on unmount removes every tween, trigger and
// pin it made (§8.1 rules 1 and 6). If the chunk fails to load, the page
// stays as it is: complete and still (rule 2.5).

import { type RefObject, useEffect, useRef } from "react";

import type { runtime } from "./gsap-runtime";

/** GSAP, and its scroll plugin under a name of our own. */
export type GsapTools = typeof runtime;

/** What a setup may return: its own clean-up, for listeners GSAP does not own. */
export type GsapSetup = (tools: GsapTools, scope: HTMLElement) => (() => void) | undefined;

/** The media query every GSAP animation of the page runs under (§8.1 rule 1). */
const MOTION_OK = "(prefers-reduced-motion: no-preference)";

let loading: Promise<GsapTools> | undefined;

/** GSAP and its scroll plugin, imported once and registered. */
export function loadGsap(): Promise<GsapTools> {
  loading ??= import("./gsap-runtime").then((module) => module.runtime);
  return loading;
}

/**
 * Runs `setup` once GSAP has loaded, inside `gsap.matchMedia()` scoped to
 * `scope`, under reduced motion's no-preference and `query` when given (for
 * example `(min-width: 1024px)`). Everything it makes is reverted on unmount.
 */
export function useGsap(scope: RefObject<HTMLElement | null>, setup: GsapSetup, query?: string): void {
  // The latest setup, so that a new function each render does not start it again.
  const latest = useRef(setup);
  useEffect(() => {
    latest.current = setup;
  });

  useEffect(() => {
    // A visitor who asks for reduced motion gets none, so GSAP is not even downloaded.
    if (!window.matchMedia(MOTION_OK).matches) {
      return undefined;
    }
    let live = true;
    let revert: (() => void) | undefined;
    void loadGsap().then(
      (tools) => {
        const element = scope.current;
        if (!live || element === null) {
          return;
        }
        const media = tools.gsap.matchMedia(element);
        media.add(query === undefined ? MOTION_OK : `${MOTION_OK} and ${query}`, () => latest.current(tools, element));
        revert = () => {
          media.revert();
        };
      },
      (): void => {
        // No GSAP: the page keeps its final, static state (rule 2.5).
        revert = undefined;
      },
    );
    return () => {
      live = false;
      revert?.();
    };
  }, [scope, query]);
}
