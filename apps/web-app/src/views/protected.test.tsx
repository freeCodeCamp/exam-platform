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

const views: [string, ComponentType, string, string][] = [
  ["Exams", Exams, "/exams", "No exams"],
  ["Exam", Exam, "/exams/e1", "Exam not found"],
  ["Profile", Profile, "/users/camperbot-id/profile", "Profile"],
  ["Certifications", Certifications, "/users/u1/certifications", "No certifications"],
  ["Certification", Certification, "/users/u1/certifications/c1", "Certification not found"],
  ["Attempt", Attempt, "/attempts/a1", "Attempt not found"],
];

describe.each(views)("%s", (_, View, path, text) => {
  test("renders for logged in users", async () => {
    window.history.replaceState(null, "", path);
    mockSession(camperbot);

    const screen = render(<View />);

    expect(await screen.findAllByText(text)).not.toHaveLength(0);
    expect(redirect).not.toHaveBeenCalled();
  });

  test("sends logged out users to /login", async () => {
    window.history.replaceState(null, "", path);
    mockSession(null);

    const screen = render(<View />);

    await vi.waitFor(() => expect(redirect).toHaveBeenCalledWith("/login"));
    expect(screen.queryAllByText(text)).toHaveLength(0);
  });
});
