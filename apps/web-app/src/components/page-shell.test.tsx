/* @vitest-environment jsdom */
import { cleanup } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";

import { AuthContext } from "@/contexts/auth";
import type { SessionUser } from "@/utils/fetch";
import { render } from "../../test-utils/render";
import { camperbot } from "../../test-utils/session";
import { PageShell } from "./page-shell";

function renderShell(user: SessionUser | null, path: string) {
  return render(
    <AuthContext value={{ user, isLoading: false, error: null }}>
      <PageShell title="Title" path={path}>
        {null}
      </PageShell>
    </AuthContext>,
  );
}

afterEach(cleanup);

describe("PageShell", () => {
  test("links logged out users to login only", () => {
    const screen = renderShell(null, "/");

    expect(screen.getByRole("link", { name: "Log in" })).toHaveAttribute("href", "/login");
    expect(screen.queryByRole("link", { name: "Exams" })).not.toBeInTheDocument();
  });

  test("links logged in users to their own pages", () => {
    const screen = renderShell(camperbot, "/");

    expect(screen.getByRole("link", { name: "Profile" })).toHaveAttribute(
      "href",
      "/users/camperbot-id/profile",
    );
    expect(screen.getByRole("link", { name: "Certifications" })).toHaveAttribute(
      "href",
      "/users/camperbot-id/certifications",
    );
    expect(screen.queryByRole("link", { name: "Log in" })).not.toBeInTheDocument();
  });

  test("marks the section of a nested page current", () => {
    const screen = renderShell(camperbot, "/users/camperbot-id/certifications/c1");

    expect(screen.getByRole("link", { name: "Certifications" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("link", { name: "Home" })).not.toHaveAttribute("aria-current");
  });
});
