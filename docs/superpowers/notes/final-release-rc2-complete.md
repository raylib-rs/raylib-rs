# Final-release workstream — rc.2 milestone

**Status:** rc.2 PUBLISHED. Both `raylib-sys 6.0.0-rc.2` and
`raylib 6.0.0-rc.2` are live on crates.io as of 2026-06-03. The
final 6.0.0 cut is intentionally held days for wider review on
the canonical merge; this rc.2 milestone is the dress rehearsal.

## What shipped

| Crate | Version | crates.io indexed |
|-------|---------|-------------------|
| `raylib-sys` | 6.0.0-rc.2 | 2026-06-03T06:17:40Z |
| `raylib`     | 6.0.0-rc.2 | 2026-06-03T06:23:07Z |

Tags on canonical:
- `v6.0.0-rc.2` on `bddca73` (the rc.2 publish source state)
- `v6.0.0` on `47e126f` (would-have-been 6.0.0 source state; kept per
  maintainer call as an accurate snapshot of the bumped tree before the
  rc.2 pivot)

PRs merged this workstream:

| # | Subject |
|----|---------|
| 297 | raylib-rs 6.0.0 (merged 47e126f) |
| 298 | ci: fix release workflows + add unstable to push triggers (post-merge) |
| 299 | ci(showcase): scope continue-on-error to wasm-build only |
| 300 | chore(release): bump 6.0.0 → 6.0.0-rc.2 (wider-review snapshot) |
| 301 | docs: flip Pages URLs from fork to canonical raylib-rs.github.io |
| 302 | docs: refresh README + CLAUDE.md for the merged 6.0 release |
| 303 | fix(showcase): synthesize per-example HTML + pin gen-thumbnails CWD |

10 community PRs closed with attribution + 2 Group C closures (#252,
#279) — see the PR-closure summary earlier in the session for the
full Group A/B/C inventory.

## Sequence executed

1. CI baseline gate at `6d1bffa` SKIPPED per maintainer call — the GHA
   artifact-cache outage wedged every sccache-using workflow.
2. Bumped `6.0.0-rc.1` → `6.0.0` on fork's `6.0-rc` branch (`6c88e14`).
3. Commented out sccache wiring across 6 workflows (`00149ce`) — outage
   workaround.
4. Fixed canonical PR #297 CI checks (typos + no_std smoke test) (`8e30e90`).
5. Tagged `v6.0.0-pre` safety snapshot on `6c88e14`.
6. Opened canonical PR #297 → merged at `47e126f` ("raylib-rs 6.0.0").
7. Tagged `v6.0.0` on the merge commit (stays in place per maintainer
   call as an accurate snapshot of the would-have-been 6.0.0 source).
8. First `release-sys.yml dry_run=true` failed at `Could NOT find X11`
   — bare `ubuntu-latest` runner missing the build deps the
   `cargo publish --dry-run` verify build needs.
9. PR #298 folded in:
   - Linux build deps for `release-sys.yml` + `release-safe.yml`
   - `unstable` added to push triggers on 6 workflows
   - `check-compiles-no-build` TODO rewritten — it's a no_std smoke
     test, not a `nobuild` test (the misnamed comment from `8e30e90`
     conflated the two)
10. PR #299 (showcase `wasm-build` continue-on-error only) + PR #300
    (bump `6.0.0` → `6.0.0-rc.2`) merged.
11. Tagged `v6.0.0-rc.2` on the post-merge HEAD `bddca73`.
12. `release-sys.yml dry_run=true` → GREEN.
13. **First real `release-sys.yml dry_run=false` FAILED** at
    `please provide a non-empty token` — the workflow indirects via
    `${{ secrets.CRATES_IO_TOKEN }}` to set the `CARGO_REGISTRY_TOKEN`
    env var. The secret had been added as `CARGO_REGISTRY_TOKEN`
    (matching the env var name, not the secret name the workflow
    reads). No tarball was uploaded; cargo bailed at the client-side
    token check before the network call.
14. Maintainer added `CRATES_IO_TOKEN` secret.
15. `release-sys.yml dry_run=false` retry → SUCCESS,
    `raylib-sys v6.0.0-rc.2` uploaded
    (run `26867245436`).
16. `raylib-sys 6.0.0-rc.2` indexed on crates.io at
    `2026-06-03T06:17:40Z` — well under 5 minutes (the typical
    indexing window is 5–15 min; this caught it almost immediately).
17. `release-safe.yml dry_run=true` → GREEN — the WS8c-flagged
    "needs real raylib-sys on registry first" limitation cleared
    automatically once step 15 published.
18. `release-safe.yml dry_run=false` → SUCCESS, `raylib v6.0.0-rc.2`
    uploaded (run `26867447920`); `Published raylib v6.0.0-rc.2 at
    registry crates-io`.
19. `raylib 6.0.0-rc.2` indexed at `2026-06-03T06:23:07Z` — under
    6 min end-to-end from sys-upload through both indexing waits.

## Incidents (for the final-cut runbook)

1. **GHA artifact-cache outage 2026-06-02** (Azure DevOps Edge
   incident: `Our services aren't available right now`). Wedged
   `mozilla-actions/sccache-action@v0.0.6`'s server startup in every
   workflow that used it. Worked around by commenting out the sccache
   wiring (`00149ce`); `Swatinem/rust-cache@v2` was unaffected and
   left intact. Re-enable path: revert `00149ce` once GHA cache
   service is back; the `TODO(re-enable)` markers point at each
   commented block. See [[post-release-web-build-speed]].

2. **Secret name confusion** (`CARGO_REGISTRY_TOKEN` env var vs.
   `CRATES_IO_TOKEN` secret name). The workflow does:
   ```yaml
   env:
     CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
   ```
   The *secret* name (what you create in GitHub repo settings) is
   `CRATES_IO_TOKEN`. The *env var* (what cargo reads) is
   `CARGO_REGISTRY_TOKEN`. They are different by design — the
   indirection is so a maintainer can rotate the secret name without
   touching the workflow files. The session's first publish attempt
   used the env-var name as the secret name and bailed at token
   check; no version was published.

3. **Linux build deps absent on the bare `ubuntu-latest` runner.**
   `cargo publish --dry-run` runs a verify build of the packaged
   tarball; that build cmake-builds the vendored raylib C source,
   which requires the X11+GL stack and ALSA/udev headers. WS8c's
   `act`-based validation didn't catch this because act's default
   container has more dev tooling pre-installed than the bare GHA
   runner. Fix in PR #298.

## Pages

Canonical Pages is configured (`build_type: workflow`, `branch:
unstable`, `path: /`, public, https enforced) and the canonical URL
is `https://raylib-rs.github.io/raylib-rs/`. PR #301 flipped the
Pages URLs in repo docs from the fork to canonical; PR #303 fixed
the per-example HTML synthesis + `gen-thumbnails` CWD. The fork's
Pages deploy at `https://dacode45.github.io/raylib-rs/` is the
historical record of the WS9 development site.

## Tracked-deferred for the 6.0.0 final cut

- Bump `Cargo.toml` versions back from `6.0.0-rc.2` → `6.0.0`
  (mirror PR #300 inverted). The pattern is the same: 3 Cargo.toml
  files + 1 inter-crate pin + README + 2 book snippets + CHANGELOG
  header flip + remove the rc-rationale blockquote.
- Decide whether `v6.0.0` (currently on `47e126f`) gets re-pointed
  at the final-cut commit or stays where it is. If staying, the
  final commit needs a fresh tag like `v6.0.0-final` or similar.
- Run `release-sys.yml dry_run=false` + `release-safe.yml
  dry_run=false` for the final cut (token is in place; both dry-run
  paths are validated).
- Write `final-release-complete.md` covering the final cut (this
  rc.2 note is the dress-rehearsal data).
- Tag `final-release-complete` once 6.0.0 final lands.
- Flip `CLAUDE.md` workstream status line to
  `final-release ✅` (currently still reads the post-merge,
  pre-final-cut state).

## Carry-forward memories

- [[post-release-bevy-raylib]] — bevy ECS integration
- [[post-release-showcase-clippy-cleanup]] — allow-attribute showcase examples
- [[post-release-web-build-speed]] — matrix-split wasm + re-enable sccache + drop legacy CI duplication
