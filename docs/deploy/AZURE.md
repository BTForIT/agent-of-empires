# Azure Container Apps deployment (`aoe serve`)

> **Scope:** this document and the files it references (`/Dockerfile`,
> `/.dockerignore`, `/.github/workflows/azure-container-build.yml`) are a
> **fork-only addition** maintained under `BTForIT/agent-of-empires`.
> Upstream `njbrake/agent-of-empires` stays unmodified so we remain
> mergeable. No Rust source or Cargo files are changed.

This is the ForIT deploy recipe for running `aoe serve` as an Azure
Container App (`aoe-serve`) alongside the existing `claude-code`
container. It is consumed by the `forit-dev` MCP server and, through
it, by the Bernard oversight loop.

See also: [`docs/plans/2026-04-22-aoe-integration-design.md`][design-doc]
in the `forit-AI` repo for the full architecture rationale.

[design-doc]: https://github.com/BTForIT/forit-AI/blob/main/docs/plans/2026-04-22-aoe-integration-design.md

## Topology

```
foritairegistry.azurecr.io/aoe-serve:<sha>
            │
            ▼
┌──────────────────────────────────────────────────┐
│  Azure Container App: aoe-serve (eastus)         │
│  ┌────────────────────────────┐                  │
│  │ main: aoe serve (port 8080)│  ← HTTPS ingress │
│  │   0.5 vCPU / 1 GiB         │     IP-restricted │
│  │   scale-to-zero (min=0)    │     to forit-dev │
│  │   tmux-only (no Docker)    │     outbound     │
│  └────────────────────────────┘                  │
│  ┌────────────────────────────┐                  │
│  │ Azure Files volume         │  ← survives      │
│  │ mount → /home/aoe/.config/ │    revision      │
│  │        agent-of-empires    │    swaps         │
│  └────────────────────────────┘                  │
└──────────────────────────────────────────────────┘
```

### Why this shape

- **Single shared `aoe-serve`, not per-user.** v1 scope. Per-user
  instances land when we onboard a second user.
- **tmux-only, no Docker sandbox.** AoE's README lists Docker as
  *optional* (for per-session sandbox isolation); tmux is the only
  required prerequisite. Azure Container Apps **forbids privileged
  containers** ([MS Learn][aca-limits]), which rules out a
  `docker:dind` sidecar; Azure Container Instances forbids them too.
  For single-tenant Bernard oversight, tmux process isolation is
  sufficient. If per-session Docker sandboxing is ever needed
  (e.g. untrusted code, multi-tenant), the v2 pivot is an Azure VM
  — not a Container App.

[aca-limits]: https://learn.microsoft.com/en-us/azure/container-apps/containers#limitations
- **Scale-to-zero** because Bernard ticks on a cron, not continuously.
  Cold-start cost eats ~5s on the first tick; acceptable for an
  oversight loop that runs every 1–5 min.
- **Azure Files mount** at the Linux app dir (`$XDG_CONFIG_HOME/
  agent-of-empires`, i.e. `/home/aoe/.config/agent-of-empires` in the
  image) preserves `serve.token`, profile data, and session state
  across revisions. The design doc's shorthand `/root/.aoe` refers to
  the same concept — in practice, on Linux with the non-root user in
  the Dockerfile, the path is `/home/aoe/.config/agent-of-empires`.

## Image

Built by `.github/workflows/azure-container-build.yml` on push to
`main` or manual dispatch. Tags:

- `foritairegistry.azurecr.io/aoe-serve:<commit-sha>` (immutable)
- `foritairegistry.azurecr.io/aoe-serve:latest`

Local sanity check:

```bash
docker build -t aoe-serve:dev .
docker run --rm -p 8080:8080 \
  -e AOE_SERVE_PASSPHRASE=local-dev \
  aoe-serve:dev
# First boot writes /home/aoe/.config/agent-of-empires/serve.token
# Grab it with: docker exec <id> cat /home/aoe/.config/agent-of-empires/serve.token
```

## Environment variables consumed by `aoe serve`

`aoe serve` is the subcommand behind the Cargo `serve` feature. It reads
very few env vars directly — most behavior is driven by CLI flags. The
meaningful ones:

| Var | Source | Effect |
|-----|--------|--------|
| `AOE_SERVE_PASSPHRASE` | `clap(env = "AOE_SERVE_PASSPHRASE")` on `--passphrase` | Enables second-factor login. **Set this in production via Container App secret.** |
| `HOME` | Std env | Required — `aoe` derives `$XDG_CONFIG_HOME` / fallback `~/.agent-of-empires` from here. Set to `/home/aoe` in the image. |
| `XDG_CONFIG_HOME` | Std env | On Linux, `aoe` stores state at `$XDG_CONFIG_HOME/agent-of-empires`. Image sets `/home/aoe/.config`. |
| `RUST_LOG` | `tracing-subscriber` EnvFilter | Log level. Default `info` in the image; set `debug` for troubleshooting. |

### Auth token — important

`aoe serve` does **not** accept an `--auth-token` CLI flag. The token is
either:

1. Read from `$APP_DIR/serve.token` if present and <24h old, or
2. Generated and written there on first boot.

For the Bernard integration, we seed the token into the Azure Files
volume out-of-band (one-time setup) so `forit-dev`'s `AoeAgent` can use
a known stable value for the `AOE_AUTH_TOKEN` Container App setting.
Rotation follows AoE's built-in grace-period logic.

The `--no-auth` flag exists but is **forbidden in our deployment** —
the Container App binds `0.0.0.0` and exposes a public HTTPS endpoint
(IP-restricted, but still reachable). Token auth must stay on.

## One-time provisioning (manual, first deploy)

1. **Build + push the image** via the GH Action (push to `main` or run
   `workflow_dispatch`). Wait for the `aoe-serve:latest` tag to appear
   in ACR.
2. **Create the Container App Environment** if one doesn't exist:
   `forit-ai-aoe-env` in `rg-forit-ai-engine` (eastus).
3. **Create Azure Files share** `aoe-state` on `stforitaiengine`.
   Mount definition in the env:
   - Account: `stforitaiengine`
   - Share: `aoe-state`
   - Access: read/write, SMB
4. **Seed the auth token** (one-time):
   ```bash
   # Generate a token locally
   TOKEN=$(python3 -c 'import secrets; print(secrets.token_hex(32))')
   # Write it to the share at agent-of-empires/serve.token
   # (Use Storage Explorer or `az storage file upload`.)
   ```
5. **Create the Container App `aoe-serve`** with:
   - Image: `foritairegistry.azurecr.io/aoe-serve:latest`
   - Ingress: external, target port 8080, transport HTTPS
   - IP restriction: forit-dev App Service outbound range (verify
     current range via the App Service "Networking" blade)
   - Scale: min 0, max 1 (single shared instance)
   - CPU/mem: 0.5 vCPU / 1 GiB
   - Volume mount: Azure Files `aoe-state` at
     `/home/aoe/.config/agent-of-empires`
   - Secret: `aoe-serve-passphrase` (random, 32+ chars)
   - Env: `AOE_SERVE_PASSPHRASE` = secretref `aoe-serve-passphrase`
   - **No sidecars.** tmux-only v1; see "Why this shape" above.
6. **Verify:**
   ```bash
   TOKEN=$(cat serve.token)  # the one we seeded
   curl -sfH "Authorization: Bearer $TOKEN" \
     https://aoe-serve.<env-suffix>.eastus.azurecontainerapps.io/api/sessions
   # → [] on cold boot, 401 if token is wrong
   ```
7. **Record values in `forit-dev`:**
   - `AOE_SERVE_URL` = the ingress FQDN
   - `AOE_AUTH_TOKEN` = the seeded token

## Ongoing operations

- **Deploy a new image revision:** push to `main` → workflow builds →
  update the Container App to use `foritairegistry.azurecr.io/aoe-serve:<new-sha>`
  (or leave it on `:latest` with revision-on-new-image auto-pull).
- **Rotate auth token:** let AoE's `TokenManager` grace-period rotation
  handle it, or manually replace `/home/aoe/.config/agent-of-empires/serve.token`
  on the volume and restart the revision.
- **Read state:** `aoe-state` is an Azure Files share; browse with
  Storage Explorer. `serve.token`, `profiles/`, session state all live
  there.
- **Smoke test after deploy:** `./scripts/aoe-serve-smoke.sh` (lives in
  `forit-AI`, not this repo) — cold-boots the container, hits
  `/api/sessions` with the bearer token, asserts `[]` + 200, asserts
  `[]` with no token gets 401.

## Known caveats

- **No Docker sandbox on ACA.** ACA (and ACI) forbid privileged
  containers, which `docker:dind` requires. AoE runs in tmux-only
  mode here, which is fine for single-tenant oversight but means
  `ensure_container_terminal` will not work. If a session tries to
  create a container-backed terminal, AoE will fail that session —
  callers must pass `container: none` (or equivalent default) when
  starting sessions. Docker sandboxing = v2 Azure VM pivot.
- **Linux path vs. design-doc shorthand:** the design doc says "Azure
  Files mount at `/root/.aoe`." The actual mount is
  `/home/aoe/.config/agent-of-empires` because (a) AoE uses
  `$XDG_CONFIG_HOME/agent-of-empires` on Linux, not `~/.aoe`, and (b)
  the image runs as non-root UID 10001. Same state, same file names,
  different path.
- **`--auth-token` is not a flag.** Token lives in `serve.token` on
  disk. Any documentation that implies a CLI flag is wrong — the
  `TokenManager` generates/persists it internally.
- **No `/api/health` endpoint.** The image's HEALTHCHECK hits
  `/api/sessions` (which always responds — 401 without auth, 200 with
  it) to prove the HTTP listener is bound. "Server up" ≠ "auth
  configured correctly"; smoke-test for the latter after deploy.

## Staying mergeable with upstream

Everything in this deploy recipe is **additive**: `Dockerfile`,
`.dockerignore`, `.github/workflows/azure-container-build.yml`,
`docs/deploy/AZURE.md`. No Rust source, no `Cargo.toml`, no
`Cargo.lock`, no upstream workflow edits. If `njbrake/agent-of-empires`
ever ships their own Dockerfile at the repo root, we resolve the
conflict in our favor (our Dockerfile encodes the ForIT deploy
assumptions) and keep the rest as-is.
