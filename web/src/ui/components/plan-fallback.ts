// V6.2: the toast told once when a run asked for an AI plan and uses the
// built-in rules (§8.2, P194). Signed out, it offers "Sign in" (P206a, D-501),
// which opens the sign-in dialog over the editor: it never leaves the page,
// because before V7 a clip that is left is not restored.

import { messages } from "../../copy/messages";
import { type ToastItem, useUiStore } from "../../state/ui-store";

export const PLAN_FALLBACK_TOAST = "plan-fallback";

/** The toast for a fallback's reason. Only "signed_out" carries an action. */
export function planFallbackToast(why: string): ToastItem {
  const toast = { id: PLAN_FALLBACK_TOAST, message: messages.plan.fallbackLine(why) };
  if (why !== "signed_out") {
    return toast;
  }
  return {
    ...toast,
    action: {
      label: messages.plan.signIn,
      run: () => {
        useUiStore.getState().showSignIn(true);
      },
    },
  };
}
