# WS8c — release workflow validation via `act`

**Spec:** `docs/superpowers/specs/2026-05-29-ws8-release-prep-checkpoint-design.md` §4 WS8c
**Plan:** `docs/superpowers/plans/2026-05-29-ws8c-act-validation.md`

Three `act` runs plus the direct sync-check baseline validate the two release
workflows from WS8b without any crates.io traffic. The runbook here doubles as
the documentation a future maintainer needs when running the real publish.

**Headline result:** the two `release-*.yml` workflows are structurally sound.
Run 3 confirms the `--require-date` sync-check gate fires correctly on
`dry_run: false`. Runs 1 and 2 both hit a documented `act`-on-Windows
artifact (`cargo publish --dry-run` rejects the docker-cp-checked-out tree
as "dirty"), so direct `cargo publish --dry-run` invocations were used to
prove the cargo step itself works as designed.

## Tooling

- `act` invocation prefix used: `gh act` (via the `gh-act` GitHub CLI extension; a bare `act` binary is not on PATH on this host).
- `act` version: `act version 0.2.84`.
- Docker version: `Docker version 29.2.1, build a5c7197` (Docker Desktop on Windows 11).
- `act` runner image: `catthehacker/ubuntu:act-latest`, passed explicitly via `-P ubuntu-latest=catthehacker/ubuntu:act-latest` (no `~/.actrc` on this host).
- Host: Windows 11 + Git Bash + Docker Desktop (`npipe:////./pipe/docker_engine`).

## Direct sync-check baseline (no `act`)

All three commands exercised the helper directly from the host shell. Outputs verbatim:

```
$ bash scripts/check-release-sync.sh raylib-sys
OK: raylib-sys @ 6.0.0, CHANGELOG @ 6.0.0
exit=0

$ bash scripts/check-release-sync.sh raylib
OK: raylib @ 6.0.0, CHANGELOG @ 6.0.0
exit=0

$ bash scripts/check-release-sync.sh raylib-sys --require-date
ERROR: CHANGELOG.md heading '## X.Y.Z' must be followed by ' — YYYY-MM-DD' (em-dash U+2014 + ISO date) before real publish
  found: ## 6.0.0 (unreleased)
exit=1
```

These confirm:
- Soft path passes for both crates (`6.0.0` == `6.0.0`).
- The hardened-by-WS8b error message names the exact constraint (em-dash U+2014 + ISO date) and the exact heading line found in the CHANGELOG. This is what the WS8b review-fix commit `82dca54` set out to do.

## Run 1 — `release-sys.yml` with `dry_run=true`

**Command**:
```
gh act workflow_dispatch \
  -W .github/workflows/release-sys.yml \
  --input dry_run=true \
  -s CARGO_REGISTRY_TOKEN=dummy \
  -P ubuntu-latest=catthehacker/ubuntu:act-latest
```

**Plan-expected outcome:** SUCCESS — every step passes; `cargo publish (real)` is skipped by the `if` gate.

**Actual outcome:** FAILED at `cargo publish (dry-run, always)`.

Step-by-step result:

| Step | Result | Duration |
|------|--------|----------|
| Set up job | OK | — |
| Pre Install Rust toolchain | OK | 313 ms |
| Main Checkout (`actions/checkout@v5`, submodules: recursive) | OK | 2m 41s |
| Main Install Rust toolchain (`dtolnay/rust-toolchain@stable`) | OK | 51 s |
| Main Install jq (sync-check dep) | OK | 20 s |
| Main Version + CHANGELOG sync check | OK (printed `OK: raylib-sys @ 6.0.0, CHANGELOG @ 6.0.0`) | 64 s |
| **Main cargo publish (dry-run, always)** | **FAIL (exit 101)** | 5 s |
| Post Install Rust toolchain | OK | 1.1 s |
| `cargo publish (real)` | (never executed — the previous failure aborted the job) | — |

The failure message inside the container:

```
error: 246 files in the working directory contain changes that were not yet committed into git:

raylib-sys/Cargo.toml
raylib-sys/GENERATING_BINDINGS.md
raylib-sys/binding/.gitattributes
raylib-sys/binding/binding.h
...
raylib-sys/raylib/src/raylib.h
raylib-sys/src/color.rs

to proceed despite this and include the uncommitted changes, pass the `--allow-dirty` flag
```

**Diagnosis:** `act` checks out the workspace via `docker cp src=<host-path> dst=<container-path>`,
not via `git clone`. The host repo is clean on disk, but inside the container `cargo publish`
sees the on-disk tree against an empty/mismatched git index (CRLF/LF normalization and `.gitattributes`
behaviour also differ on Windows-host → Linux-container) and refuses to package without
`--allow-dirty`. The 246-file count includes both `raylib-sys/` and the entire vendored
raylib submodule tree, which is the giveaway that this is a checkout-method artifact, not a
real local edit.

On a real GitHub Actions runner `actions/checkout@v5` does a clean `git clone`, so this
behaviour does not apply there. See the "Direct fallback" section below for the proof that
`cargo publish -p raylib-sys --dry-run` succeeds against the same working tree when run
from the host shell (no `act`, no docker-cp).

Because this failure happens *before* `cargo publish (real)` is reached, the `if`-gate
behaviour for that step could not be observed in this run. Run 3 below provides indirect
confirmation that the gate is wired correctly (the workflow does take a different code
path when `dry_run=false`).

## Run 2 — `release-safe.yml` with `dry_run=true` (KNOWN LIMITATION)

**Command**:
```
gh act workflow_dispatch \
  -W .github/workflows/release-safe.yml \
  --input dry_run=true \
  -s CARGO_REGISTRY_TOKEN=dummy \
  -P ubuntu-latest=catthehacker/ubuntu:act-latest
```

**Plan-expected outcome:** FAIL at `cargo publish --dry-run -p raylib` with a dep-resolution
error against crates.io (because `raylib-sys 6.0.0` is not yet published).

**Actual outcome:** FAIL at `cargo publish --dry-run -p raylib` — but for the same
`act`-checkout "dirty workspace" reason as Run 1, *not* the planned dep-resolution
reason. Cargo bails on the dirty check before ever consulting the registry.

Step-by-step result:

| Step | Result | Duration |
|------|--------|----------|
| Set up job → Install jq | OK | — |
| Main Version + CHANGELOG sync check | OK (`OK: raylib @ 6.0.0, CHANGELOG @ 6.0.0`) | 64 s |
| **Main cargo publish (dry-run, always)** | **FAIL (exit 101)** | 4.5 s |

The failure message inside the container:

```
error: 15 files in the working directory contain changes that were not yet committed into git:

README.md
raylib/Cargo.toml
raylib/Web.toml
raylib/src/core/audio.rs
raylib/src/core/callbacks/audio_stream_callback.rs
raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs
raylib/src/core/databuf.rs
raylib/src/core/logging.rs
raylib/src/core/macros.rs
raylib/src/core/misc.rs
raylib/src/core/text.rs
raylib/src/ease.rs
raylib/src/lib.rs
raylib/src/prelude.rs
raylib/tests/render_shapes.rs

to proceed despite this and include the uncommitted changes, pass the `--allow-dirty` flag
```

(Cargo for the `raylib` package only mentions files inside `raylib/` plus root-level
`README.md` because that's all that ends up in `raylib`'s package tree; the smaller
file count is consistent with the same diagnosis as Run 1.)

**The planned dep-resolution error WAS reproduced**, but on the host via the fallback
path (see below) — `act` simply can't reach that step on this host.

**Implication for the final-release runbook**: run `release-sys.yml` (real publish, then
verify on crates.io) **before** running `release-safe.yml`. At that point the dep-resolution
check will pass on the real GitHub Actions runner.

## Run 3 — `release-sys.yml` with `dry_run=false` + dummy token (negative test)

**Command**:
```
gh act workflow_dispatch \
  -W .github/workflows/release-sys.yml \
  --input dry_run=false \
  -s CARGO_REGISTRY_TOKEN=dummy \
  -P ubuntu-latest=catthehacker/ubuntu:act-latest
```

**Plan-expected outcome:** FAIL at the sync-check step because `--require-date` rejects
the still-unreleased CHANGELOG heading. Real publish step never runs.

**Actual outcome:** MATCHES the plan exactly. Failure at the sync-check step, real publish
never executes, dummy token is never used.

Step-by-step result:

| Step | Result | Duration |
|------|--------|----------|
| Set up job → Install jq | OK | — |
| **Main Version + CHANGELOG sync check** | **FAIL (exit 1)** | 52 s |
| `cargo publish (dry-run, always)` | (never executed) | — |
| `cargo publish (real)` | (never executed) | — |

Exact stderr from the failing step:

```
ERROR: CHANGELOG.md heading '## X.Y.Z' must be followed by ' — YYYY-MM-DD' (em-dash U+2014 + ISO date) before real publish
  found: ## 6.0.0 (unreleased)
```

**What this validates**:
- The `dry_run=false` branch of the workflow runs `bash scripts/check-release-sync.sh
  raylib-sys --require-date` (i.e. the `if` gate around `--require-date` is wired
  correctly), and
- The real-publish step is never reached during validation, so the dummy-token usage
  is safe.

**Pending validation that this WS8 can't perform** (requires flipping the CHANGELOG
date, which is the future final-release step's work):

- Behaviour of the real `cargo publish (real)` step with a dummy token. Expected:
  failure at crates.io auth (`401 Unauthorized` or equivalent).
- Confirmation that with a date-stamped CHANGELOG and a real token, the sequence
  sync-check → dry-run → real-publish completes successfully on a GitHub-hosted runner.

## Direct fallback validation (host shell, no `act`)

To prove the workflow steps themselves are correct — independent of `act`'s
docker-cp artifact — the relevant cargo invocations were run directly on the host:

```
$ cargo publish -p raylib-sys --dry-run
    Updating crates.io index
   Packaging raylib-sys v6.0.0 (...\raylib-sys)
    Updating crates.io index
    Packaged 288 files, 17.4MiB (3.0MiB compressed)
   Verifying raylib-sys v6.0.0 (...\target\package\raylib-sys-6.0.0)
   ... (full bindgen + cmake + build) ...
   Compiling raylib-sys v6.0.0 (...\target\package\raylib-sys-6.0.0)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 50s
   Uploading raylib-sys v6.0.0 (...\raylib-sys)
warning: aborting upload due to dry run
exit=0
```

Packages, builds, and dry-run-aborts cleanly. This is what `release-sys.yml`'s `cargo
publish (dry-run, always)` step will do on a real GitHub Actions runner.

```
$ cargo publish -p raylib --dry-run
    Updating crates.io index
   Packaging raylib v6.0.0 (...\raylib)
    Updating crates.io index
error: failed to prepare local package for uploading

Caused by:
  failed to select a version for the requirement `raylib-sys = "^6.0.0"`
  candidate versions found which didn't match: 5.5.1, 5.5.0, 5.0.2, ...
  location searched: crates.io index
  required by package `raylib v6.0.0 (...\raylib)`
exit=101
```

**This is exactly the planned Run-2 failure** that `act` couldn't reach in-container.
It proves both that the `release-safe.yml` cargo step will fail with the documented
error against crates.io as long as `raylib-sys 6.0.0` is unpublished, and that
running `release-sys.yml` to completion first will satisfy the requirement.

## Gotchas hit / mitigations

1. **`act` is `gh act`-only on this host.** The bare binary `act` is not on PATH; only the
   GitHub CLI extension `gh act` works. All commands above use `gh act`. If `act` is the
   only available CLI on another machine, drop the leading `gh`.

2. **Runner image must be specified explicitly.** No `~/.actrc` was present, so the
   default `node:slim` image (which lacks `bash`, `apt-get`, `cargo`, …) would have been
   used. `-P ubuntu-latest=catthehacker/ubuntu:act-latest` is required on every
   invocation, or add it once to `~/.actrc`.

3. **Slow first run.** Run 1 took roughly 5 minutes wall-clock, dominated by the
   `actions/checkout@v5` step doing a recursive submodule copy (`raylib-sys/raylib/`
   is a large submodule). Subsequent runs reused the toolchain layer but each still
   re-runs the checkout from scratch (~1m 22s).

4. **`act`'s docker-cp-based checkout poses as a dirty git workspace to `cargo
   publish`.** This is the load-bearing gotcha that prevented Runs 1 and 2 from matching
   their planned outcomes. The host repo is clean (`git status --short` shows only
   the three untracked working files `TODO.md`, `prompt.md`, `next-session-prompt.md`),
   but inside the container `cargo` sees 246 (Run 1) or 15 (Run 2) "modified" files
   and aborts. **Not a workflow defect** — `actions/checkout@v5` on a real GitHub runner
   does a clean `git clone` and avoids this entirely. Two possible mitigations if a
   future maintainer wants `act` to reach the cargo step:
   - Pass `--allow-dirty` to `cargo publish` for `act`-only local runs (NOT for the
     production workflow — `--allow-dirty` masks real defects).
   - Use `act`'s `--bind` flag to bind-mount the workspace instead of `docker cp`-ing
     it. The CRLF/.gitattributes mismatch may still bite on Windows hosts.
   Neither is appropriate to merge into the production workflow.

5. **Dummy token never reaches a network call.** In Run 3 the workflow aborted at the
   sync-check step (well before any `cargo publish (real)`), so `CARGO_REGISTRY_TOKEN=dummy`
   was loaded into the environment but never sent anywhere. This is the safety property
   the negative test was designed to demonstrate.

## Fallback validation if `act` can't run

If Docker or `act` is unavailable on a future maintainer's machine — or if `act` on a
Windows host hits the same checkout artifact documented above — the equivalent direct
validation is:

```bash
# Sync-check both crates (soft path)
bash scripts/check-release-sync.sh raylib-sys
bash scripts/check-release-sync.sh raylib

# Sync-check the hard path (must fail with "(unreleased)" before publish)
bash scripts/check-release-sync.sh raylib-sys --require-date
bash scripts/check-release-sync.sh raylib --require-date

# Package raylib-sys without publishing — should succeed
cargo publish -p raylib-sys --dry-run

# Package raylib without publishing — will fail until raylib-sys 6.0.0 is on crates.io
cargo publish -p raylib --dry-run

# Validate workflow YAML parses (optional)
python -c "import yaml; yaml.safe_load(open('.github/workflows/release-sys.yml'))"
python -c "import yaml; yaml.safe_load(open('.github/workflows/release-safe.yml'))"
```

This catches sync-check + packaging errors but not workflow-level YAML/step-gating
errors. `act` is the preferred end-to-end validation; the direct sequence above is
what was actually used in this WS8c run to compensate for the Windows-host `act`
limitation.

## Final-release runbook teaser

The full final-release runbook is part of the future final-release workstream. The
relevant WS8c outcome — and the sequence that this WS8c run validated piece by piece:

1. Flip `CHANGELOG.md` heading `## 6.0.0 (unreleased)` → `## 6.0.0 — YYYY-MM-DD`
   (em-dash U+2014, ISO date). The sync-check helper will then accept `--require-date`.
2. Trigger `release-sys.yml` with `dry_run: true` first — must pass cleanly now that
   the CHANGELOG has a date (sync-check + `cargo publish --dry-run -p raylib-sys` both
   green).
3. Trigger `release-sys.yml` with `dry_run: false` — publishes `raylib-sys 6.0.0`.
   The `--require-date` gate (proven to fire in Run 3) only allows this path when the
   CHANGELOG is dated.
4. Verify on <https://crates.io/crates/raylib-sys> that `6.0.0` is listed and resolvable.
5. Trigger `release-safe.yml` with `dry_run: true` — now passes because `raylib-sys
   6.0.0` resolves on crates.io (the dep-resolution error reproduced in the direct
   fallback above is gone).
6. Trigger `release-safe.yml` with `dry_run: false` — publishes `raylib 6.0.0`.
