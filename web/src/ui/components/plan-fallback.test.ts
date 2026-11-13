// V6.2 (P206a, D-501): the AI plan's fallback toast. Signed out, it carries
// "Sign in", which opens the sign-in dialog over the editor (a flag of the UI
// store, which the editor's layout draws) and never navigates; every other
// reason carries no action.

import { beforeEach, describe, expect, it } from "vitest";

import { messages } from "../../copy/messages";
import { useUiStore } from "../../state/ui-store";
import { PLAN_FALLBACK_TOAST, planFallbackToast } from "./plan-fallback";

const OTHER_REASONS = ["quota", "offline", "unavailable", "invalid", "timeout", "catalog"];

describe("the plan's fallback toast", () => {
  beforeEach(() => {
    useUiStore.setState({ signIn: false });
  });

  it("signed out: the line, and a Sign in action that opens the dialog in place", () => {
    const toast = planFallbackToast("signed_out");
    expect(toast.id).toBe(PLAN_FALLBACK_TOAST);
    expect(toast.message).toBe(messages.plan.fallback.signedOut);
    expect(toast.action?.label).toBe(messages.plan.signIn);
    expect(useUiStore.getState().signIn).toBe(false);
    toast.action?.run();
    expect(useUiStore.getState().signIn).toBe(true);
  });

  it("every other reason: its line and no action", () => {
    for (const why of OTHER_REASONS) {
      const toast = planFallbackToast(why);
      expect(toast, why).toEqual({ id: PLAN_FALLBACK_TOAST, message: messages.plan.fallbackLine(why) });
    }
  });

  it("the dialog's flag closes again", () => {
    useUiStore.getState().showSignIn(true);
    useUiStore.getState().showSignIn(false);
    expect(useUiStore.getState().signIn).toBe(false);
  });
});
