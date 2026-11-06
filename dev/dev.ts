#!/usr/bin/env bun
// Local development front end for compose.dev.yaml. `bun run dev help` lists commands.
import { existsSync } from "node:fs";
import { randomBytes } from "node:crypto";
import { join } from "node:path";

const ROOT = join(import.meta.dir, "..");

// Slice -> compose services. Compose starts each service's `depends_on` too.
const SLICES: Record<string, { services: string[]; about: string }> = {
  edb: { services: ["migrate-edb"], about: "eDb, migrated" },
  cdb: { services: ["migrate-cdb"], about: "cDb, database created and migrated" },
  s3: { services: ["s3-init"], about: "SeaweedFS with bucket `curriculum`" },
  edd: { services: ["examiner-dashboard"], about: "eDd server + built client, with eDb, S3" },
  aapi: { services: ["auth-api"], about: "aAPI, with eDb" },
  wapp: { services: ["web-app"], about: "wApp, with aAPI and eDb" },
  all: { services: ["examiner-dashboard", "web-app", "migrate-cdb"], about: "everything" },
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

async function compose(args: string[]): Promise<number> {
  const envFiles = ENV_FILES.filter((f) => existsSync(join(ROOT, f))).flatMap((f) => [
    "--env-file",
    f,
  ]);
  const proc = Bun.spawn(["docker", "compose", "-f", "compose.dev.yaml", ...envFiles, ...args], {
    cwd: ROOT,
    stdio: ["inherit", "inherit", "inherit"],
  });
  return proc.exited;
}

function servicesFor(slices: string[]): string[] {
  const unknown = slices.filter((s) => !SLICES[s]);
  if (unknown.length) {
    console.error(`unknown slice: ${unknown.join(", ")}`);
    help();
    process.exit(2);
  }
  return [...new Set(slices.flatMap((s) => SLICES[s]!.services))];
}

function help() {
  const slices = Object.entries(SLICES)
    .map(([name, { about }]) => `  ${name.padEnd(6)} ${about}`)
    .join("\n");
  console.log(`Usage: bun run dev <slice>...        build, start and follow logs (Ctrl-C detaches)
       bun run dev logs [slice]...    follow logs
       bun run dev ps                 list containers
       bun run dev down               stop everything, keep data
       bun run dev reset              stop everything, delete data volumes
       bun run dev compose <args>...  any docker compose command on compose.dev.yaml

Slices:
${slices}

Example: bun run dev edd`);
}

const argv = process.argv.slice(2);
const [cmd, ...rest] = argv[0] === "up" ? argv.slice(1) : argv;

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
    process.exit(await compose(["logs", "--follow", "--tail", "100", ...servicesFor(rest)]));
  default: {
    const slices = [cmd, ...rest];
    const services = servicesFor(slices);
    await bootstrap();
    const code = await compose(["up", "--detach", "--build", "--wait", ...services]);
    if (code !== 0) {
      console.error("startup failed: `bun run dev ps`, then `bun run dev logs <slice>`");
      process.exit(code);
    }
    // Long-running services only: one-shots have exited.
    const longRunning = services.filter((s) => !/^(migrate|s3-init)/.test(s));
    if (longRunning.length === 0) {
      await compose(["ps", "--all"]);
      break;
    }
    await compose(["logs", "--follow", "--tail", "50", ...longRunning]);
  }
}
