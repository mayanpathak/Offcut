// The one function every state machine goes through (TS §12.3).

/** For each state, the events it accepts and the state each leads to. A pair that is missing is illegal. */
export type MachineDef<S extends string, E extends string> = Readonly<Record<S, Partial<Record<E, S>>>>;

export class IllegalTransitionError extends Error {
  override name = "IllegalTransitionError";
}

type Reporter = (machine: string, from: string, event: string) => void;

let reporter: Reporter | undefined;

/**
 * Makes an illegal transition a report and not a throw. Production builds
 * call this once at app start; development and test builds do not, so there
 * an illegal transition throws. `state/` can neither read the build mode nor
 * send an analytics event, which is why both are handed in this way.
 */
export function setIllegalTransitionReporter(fn: Reporter): void {
  reporter = fn;
}

/**
 * The state that `event` leads to from `state`. On an illegal pair: throws
 * `IllegalTransitionError`, or, once a reporter is set, reports the attempt
 * and returns `state` unchanged.
 */
export function transition<S extends string, E extends string>(
  name: string,
  def: MachineDef<S, E>,
  state: S,
  event: E,
): S {
  const next = def[state][event];
  if (next !== undefined) {
    return next;
  }
  if (reporter === undefined) {
    throw new IllegalTransitionError(`${name}: "${event}" is not allowed in state "${state}"`);
  }
  reporter(name, state, event);
  return state;
}
