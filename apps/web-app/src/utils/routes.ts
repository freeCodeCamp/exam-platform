export const ROUTES = {
  home: "/",
  login: "/login",
  exams: "/exams",
  exam: "/exams/:examId",
  profile: "/users/:userId/profile",
  certifications: "/users/:userId/certifications",
  certification: "/users/:userId/certifications/:certificationId",
  attempt: "/attempts/:attemptId",
} as const;

export const PLACEHOLDER = "_";

type ParamNames<P extends string> = P extends `${string}:${infer Name}/${infer Rest}`
  ? Name | ParamNames<Rest>
  : P extends `${string}:${infer Name}`
    ? Name
    : never;

export type RouteParams<P extends string> = Record<ParamNames<P>, string>;

const PARAM = /:(\w+)/g;

export const DYNAMIC_ROUTES: string[] = Object.values(ROUTES).filter((pattern) =>
  pattern.includes(":"),
);

export function buildPath<P extends string>(pattern: P, params: RouteParams<P>): string {
  return pattern.replace(PARAM, (_, name: ParamNames<P>) => encodeURIComponent(params[name]));
}

export function placeholderPath(pattern: string): string {
  return pattern.replace(PARAM, PLACEHOLDER);
}

export function matchPath<P extends string>(pattern: P, pathname: string): RouteParams<P> | null {
  const names: string[] = [];
  const source = pattern.replace(PARAM, (_, name: string) => {
    names.push(name);
    return "([^/]+)";
  });
  const match = new RegExp(`^${source}/?$`).exec(pathname);
  if (!match) return null;

  try {
    return Object.fromEntries(
      names.map((name, i) => [name, decodeURIComponent(match[i + 1]!)]),
    ) as RouteParams<P>;
  } catch {
    return null;
  }
}

export function currentRouteParams<P extends string>(pattern: P): RouteParams<P> {
  const { pathname } = window.location;
  const params = matchPath(pattern, pathname);
  if (!params) throw new Error(`${pathname} does not match ${pattern}`);
  return params;
}
