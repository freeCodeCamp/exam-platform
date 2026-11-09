import { afterEach, describe, expect, test } from "vitest";

import {
  buildPath,
  currentRouteParams,
  DYNAMIC_ROUTES,
  matchPath,
  placeholderPath,
  ROUTES,
} from "./routes";

describe("matchPath", () => {
  test("extracts every param", () => {
    expect(matchPath(ROUTES.certification, "/users/u1/certifications/c2")).toEqual({
      userId: "u1",
      certificationId: "c2",
    });
  });

  test("allows a trailing slash", () => {
    expect(matchPath(ROUTES.exam, "/exams/e1/")).toEqual({ examId: "e1" });
  });

  test("decodes params", () => {
    expect(matchPath(ROUTES.attempt, "/attempts/a%20b")).toEqual({ attemptId: "a b" });
  });

  test("rejects other paths", () => {
    expect(matchPath(ROUTES.exam, "/exams")).toBeNull();
    expect(matchPath(ROUTES.exam, "/exams/e1/extra")).toBeNull();
    expect(matchPath(ROUTES.certifications, "/users/u1/certifications/c2")).toBeNull();
  });

  test("rejects malformed encodings", () => {
    expect(matchPath(ROUTES.exam, "/exams/%E0")).toBeNull();
  });
});

describe("buildPath", () => {
  test("encodes params", () => {
    expect(buildPath(ROUTES.profile, { userId: "a/b" })).toBe("/users/a%2Fb/profile");
  });

  test("round trips through matchPath", () => {
    const params = { userId: "u 1", certificationId: "c?2" };
    expect(matchPath(ROUTES.certification, buildPath(ROUTES.certification, params))).toEqual(
      params,
    );
  });
});

test("placeholderPath replaces every param", () => {
  expect(placeholderPath(ROUTES.certification)).toBe("/users/_/certifications/_");
});

test("DYNAMIC_ROUTES lists only patterns with params", () => {
  expect(DYNAMIC_ROUTES).toEqual([
    ROUTES.exam,
    ROUTES.profile,
    ROUTES.certifications,
    ROUTES.certification,
    ROUTES.attempt,
  ]);
});

describe("currentRouteParams", () => {
  afterEach(() => {
    window.history.replaceState(null, "", "/");
  });

  test("reads params from the current URL", () => {
    window.history.replaceState(null, "", "/exams/e1?tab=1");

    expect(currentRouteParams(ROUTES.exam)).toEqual({ examId: "e1" });
  });

  test("throws when the URL does not match", () => {
    window.history.replaceState(null, "", "/exams");

    expect(() => currentRouteParams(ROUTES.exam)).toThrow("/exams does not match /exams/:examId");
  });
});
