// The hosts the client may contact (TS §24.1). This is the only such list.

import { env } from "./env";

/** The host of the app itself, then the asset host. */
export function allowedHosts(): readonly string[] {
  return [window.location.host, new URL(env.assetBaseUrl).host];
}

/** Whether `url`, read relative to the current page, points at an allowed host. */
export function isAllowedUrl(url: string): boolean {
  const base = window.location.href;
  if (!URL.canParse(url, base)) {
    return false;
  }
  const { protocol, host } = new URL(url, base);
  const isHttp = protocol === "https:" || protocol === "http:";
  return isHttp && allowedHosts().includes(host);
}
