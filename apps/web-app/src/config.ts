// Set by /config.js, which pages load before any module script.
declare global {
  var AUTH_API_URL: string | undefined;
}

/** aAPI origin the browser calls. Throws if /config.js did not load or set a URL. */
export function authApiUrl(): URL {
  if (!globalThis.AUTH_API_URL) throw new Error("AUTH_API_URL unset: /config.js did not load");
  return new URL(globalThis.AUTH_API_URL);
}
