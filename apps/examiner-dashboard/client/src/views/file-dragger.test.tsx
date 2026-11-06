/* @vitest-environment jsdom */
import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

import type { StoredFile } from "@/utils/fetch";
import { camperbot } from "../../test-utils/session";
import { FileDragger, formatBytes } from "./file-dragger.tsx";

vi.mock("@/utils/navigate");

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});

const diagramHash = "a".repeat(64);
const diagram: StoredFile = {
  sha256: diagramHash,
  key: `sha256/${diagramHash}.png`,
  contentType: "image/png",
  size: 2048,
  fileName: "diagram.png",
  createdAt: "2026-11-05T10:00:00Z",
  url: "https://bucket.example/diagram.png?X-Amz-Signature=abc",
};

const listeningHash = "b".repeat(64);
const listening: StoredFile = {
  ...diagram,
  sha256: listeningHash,
  key: `sha256/${listeningHash}.mp3`,
  contentType: "audio/mpeg",
  fileName: "listening-1.mp3",
  url: "https://bucket.example/listening.mp3?X-Amz-Signature=abc",
};

// A logged-in session, and in-memory assets behind /api/files.
function mockServer(initial: StoredFile[]) {
  const stored = [...initial];
  const fetch = vi.fn((url: string, init?: RequestInit) => {
    if (url === "/auth/session") {
      return Promise.resolve(Response.json(camperbot));
    }
    if (url === "/api/files" && init?.method === "POST") {
      const body = init.body as FormData;
      const uploaded: StoredFile[] = body.getAll("file").map((file, i) => ({
        sha256: String(i).repeat(64),
        key: `sha256/${String(i).repeat(64)}.png`,
        contentType: "image/png",
        size: (file as File).size,
        fileName: (file as File).name,
        createdAt: null,
        url: "https://bucket.example/new",
      }));
      stored.unshift(...uploaded);
      return Promise.resolve(Response.json(uploaded, { status: 201 }));
    }
    if (url === "/api/files") {
      return Promise.resolve(Response.json(stored));
    }
    return Promise.resolve(new Response("not found", { status: 404 }));
  });
  vi.stubGlobal("fetch", fetch);
  return fetch;
}

test("lists uploaded files by name and hash, with image previews", async () => {
  mockServer([diagram]);

  const screen = render(<FileDragger />);

  expect(await screen.findByText(diagram.sha256)).toBeInTheDocument();
  expect(screen.getByText(diagram.fileName)).toBeInTheDocument();
  expect(screen.getByRole("img", { name: diagram.fileName })).toHaveAttribute("src", diagram.url);
  expect(screen.getByText(/2\.0 KiB/)).toBeInTheDocument();
});

test("previews audio with a player", async () => {
  mockServer([listening]);

  const screen = render(<FileDragger />);

  await screen.findByText(listening.fileName);
  const audio = screen.container.querySelector("audio");
  expect(audio).toHaveAttribute("src", listening.url);
  expect(audio).toHaveAttribute("controls");
});

test("says when there are no files", async () => {
  mockServer([]);

  const screen = render(<FileDragger />);

  expect(await screen.findByText("No files yet.")).toBeInTheDocument();
});

test("uploads chosen files, then lists them", async () => {
  const fetch = mockServer([]);
  const screen = render(<FileDragger />);
  await screen.findByText("No files yet.");
  const image = new File(["png bytes"], "diagram.png", { type: "image/png" });

  // Hidden from the accessibility tree: the dropzone is the accessible control.
  const input = screen.container.querySelector<HTMLInputElement>("input[type=file]");
  fireEvent.input(input!, { target: { files: [image] } });
  fireEvent.click(await screen.findByRole("button", { name: /Upload 1 file/ }));

  expect(await screen.findByText("0".repeat(64))).toBeInTheDocument();
  const post = fetch.mock.calls.find(([, init]) => init?.method === "POST");
  const body = post?.[1]?.body;
  expect(body).toBeInstanceOf(FormData);
  expect((body as FormData).getAll("file")).toEqual([image]);
  expect(screen.getByRole("button", { name: /Upload 0 files/ })).toBeDisabled();
});

test("upload button is disabled until files are chosen", async () => {
  mockServer([]);

  const screen = render(<FileDragger />);

  expect(await screen.findByRole("button", { name: /Upload 0 files/ })).toBeDisabled();
});

test("formats sizes", () => {
  expect(formatBytes(512)).toBe("512 B");
  expect(formatBytes(1536)).toBe("1.5 KiB");
  expect(formatBytes(10 * 1024 * 1024)).toBe("10.0 MiB");
});
