// GSAP, loaded once the landing page has rendered (designimplementation D-318).
// `import()` puts it in a chunk of its own: it never blocks the first paint,
// and the editor's route never downloads it (case L-8). Only files under
// ui/landing/ may import GSAP (D-319).
//
// Every animation runs inside `gsap.matchMedia()` under
// `(prefers-reduced-motion: no-preference)`, so with reduced motion nothing
// is registered, and `revert()` on unmount removes every tween, ScrollTrigger
// and pin it made (§8.1 rules 1 and 6). If the chunk fails to load, the page
// stays as it is: complete and still (rule 2.5).

import { type RefObject, useEffect, useRef } from "react";

export type GsapTools = {
  gsap: (typeof import("gsap"))["gsap"];
  ScrollTrigger: (typeof import("gsap/ScrollTrigger"))["ScrollTrigger"];
};

/** What a setup may return: its own clean-up, for listeners GSAP does not own. */
export type GsapSetup = (tools: GsapTools, scope: HTMLElement) => (() => void) | undefined;

/** The media query every GSAP animation of the page runs under (§8.1 rule 1). */
const MOTION_OK = "(prefers-reduced-motion: no-preference)";

let loading: Promise<GsapTools> | undefined;

/** GSAP and ScrollTrigger, imported once and registered. */
export function loadGsap(): Promise<GsapTools> {
  loading ??= Promise.all([import("gsap"), import("gsap/ScrollTrigger")]).then(([core, scroll]) => {
    core.gsap.registerPlugin(scroll.ScrollTrigger);
    return { gsap: core.gsap, ScrollTrigger: scroll.ScrollTrigger };
  });
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
