// A fake API for the E2E tests. It answers every request under /api/v1 in the
// browser, before it reaches the network, so no test touches a real server.

import type { Page } from "@playwright/test";

export type RecordedRequest = { method: string; path: string; body: unknown };

export type FakeApi = {
  /** Every request to /api/v1, in the order it arrived. `path` is relative to /api/v1. */
  requests: RecordedRequest[];
  /** Every analytics event of that name, from all recorded `POST /events` bodies. */
  eventsOf(name: string): unknown[];
};

export type FakeApiOptions = {
  /** How `POST /notify-me` answers. The default is 204 at once. */
  notifyMe?: { status: number; delayMs?: number };
  healthz?: { delayMs?: number };
};

const API_PREFIX = "/api/v1";
const JSON_TYPE = "application/json";

const wait = (ms: number) =>
  new Promise<void>((resolve) => {
    setTimeout(resolve, ms);
  });

function parse(text: string | null): unknown {
  if (text === null || text === "") {
    return undefined;
  }
  try {
    return JSON.parse(text) as unknown;
  } catch {
    return text;
  }
}

/** The events of one recorded body, if it has the shape of a batch. */
function eventsIn(body: unknown): unknown[] {
  if (typeof body === "object" && body !== null && "events" in body && Array.isArray(body.events)) {
    return body.events as unknown[];
  }
  return [];
}

/**
 * Installs the fake. `GET /healthz` answers 200, `POST /events` 204, and
 * `POST /notify-me` the configured status; an error status carries the body
 * the real server sends. Anything else is 404 `not_found`, as on the server.
 */
export async function installFakeApi(page: Page, options: FakeApiOptions = {}): Promise<FakeApi> {
  const requests: RecordedRequest[] = [];
  const notifyMe = options.notifyMe ?? { status: 204 };

  await page.route(`**${API_PREFIX}/**`, async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname.slice(API_PREFIX.length);
    requests.push({ method: request.method(), path, body: parse(request.postData()) });

    if (path === "/healthz") {
      await wait(options.healthz?.delayMs ?? 0);
      await route.fulfill({ status: 200, contentType: JSON_TYPE, body: '{"ok":true,"version":"fake"}' });
    } else if (path === "/events") {
      await route.fulfill({ status: 204 });
    } else if (path === "/notify-me") {
      await wait(notifyMe.delayMs ?? 0);
      if (notifyMe.status === 429) {
        const body = JSON.stringify({ code: "rate_limited", retry_after_secs: 705 });
        await route.fulfill({ status: 429, contentType: JSON_TYPE, headers: { "retry-after": "705" }, body });
      } else if (notifyMe.status >= 400) {
        const code = notifyMe.status >= 500 ? "internal" : "bad_request";
        const body = JSON.stringify({ code, retry_after_secs: null });
        await route.fulfill({ status: notifyMe.status, contentType: JSON_TYPE, body });
      } else {
        await route.fulfill({ status: notifyMe.status });
      }
    } else {
      const body = JSON.stringify({ code: "not_found", retry_after_secs: null });
      await route.fulfill({ status: 404, contentType: JSON_TYPE, body });
    }
  });

  return {
    requests,
    eventsOf: (name) =>
      requests
        .filter((request) => request.path === "/events")
        .flatMap((request) => eventsIn(request.body))
        .filter((event) => typeof event === "object" && event !== null && "name" in event && event.name === name),
  };
}
