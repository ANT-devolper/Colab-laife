# ColabLife task runner. Run `just` to list recipes.
# Requires: just, the Rust toolchain, Node, and (for the database) Docker.

# Local dev database connection string used by `run` and `migrate`.
export DATABASE_URL := env_var_or_default("DATABASE_URL", "postgres://colab:colab@localhost:5432/colab_life")

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

# Run the backend API.
run:
    cd backend && cargo run -p api

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
