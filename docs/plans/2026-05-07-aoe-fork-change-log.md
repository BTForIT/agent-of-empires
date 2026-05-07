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

### End state (the only acceptable target)

**`agent-of-empires` checkout = `upstream/main`. Period.** No `local/deploy`. No `our/integration`. No long-lived divergent branches that ship binaries. The only reason to ever check out something other than `upstream/main` is to author a PR branch that will be merged or deleted within days.

Updating becomes:

```bash
cd ~/GitProjects/personal-dev/forks/agent-of-empires
git fetch upstream && git checkout main && git merge --ff-only upstream/main
cargo build --release --features serve && cp target/release/aoe ~/.cargo/bin/aoe
```

That's it. No conflict resolution. No "which branch am I deploying from."

### Per-theme fate (every commit goes somewhere)

Each of the 73 local-only commits resolves to exactly one of three outcomes. Nothing stays local in the fork.

| Theme | Disposition | Action |
|-------|-------------|--------|
| A — Attention/sort/cursor (archive/favorite/snooze) | **Upstream PR** `feat/attention-system` | Biggest single PR. Bundle cohesively. |
| B — Cursor jumping | **Upstream PR** `feat/cursor-jumps` | Small, self-contained. |
| C — last_accessed_at follow-ups | **Upstream PR** `fix/attention-aging-followups` | Stacks on already-merged #762/#777. |
| D — Strict-hotkeys regressions | **Upstream PR(s)** `fix/strict-hotkeys-*` | Each fix is tiny; can be one PR or several. |
| E — Mosh/iPad layout polish | **Upstream PR** `feat/responsive-mosh-followups` | Stacks on already-merged #865. |
| F — Spawn pickers | **Upstream PR** `feat/spawn-pickers` | Small, self-contained, easy review. |
| G — Headless/wedge fixes | **Upstream PR(s)** `fix/headless-*` | Cheap wins, file individually. |
| H — cs-aliases | **Move out of fork** | Not AoE concern. → `personal-dev/cx-scripts/`. |
| I — Hooks shimmed into AoE | **Move out of fork** | Not AoE concern. → `personal-dev/claude-hooks/`. |
| J — Misc UI tweaks | **Upstream PR(s)** | File individually as makes sense. |
| K — PR #778 (default-view-mode) | **Already open upstream** | Just ride it in; nudge if stale. |

**Themes H and I are the critical insight.** They're the reason `our/integration` exists at all — local-only customizations that never had any business inside the AoE source tree. Their content already has a proper home in `personal-dev/`. Move them, then they stop being a fork-divergence problem forever.

### Pulling upstream while drain is in progress

Until every theme is drained, the only branch that should ship binaries is the one with the fewest pending PRs. Right now that's `feat/attention-flat-no-groups` (94 ahead). To pull a new upstream release without making the situation worse:

```bash
git fetch upstream
git checkout feat/attention-flat-no-groups
git rebase upstream/main         # already-merged PR commits drop via patch-id equivalence
# Build + install
cargo build --release --features serve && cp target/release/aoe ~/.cargo/bin/aoe
```

**Rule during drain:** when a theme lands upstream, immediately rebase the deploy branch on upstream/main so those commits drop. The deploy branch only ever shrinks.

### Rules to keep this clean

1. **Never merge upstream into anything.** Rebase only.
2. **No new local-only commits to the AoE fork.** If it's not going upstream, it doesn't belong in this repo. Themes H and I prove the point.
3. **One feature = one branch = one PR.** Don't accumulate work into mega-branches.
4. **Once a PR merges, delete its local branch and rebase the deploy branch immediately.**
5. **Drain bias:** prioritize PRs by ratio of (commits removed from local) ÷ (review effort). Themes F, G, B clear quickly.

## Action items (concrete)

- [ ] **Move Theme H commits out of fork** → `personal-dev/cx-scripts/` (cs-aliases). Revert in fork.
- [ ] **Move Theme I commits out of fork** → `personal-dev/claude-hooks/`. Revert in fork. (Most content already lives there; fork copies are stale duplicates.)
- [ ] PR Theme F (`feat/spawn-pickers`) — easiest first PR.
- [ ] PR Theme G individually (4 tiny fixes — headless, send-keys separator, send_message profile save).
- [ ] PR Theme B (`feat/cursor-jumps`).
- [ ] PR Theme D individually (`fix/strict-hotkeys-*`).
- [ ] PR Theme C (`fix/attention-aging-followups`).
- [ ] PR Theme E (`feat/responsive-mosh-followups`).
- [ ] PR Theme J individually as makes sense.
- [ ] PR Theme A (`feat/attention-system`) — biggest, save for last; review cycle will be longest.
- [ ] Nudge upstream on PR #778 if still open.
- [ ] After every PR merges or every revert lands: rebase `feat/attention-flat-no-groups` on `upstream/main` so already-merged commits drop. Watch the "ahead by N" count fall.
- [ ] **Once `feat/attention-flat-no-groups` is 0 commits ahead of `upstream/main`:** delete the branch, delete `our/integration`, switch installed binary to plain `main`. Update top-level `CLAUDE.md` "Where things live" to drop the "active deploy branch" row.

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
