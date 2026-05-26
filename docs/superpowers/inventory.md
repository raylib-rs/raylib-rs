# raylib-rs 6.0 — Backlog Inventory & Triage

Source: `raylib-rs/raylib-rs` open PRs/issues + community branches, as of 2026-05-25.
Disposition rubric: **merge** (take as-is) · **adapt** (rework into a workstream) ·
**superseded** (6.0 rewrite replaces it; record why) · **decline** (out of scope; record why).

## Pull requests

| PR | Title | Author | Branch | Disposition | Target WS / reason |
|----|-------|--------|--------|-------------|--------------------|
| [#296](https://github.com/raylib-rs/raylib-rs/pull/296) | DRAFT: feat: expose gui_load_style_from_memory() | jgabaut | work/gui_load_style_from_memory | adapt | WS5 (raygui) — depends on upstream raygui update not yet released; cherry-pick the safe-wrapper portion when raygui vendored header advances |
| [#295](https://github.com/raylib-rs/raylib-rs/pull/295) | Sola merge | Dacode45 | sola-merge | adapt | WS9 — large merge of brettchalupa/sola-raylib content; extract example ports and integrate into showcase pipeline rather than merging whole branch as-is |
| [#289](https://github.com/raylib-rs/raylib-rs/pull/289) | Build/nix shell for x11 | Seowlfh | build/nix_shell_for_X11 | adapt | WS6 — nix shell X11 support is valuable; rebase against 6.0 build system and merge with the full platform matrix |
| [#284](https://github.com/raylib-rs/raylib-rs/pull/284) | updated docstrings in color.rs to match the raylib cheatsheet | LBreede | unstable | merge | WS7 — doc-only fix, applies cleanly; cherry-pick with attribution before color API is touched |
| [#282](https://github.com/raylib-rs/raylib-rs/pull/282) | Fixed gen_image_perlin_noise definition | EasyGameT | fix-definition | adapt | WS3 — function signature fix; verify against raylib 6.0 C header, then cherry-pick if signature matches |
| [#279](https://github.com/raylib-rs/raylib-rs/pull/279) | Add links setting to raylib-sys and export include path | themkat | feature/links-include-var | merge | WS1 — tiny, correct, enables downstream sys-crate composition; cherry-pick as-is |
| [#277](https://github.com/raylib-rs/raylib-rs/pull/277) | Remove unsound trait implementations on thin wrappers with pointer fields | AmityWilder | thick-wrappers | adapt | WS3 — soundness fix for thin/thick wrapper design; WS3 will redesign wrappers in 6.0, absorb the intent (drop unsound impls) during that rewrite |
| [#275](https://github.com/raylib-rs/raylib-rs/pull/275) | Add missing config flags to `RaylibBuilder` | AmityWilder | more-config-flags | adapt | WS3 — builder additions; cross-check against 6.0 raylib config flags and re-add any that still exist |
| [#273](https://github.com/raylib-rs/raylib-rs/pull/273) | Fix module doc comment in `logging.rs` | AmityWilder | outer-logging-comment | merge | WS7 — one-line doc fix; cherry-pick immediately |
| [#272](https://github.com/raylib-rs/raylib-rs/pull/272) | Replace `CStr::from_bytes_with_nul` for literals with `c"..."` literals | AmityWilder | cstr-literals | merge | WS3 — mechanical modernisation, no logic change; cherry-pick as-is |
| [#270](https://github.com/raylib-rs/raylib-rs/pull/270) | Create a macro for repetitive `ShaderV` impls | AmityWilder | shader_v-macro | adapt | WS3 — macro cleanup; review shader API surface in 6.0 first, then apply or subsume |
| [#268](https://github.com/raylib-rs/raylib-rs/pull/268) | Convert `Into<T> for U` to `From<U> for T` | AmityWilder | into-to-from | merge | WS3 — idiomatic Rust convention fix, no semantics change; cherry-pick with attribution |
| [#266](https://github.com/raylib-rs/raylib-rs/pull/266) | Seal `AudioSample` | AmityWilder | sealed-audiosample | merge | WS3 — API hygiene, closes issue #213; cherry-pick as-is |
| [#263](https://github.com/raylib-rs/raylib-rs/pull/263) | Change `get_random_value` to use `RangeInclusive` instead of `Range` | AmityWilder | get_random_value_inclusive | merge | WS3 — correctness fix for inclusive semantics, closes #255; cherry-pick |
| [#259](https://github.com/raylib-rs/raylib-rs/pull/259) | Fix mipmaps accessor | AmityWilder | mipmaps-accessor | merge | WS3 — one-line bug fix, closes #258; cherry-pick |
| [#257](https://github.com/raylib-rs/raylib-rs/pull/257) | Fix soundness hole in RaylibMesh accessors | AmityWilder | mesh-accessor-nulls | adapt | WS3 — soundness fix for mesh accessors; WS3 will rework the Mesh API surface, absorb this fix during that work |
| [#256](https://github.com/raylib-rs/raylib-rs/pull/256) | Mesh general safety/soundness fixes for 3D | meisei4 | models-soundess-safety-idea | adapt | WS3 — draft with broader mesh soundness work; author notes it targets 6.0 parity; review patches during WS3 mesh API pass |
| [#252](https://github.com/raylib-rs/raylib-rs/pull/252) | Fix RaylibHandle::get_window_state | zacharysiegel | zach/fix-get-window-state | merge | WS3 — targeted bug fix, closes #287; cherry-pick with attribution |
| [#251](https://github.com/raylib-rs/raylib-rs/pull/251) | Port raylib-sys to work with #![no_std] environments | nbe1233 | unstable | adapt | WS1 — no_std support is desirable; the 6.0 sys regeneration (WS1) should incorporate the core/std split properly rather than applying the patch to the old binding |
| [#250](https://github.com/raylib-rs/raylib-rs/pull/250) | core: fix memory leak in export_image_to_memory() (fixes #247) | roccoblues | ds/fix-export-image-memory-leak | merge | WS3 — correct, targeted memory-safety fix; cherry-pick as-is |
| [#249](https://github.com/raylib-rs/raylib-rs/pull/249) | fixing test issue with TODO for next raylib release | meisei4 | file-paths-for-screenshot-todo | superseded | WS1 — the TODO referenced was for a future raylib release; raylib 6.0 is that release; WS1 re-pins raylib and regenerates bindings, resolving the underlying issue |
| [#242](https://github.com/raylib-rs/raylib-rs/pull/242) | Migrate Samples | Dacode45 | m/samples | adapt | WS9 — sample migration work in progress by maintainer; integrate into WS9 showcase pipeline |
| [#224](https://github.com/raylib-rs/raylib-rs/pull/224) | Clippy Cleanup | AmityWilder | clippy-cleanup | adapt | WS3 — large clippy pass; apply incrementally during WS3 as each module is touched, to avoid conflicts with rewrite |
| [#214](https://github.com/raylib-rs/raylib-rs/pull/214) | Working on CI | Dacode45 | all_platform_ci | adapt | WS6 — CI scaffolding by maintainer; subsume into WS6 full platform CI/CD redesign |
| [#211](https://github.com/raylib-rs/raylib-rs/pull/211) | Safety audit | AmityWilder | safety-audit | adapt | WS3 — large draft adding Safety docs to every unsafe fn; cherry-pick the doc additions module by module as WS3 touches each file, giving author attribution per file |
| [#204](https://github.com/raylib-rs/raylib-rs/pull/204) | Move Rectangle from raylib-sys -> raylib safe bindings and Add Rectangle methods | jestarray | rect-updates | superseded | WS2 — WS2 will decouple math types and introduce native types; the Rectangle move will happen as part of that redesign, rendering this standalone PR obsolete |
| [#118](https://github.com/raylib-rs/raylib-rs/pull/118) | Expose texcoords in a safe manner for Mesh | strizhkindenis | unstable | adapt | WS3 — safe texcoord accessor; WS3 mesh API pass will handle this with the broader accessor redesign |
| [#85](https://github.com/raylib-rs/raylib-rs/pull/85) | Port initial examples | t4ccer | t4/examples | adapt | WS9 — 4 examples ported; note color constant discrepancy flagged by author (LIGHTGRAY); absorb into WS9 example tracking, fix colors from raylib 6.0 constants |

## Issues

| Issue | Title | Labels | Disposition | Target WS / reason |
|-------|-------|--------|-------------|--------------------|
| [#294](https://github.com/raylib-rs/raylib-rs/issues/294) | Support Raylib 6.0 | — | adapt | WS1 — the primary tracking issue for 6.0 parity; WS1 is the direct response |
| [#291](https://github.com/raylib-rs/raylib-rs/issues/291) | Documentation recommends broken API in latest release (5.5.1) | — | adapt | WS7 — broken doc examples; fix during WS7 doc pass |
| [#290](https://github.com/raylib-rs/raylib-rs/issues/290) | Documentation links broken | — | adapt | WS7 — dead links in docs; sweep during WS7 |
| [#288](https://github.com/raylib-rs/raylib-rs/issues/288) | Add support for X11 when building with nix | — | adapt | WS6 — nix X11 build support; companion to PR #289, absorb in WS6 platform matrix |
| [#287](https://github.com/raylib-rs/raylib-rs/issues/287) | get_window_state not working as expected? | — | adapt | WS3 — bug closed by PR #252; track in WS3 merge of that PR |
| [#286](https://github.com/raylib-rs/raylib-rs/issues/286) | Flickering when using draw closure function | — | adapt | WS3 — investigate whether 6.0 draw API changes resolve this; if not, fix in WS3 |
| [#285](https://github.com/raylib-rs/raylib-rs/issues/285) | `get_monitor_count()` does not require a `RaylibHandle` | — | adapt | WS3 — API design issue; address during WS3 handle API review |
| [#283](https://github.com/raylib-rs/raylib-rs/issues/283) | Improper calculation of indices | — | adapt | WS3 — mesh index calculation bug; review during WS3 mesh pass |
| [#278](https://github.com/raylib-rs/raylib-rs/issues/278) | [Documentation/Policy] raylib-sys submodule version pin alignment policy | — | adapt | WS1 — submodule pinning policy should be documented as part of WS1 sys-parity work |
| [#276](https://github.com/raylib-rs/raylib-rs/issues/276) | `AsRef` and `AsMut` undermine safety | — | adapt | WS3 — closed by PR #277; track with that PR in WS3 wrapper redesign |
| [#274](https://github.com/raylib-rs/raylib-rs/issues/274) | Many config flags missing from `WindowBuilder` | — | adapt | WS3 — closed by PR #275; absorb in WS3 builder API pass |
| [#271](https://github.com/raylib-rs/raylib-rs/issues/271) | Unnecessary `CStr::from_bytes` on bytestring literals | — | adapt | WS3 — closed by PR #272; apply modernisation during WS3 |
| [#269](https://github.com/raylib-rs/raylib-rs/issues/269) | `ShaderV` impls are repetitive | — | adapt | WS3 — closed by PR #270; apply during WS3 shader pass |
| [#267](https://github.com/raylib-rs/raylib-rs/issues/267) | Types implement `Into` instead of `From` | enhancement | adapt | WS3 — closed by PR #268; convention fix, apply during WS3 |
| [#262](https://github.com/raylib-rs/raylib-rs/issues/262) | Accessors for `Mesh` are unsound | — | adapt | WS3 — closed by PRs #257 and #256; handle during WS3 mesh pass |
| [#258](https://github.com/raylib-rs/raylib-rs/issues/258) | Accessor for `mipmaps` returns width | — | adapt | WS3 — closed by PR #259; apply cherry-pick early |
| [#255](https://github.com/raylib-rs/raylib-rs/issues/255) | get_random_value uses exclusive range | — | adapt | WS3 — closed by PR #263; apply cherry-pick early |
| [#254](https://github.com/raylib-rs/raylib-rs/issues/254) | Fix for "Unable to find libclang" | — | adapt | WS1 — libclang/bindgen setup issue; document fix in WS1 build/dev-env setup |
| [#248](https://github.com/raylib-rs/raylib-rs/issues/248) | Task [opengl11] Bindgen omits opengl_<version> guarded prototypes | — | adapt | WS1 — bindgen cfg-guard handling; address during WS1 sys regeneration with proper feature flags |
| [#247](https://github.com/raylib-rs/raylib-rs/issues/247) | Memory leak in `Image.export_image_to_memory()` | — | adapt | WS3 — closed by PR #250; cherry-pick that PR early |
| [#246](https://github.com/raylib-rs/raylib-rs/issues/246) | Positions are messed up on multi-monitor setup | — | adapt | WS3/WS6 — multi-monitor coordinate bug; investigate during platform matrix work (WS6) and fix in safe API (WS3) |
| [#238](https://github.com/raylib-rs/raylib-rs/issues/238) | Carried bug from old raylib version — buffer overflow with PS controller | — | superseded | WS1 — upstream raylib bug in an old version; verify whether raylib 6.0 fixes it; close with a note if confirmed fixed upstream |
| [#237](https://github.com/raylib-rs/raylib-rs/issues/237) | Task: nobuild Issues and Investigation | — | adapt | WS1 — nobuild/pre-built binaries investigation; address in WS1 build system work |
| [#234](https://github.com/raylib-rs/raylib-rs/issues/234) | Safe bindings for raylib's OpenGL draw calls | — | adapt | WS5 (rlgl) — rlgl safe wrappers; WS5 covers rlgl bindings |
| [#227](https://github.com/raylib-rs/raylib-rs/issues/227) | broken aarch64-linux build | — | adapt | WS6 — aarch64 Linux build failure; address in WS6 platform matrix and CI |
| [#223](https://github.com/raylib-rs/raylib-rs/issues/223) | add topmost method to RaylibBuilder for FLAG_WINDOW_TOPMOST | — | adapt | WS3 — builder flag; companion to PR #275 discussion; add during WS3 builder pass |
| [#220](https://github.com/raylib-rs/raylib-rs/issues/220) | OsStr::to_string_lossy does not replace nul bytes | — | adapt | WS3 — string handling edge case; fix during WS3 string API review |
| [#213](https://github.com/raylib-rs/raylib-rs/issues/213) | `AudioSample` should be sealed | — | adapt | WS3 — closed by PR #266; cherry-pick that PR |
| [#210](https://github.com/raylib-rs/raylib-rs/issues/210) | Safety Audit | — | adapt | WS3 — tracking issue for PR #211 safety audit; absorb doc additions module by module during WS3 |
| [#209](https://github.com/raylib-rs/raylib-rs/issues/209) | Only sync raylib src folder | — | adapt | WS1 — submodule/vendoring hygiene; address in WS1 sys update |
| [#207](https://github.com/raylib-rs/raylib-rs/issues/207) | 3D Needs Work | — | adapt | WS3 — catch-all tracking issue for 3D API gaps; use as checklist during WS3 3D/mesh pass |
| [#181](https://github.com/raylib-rs/raylib-rs/issues/181) | Issue when cross-compiling from linux to windows in debug mode | — | adapt | WS6 — cross-compilation issue; address in WS6 platform/CI work |
| [#179](https://github.com/raylib-rs/raylib-rs/issues/179) | Implement ffi::rlPushMatrix family of functions | — | adapt | WS5 (rlgl) — rlgl matrix stack; WS5 covers these bindings |
| [#156](https://github.com/raylib-rs/raylib-rs/issues/156) | full recompilation every time | bug, question | adapt | WS1 — incremental build fingerprinting; investigate in WS1 build system work |
| [#137](https://github.com/raylib-rs/raylib-rs/issues/137) | deltaphc/raylib-rs issues | — | decline | Legacy issue referencing the original fork by deltaphc; all relevant bugs have been filed as separate issues; close with a comment directing to specific issues |
| [#135](https://github.com/raylib-rs/raylib-rs/issues/135) | Have the CI build for Windows again. | — | adapt | WS6 — Windows CI; WS6 platform/CI work will restore and expand Windows builds |

## Community branches

| Ref | What it is | Disposition | Target WS / reason |
|-----|-----------|-------------|--------------------|
| `origin/web` | Web/WASM build support branch | adapt | WS6 — Emscripten/WASM platform is in the platform matrix; rebase against 6.0 build system and integrate in WS6 platform CI pass |
| `origin/linux` | Linux-specific build fixes | adapt | WS6 — Linux build improvements; review commits and cherry-pick valid fixes during WS6 platform work |
| `origin/m/callback-fixes` | Fixes for Rust callback safety | adapt | WS3 — callback safety is part of the safe API surface; review and apply during WS3 |
| `origin/samples/platformer` | Platformer sample game | adapt | WS9 — example port; integrate into WS9 showcase pipeline |
| `origin/samples/space-eaters` | Space-eaters sample game | adapt | WS9 — example port; integrate into WS9 showcase pipeline |
| `origin/sample/roguelike` | Roguelike sample game | adapt | WS9 — example port; integrate into WS9 showcase pipeline |
| `origin/showcase-all` | All upstream raylib examples ported | adapt | WS9 — bulk example port; review completeness against raylib 6.0 example set, then use as basis for WS9 |
| `origin/tcod-remove` | Remove tcod dependency | adapt | WS1 — dependency cleanup; review and apply during WS1 sys build cleanup |
| `origin/sola-merge` | Merge of brettchalupa/sola-raylib | adapt | WS9 — sola game engine built on raylib-rs; extract useful example patterns and API feedback, integrate examples in WS9; do not bulk-merge |
| `sola/main` | brettchalupa/sola-raylib main branch | adapt | WS9 — upstream sola reference; monitor for API usage patterns that inform WS3/WS9 design; cherry-pick examples with attribution |
| `origin/m/fix-compilation-issues` | Miscellaneous compilation fixes | adapt | WS1/WS6 — review commits; apply valid fixes during WS1 sys work and WS6 platform/CI pass |
| `origin/indices_fix` | Mesh indices bug fix | adapt | WS3 — relates to #283 (improper index calculation); review and apply during WS3 mesh pass |
| `origin/contributing-safety` | Safety contribution guidelines | merge | WS7 — documentation improvement; cherry-pick into docs during WS7 |
| `origin/bindgen` | Experimental bindgen configuration | adapt | WS1 — bindgen tuning; review during WS1 sys regeneration for useful configuration options |
| `origin/pong` | Pong sample game | adapt | WS9 — classic example; integrate into WS9 showcase pipeline |
| `origin/m/samples` | In-progress sample migration (maintainer branch) | adapt | WS9 — Dacode45's sample migration work; this is the backing branch for PR #242; merge via PR process in WS9 |
