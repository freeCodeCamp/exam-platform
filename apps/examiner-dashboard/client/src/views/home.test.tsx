/* @vitest-environment jsdom */
import { cleanup, render } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

import { redirect } from "@/utils/navigate";
import { camperbot, mockFetchResponse, mockSession } from "../../test-utils/session";
import { Home } from "./home.tsx";

vi.mock("@/utils/navigate");

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});

test("shows the dashboard to logged in users", async () => {
  mockSession(camperbot);

  const screen = render(<Home />);

  expect(await screen.findByText(/Author/)).toBeInTheDocument();
});

test("sends logged out users to /login", async () => {
  mockSession(null);

  const screen = render(<Home />);

  await vi.waitFor(() => expect(redirect).toHaveBeenCalledWith("/login"));
  expect(screen.queryByText(/Author/)).not.toBeInTheDocument();
});

test("sends forbidden users to /login", async () => {
  mockFetchResponse(() => new Response("only @freecodecamp.org", { status: 403 }));

  const screen = render(<Home />);

  await vi.waitFor(() => expect(redirect).toHaveBeenCalledWith("/login"));
  expect(screen.queryByText(/Author/)).not.toBeInTheDocument();
});

test("reuses the session when a page remounts", async () => {
  mockSession(camperbot);
  const firstPage = render(<Home />);
  await firstPage.findByText(/Author/);

  firstPage.unmount();
  const nextPage = render(<Home />);
  await nextPage.findByText(/Author/);

  expect(fetch).toHaveBeenCalledTimes(1);
});
