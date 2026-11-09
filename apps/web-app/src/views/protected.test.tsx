/* @vitest-environment jsdom */
import { cleanup, render } from "@testing-library/react";
import type { ComponentType } from "react";
import { afterEach, describe, expect, test, vi } from "vitest";

import { redirect } from "@/utils/navigate";
import { camperbot, mockSession } from "../../test-utils/session";
import { Attempt } from "./attempt";
import { Certification } from "./certification";
import { Certifications } from "./certifications";
import { Exam } from "./exam";
import { Exams } from "./exams";
import { Profile } from "./profile";

vi.mock("@/utils/navigate");

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
  window.history.replaceState(null, "", "/");
});

const views: [string, ComponentType, string][] = [
  ["Exams", Exams, "/exams"],
  ["Exam", Exam, "/exams/e1"],
  ["Profile", Profile, "/users/u1/profile"],
  ["Certifications", Certifications, "/users/u1/certifications"],
  ["Certification", Certification, "/users/u1/certifications/c1"],
  ["Attempt", Attempt, "/attempts/a1"],
];

describe.each(views)("%s", (title, View, path) => {
  test("renders for logged in users", async () => {
    window.history.replaceState(null, "", path);
    mockSession(camperbot);

    const screen = render(<View />);

    expect(await screen.findByRole("heading", { name: title })).toBeInTheDocument();
    expect(redirect).not.toHaveBeenCalled();
  });

  test("sends logged out users to /login", async () => {
    window.history.replaceState(null, "", path);
    mockSession(null);

    const screen = render(<View />);

    await vi.waitFor(() => expect(redirect).toHaveBeenCalledWith("/login"));
    expect(screen.queryByRole("heading", { name: title })).not.toBeInTheDocument();
  });
});
