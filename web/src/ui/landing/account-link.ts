// V6.2 (P206a, D-501): the landing page's way to the account page. One link
// in the top bar, to `/account`, where the sign-in form already lives: "Sign
// in" until this tab is signed in, then "Account". The landing page holds no
// clip, so leaving it loses nothing.

import { messages } from "../../copy/messages";
import type { AuthStatus } from "../../state/machines/auth-machine";

/** The link's words for this tab's account state. Only a tab signed in reads "Account". */
export function accountLinkLabel(status: AuthStatus): string {
  const copy = messages.landing.account;
  return status === "signed_in" ? copy.open : copy.signIn;
}
