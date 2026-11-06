// eDd server serves the client, so every request is same-origin and carries the sid cookie.
import { redirect } from "@/utils/navigate";

export interface SessionUser {
  name: string;
  email: string;
  picture: string | null;
}

// Navigated to, not fetched: the server redirects to Google's consent screen.
export const GOOGLE_LOGIN_PATH = "/auth/google/login";

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

// For every request needing login. 401 (no or expired session) sends the user to /login; other
// failures, 403 included, reject with the server's message.
export async function authorizedFetch(url: string, options?: RequestInit): Promise<Response> {
  const res = await fetch(url, options);

  if (res.status === 401) {
    redirect("/login");
  }
  if (!res.ok) {
    throw await responseError(res);
  }

  return res;
}

// `null` when logged out.
export async function getSessionUser(): Promise<SessionUser | null> {
  const res = await fetch("/auth/session");

  if (res.status === 401) {
    return null;
  }
  if (!res.ok) {
    throw await responseError(res);
  }

  return (await res.json()) as SessionUser;
}

// Only mounted when the server runs with MOCK_AUTH=true. Sets the sid cookie.
export async function postDevLogin(user: { name: string; email: string }): Promise<void> {
  const res = await fetch("/auth/dev/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(user),
  });

  if (!res.ok) {
    throw await responseError(res);
  }
}

export interface StoredObject {
  // What a question's `question_objects` row stores.
  sha256: string;
  // In the curriculum bucket: `sha256/<sha256>.<ext>`.
  objectKey: string;
  // Sniffed by the server from the bytes.
  contentType: string;
  byteSize: number;
  // As first uploaded.
  fileName: string;
  // RFC 3339.
  createdAt: string;
  // Presigned, expires in minutes. Display only: never store it.
  url: string;
}

export async function getObjects(): Promise<StoredObject[]> {
  const res = await authorizedFetch("/api/objects");
  return (await res.json()) as StoredObject[];
}

// Already-stored files come back as stored, under their first upload's name.
export async function uploadObjects(files: File[]): Promise<StoredObject[]> {
  const body = new FormData();
  for (const file of files) {
    body.append("file", file);
  }

  const res = await authorizedFetch("/api/objects", { method: "POST", body });
  return (await res.json()) as StoredObject[];
}
