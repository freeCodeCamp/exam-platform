import { afterEach, describe, expect, test, vi } from "vitest";

import { mockFetchResponse } from "../../test-utils/session";
import { authorizedFetch, getSessionUser, HttpError } from "./fetch";
import { redirect } from "./navigate";

vi.mock("./navigate");

afterEach(() => {
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});

describe("authorizedFetch", () => {
  test("calls aAPI with credentials", async () => {
    mockFetchResponse(() => Response.json([]));

    await authorizedFetch("/exams", { method: "POST" });

    expect(fetch).toHaveBeenCalledWith(new URL("http://127.0.0.1:13001/exams"), {
      method: "POST",
      credentials: "include",
    });
  });

  test("sends users without a session to /login", async () => {
    mockFetchResponse(() => new Response("no session", { status: 401 }));

    await expect(authorizedFetch("/exams")).rejects.toMatchObject({ status: 401 });
    expect(redirect).toHaveBeenCalledWith("/login");
  });

  test("rejects forbidden users with the server's message", async () => {
    mockFetchResponse(() => new Response("not your attempt", { status: 403 }));

    const result = authorizedFetch("/attempts/a1");

    await expect(result).rejects.toBeInstanceOf(HttpError);
    await expect(result).rejects.toThrow("403 - not your attempt");
    expect(redirect).not.toHaveBeenCalled();
  });
});

describe("getSessionUser", () => {
  test("returns null when logged out", async () => {
    mockFetchResponse(() => new Response("no session", { status: 401 }));

    await expect(getSessionUser()).resolves.toBeNull();
    expect(fetch).toHaveBeenCalledWith(new URL("http://127.0.0.1:13001/auth/session"), {
      credentials: "include",
    });
    expect(redirect).not.toHaveBeenCalled();
  });

  test("rejects other failures", async () => {
    mockFetchResponse(() => new Response("boom", { status: 500 }));

    await expect(getSessionUser()).rejects.toThrow("500 - boom");
  });
});
