/* @vitest-environment jsdom */
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";

import { redirect } from "@/utils/navigate";
import { camperbot, mockFetchResponse, mockSession } from "../../test-utils/session";
import { Login } from "./login";

vi.mock("@/utils/navigate");

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});

describe("Login", () => {
  test("is public", async () => {
    mockSession(null);

    const screen = render(<Login />);

    expect(await screen.findByRole("heading", { name: "Log in" })).toBeInTheDocument();
    expect(redirect).not.toHaveBeenCalled();
  });

  test("sends logged in users home", async () => {
    mockSession(camperbot);

    const screen = render(<Login />);

    await vi.waitFor(() => expect(redirect).toHaveBeenCalledWith("/"));
    expect(screen.queryByRole("heading", { name: "Log in" })).not.toBeInTheDocument();
  });

  test("warns when the session check fails", async () => {
    mockFetchResponse(() => new Response("boom", { status: 500 }));

    const screen = render(<Login />);

    expect(await screen.findByText("Could not check your session")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Log in" })).toBeInTheDocument();
  });
});
