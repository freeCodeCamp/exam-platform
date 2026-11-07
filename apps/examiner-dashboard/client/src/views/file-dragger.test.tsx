/* @vitest-environment jsdom */
import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

import type { RecordedObject } from "@/utils/fetch";
import { camperbot } from "../../test-utils/session";
import { formatBytes } from "@/utils/objects";
import { FileDragger } from "./file-dragger.tsx";

vi.mock("@/utils/navigate");

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});

const diagramHash = "a".repeat(64);
const diagram: RecordedObject = {
  sha256: diagramHash,
  objectKey: `sha256/${diagramHash}.png`,
  contentType: "image/png",
  byteSize: 2048,
  fileName: "diagram.png",
  createdAt: "2026-11-05T10:00:00Z",
  url: "https://bucket.example/diagram.png?X-Amz-Signature=abc",
};

const listeningHash = "b".repeat(64);
const listening: RecordedObject = {
  ...diagram,
  sha256: listeningHash,
  objectKey: `sha256/${listeningHash}.mp3`,
  contentType: "audio/mpeg",
  fileName: "listening-1.mp3",
  url: "https://bucket.example/listening.mp3?X-Amz-Signature=abc",
};

// A logged-in session, and in-memory objects behind /api/objects. `failUpload`: POST stores the
// first file, then fails.
function mockServer(initial: RecordedObject[], { failUpload = false } = {}) {
  const stored = [...initial];
  const fetch = vi.fn((url: string, init?: RequestInit) => {
    if (url === "/auth/session") {
      return Promise.resolve(Response.json(camperbot));
    }
    if (url === "/api/objects" && init?.method === "POST") {
      const body = init.body as FormData;
      const uploaded: RecordedObject[] = body.getAll("file").map((file, i) => ({
        sha256: String(i).repeat(64),
        objectKey: `sha256/${String(i).repeat(64)}.png`,
        contentType: "image/png",
        byteSize: (file as File).size,
        fileName: (file as File).name,
        createdAt: "2026-11-06T10:00:00Z",
        url: "https://bucket.example/new",
      }));
      if (failUpload) {
        stored.unshift(...uploaded.slice(0, 1));
        return Promise.resolve(
          new Response("b.png: object storage request failed. Stored before it: a.png", {
            status: 502,
          }),
        );
      }
      stored.unshift(...uploaded);
      return Promise.resolve(Response.json(uploaded, { status: 201 }));
    }
    if (url === "/api/objects") {
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

  choose(screen, [image]);
  fireEvent.click(await screen.findByRole("button", { name: /Upload 1 file/ }));

  expect(await screen.findByText("0".repeat(64))).toBeInTheDocument();
  const post = fetch.mock.calls.find(([, init]) => init?.method === "POST");
  const body = post?.[1]?.body;
  expect(body).toBeInstanceOf(FormData);
  expect((body as FormData).getAll("file")).toEqual([image]);
  expect(screen.getByRole("button", { name: /Upload 0 files/ })).toBeDisabled();
});

function choose(screen: ReturnType<typeof render>, files: File[]) {
  // Hidden from the accessibility tree: the dropzone is the accessible control.
  const input = screen.container.querySelector<HTMLInputElement>("input[type=file]");
  fireEvent.input(input!, { target: { files } });
}

test("accepts types browsers report under other names, by extension", async () => {
  mockServer([]);
  const screen = render(<FileDragger />);
  await screen.findByText("No files yet.");

  choose(screen, [
    new File(["m4a"], "listening.m4a", { type: "audio/x-m4a" }),
    new File(["wav"], "listening.wav", { type: "audio/x-wav" }),
    new File(["svg"], "diagram.svg", { type: "image/svg+xml" }),
  ]);

  expect(await screen.findByRole("button", { name: /Upload 2 files/ })).toBeEnabled();
  expect(screen.getByText(/diagram\.svg: file invalid type/)).toBeInTheDocument();
});

test("after a failed upload, shows the error and lists what was stored", async () => {
  mockServer([], { failUpload: true });
  const screen = render(<FileDragger />);
  await screen.findByText("No files yet.");

  choose(screen, [
    new File(["a"], "a.png", { type: "image/png" }),
    new File(["b"], "b.png", { type: "image/png" }),
  ]);
  fireEvent.click(await screen.findByRole("button", { name: /Upload 2 files/ }));

  expect(await screen.findByText(/Stored before it: a\.png/)).toBeInTheDocument();
  expect(await screen.findByText("0".repeat(64))).toBeInTheDocument();
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
