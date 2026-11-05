/* @vitest-environment jsdom */
import { cleanup, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { redirect } from "@/utils/navigate";
import { camperbot, mockFetchResponse, mockSession } from "../../test-utils/session";
import { Login, loginErrorMessage } from "./login.tsx";

vi.mock("@/utils/navigate");

beforeEach(() => {
  window.history.replaceState(null, "", "/login");
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});

describe("Login", () => {
  test("links to Google login when logged out", async () => {
    mockSession(null);

    const screen = render(<Login />);

    const link = await screen.findByRole("link", { name: /continue with google/i });
    expect(link).toHaveAttribute("href", "/auth/google/login");
    expect(fetch).toHaveBeenCalledWith("/auth/session");
    expect(redirect).not.toHaveBeenCalled();
  });

  test("sends logged in users home", async () => {
    mockSession(camperbot);

    const screen = render(<Login />);

    await vi.waitFor(() => expect(redirect).toHaveBeenCalledWith("/"));
    expect(screen.queryByRole("link", { name: /continue with google/i })).not.toBeInTheDocument();
  });

  test("shows the callback error and removes it from the URL", async () => {
    mockSession(null);
    window.history.replaceState(null, "", "/login?error=forbidden_domain");

    const screen = render(<Login />);

    expect(await screen.findByText(loginErrorMessage("forbidden_domain"))).toBeInTheDocument();
    expect(window.location.search).toBe("");
  });

  test("warns when the session check fails", async () => {
    mockFetchResponse(() => new Response("boom", { status: 500 }));

    const screen = render(<Login />);

    expect(await screen.findByText("Could not check your session")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /continue with google/i })).toBeInTheDocument();
  });
});

describe("loginErrorMessage", () => {
  test("falls back to the generic message for unknown codes", () => {
    expect(loginErrorMessage("<b>call 555-0100</b>")).toBe(loginErrorMessage("login_failed"));
  });
});
