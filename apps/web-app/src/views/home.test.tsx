/* @vitest-environment jsdom */
import { cleanup, render } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

import { redirect } from "@/utils/navigate";
import { camperbot, mockSession } from "../../test-utils/session";
import { Home } from "./home";

vi.mock("@/utils/navigate");

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});

test("is public", async () => {
  mockSession(null);

  const screen = render(<Home />);

  expect(await screen.findByRole("heading", { name: "Exam Platform" })).toBeInTheDocument();
  expect(screen.getAllByRole("link", { name: "Sign in" })).toHaveLength(2);
  expect(redirect).not.toHaveBeenCalled();
});

test("hides sign in from logged in users", async () => {
  mockSession(camperbot);

  const screen = render(<Home />);

  expect(await screen.findByRole("link", { name: "Profile" })).toBeInTheDocument();
  expect(screen.queryByRole("link", { name: "Sign in" })).not.toBeInTheDocument();
});
