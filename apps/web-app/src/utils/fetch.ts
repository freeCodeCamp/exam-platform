import { authApiUrl } from "@/config";
import { redirect } from "@/utils/navigate";
import { ROUTES } from "@/utils/routes";

export interface SessionUser {
  id: string;
  name: string;
  email: string;
}

export class HttpError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "HttpError";
  }
}

async function responseError(res: Response): Promise<HttpError> {
  const body = await res.text();
  return new HttpError(res.status, `${res.status} - ${body || res.statusText}`);
}

function authApiFetch(path: string, options?: RequestInit): Promise<Response> {
  return fetch(new URL(path, authApiUrl()), { ...options, credentials: "include" });
}

export async function authorizedFetch(path: string, options?: RequestInit): Promise<Response> {
  const res = await authApiFetch(path, options);

  if (res.status === 401) {
    redirect(ROUTES.login);
  }
  if (!res.ok) {
    throw await responseError(res);
  }

  return res;
}

export async function getSessionUser(): Promise<SessionUser | null> {
  const res = await authApiFetch("/auth/session");

  if (res.status === 401) {
    return null;
  }
  if (!res.ok) {
    throw await responseError(res);
  }

  return (await res.json()) as SessionUser;
}

export async function postDevLogin(user: { name: string; email: string }): Promise<SessionUser> {
  const res = await authApiFetch("/auth/dev/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(user),
  });

  if (!res.ok) {
    throw await responseError(res);
  }

  return (await res.json()) as SessionUser;
}

export async function deleteSession(): Promise<void> {
  await authorizedFetch("/auth/logout", { method: "DELETE" });
}
