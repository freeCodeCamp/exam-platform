import { vi } from "vitest";

import type { SessionUser } from "@/utils/fetch";

export const camperbot: SessionUser = {
  name: "Camperbot",
  email: "camperbot@freecodecamp.org",
  picture: null,
};

// Stubs `fetch`, so every request (the session check included) gets `response`. Undo with `vi.unstubAllGlobals()`.
export function mockFetchResponse(response: () => Response) {
  vi.stubGlobal(
    "fetch",
    vi.fn(() => Promise.resolve(response())),
  );
}

export function mockSession(user: SessionUser | null) {
  mockFetchResponse(() =>
    user ? Response.json(user) : new Response("not logged in", { status: 401 }),
  );
}
