# syntax=docker/dockerfile:1.7
#
# Azure Container Apps deployment image for `aoe serve`.
#
# This Dockerfile is a FORK-ONLY addition (BTForIT/agent-of-empires). Upstream
# njbrake/agent-of-empires is not modified. See docs/deploy/AZURE.md for the
# deploy shape (DinD sidecar, Azure Files mount, auth).
#
# Build:
#   docker build -t aoe-serve:dev .
# Run (local smoke test):
#   docker run --rm -p 8080:8080 aoe-serve:dev
#
# NOTE: `aoe serve` does NOT accept an --auth-token flag. It generates a
# token at first boot and stores it at $APP_DIR/serve.token
# (Linux: $XDG_CONFIG_HOME/agent-of-empires/serve.token, i.e.
# /home/aoe/.config/agent-of-empires/serve.token in this image). To inject
# a known token, mount a file at that path or seed it via an init script /
# Azure Files volume. The container listens on 0.0.0.0:8080 so Azure
# Container Apps ingress can reach it; protect it with IP restriction and
# a passphrase (AOE_SERVE_PASSPHRASE).

########################
# Stage 1: build
########################
FROM rust:1-bookworm AS builder

WORKDIR /build

# System deps for the build.
# - pkg-config / perl / make / cmake / ca-certificates: C build for the
#   vendored openssl (git2 feature) + general build glue.
# - nodejs + npm: build.rs invokes `npm ci` + `npm run build` against
#   web/ when the `serve` feature is enabled — it's NOT optional.
#   bookworm ships Node 18 which satisfies Vite; if the web/ project
#   ever pins a newer Node we switch to the NodeSource apt repo.
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        perl \
        make \
        cmake \
        ca-certificates \
        curl \
        gnupg \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# Copy the full source. We don't do the dependency-cache dance because
# the web assets are embedded via rust-embed and build.rs reads them at
# compile time — splitting the copy into a deps-only layer has no net
# speedup for this repo's layout.
COPY . .

# Build the release binary with the `serve` feature (required — the
# `aoe serve` subcommand is behind it per Cargo.toml [features]).
# --locked: fail if Cargo.lock is out of sync, no silent resolution drift.
#
# We deliberately do NOT use --mount=type=cache for cargo registry /
# target/ here. The `serve` build is split across two tools: build.rs
# drives `npm ci` + `npm run build` to produce web/dist/, then cargo
# compiles the Rust crate which embeds web/dist/ via rust-embed. The
# workdir (web/dist/) is NOT on a cache mount, but target/ would be —
# so a re-run with a stale target/ cache can resurrect macro-expanded
# artifacts that reference a web/dist/ the fresh workdir doesn't have,
# producing `folder ... does not exist` errors. CI uses BuildKit GHA
# cache which operates at the layer level and avoids this class of bug.
RUN cargo build --release --locked --features serve --bin aoe \
    && cp target/release/aoe /usr/local/bin/aoe

########################
# Stage 2: runtime
########################
FROM debian:bookworm-slim AS runtime

# Runtime deps:
# - git: AoE clones repos for sessions
# - tmux: AoE's session manager (every session is a tmux pane)
# - openssh-client: git-over-ssh + AoE's remote worktree flows
# - ca-certificates: HTTPS to GitHub/Anthropic/etc.
# - curl: HEALTHCHECK + debugging
# - tini: PID 1 signal forwarding so Container Apps can terminate cleanly
RUN apt-get update && apt-get install -y --no-install-recommends \
        git \
        tmux \
        openssh-client \
        ca-certificates \
        curl \
        tini \
    && rm -rf /var/lib/apt/lists/*

# Non-root user. UID 10001 avoids collisions with host UIDs and matches
# Azure Container Apps best practice (no shared UID range with hosts).
RUN groupadd --system --gid 10001 aoe \
    && useradd --system --uid 10001 --gid aoe --create-home --shell /bin/bash aoe

# Copy the built binary.
COPY --from=builder /usr/local/bin/aoe /usr/local/bin/aoe

# Pre-create the app dir so a volume mount inherits correct ownership.
# On Linux, AoE stores state under $XDG_CONFIG_HOME/agent-of-empires
# (default ~/.config/agent-of-empires). That's where serve.token,
# serve.pid, profiles/, and all session state live.
RUN mkdir -p /home/aoe/.config/agent-of-empires \
    && chown -R aoe:aoe /home/aoe

USER aoe
WORKDIR /home/aoe
ENV HOME=/home/aoe \
    XDG_CONFIG_HOME=/home/aoe/.config \
    RUST_LOG=info

EXPOSE 8080

# Lenient healthcheck: the server requires bearer auth, so any unauth
# request to /api/sessions returns 401 — but that still proves the
# process is up and the HTTP listener is bound. `curl --fail` treats
# 4xx/5xx as errors, so we test the TCP connect + HTTP response line
# via --output /dev/null and accept any status that came back at all.
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -sS -o /dev/null -m 4 http://127.0.0.1:8080/api/sessions || exit 1

# tini as PID 1 so SIGTERM from Container Apps reaches aoe cleanly.
ENTRYPOINT ["/usr/bin/tini", "--"]

# Bind to 0.0.0.0 so the Container Apps ingress proxy can reach us.
# AOE_SERVE_PASSPHRASE is read from env by `aoe serve` (clap `env =
# "AOE_SERVE_PASSPHRASE"` on the --passphrase flag). In production, set
# it via Container App secret binding.
CMD ["aoe", "serve", "--host", "0.0.0.0", "--port", "8080"]
