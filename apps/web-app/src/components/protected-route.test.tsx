/* @vitest-environment jsdom */
import { cleanup } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";

import { AuthContext } from "@/contexts/auth";
import { redirect } from "@/utils/navigate";
import { render } from "../../test-utils/render";
import { camperbot } from "../../test-utils/session";
import { ProtectedRoute } from "./protected-route";

vi.mock("@/utils/navigate");

function renderWithAuth(auth: React.ContextType<typeof AuthContext>) {
  return render(
    <AuthContext value={auth}>
      <ProtectedRoute>Protected content</ProtectedRoute>
    </AuthContext>,
  );
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("ProtectedRoute", () => {
  test("waits for the session check", () => {
    const screen = renderWithAuth({ user: null, isLoading: true, error: null });

    expect(screen.getByText("Checking session...")).toBeInTheDocument();
    expect(screen.queryByText("Protected content")).not.toBeInTheDocument();
    expect(redirect).not.toHaveBeenCalled();
  });

  test("redirects logged out users to /login", () => {
    const screen = renderWithAuth({ user: null, isLoading: false, error: null });

    expect(screen.queryByText("Protected content")).not.toBeInTheDocument();
    expect(redirect).toHaveBeenCalledWith("/login");
  });

  test("redirects to /login when the session check fails", () => {
    renderWithAuth({ user: null, isLoading: false, error: new Error("500 - boom") });

    expect(redirect).toHaveBeenCalledWith("/login");
  });

  test("renders children for logged in users", () => {
    const screen = renderWithAuth({ user: camperbot, isLoading: false, error: null });

    expect(screen.getByText("Protected content")).toBeInTheDocument();
    expect(redirect).not.toHaveBeenCalled();
  });
});
