// Boots the full ColabLife stack for the end-to-end tests and runs the API in
// the foreground so Playwright's `webServer` manages its lifecycle.
//
// Steps: ensure a dedicated PostgreSQL container is up, reset its database to a
// clean slate, apply the public migrations, build the Elm SPA, then exec the API
// binary (serving the SPA on the same origin — see ADR 0011).
//
// A dedicated container, database port and API port keep this isolated from any
// local dev stack. PostgreSQL is provisioned with `docker run` (not compose) so
// it works wherever the Docker daemon is available.

import { execFileSync, execSync, spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const BACKEND = join(ROOT, "backend");
const FRONTEND = join(ROOT, "frontend");

const PG_CONTAINER = "colab-life-e2e-postgres";
const PG_PORT = 5544;
const DATABASE_URL = `postgres://colab:colab@localhost:${PG_PORT}/colab_life`;
const API_PORT = process.env.E2E_PORT ?? "8081";

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function run(cmd, args, opts = {}) {
  execFileSync(cmd, args, { stdio: "inherit", ...opts });
}

async function ensurePostgres() {
  const running = execSync(`docker ps -q -f name=^${PG_CONTAINER}$`).toString().trim();
  if (!running) {
    execSync(`docker rm -f ${PG_CONTAINER} 2>/dev/null || true`, { shell: "/bin/bash" });
    run("docker", [
      "run", "-d", "--name", PG_CONTAINER,
      "-e", "POSTGRES_USER=colab",
      "-e", "POSTGRES_PASSWORD=colab",
      "-e", "POSTGRES_DB=colab_life",
      "-p", `${PG_PORT}:5432`,
      "postgres:16.14",
    ]);
  }

  for (let i = 0; i < 60; i += 1) {
    try {
      execSync(`docker exec ${PG_CONTAINER} pg_isready -U colab -d colab_life`, { stdio: "ignore" });
      return;
    } catch {
      await sleep(1000);
    }
  }
  throw new Error("PostgreSQL did not become ready in time");
}

function resetDatabase() {
  // Drop and recreate so every run starts from a clean slate.
  run("docker", [
    "exec", PG_CONTAINER,
    "psql", "-U", "colab", "-d", "postgres", "-v", "ON_ERROR_STOP=1",
    "-c", "DROP DATABASE IF EXISTS colab_life WITH (FORCE)",
    "-c", "CREATE DATABASE colab_life",
  ]);
}

async function main() {
  await ensurePostgres();
  resetDatabase();

  // Public-schema migrations (tenant schemas are migrated on provisioning).
  run("cargo", ["run", "-p", "migration", "--", "up"], {
    cwd: BACKEND,
    env: { ...process.env, DATABASE_URL },
  });

  // Build the SPA the API will serve.
  run("npx", ["--no-install", "elm", "make", "src/Main.elm", "--output=dist/app.js"], { cwd: FRONTEND });
  run("cp", [join(FRONTEND, "index.html"), join(FRONTEND, "dist", "index.html")]);

  // Build then run the API binary directly so Playwright can terminate it
  // cleanly (no `cargo run` wrapper holding the real process as a grandchild).
  run("cargo", ["build", "-p", "api"], { cwd: BACKEND });
  const api = spawn(join(BACKEND, "target", "debug", "api"), [], {
    stdio: "inherit",
    env: {
      ...process.env,
      DATABASE_URL,
      JWT_SECRET: "e2e-only-insecure-secret",
      FRONTEND_DIST: join(FRONTEND, "dist"),
      PORT: API_PORT,
    },
  });

  const forward = (signal) => api.kill(signal);
  process.on("SIGTERM", () => forward("SIGTERM"));
  process.on("SIGINT", () => forward("SIGINT"));
  api.on("exit", (code) => process.exit(code ?? 0));
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
