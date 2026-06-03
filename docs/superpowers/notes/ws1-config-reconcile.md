# WS1 — SUPPORT_* Feature Flag Reconciliation with raylib 6.0 config.h

**Date:** 2026-05-25
**Branch:** 6.0-rc
**Scope:** `raylib-sys/build.rs`, `raylib-sys/Cargo.toml`, `raylib/Cargo.toml`

## Methodology

Authoritative 6.0 flag set: `grep -oE "SUPPORT_[A-Z0-9_]+" raylib-sys/raylib/src/config.h | sort -u` → 55 flags.

Compared against flags in `build.rs` (`cmake.define` calls in `features_from_env`) and both Cargo manifests. Flags absent from the entire raylib 6.0 source tree (not just config.h, but also all `.c`/`.h` files) were removed. New flags present in config.h were added.

CMake note: `raylib-sys/raylib/cmake/CompileDefinitions.cmake` reads SUPPORT_ flags exclusively from `config.h` via `ParseConfigHeader.cmake`, so config.h is the authoritative source; no cmake-only SUPPORT_ overrides exist.

## Added (in 6.0 config.h, not in our files)

- `SUPPORT_FILEFORMAT_PNM` — PNM image format support; disabled by default in 6.0 (`#define SUPPORT_FILEFORMAT_PNM 0`). NOT added to `default` feature list.
- `SUPPORT_GPU_SKINNING` — GPU skinning for models; disabled by default in 6.0 (`#define SUPPORT_GPU_SKINNING 0`, comment: "some GPUs do not support more than 8 VBOs"). NOT added to `default` feature list.

## Removed (in our files, gone from all of raylib 6.0 source)

- `SUPPORT_DEFAULT_FONT` — removed from config.h and all raylib src in 6.0. Was in `default` features; removed from there too.
- `SUPPORT_DISTORTION_SHADER` — removed from config.h and all raylib src in 6.0.
- `SUPPORT_FONT_ATLAS_WHITE_REC` — removed from config.h and all raylib src in 6.0. Replaced by a non-configurable `FONT_ATLAS_CORNER_REC_SIZE` constant in rtext.c. Was in `default` features; removed from there too.
- `SUPPORT_FONT_TEXTURE` — removed from config.h and all raylib src in 6.0.
- `SUPPORT_GIF_RECORDING` — removed from config.h and all raylib src in 6.0. Was in `default` features; removed from there too.
- `SUPPORT_IMAGE_MANIPULATION` — removed from config.h and all raylib src in 6.0. Was in `default` features; removed from there too.
- `SUPPORT_STANDARD_FILEIO` — removed from config.h and all raylib src in 6.0. Was in `default` features; removed from there too. (Previously noted as "critical" for LoadShader — raylib 6.0 no longer gates this behind a compile flag.)
- `SUPPORT_TEXT_MANIPULATION` — removed from config.h and all raylib src in 6.0. Was in `default` features; removed from there too.
- `SUPPORT_VR_SIMULATOR` — removed from config.h and all raylib src in 6.0.

## Renamed

None identified. The removals above are true removals (functionality either unconditionally enabled, removed, or restructured), not renames.

## Kept-but-verify

None. All flags passing through `features_from_env` have been verified against config.h. The two new flags (`SUPPORT_FILEFORMAT_PNM`, `SUPPORT_GPU_SKINNING`) are confirmed present in config.h with default=0.

## Default Feature List Changes

The following flags were removed from `default = [...]` in `raylib-sys/Cargo.toml` because they no longer exist in raylib 6.0:

- `SUPPORT_GIF_RECORDING`
- `SUPPORT_IMAGE_MANIPULATION`
- `SUPPORT_DEFAULT_FONT`
- `SUPPORT_FONT_ATLAS_WHITE_REC`
- `SUPPORT_TEXT_MANIPULATION`
- `SUPPORT_STANDARD_FILEIO`

The two newly added flags (`SUPPORT_FILEFORMAT_PNM`, `SUPPORT_GPU_SKINNING`) were not added to `default` because raylib 6.0 disables them by default.

## Flag Counts

| Category | Count |
|----------|-------|
| Flags in 6.0 config.h | 55 |
| Added | 2 |
| Removed | 9 |
| Renamed | 0 |
| Kept-but-verify | 0 |
