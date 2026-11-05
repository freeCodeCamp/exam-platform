import { afterEach, describe, expect, test, vi } from "vitest";

import { mockFetchResponse } from "../../test-utils/session";
import { authorizedFetch, HttpError } from "./fetch";
import { redirect } from "./navigate";

vi.mock("./navigate");

afterEach(() => {
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});

describe("authorizedFetch", () => {
  test("sends users without a session to /login", async () => {
    mockFetchResponse(() => new Response("no session", { status: 401 }));

    await expect(authorizedFetch("/api/exams")).rejects.toMatchObject({ status: 401 });
    expect(redirect).toHaveBeenCalledWith("/login");
  });

  test("rejects forbidden users with the server's message", async () => {
    mockFetchResponse(() => new Response("only @freecodecamp.org", { status: 403 }));

    const result = authorizedFetch("/api/exams");

    await expect(result).rejects.toBeInstanceOf(HttpError);
    await expect(result).rejects.toThrow("403 - only @freecodecamp.org");
    expect(redirect).not.toHaveBeenCalled();
  });

  test("resolves successful responses", async () => {
    mockFetchResponse(() => Response.json([]));

    const res = await authorizedFetch("/api/exams");

    expect(await res.json()).toEqual([]);
    expect(redirect).not.toHaveBeenCalled();
  });
});
