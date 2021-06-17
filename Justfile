shell_bin := env_var_or_default("SHELL", "bash")

# ---
# TOP LEVEL COMMANDS
# ---

# Build everything
#build: build-client build-server

# Format everything using rustfmt and prettier
fmt: (cargo-fmt-all)

@cargo-fmt-all *args:
    cargo fmt --all {{args}}

# Check Rust formatting
check-rust-formatting: (cargo-fmt-all '--' '--check')

# ---
# SERVER
# ---

# Build the cloudapi in release mode, equivalent to `cargo build --release`
build-server-release:
    cargo build --package server --release

# Start a local instance of the Rust cloudapi
start-server:
    #!/usr/bin/env {{shell_bin}}

    cd server

    RUST_LOG=server=debug \
    POSTGRES_URL=postgres://postgres:postgres@localhost:5431/contacts \
    cargo run

# Run Rust tests in the cloudapi folder
test-server:
    #!/usr/bin/env bash
    cd server
    cargo test --package server --release

# Generate Rustdoc cloudapi docs
doc-server:
    cargo doc --package server --no-deps --document-private-items --release

# Build the `seed_db` binary

# ---
# FRONTEND
# ---

# Start the frontend on http://localhost:1234
start-client:
    #!/usr/bin/env bash
    cd client
    trunk serve

# Run frontend tests
test-client:
    cd client
    cargo test --package client --release

# ---
# DOCKER
# ---

# Start required Docker images for local development
dc-up *args:
    docker-compose -f docker-compose.dev.yaml up {{args}}