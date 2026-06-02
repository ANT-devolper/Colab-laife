# ColabLife task runner. Run `just` to list recipes.
# Requires: just, the Rust toolchain, Node, and (for the database) Docker.

# Local dev database connection string used by `run` and `migrate`.
export DATABASE_URL := env_var_or_default("DATABASE_URL", "postgres://colab:colab@localhost:5432/colab_life")

# Secret used to sign session JWTs locally. Dev-only — set a strong, protected
# secret via the environment in real deployments.
export JWT_SECRET := env_var_or_default("JWT_SECRET", "dev-only-insecure-secret-change-me")

# Directory holding the built Elm SPA. When set, `run` serves it on the same
# origin as the API (see ADR 0011); built by `frontend-build`.
export FRONTEND_DIST := justfile_directory() / "frontend/dist"

# List available recipes.
default:
    @just --list

# Install/download every dependency and prepare the dev environment.
setup:
    cd backend && cargo build
    cd frontend && npm install
    cd e2e && npm install
    cd e2e && npx playwright install

# Run the full test suite (backend unit + integration, frontend unit, e2e).
test:
    cd backend && cargo test
    cd frontend && npm test
    cd e2e && npm test

# Build the Elm SPA into frontend/dist (compiled JS + the HTML shell).
frontend-build:
    cd frontend && npx elm make src/Main.elm --output=dist/app.js
    cp frontend/index.html frontend/dist/index.html

# Run the backend API, serving the built SPA from the same origin (see ADR 0011).
run: frontend-build
    cd backend && cargo run -p api

# Run the end-to-end tests (Playwright boots the full stack via scripts/e2e-stack.mjs).
e2e *ARGS:
    cd e2e && npm test {{ARGS}}

# Run database migrations: `just migrate up`, `just migrate down`, `just migrate status`...
migrate *ARGS:
    cd backend && cargo run -p migration -- {{ARGS}}

# Format Rust and Elm code.
fmt:
    cd backend && cargo fmt
    cd frontend && npm run format

# Start the dev PostgreSQL (needs Docker).
db-up:
    docker compose up -d postgres

# Stop the dev PostgreSQL.
db-down:
    docker compose down
