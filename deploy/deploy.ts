// Runs Komodo's exam-platform-deploy Action (exam-platform.toml) and waits for it. Run by
// .github/workflows/deploy.yml. Env:
//   APP, IMAGE                          the Action's ARGS
//   KOMODO_URL, KOMODO_API_KEY, KOMODO_API_SECRET  an API key of a user with Execute on the Action
// Exits non-zero, with a GitHub annotation, unless the Action succeeds.
import { randomBytes } from "node:crypto";
import type { Types } from "komodo_client";

// komodo_client's auth dependency reads localStorage on import, for browser logins. API keys never touch it.
globalThis.localStorage ??= {
  getItem: () => null,
  setItem: () => {},
} as unknown as Storage;
const { KomodoClient } = await import("komodo_client");

// The Action gives migrations MIGRATE_SECONDS (420s), then a rollout and its rollback DEPLOY_SECONDS (420s)
// each: keep this above the sum.
const DEADLINE_SECONDS = 1500;
const POLL_SECONDS = 5;
const REQUEST_SECONDS = 30;

function fail(message: string): never {
  console.log(`::error::${message}`);
  process.exit(1);
}

function env(name: string): string {
  return process.env[name] || fail(`${name} is not set`);
}

const sleep = (seconds: number) =>
  new Promise((resolve) => setTimeout(resolve, seconds * 1000));
// The client rejects with { status, result: { error, trace } }.
const describe = (err: unknown) =>
  err instanceof Error
    ? err.message
    : JSON.stringify((err as { result?: unknown })?.result ?? err);

async function within<T>(call: Promise<T>): Promise<T> {
  let timer: Timer | undefined;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(
      () => reject(new Error(`No response in ${REQUEST_SECONDS}s`)),
      REQUEST_SECONDS * 1000,
    );
  });
  try {
    return await Promise.race([call, timeout]);
  } finally {
    clearTimeout(timer);
  }
}

const app = env("APP");
const image = env("IMAGE");
// Fails fast with an annotation. The Action validates IMAGE again and is authoritative.
const digest = new RegExp(
  `^registry\\.digitalocean\\.com/[a-z0-9-]+/${app}@sha256:[0-9a-f]{64}$`,
);
if (image !== "previous" && !digest.test(image)) {
  fail(
    `image must be registry.digitalocean.com/<registry>/${app}@sha256:<digest>, or previous`,
  );
}

const komodo = KomodoClient(env("KOMODO_URL").replace(/\/$/, ""), {
  type: "api-key",
  params: { key: env("KOMODO_API_KEY"), secret: env("KOMODO_API_SECRET") },
});

// Rides out a Core restart or a dropped connection: fails after 4 polls in a row fail.
async function waitFor(id: string): Promise<Types.Update> {
  const deadline = Date.now() + DEADLINE_SECONDS * 1000;
  let failures = 0;
  while (Date.now() < deadline) {
    await sleep(POLL_SECONDS);
    try {
      const update = await within(komodo.read("GetUpdate", { id }));
      failures = 0;
      if (update.status === "Complete") return update;
    } catch (err) {
      if (++failures === 4)
        fail(`GetUpdate failed for Komodo update ${id}: ${describe(err)}`);
    }
  }
  fail(`Timed out waiting for Komodo update ${id}`);
}

let started: Types.Update;
try {
  // No retry: a repeat would start a second deploy.
  started = await within(
    komodo.execute("RunAction", {
      action: "exam-platform-deploy",
      args: { APP: app, IMAGE: image },
    }),
  );
} catch (err) {
  fail(`Komodo rejected RunAction: ${describe(err)}`);
}
const id = started._id?.$oid ?? fail("RunAction returned no update id");
console.log(`Komodo update ${id}`);

const update = await waitFor(id);
// Komodo's output must not run workflow commands.
const stop = randomBytes(16).toString("hex");
console.log(`::stop-commands::${stop}`);
for (const log of update.logs)
  console.log(`--- ${log.stage}\n${log.stdout}${log.stderr}`);
console.log(`::${stop}::`);
if (!update.success)
  fail(`Deploy failed: see the Komodo log above (update ${id})`);
// A request abandoned by `within` would otherwise hold the process open.
process.exit(0);
