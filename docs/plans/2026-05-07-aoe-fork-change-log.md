# AoE fork — change log + upstream-update strategy

**Generated 2026-05-07.** Purpose: end the "we can't update because branches are a mess" pain. Every divergent commit is inventoried; every commit is mapped to a PR (open / merged / not-yet) or marked local-only; and there is one concrete workflow for keeping current with upstream.

## Topology

| | |
|-|-|
| Upstream | `https://github.com/njbrake/agent-of-empires` (njbrake) |
| Our public fork | `origin` = `https://github.com/BTForIT/agent-of-empires` |
| Upstream HEAD | `64a3998 chore: bump version to 1.5.2` (2026-05-05) |
| `origin/main` vs `upstream/main` | identical (0 commits ahead) — our `main` is a clean upstream mirror |
| Installed binary | `~/.cargo/bin/aoe` v1.5.0, built 2026-05-06 from `feat/attention-flat-no-groups` |
| Active "deploy" branch | `feat/attention-flat-no-groups` (94 commits ahead of upstream/main) |
| Working integration branch | `our/integration` (81 commits ahead) — being phased out |

## What's already in upstream (8 PRs merged)

These features came from us and are now in `upstream/main` ≥ 1.5.0. **No further action needed** — they auto-vanish when we rebase on upstream/main.

| PR | Branch | Title | Status |
|----|--------|-------|--------|
| [#910](https://github.com/njbrake/agent-of-empires/pull/910) | `feat/restart-all` | `aoe session restart --all` | MERGED 2026-05-05 |
| [#865](https://github.com/njbrake/agent-of-empires/pull/865) | `feat/responsive-mosh` | Responsive layout for narrow viewports | MERGED 2026-04-29 |
| [#862](https://github.com/njbrake/agent-of-empires/pull/862) | `feat/extra-nav-keys` | iPad-friendly ±10 nav | MERGED 2026-04-29 |
| [#861](https://github.com/njbrake/agent-of-empires/pull/861) | `feat/api-control` | `POST /sessions/{id}/send` + `GET /sessions/{id}/output` | MERGED 2026-04-29 |
| [#777](https://github.com/njbrake/agent-of-empires/pull/777) | `pr/attention-aging-correctness` | Last-activity column at narrow widths | MERGED 2026-04-23 |
| [#762](https://github.com/njbrake/agent-of-empires/pull/762) | `feat/last-activity-column` | Last-activity column + LastActivity sort | MERGED 2026-04-21 |
| [#756](https://github.com/njbrake/agent-of-empires/pull/756) | `feat/palette-color-mode` | Opt-in 256-color palette mode | MERGED 2026-04-20 |
| [#755](https://github.com/njbrake/agent-of-empires/pull/755) | `feat/strict-hotkeys` | Strict hotkeys mode (Shift/Ctrl on destructive) | MERGED 2026-04-20 |

## What's open in upstream (1 PR awaiting review)

| PR | Branch | Title | Status |
|----|--------|-------|--------|
| [#778](https://github.com/njbrake/agent-of-empires/pull/778) | `feat/default-view-mode` | Configurable `default_view_mode` for home screen | OPEN since 2026-04-23 |

## What is local-only and NOT yet upstreamed (73 commits)

The 73 commits below are pending in `our/integration` over `upstream/main`. Many are downstream rework of already-merged PRs (rebases produce non-equivalent patch-ids). The categorization below is what matters for upstream PR work.

### Theme A — Attention/sort/cursor behavior (the biggest local-only work)

Has not been PR'd yet. Big, coherent feature. Should become **one** upstream PR called `feat/attention-system` (folder + session archive, favorite, snooze, sort modes, cursor jumps).

| Commit | Title |
|--------|-------|
| `e4f9945` | feat(attention): folder + session archive (tier 99 + italic+dim) |
| `8f9f12f` | feat(attention): favorite session — opposite of archive, pins needs-help to top |
| `a19337b` | feat(attention): snooze session — temporary archive with auto-wake |
| `bfe694e` | feat(attention): mutual exclusion between archive / favorite / snooze |
| `48ddccf` | feat(attention): auto-unarchive/unsnooze on user interaction |
| `67592cd` | feat(attention): favorite = within-tier pin (top of respective category) |
| `4526f49` | feat(web): attention overlay (archive/favorite/snooze) in dashboard |
| `e6c973f` | fix(tui): add Snooze hotkey hint to bottom status bar |
| `4495f29`, `e1bf724` | favorite glyph rendering fixes |
| `9fec812` | fix(tui): archive/snooze override status fg color so rows visually sink |
| `e1db5c1` | feat(snooze): expand TUI duration presets 1-9 (15m → 1 week) |
| `6a91fa4` | feat(tui): snooze duration picker — 30m/1h/24h single-key choice |
| `f2d8427` | fix(tui): kill spinner on archived/snoozed rows |
| `9d63a40` | fix(session): refuse to restart archived sessions |
| `25307f0` | feat(tui): restart-session keybind (e/E/F5) + footer hints |

**Status:** branch `feat/attention-archive-and-signal-hook` (32 commits ahead) is the source. Needs cleanup, splitting, then PR.

### Theme B — Cursor jumping after operations

Should bundle with Theme A or PR separately as `feat/attention-cursor-jumps`.

| Commit | Title |
|--------|-------|
| `0635fb8` | feat(tui): jump cursor to next attention item after archive |
| `7510add` | feat(tui): jump cursor to top of Attention after message send |
| `54c2e95` | feat(tui): Attention sort jumps cursor to top on attach return |
| `6fd2d7b` | fix(tui): Attention cursor skips returning session on attach return |
| `8976cdb` | fix(tui): cursor-jump-after-send survives reload |

### Theme C — last_accessed_at correctness (foundation for attention sorting)

Already partially upstreamed in #762/#777 but several follow-up fixes remain local.

| Commit | Title |
|--------|-------|
| `54ffc63` | feat(session): track last_accessed_at on status change |
| `6a527ad` | fix(tui): populate last_accessed_at from tmux session activity |
| `2d3db7c` | fix(tui): plumb last_accessed_at through StatusUpdate |
| `d40994d` | feat(tui): Attention sort + drop flickering last-activity column |
| `324c378` | fix(attention): stop poller from bumping last_accessed_at — aging was dead |
| `d77b861` | fix(attention): persist last_accessed_at on send + attach-return |
| `32e5964` | fix(attention): flip status to Running synchronously on send |
| `8114868` | fix(attention): show age column at width 45 + group tiebreaker ASC |
| `0bc3fbf` | fix(attention): lower age-column threshold to 30 |

**Status:** these are bug fixes layered on top of the already-merged PRs. Each is small, individually upstreamable. Bundle as `fix/attention-aging-followups`.

### Theme D — Strict-hotkeys follow-ups

#755 merged. These are post-merge regression fixes and extensions — should each be its own small PR.

| Commit | Title |
|--------|-------|
| `690c29a`, `b86b13e` | strict_hotkeys mode + complete (likely rebase-artifact, may already be in upstream/main) |
| `39fe7ac` | feat(tui): strict-mode lowercase → compose dialog capture |
| `4c55508` | fix(tui): Shift+O cycles sort in strict mode |
| `2f4613b`, `e61d79c` | fix(tui): restore 'no destructive lowercase' for 'o' sort |
| `2927337` | fix(tui): strict-mode Q quit + iOS Mosh |
| `3d79573` | fix(tui): strict-mode Shift+letter regression for N/X/S/M/T/C |

**Status:** each bug-fix commit is a candidate `fix/strict-hotkeys-*` PR.

### Theme E — Mosh/iPad/responsive follow-ups

#862 + #865 merged. These are layered iterations.

| Commit | Title |
|--------|-------|
| `209f067` | feat(tui): Shift+Up/Down and {/} as iPad-friendly ±10 nav |
| `14cee31` | fix(tui): handle Event::Resize so iPad/iPhone Mosh redraws |
| `f0e52de` | fix(tui): Ctrl+q quits in strict_hotkeys mode (iPad rescue) |
| `fb16e10` | revert: drop iPad-divergent alt bindings |
| `5c49a50` | feat(tui): restore { / } as ±10 nav |
| `ce3a4f7` | feat(tui): iPad/iPhone Mosh — < > pane-resize + list-pane mouse scroll |
| `ce31939` | feat(tui): stacked layout below 60 cols |
| `ef32f8b` | fix(tui): stacked layout — list on top, preview below |
| `2f59d6e` | fix(tui): paste-burst detector for VoiceInk over Mosh |
| `a390257` | fix(tui): position-aware scroll routing |
| `e5807b0` | fix(tui): gate EnableMouseCapture behind `AOE_MOUSE_CAPTURE=1` |
| `2f6ba0e` | debug(tui): trace mouse events (debug-only — drop or PR) |

**Status:** branch `feat/responsive-mosh-rebased` (90 ahead). Bundle as `feat/responsive-mosh-followups` or split into `feat/stacked-layout`, `fix/voiceink-paste`, `fix/mouse-capture-gate`.

### Theme F — Single-spawn picker + batch-spawn

| Commit | Title |
|--------|-------|
| `2ec88ad` | feat(tui): `b`/`B` shortcut to batch-spawn sessions via cxs |
| `13e9889` | feat(tui): surface `b`/`B` batch-spawn + paste in UI chrome |
| `d49496f` | feat(tui): ship §9.13 a/A single-spawn picker |
| `32c001f` | feat(tui): capture bracketed paste in home view (VoiceInk guard) |

**Status:** PR-ready as `feat/spawn-pickers`.

### Theme G — Headless / wedge-size / message-routing fixes

| Commit | Title |
|--------|-------|
| `d02feb3` | fix(tmux): default headless size to 240x80 to avoid 80x24 wedge |
| `7f9ccdc` | fix(tmux): substitute DEFAULT_HEADLESS_SIZE for wedge-sized terminals |
| `5ff89a1` | fix(tmux): use `--` separator in send-keys -l for dash-prefixed lines |
| `c82c95c` | fix(api): send_message must save only the touched profile |

**Status:** each its own tiny upstream PR. Cheap wins.

### Theme H — Multi-account "cs aliases" symlink-skip (LOCAL-ONLY by design)

| Commit | Title |
|--------|-------|
| `942af48` | fix(session): skip symlinks in list_profiles() |
| `53d4eef` | test+harden: pin symlink-skip in profile listing |

**Status:** depends on the cs/cxa account-switcher pattern that's specific to this Mac Mini setup. Upstream users don't have it. **Keep local-only.**

### Theme I — Hook integration (LOCAL-ONLY)

| Commit | Title |
|--------|-------|
| `e1957d4` | fix(hooks): Stop event writes "waiting" not "idle" |
| `40952b8` | feat(session): send wake-up prompt after restart |

**Status:** tied to our personal-dev hook stack. **Keep local-only.**

### Theme J — Misc UI polish

| Commit | Title |
|--------|-------|
| `9ebd2db` | feat(tui): make `q` aggressively quit |
| `0017f01` | fix(tui): width-adaptive status bar |
| `8e6e177` | fix(tui): selected row overrides fg to theme.text |
| `fc119ba` | fix(tui): promote Msg/Archive/Fav/Snooze to priority 1 in status bar |
| `5e93524` | fix(tui): reset field (r) should not change focus or scroll |
| `78b24e5` | chore: cargo fmt drift |
| `f27dd44` | fix(tui): Settings theme preview honors color_mode=Palette |

**Status:** PR-ready, individually small.

### Theme K — Already-in-flight as `feat/default-view-mode` (PR #778 OPEN)

| Commit | Title |
|--------|-------|
| `8ac1a73` | feat(tui): configurable default_view_mode for home screen |

## How to keep up to date with upstream — the workflow

### Branch model going forward

```
upstream/main        ← single source of truth
   │
   ├── feat/attention-system    ← clean PR candidate, rebased on upstream/main
   ├── feat/spawn-pickers       ← clean PR candidate
   ├── fix/attention-aging-followups
   ├── fix/voiceink-paste
   ├── ...
   │
   └── local/deploy             ← upstream/main + cherry-picks of accepted PRs
                                  + LOCAL-ONLY patches (Themes H, I)
                                  This is what gets installed.
```

### Pulling upstream

```bash
cd ~/GitProjects/personal-dev/forks/agent-of-empires
git fetch upstream
git fetch origin

# Update local main mirror
git checkout main && git merge --ff-only upstream/main && git push origin main

# Rebase the deploy branch
git checkout local/deploy
git rebase upstream/main           # already-merged commits drop automatically
                                   # via patch-id equivalence (cherry-pick aware)
# Resolve any conflicts (rare if PRs land cleanly)

# Rebuild + install
cargo build --release --features serve
cp target/release/aoe ~/.cargo/bin/aoe
```

### Why this model and not "merge upstream into our/integration"

`our/integration` accumulates merge commits (8 visible in `git log --merges`). Each merge changes commit hashes downstream, breaks `git cherry`'s already-merged detection, and forces conflict resolution every time you pull upstream. **A rebase-only deploy branch never has merge commits, so already-merged PRs disappear cleanly.**

### Migration: turning the current state into the new model

1. Create `local/deploy` from `feat/attention-flat-no-groups` (the currently-installed branch).
2. Rebase `local/deploy` on `upstream/main` — drops anything already merged upstream.
3. For each Theme above marked "PR-ready" or "PR candidate":
   - Create branch from `upstream/main`
   - `git cherry-pick` the relevant commits
   - Push to `origin`, open PR upstream
4. Delete `our/integration` once `local/deploy` is green and installed.
5. Update CLAUDE.md table to point at `local/deploy` as the live branch.

### Rules to keep this clean

1. **Never merge upstream into a feature branch.** Rebase only.
2. **Never commit to `local/deploy` directly.** Always cherry-pick from a feature branch.
3. **One feature = one branch = one PR.** Don't pile work into `our/integration`-style mega-branches.
4. **LOCAL-ONLY commits go on `local/deploy` only**, with commit message prefix `local:` so they're trivially identifiable.
5. **Pull upstream weekly** — the longer between rebases, the worse the conflicts.

## Action items (concrete)

- [ ] Create `local/deploy` from current installed HEAD.
- [ ] Rebase `local/deploy` onto `upstream/main` (v1.5.2). Bump `~/.cargo/bin/aoe` to 1.5.2.
- [ ] PR Theme F (`feat/spawn-pickers`) — small, self-contained, easy review.
- [ ] PR Theme G (4 tiny fixes — headless, send-keys separator, send_message profile save) — cheap wins.
- [ ] PR Theme A (`feat/attention-system`) — biggest unmerged chunk; bundle archive/favorite/snooze cohesively.
- [ ] PR Theme C (`fix/attention-aging-followups`) — layered fixes on already-merged work.
- [ ] PR Theme D individual `fix/strict-hotkeys-*` — each is small.
- [ ] PR Theme J individually as it makes sense.
- [ ] Delete `our/integration` after `local/deploy` is the source of truth.
- [ ] Update top-level `CLAUDE.md` "## Where things live" + "## Outstanding issues" to point at `local/deploy`.

## Currently open / WIP feature branches (snapshot 2026-05-07)

| Branch | Ahead of upstream/main | Status |
|--------|---|---|
| `our/integration` | 81 | Phase out |
| `feat/attention-flat-no-groups` | 94 | Currently installed — becomes `local/deploy` |
| `feat/responsive-mosh-rebased` | 90 | Source for Theme E PRs |
| `fix/strict-shift-d-r-regression` | 77 | Source for Theme D PRs |
| `feat/snooze` | 50 | Source for Theme A PRs |
| `feat/attention-archive-and-signal-hook` | 32 | Source for Theme A PRs |
| `feat/restart-all` | 4 | Already merged upstream — delete |
| `fix/hook-stop-emits-waiting` | 3 | LOCAL-ONLY (Theme I) — keep local |
| `feat/azure-container-deploy` | 2 | Untouched, status TBD |
| `feat/extra-nav-keys` | 2 | Already merged upstream — delete |
| `feat/api-control` | 1 | Already merged upstream — delete |
| `feat/default-view-mode` | 1 | PR #778 OPEN — wait for merge |
