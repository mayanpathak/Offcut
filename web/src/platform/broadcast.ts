// Tells this browser's other tabs that a sign-in happened (v6implementation
// §11.6). One word crosses it and nothing else: no token, no address. The tab
// that hears it asks the server for itself.

const CHANNEL = "offcut-auth";
const SIGNED_IN = "signed_in";

let channel: BroadcastChannel | undefined;

/** `undefined` on a browser without it, as the page that says it cannot run Offcut may be: its tabs just do not hear each other. */
function shared(): BroadcastChannel | undefined {
  if (typeof BroadcastChannel === "undefined") {
    return undefined;
  }
  channel ??= new BroadcastChannel(CHANNEL);
  return channel;
}

/** A channel does not hear its own posts: only the other tabs do. */
export function postSignedIn(): void {
  shared()?.postMessage(SIGNED_IN);
}

export function onSignedIn(handler: () => void): void {
  shared()?.addEventListener("message", (event: MessageEvent<unknown>) => {
    if (event.data === SIGNED_IN) {
      handler();
    }
  });
}
