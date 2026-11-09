/* @vitest-environment jsdom */
import { cleanup, render } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

import { redirect } from "@/utils/navigate";
import { mockSession } from "../../test-utils/session";
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

  expect(await screen.findByRole("link", { name: "Log in" })).toBeInTheDocument();
  expect(redirect).not.toHaveBeenCalled();
});
