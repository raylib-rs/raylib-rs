# Open-PR triage workstream (2026-06-09)

Triage of every open PR after the 6.0 merge + the post-release queue work.
Split into **mergeable now** (CI-validated, ready for the merge button) and
**needs work / a decision** (this workstream). Supersedes the relevant rows of
`docs/superpowers/inventory.md` for these PR numbers.

## Snapshot (6 open PRs)

| PR | Author | Draft | Mergeable | CI | Disposition |
|----|--------|-------|-----------|----|-------------|
| [#335](https://github.com/raylib-rs/raylib-rs/pull/335) | Dacode45 | no | CLEAN | 31 ✅ | **MERGE** — item 13 re-land (nobuild-prebuilt job onto unstable) |
| [#319](https://github.com/raylib-rs/raylib-rs/pull/319) | Dacode45 | no | conflicting | 27 ✅ (stale) | **CLOSE — superseded** |
| [#270](https://github.com/raylib-rs/raylib-rs/pull/270) | AmityWilder | no | behind | stale | **REWORK** — adapt macro to current impl set |
| [#118](https://github.com/raylib-rs/raylib-rs/pull/118) | strizhkindenis | no | conflicting | none | **REWORK** — Mesh accessor redesign |
| [#211](https://github.com/raylib-rs/raylib-rs/pull/211) | AmityWilder | yes | conflicting | stale | **REWORK** — safety-audit, cherry-pick |
| [#325](https://github.com/raylib-rs/raylib-rs/pull/325) | jgabaut | yes | conflicting | 27 ✅ | **REWORK** — finish the cross-target bindgen fix |

## Mergeable now

### #335 — nobuild-prebuilt matrix re-land
- CLEAN against current `unstable` tip (6e02979); 31/31 checks green, including
  the `nobuild-prebuilt` job on all 3 OSes. CI is current — no refresh needed.
- **Action:** merge. Completes post-release queue item 13 on `unstable`.

## Close — superseded (no rework, just a decision + courteous comment)

### #319 — drop the in-overlay GitHub-URL footer
- **Already done on `unstable`.** `showcase/src/viewer.rs` was reworked after the
  PR (368 lines changed); the GitHub-URL footer it removes no longer exists (0
  references to `FOOTER_FONT_SIZE` / "Source on GitHub" on `unstable`; GitHub
  links now live on the Pages gallery, viewer.rs:7). The conflict is the rewrite,
  not a real divergence of intent.
- **Action:** close as superseded; no code needed.

## Rework — genuine workstream items

### #270 — macro for repetitive `ShaderV` impls
- **Not superseded** (verified): 12 repetitive hand-written `impl ShaderV for
  <type>` blocks still exist in `raylib/src/core/shaders.rs` (f32, Vector2/3/4,
  i32, `[i32;2/3/4]`, `[f32;2/3/4]`, `&[i32]`, …), so the cleanup the PR proposes
  is still relevant. The macro is a plain `macro_rules!` (no `paste`), so it
  aligns with the deliberate `paste` removal (#329 / b4cf538) — good.
- **But it can't be merged as-is:** `shaders.rs` diverged +151/-15 on `unstable`
  since the PR's base (closes issue #269's pre-6.0 state). GitHub reports BEHIND
  (textually auto-mergeable), but the macro enumerates the *old* impl set —
  auto-merging would silently drop any impls added during the 6.0 trait redesign
  (per-uniform fast paths). A naive branch-update + merge would compile-fail or
  lose coverage.
- **Plan:** re-derive the macro against the *current* impl set: enumerate every
  `impl ShaderV` on `unstable`, confirm the macro generates exactly that set
  (including the per-uniform fast paths and any `&[f32]`/Matrix additions), then
  update the branch and let CI validate. Small, self-contained, paste-free —
  good first rework item. Credit the author. Also resolves #269.

### #118 — expose Mesh texcoords safely
- Safe accessor for `Mesh` texcoords (4 files, +205/-1). CONFLICTING; predates the
  6.0 Mesh/`DataBuf` changes.
- **Plan:** fold into the deferred **Mesh-accessor redesign** (the DataBuf+Mesh
  testing umbrella in the post-release queue) rather than merging the standalone
  accessor. Reconcile against the current `Mesh` API + `DataBuf` lifetimes,
  add Tier-1 tests, credit the original author. Coordinate with the broader
  vertex-attribute accessor surface so texcoords/normals/colors land consistently.

### #211 — safety audit
- Large draft (+4783/-5, 11 commits, 2 files) adding `# Safety` docs to unsafe
  fns. CONFLICTING against the 6.0 tree.
- **Plan:** do not merge wholesale. Cherry-pick the Safety-doc additions
  **module by module** as we touch each file, preserving per-file author
  attribution (matches the inventory's WS3 guidance for #210/#211). Cross-check
  each against the soundness changes already made post-6.0 (wrapper soundness
  #331, Mesh soundness) so docs describe the *current* invariants. Track as an
  ongoing absorb, not a single PR.

### #325 — explicit bindgen target for `*-windows-gnu`
- Draft/WIP (+5, 1 file) setting the bindgen `--target` explicitly for
  `x86_64-pc-windows-gnu` cross builds. CONFLICTING.
- **Plan:** finish + de-conflict against the current `build.rs` (which now carries
  the `nobuild`/`nobindgen` gating + the raymath-shim logic). Decide whether the
  explicit-target arg belongs in the shared bindgen builder or behind a
  cross-target guard, and add a CI leg (or extend an existing cross-check) that
  actually exercises a `*-windows-gnu` target so the fix is regression-guarded.
  Confirm it doesn't perturb the host/`nobindgen` paths.

## Recommended order
1. Merge **#335** (closes item 13 on `unstable`).
2. Close **#319** (superseded — footer already removed; immediate, no code).
3. Schedule the rework items: **#270** (small, self-contained, paste-free — good
   first) and **#325** (small cross-target bindgen fix), then **#118** into the
   Mesh-accessor redesign, then **#211** as a rolling module-by-module absorb.
