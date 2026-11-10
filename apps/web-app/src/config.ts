// Set by /config.js, which pages load before any module script.
declare global {
  var AUTH_API_URL: string | undefined;
}

const LOOPBACK_HOSTNAMES = new Set(["localhost", "127.0.0.1"]);

export function matchLoopbackHostname(apiUrl: URL, pageHostname: string): URL {
  if (LOOPBACK_HOSTNAMES.has(apiUrl.hostname) && LOOPBACK_HOSTNAMES.has(pageHostname)) {
    const url = new URL(apiUrl);
    url.hostname = pageHostname;
    return url;
  }
  return apiUrl;
}

/** aAPI origin the browser calls. Throws if /config.js did not load or set a URL. */
export function authApiUrl(): URL {
  if (!globalThis.AUTH_API_URL) throw new Error("AUTH_API_URL unset: /config.js did not load");
  return matchLoopbackHostname(new URL(globalThis.AUTH_API_URL), globalThis.location.hostname);
}
