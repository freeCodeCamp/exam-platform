/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, within } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

import type { StoreEntry } from "@/utils/fetch";
import { camperbot } from "../../test-utils/session";
import { ObjectStore } from "./object-store.tsx";

vi.mock("@/utils/navigate");

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});

const diagramHash = "a".repeat(64);
const diagram: StoreEntry = {
  key: `sha256/${diagramHash}.png`,
  status: "recorded",
  record: {
    sha256: diagramHash,
    contentType: "image/png",
    fileName: "diagram.png",
    createdAt: "2026-11-05T10:00:00Z",
  },
  byteSize: 2048,
  lastModified: "2026-11-05T10:00:00Z",
  url: "https://bucket.example/diagram.png?X-Amz-Signature=abc",
};

const stray: StoreEntry = {
  key: "stray/notes.txt",
  status: "unrecorded",
  record: null,
  byteSize: 12,
  lastModified: "2026-11-04T10:00:00Z",
  url: "https://bucket.example/stray/notes.txt?X-Amz-Signature=abc",
};

const goneHash = "c".repeat(64);
const gone: StoreEntry = {
  ...diagram,
  key: `sha256/${goneHash}.png`,
  status: "missing",
  record: { ...diagram.record!, sha256: goneHash, fileName: "gone.png" },
  lastModified: null,
  url: null,
};

const diagramUrl = `/api/object-store/object?key=sha256%2F${diagramHash}.png`;

// A logged-in session, and in-memory objects behind /api/object-store.
function mockServer(initial: StoreEntry[]) {
  let stored = [...initial];
  const fetch = vi.fn((url: string, init?: RequestInit) => {
    if (url === "/auth/session") {
      return Promise.resolve(Response.json(camperbot));
    }
    if (url === "/api/object-store") {
      return Promise.resolve(Response.json(stored));
    }
    const key = new URL(url, "http://localhost").searchParams.get("key");
    if (init?.method === "PATCH") {
      const { fileName } = JSON.parse(init.body as string) as { fileName: string };
      stored = stored.map((o) =>
        o.key === key && o.record ? { ...o, record: { ...o.record, fileName } } : o,
      );
      return Promise.resolve(new Response(null, { status: 204 }));
    }
    if (init?.method === "DELETE") {
      stored = stored.filter((o) => o.key !== key);
      return Promise.resolve(new Response(null, { status: 204 }));
    }
    return Promise.resolve(new Response("not found", { status: 404 }));
  });
  vi.stubGlobal("fetch", fetch);
  return fetch;
}

// URL and body of each request with `method`.
function requests(fetch: ReturnType<typeof mockServer>, method: string) {
  return fetch.mock.calls
    .filter(([, init]) => init?.method === method)
    .map(([url, init]) => ({ url, body: init?.body }));
}

test("lists every object with its status", async () => {
  mockServer([diagram, stray, gone]);

  const screen = render(<ObjectStore />);

  expect(await screen.findByText("3 objects")).toBeInTheDocument();
  expect(screen.getByRole("img", { name: "diagram.png" })).toHaveAttribute("src", diagram.url);
  expect(screen.getByText(stray.key)).toBeInTheDocument();
  expect(screen.getByText("Unrecorded")).toBeInTheDocument();
  expect(screen.getByText("Bytes missing")).toBeInTheDocument();
  expect(screen.getAllByText(/2\.0 KiB/)).toHaveLength(2);
});

test("says when the bucket is empty", async () => {
  mockServer([]);

  const screen = render(<ObjectStore />);

  expect(await screen.findByText("The bucket is empty.")).toBeInTheDocument();
});

test("renames recorded objects only", async () => {
  const fetch = mockServer([diagram, stray]);
  const screen = render(<ObjectStore />);

  fireEvent.click(await screen.findByRole("button", { name: "Rename diagram.png" }));
  const input = screen.getByRole("textbox", { name: "File name" });
  fireEvent.change(input, { target: { value: "triangle.png" } });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));

  expect(await screen.findByText("triangle.png")).toBeInTheDocument();
  expect(requests(fetch, "PATCH")).toEqual([
    { url: diagramUrl, body: JSON.stringify({ fileName: "triangle.png" }) },
  ]);
  expect(screen.queryByRole("button", { name: `Rename ${stray.key}` })).not.toBeInTheDocument();
});

test("will not save an empty name", async () => {
  mockServer([diagram]);
  const screen = render(<ObjectStore />);

  fireEvent.click(await screen.findByRole("button", { name: "Rename diagram.png" }));
  fireEvent.change(screen.getByRole("textbox", { name: "File name" }), {
    target: { value: "  " },
  });

  expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
});

test("deletes only after confirming the consequences", async () => {
  const fetch = mockServer([diagram]);
  const screen = render(<ObjectStore />);

  fireEvent.click(await screen.findByRole("button", { name: "Delete diagram.png" }));
  const dialog = await screen.findByRole("alertdialog");
  expect(within(dialog).getByText(/loses the file/)).toBeInTheDocument();
  expect(within(dialog).getByText(/cannot be undone/)).toBeInTheDocument();
  expect(requests(fetch, "DELETE")).toHaveLength(0);

  fireEvent.click(within(dialog).getByRole("button", { name: "Delete permanently" }));

  expect(await screen.findByText("The bucket is empty.")).toBeInTheDocument();
  expect(requests(fetch, "DELETE").map((r) => r.url)).toEqual([diagramUrl]);
});

test("cancelling a delete sends nothing", async () => {
  const fetch = mockServer([diagram]);
  const screen = render(<ObjectStore />);

  fireEvent.click(await screen.findByRole("button", { name: "Delete diagram.png" }));
  const dialog = await screen.findByRole("alertdialog");
  fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));

  expect(requests(fetch, "DELETE")).toHaveLength(0);
  expect(screen.getByText("diagram.png")).toBeInTheDocument();
});

test("sends the key whole in the query, dot segments included", async () => {
  const odd: StoreEntry = { ...stray, key: "stray/../a b?.txt" };
  const fetch = mockServer([odd]);
  const screen = render(<ObjectStore />);

  fireEvent.click(await screen.findByRole("button", { name: `Delete ${odd.key}` }));
  fireEvent.click(
    within(await screen.findByRole("alertdialog")).getByRole("button", {
      name: "Delete permanently",
    }),
  );

  expect(await screen.findByText("The bucket is empty.")).toBeInTheDocument();
  expect(requests(fetch, "DELETE").map((r) => r.url)).toEqual([
    "/api/object-store/object?key=stray%2F..%2Fa+b%3F.txt",
  ]);
});
