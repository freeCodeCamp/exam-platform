// Runs exam-platform.toml's deploy Action script against a fake Komodo and clock. `bun test`.
import { describe, expect, test } from "bun:test";
import config from "./exam-platform.toml";

const script = new Bun.Transpiler({ loader: "ts" }).transformSync(
  config.action.find((a: { name: string }) => a.name === "exam-platform-deploy").config.file_contents,
);
const AsyncFunction = (async () => {}).constructor as new (...args: string[]) => (...args: unknown[]) => Promise<void>;
const action = new AsyncFunction("komodo", "ARGS", "setTimeout", "Date", script);

const image = (app: string, digit: string) => `registry.digitalocean.com/exam-platform/${app}@sha256:${digit.repeat(64)}`;

type Result = "success" | "failed" | "unknown";

// `results` answers each execution in order; "unknown" never completes.
function run(app: string, requested: string, results: Result[], vars: Record<string, string> = {}) {
  const variables = new Map(Object.entries(vars));
  const executed: { operation: string; image?: string }[] = [];
  const updates = new Map<string, Result>();
  let clock = 0;
  const komodo = {
    async read(request: string, params: { id?: string }) {
      switch (request) {
        case "ListVariables":
          return [...variables].map(([name, value]) => ({ name, value }));
        case "GetStack":
          return { _id: { $oid: "stack" } };
        case "GetStackActionState":
          return { deploying: false };
        case "GetUpdate": {
          const result = updates.get(params.id!);
          return result === "unknown"
            ? { status: "InProgress", logs: [] }
            : { status: "Complete", success: result === "success", logs: [] };
        }
      }
      throw new Error(`unexpected read ${request}`);
    },
    async write(request: string, params: { name: string; value: string }) {
      if (request !== "UpdateVariableValue" && request !== "CreateVariable") throw new Error(request);
      variables.set(params.name, params.value);
    },
    async execute(operation: string) {
      const prefix = `EXAM_PLATFORM_${app.toUpperCase().replaceAll("-", "_")}`;
      const name = operation === "RunStackService" ? `${prefix}_MIGRATE_IMAGE` : `${prefix}_IMAGE`;
      executed.push({ operation, image: variables.get(name) });
      const id = String(executed.length);
      updates.set(id, results[executed.length - 1] ?? "success");
      return { _id: { $oid: id } };
    },
  };
  const fakeTimeout = (resolve: () => void, ms: number) => {
    clock += ms;
    queueMicrotask(resolve);
  };
  const done = action(komodo, { APP: app, IMAGE: requested }, fakeTimeout, { now: () => clock });
  return { done, executed, variables };
}

const healthy = {
  EXAM_PLATFORM_AUTH_API_IMAGE: image("auth-api", "a"),
  EXAM_PLATFORM_AUTH_API_IMAGE_HEALTHY: image("auth-api", "a"),
  EXAM_PLATFORM_AUTH_API_IMAGE_PREVIOUS: image("auth-api", "0"),
};

describe("backend deploy", () => {
  test("migrates from the new image, then deploys it", async () => {
    const { done, executed, variables } = run("auth-api", image("auth-api", "b"), ["success", "success"], healthy);
    await done;
    expect(executed).toEqual([
      { operation: "RunStackService", image: image("auth-api", "b") },
      { operation: "DeployStack", image: image("auth-api", "b") },
    ]);
    expect(variables.get("EXAM_PLATFORM_AUTH_API_IMAGE_HEALTHY")).toBe(image("auth-api", "b"));
    expect(variables.get("EXAM_PLATFORM_AUTH_API_IMAGE_PREVIOUS")).toBe(image("auth-api", "a"));
  });

  for (const result of ["failed", "unknown"] as const) {
    test(`${result} migration leaves the running app untouched`, async () => {
      const { done, executed, variables } = run("auth-api", image("auth-api", "b"), [result], healthy);
      await expect(done).rejects.toThrow(/not deployed/);
      expect(executed.map((e) => e.operation)).toEqual(["RunStackService"]);
      for (const [name, value] of Object.entries(healthy)) expect(variables.get(name)).toBe(value);
    });
  }

  test("`previous` migrates from the previous image too", async () => {
    const { done, executed } = run("auth-api", "previous", ["success", "success"], healthy);
    await done;
    expect(executed).toEqual([
      { operation: "RunStackService", image: image("auth-api", "0") },
      { operation: "DeployStack", image: image("auth-api", "0") },
    ]);
  });

  test("rolls back the app image only, without migrating again", async () => {
    const { done, executed, variables } = run("auth-api", image("auth-api", "b"), ["success", "failed", "success"], healthy);
    await expect(done).rejects.toThrow(/rolled back/);
    expect(executed).toEqual([
      { operation: "RunStackService", image: image("auth-api", "b") },
      { operation: "DeployStack", image: image("auth-api", "b") },
      { operation: "DeployStack", image: image("auth-api", "a") },
    ]);
    expect(variables.get("EXAM_PLATFORM_AUTH_API_IMAGE_HEALTHY")).toBe(image("auth-api", "a"));
  });

  test("either backend can deploy first", async () => {
    const { done, executed } = run("examiner-dashboard", image("examiner-dashboard", "c"), ["success", "success"]);
    await done;
    expect(executed.map((e) => e.operation)).toEqual(["RunStackService", "DeployStack"]);
  });
});

test("web-app skips migrations", async () => {
  const { done, executed } = run("web-app", image("web-app", "d"), ["success"]);
  await done;
  expect(executed.map((e) => e.operation)).toEqual(["DeployStack"]);
});
