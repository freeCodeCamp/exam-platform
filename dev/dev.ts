#!/usr/bin/env bun
// Local development front end for compose.dev.yaml. `bun run dev help` lists commands.
import { existsSync } from "node:fs";
import { randomBytes } from "node:crypto";
import { join } from "node:path";

const ROOT = join(import.meta.dir, "..");

// Slice -> compose services to start (compose starts each `depends_on` too) and long-running
// services whose logs to follow.
type Slice = { services: string[]; logs: string[]; about: string };
const SLICES: Record<string, Slice> = {
  edb: {
    services: ["migrate-edb"],
    logs: ["examiner-db"],
    about: "eDb, migrated",
  },
  cdb: {
    services: ["migrate-cdb"],
    logs: ["curriculum-db"],
    about: "cDb, database created and migrated",
  },
  s3: {
    services: ["s3-init"],
    logs: ["s3"],
    about: "SeaweedFS with bucket `curriculum`",
  },
  edd: {
    services: ["examiner-dashboard"],
    logs: ["examiner-dashboard"],
    about: "eDd server + built client, with eDb, S3",
  },
  aapi: { services: ["auth-api"], logs: ["auth-api"], about: "aAPI, with eDb" },
  wapp: {
    services: ["web-app"],
    logs: ["web-app"],
    about: "wApp, with aAPI and eDb",
  },
  all: {
    services: ["examiner-dashboard", "web-app", "migrate-cdb"],
    logs: ["examiner-dashboard", "auth-api", "web-app"],
    about: "everything",
  },
};

// Interpolation only: database passwords. Apps read their own `.env` as `env_file`.
const ENV_FILES = ["databases/examiner/.env", "databases/curriculum/.env"];

// Generated when a `.env` is created from `.env.example` and the value is empty.
const SECRETS: Record<string, () => string> = {
  EDB_PASSWORD: () => hex(32),
  EDB_MIGRATOR_PASSWORD: () => hex(32),
  EDB_EDD_PASSWORD: () => hex(32),
  EDB_AAPI_PASSWORD: () => hex(32),
  CDB_ROOT_PASSWORD: () => hex(32),
};

const hex = (bytes: number) => randomBytes(bytes).toString("hex");

function parseEnv(text: string): Record<string, string> {
  const vars: Record<string, string> = {};
  for (const line of text.split("\n")) {
    const m = line.match(/^([A-Z0-9_]+)=(.*)$/);
    if (m) vars[m[1]!] = m[2]!;
  }
  return vars;
}

// Creates `<dir>/.env` from `.env.example`: fills empty secrets, then `<VAR>` placeholders from
// this file. Never touches an existing `.env`.
async function ensureEnv(dir: string): Promise<boolean> {
  const target = join(ROOT, dir, ".env");
  if (existsSync(target)) return false;
  let text = await Bun.file(join(ROOT, dir, ".env.example")).text();
  text = text.replace(/^([A-Z0-9_]+)=$/gm, (line, key: string) =>
    SECRETS[key] ? `${key}=${SECRETS[key]()}` : line,
  );
  const vars = parseEnv(text);
  text = text.replace(/<([A-Z0-9_]+)>/g, (m, key: string) => vars[key] ?? m);
  await Bun.write(target, text);
  console.log(`created ${dir}/.env from .env.example`);
  return true;
}

// Database passwords. Apps need no `.env` in development: eDd runs with MOCK_AUTH.
async function bootstrap() {
  await ensureEnv("databases/examiner");
  await ensureEnv("databases/curriculum");
}

function composeCmd(args: string[]): string[] {
  const envFiles = ENV_FILES.filter((f) => existsSync(join(ROOT, f))).flatMap((f) => [
    "--env-file",
    f,
  ]);
  return ["docker", "compose", "-f", "compose.dev.yaml", ...envFiles, ...args];
}

async function compose(args: string[]): Promise<number> {
  const proc = Bun.spawn(composeCmd(args), {
    cwd: ROOT,
    stdio: ["inherit", "inherit", "inherit"],
  });
  return proc.exited;
}

async function composeOutput(args: string[]): Promise<string> {
  const proc = Bun.spawn(composeCmd(args), { cwd: ROOT, stdio: ["ignore", "pipe", "inherit"] });
  const out = await proc.stdout.text();
  if ((await proc.exited) !== 0) process.exit(1);
  return out;
}

// `services` plus everything they transitively `depends_on`, split by restart policy:
// `restart: "no"` marks a one-shot (database init, migration, bucket creation).
async function withDependencies(
  services: string[],
): Promise<{ all: string[]; oneShots: string[]; longRunning: string[] }> {
  const config = JSON.parse(await composeOutput(["config", "--format", "json"])) as {
    services: Record<string, { restart?: string; depends_on?: Record<string, unknown> }>;
  };
  const seen = new Set<string>();
  const visit = (name: string) => {
    if (seen.has(name)) return;
    seen.add(name);
    Object.keys(config.services[name]?.depends_on ?? {}).forEach(visit);
  };
  services.forEach(visit);
  const all = [...seen];
  const isOneShot = (name: string) => config.services[name]?.restart === "no";
  return {
    all,
    oneShots: all.filter(isOneShot),
    longRunning: all.filter((name) => !isOneShot(name)),
  };
}

// Like `docker compose up -d`, but one-shots count as ready once they exit 0. `--wait` treats an
// exited one-shot as a failure, or returns before a slow one has finished.
async function upDetached(services: string[]): Promise<boolean> {
  const { all, oneShots, longRunning } = await withDependencies(services);
  if ((await compose(["up", "--detach", "--build", ...all])) !== 0) return false;
  if (oneShots.length) {
    await compose(["wait", ...oneShots]);
    const containers = (await composeOutput(["ps", "--all", "--format", "json", ...oneShots]))
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line) as { Service: string; State: string; ExitCode: number });
    const failed = oneShots.filter(
      (name) =>
        !containers.some((c) => c.Service === name && c.State === "exited" && c.ExitCode === 0),
    );
    if (failed.length) {
      console.error(`failed: ${failed.join(", ")}`);
      return false;
    }
  }
  return !longRunning.length || (await compose(["up", "--detach", "--wait", ...longRunning])) === 0;
}

function resolve(slices: string[], key: "services" | "logs"): string[] {
  const unknown = slices.filter((s) => !SLICES[s]);
  if (unknown.length) {
    console.error(`unknown slice: ${unknown.join(", ")}`);
    help();
    process.exit(2);
  }
  return [...new Set(slices.flatMap((s) => SLICES[s]![key]))];
}

function help() {
  const slices = Object.entries(SLICES)
    .map(([name, { about }]) => `  ${name.padEnd(6)} ${about}`)
    .join("\n");
  console.log(`Usage: bun run dev <slice>...        build, start, follow logs; Ctrl-C stops
       bun run dev -d <slice>...      build, start in background, wait until healthy
       bun run dev logs [slice]...    follow logs
       bun run dev ps                 list containers
       bun run dev down               stop everything, keep data
       bun run dev reset              stop everything, delete data volumes
       bun run dev compose <args>...  any docker compose command on compose.dev.yaml

Slices:
${slices}

Example: bun run dev edd`);
}

let argv = process.argv.slice(2);
if (argv[0] === "up") argv = argv.slice(1);
const [cmd, ...rest] = argv;

switch (cmd) {
  case undefined:
  case "help":
  case "-h":
  case "--help":
    help();
    break;
  case "down":
    process.exit(await compose(["down", ...rest]));
  case "reset":
    process.exit(await compose(["down", "--volumes", ...rest]));
  case "ps":
    process.exit(await compose(["ps", "--all", ...rest]));
  case "compose":
    process.exit(await compose(rest));
  case "logs":
    process.exit(await compose(["logs", "--follow", "--tail", "100", ...resolve(rest, "logs")]));
  default: {
    // Slices only: other commands pass `-d` through unchanged.
    const DETACH = ["-d", "--detach"];
    const detach = argv.some((a) => DETACH.includes(a));
    const slices = argv.filter((a) => !DETACH.includes(a));
    const services = resolve(slices, "services");
    await bootstrap();
    if (detach) {
      if (!(await upDetached(services))) {
        console.error("startup failed: `bun run dev ps`, then `bun run dev logs <slice>`");
        process.exit(1);
      }
      process.exit(await compose(["ps", "--all"]));
    }
    // Like `docker compose up`: attached, Ctrl-C stops the slices and their dependencies. Logs
    // from the slices' long-running services only. Ctrl-C reaches compose through the terminal:
    // ignore it here and wait for compose to finish stopping.
    process.on("SIGINT", () => {});
    const attach = resolve(slices, "logs").flatMap((s) => ["--attach", s]);
    process.exit(
      await compose(["up", "--build", ...attach, ...(await withDependencies(services)).all]),
    );
  }
}
