// The account of this tab: its state, its email and its access token. The
// token is held here, in memory, and is never written to storage (TS §22.2).

import { create } from "zustand";

import { AUTH_MACHINE, type AuthEvent, type AuthStatus } from "./machines/auth-machine";
import { transition } from "./machines/transition";

export type AuthState = {
  status: AuthStatus;
  email: string | null;
  accessToken: string | null;
};

export const useAuthStore = create<AuthState>(() => ({ status: "unknown", email: null, accessToken: null }));

/** Applies `event`; `change` is stored only if the transition was legal. */
function apply(event: AuthEvent, change: Partial<Omit<AuthState, "status">>): void {
  const current = useAuthStore.getState().status;
  const status = transition("auth", AUTH_MACHINE, current, event);
  // Asked of the table, as a resend leads back to `link_sent` and changes the email.
  if (AUTH_MACHINE[current][event] !== undefined) {
    useAuthStore.setState({ status, ...change });
  }
}

/** The magic-link request was accepted. A resend may name another address. */
export function linkAccepted(email: string): void {
  apply("link_accepted", { email });
}

/** The dialog closed while waiting for the link. */
export function dialogClosed(): void {
  apply("dialog_closed", { email: null });
}

/** A verify succeeded in this tab. */
export function verified(accessToken: string): void {
  apply("verify_ok", { accessToken });
}

/** A refresh succeeded. A tab already signed in only takes the new token. */
export function refreshOk(accessToken: string): void {
  if (useAuthStore.getState().status === "signed_in") {
    useAuthStore.setState({ accessToken });
  } else {
    apply("refresh_ok", { accessToken });
  }
}

/**
 * A refresh answered that there is no session. It may answer in any state: a
 * broadcast asks for one while the dialog waits, and a person may already have
 * sent for a link. Those states stay as they are.
 */
export function refreshRefused(): void {
  const { status } = useAuthStore.getState();
  if (status === "signed_in") {
    apply("session_expired", { email: null, accessToken: null });
  } else if (status === "unknown" || status === "anonymous_offline") {
    apply("refresh_401", {});
  }
}

/** A refresh got no answer. Only the first says anything: every other state stands. */
export function refreshUnreachable(): void {
  if (useAuthStore.getState().status === "unknown") {
    apply("refresh_network", {});
  }
}

/** The person signed out. The entitlement token goes too, in the use-case (D-291). */
export function signedOut(): void {
  apply("signed_out", { email: null, accessToken: null });
}

export function accountDeleted(): void {
  apply("account_deleted", { email: null, accessToken: null });
}

/** The address, once `GET /me` has told it: a refresh does not. */
export function setEmail(email: string): void {
  useAuthStore.setState({ email });
}
