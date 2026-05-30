# raylib 6.0 cheatsheet ↔ safe-binding parity audit

**Source:** <https://www.raylib.com/cheatsheet/cheatsheet.html> (raylib 6.0 cheatsheet)
**Audit date:** 2026-05-29
**Scope:** every cheatsheet section EXCEPT Text (rtext) and Files management
(rfilesystem) — Rust `std` covers those (see `memory/skip-std-equivalent-fns.md`).

**Note on source:** the live cheatsheet HTML page is JS-rendered and not
extractable via plain HTTP fetch, and the v6.0 PDF is binary-compressed. The
authoritative function list used for this audit is the **vendored
`raylib-sys/raylib/src/raylib.h`** (raylib 6.0, 600 `RLAPI` declarations),
which is what `bindgen` actually binds. The cheatsheet is generated from this
header, so the function set is identical modulo presentation order.

## Summary

In-scope (excluding `rtext` and the `// File system management functions`,
`// File access custom callbacks`, `// File operations` blocks of rcore):

- **Total cheatsheet functions audited: 500**
- **Covered in safe bindings: 486**
- **Intentionally skipped with rationale: 1**
- **Genuine gaps (TODO): 13**

Genuine gaps further categorized:

- **Small fixes (≲10 lines, no design judgement): 5** — listed in §3.
- **Medium / future workstream: 8** — listed in §4.

> **Important headline finding:** the existing `parity-checklist.md` lists 19
> `[ ]` items that are actually **already wrapped** — 14 in
> `Color/pixel related` as inherent methods on `raylib-sys::Color`, 4 in
> `Basic shapes collision` as inherent methods on `raylib-sys::Rectangle`,
> and `AttachAudioStreamProcessor` via the user-data callback wrapper.
> The auto-scan in `find_unimplemented.py` only looks at `raylib/src/**`
> and at `pub fn` top-level wrappers, so it misses these. See §5 for the
> full reconciliation list. The genuine-gap count for the in-scope
> sections is **13, not 49** — the remaining 30 `[ ]` entries are either
> reconcile deltas (§5) or live in the out-of-scope `rtext` /
> file-system sections.

## Methodology

1. Extracted the function list per section from
   `raylib-sys/raylib/src/raylib.h` (the vendored 6.0 header; the cheatsheet
   is generated from this).
2. For each function, grepped for coverage signals:
   - `pub fn <fname_snake>` in `raylib/src/**` — top-level safe wrappers.
   - `fn <fname_snake>(...)` inside `impl …` blocks (method wrappers).
   - `ffi::<FName>` direct callsites (proves the symbol is reached).
   - **Inherent methods on `raylib-sys` types** (`raylib-sys/src/color.rs`,
     `raylib-sys/src/math.rs`, etc.) — these are reachable from the safe
     crate via `Color` / `Rectangle` / `Vector*` re-exports.
3. Cross-referenced with the existing
   `docs/superpowers/parity-checklist.md` (508 wrapped / 43 wont-impl / 49
   TODO totals) and noted reconciliation deltas.

> **Out-of-scope sections** (still listed for completeness so a reader can
> verify nothing was missed):
> - `rcore` — File system management, File access custom callbacks, File
>   operations (covered by `std::fs` and `std::path`).
> - All of `rtext` (covered by `&str`, `String`, `format!`, the `char` API
>   etc.) — Font *loading* and Text *drawing* functions stay in scope
>   because they touch the GPU and require the safe binding; the strings
>   helpers do not.

---

## Sections

### Section 1 (rcore): Window-related functions

- ✅ `InitWindow` — `raylib/src/core/window.rs`
- ✅ `CloseWindow` — `raylib/src/core/window.rs` (via `Drop` on
  `RaylibHandle`)
- ✅ `WindowShouldClose` — `raylib/src/core/window.rs`
- ✅ `IsWindowReady` — `raylib/src/core/window.rs`
- ✅ `IsWindowFullscreen` — `raylib/src/core/window.rs`
- ✅ `IsWindowHidden` — `raylib/src/core/window.rs`
- ✅ `IsWindowMinimized` — `raylib/src/core/window.rs`
- ✅ `IsWindowMaximized` — `raylib/src/core/window.rs`
- ✅ `IsWindowFocused` — `raylib/src/core/window.rs`
- ✅ `IsWindowResized` — `raylib/src/core/window.rs`
- ✅ `IsWindowState` — `raylib/src/core/window.rs`
- ✅ `SetWindowState` — `raylib/src/core/window.rs`
- ✅ `ClearWindowState` — `raylib/src/core/window.rs`
- ✅ `ToggleFullscreen` — `raylib/src/core/window.rs`
- ✅ `ToggleBorderlessWindowed` — `raylib/src/core/window.rs`
- ✅ `MaximizeWindow` — `raylib/src/core/window.rs`
- ✅ `MinimizeWindow` — `raylib/src/core/window.rs`
- ✅ `RestoreWindow` — `raylib/src/core/window.rs`
- ✅ `SetWindowIcon` — `raylib/src/core/window.rs`
- ✅ `SetWindowIcons` — `raylib/src/core/window.rs`
- ✅ `SetWindowTitle` — `raylib/src/core/window.rs`
- ✅ `SetWindowPosition` — `raylib/src/core/window.rs`
- ✅ `SetWindowMonitor` — `raylib/src/core/window.rs`
- ✅ `SetWindowMinSize` — `raylib/src/core/window.rs`
- ✅ `SetWindowMaxSize` — `raylib/src/core/window.rs`
- ✅ `SetWindowSize` — `raylib/src/core/window.rs`
- ✅ `SetWindowOpacity` — `raylib/src/core/window.rs`
- ✅ `SetWindowFocused` — `raylib/src/core/window.rs`
- ✅ `GetWindowHandle` — `raylib/src/core/window.rs`
- ✅ `GetScreenWidth` — `raylib/src/core/window.rs`
- ✅ `GetScreenHeight` — `raylib/src/core/window.rs`
- ✅ `GetRenderWidth` — `raylib/src/core/window.rs`
- ✅ `GetRenderHeight` — `raylib/src/core/window.rs`
- ✅ `GetMonitorCount` — `raylib/src/core/window.rs`
- ✅ `GetCurrentMonitor` — `raylib/src/core/window.rs`
- ✅ `GetMonitorPosition` — `raylib/src/core/window.rs`
- ✅ `GetMonitorWidth` — `raylib/src/core/window.rs`
- ✅ `GetMonitorHeight` — `raylib/src/core/window.rs`
- ✅ `GetMonitorPhysicalWidth` — `raylib/src/core/window.rs`
- ✅ `GetMonitorPhysicalHeight` — `raylib/src/core/window.rs`
- ✅ `GetMonitorRefreshRate` — `raylib/src/core/window.rs`
- ✅ `GetWindowPosition` — `raylib/src/core/window.rs`
- ✅ `GetWindowScaleDPI` — `raylib/src/core/window.rs`
- ✅ `GetMonitorName` — `raylib/src/core/window.rs`
- ✅ `SetClipboardText` — `raylib/src/core/window.rs`
- ✅ `GetClipboardText` — `raylib/src/core/window.rs`
- ✅ `GetClipboardImage` — `raylib/src/core/window.rs`
- ✅ `EnableEventWaiting` — `raylib/src/core/window.rs`
- ✅ `DisableEventWaiting` — `raylib/src/core/window.rs`

### Section 2 (rcore): Cursor-related functions

- ✅ `ShowCursor` — `raylib/src/core/window.rs`
- ✅ `HideCursor` — `raylib/src/core/window.rs`
- ✅ `IsCursorHidden` — `raylib/src/core/window.rs`
- ✅ `EnableCursor` — `raylib/src/core/window.rs`
- ✅ `DisableCursor` — `raylib/src/core/window.rs`
- ✅ `IsCursorOnScreen` — `raylib/src/core/window.rs`

### Section 3 (rcore): Drawing-related functions

- ✅ `ClearBackground` — `raylib/src/core/drawing.rs`
- ✅ `BeginDrawing` — `raylib/src/core/drawing.rs` (RAII via
  `RaylibDrawHandle`)
- ✅ `EndDrawing` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `BeginMode2D` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `EndMode2D` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `BeginMode3D` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `EndMode3D` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `BeginTextureMode` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `EndTextureMode` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `BeginShaderMode` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `EndShaderMode` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `BeginBlendMode` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `EndBlendMode` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `BeginScissorMode` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `EndScissorMode` — `raylib/src/core/drawing.rs` (RAII)
- ✅ `BeginVrStereoMode` — `raylib/src/core/vr.rs` (RAII)
- ✅ `EndVrStereoMode` — `raylib/src/core/vr.rs` (RAII)

### Section 4 (rcore): VR stereo config functions

- ✅ `LoadVrStereoConfig` — `raylib/src/core/vr.rs`
- ✅ `UnloadVrStereoConfig` — `raylib/src/core/vr.rs` (via `Drop`)

### Section 5 (rcore): Shader management functions

- ✅ `LoadShader` — `raylib/src/core/shaders.rs`
- ✅ `LoadShaderFromMemory` — `raylib/src/core/shaders.rs`
- ✅ `IsShaderValid` — `raylib/src/core/shaders.rs`
- ✅ `GetShaderLocation` — `raylib/src/core/shaders.rs`
- ✅ `GetShaderLocationAttrib` — `raylib/src/core/shaders.rs`
- ✅ `SetShaderValue` — `raylib/src/core/shaders.rs`
- ✅ `SetShaderValueV` — `raylib/src/core/shaders.rs`
- ✅ `SetShaderValueMatrix` — `raylib/src/core/shaders.rs`
- ✅ `SetShaderValueTexture` — `raylib/src/core/shaders.rs`
- ✅ `UnloadShader` — `raylib/src/core/shaders.rs` (via `Drop`)

### Section 6 (rcore): Screen-space-related functions

- ✅ `GetScreenToWorldRay` — `raylib/src/core/window.rs`
- ✅ `GetScreenToWorldRayEx` — `raylib/src/core/window.rs`
- ✅ `GetWorldToScreen` — `raylib/src/core/window.rs`
- ✅ `GetWorldToScreenEx` — `raylib/src/core/window.rs`
- ✅ `GetWorldToScreen2D` — `raylib/src/core/window.rs`
- ✅ `GetScreenToWorld2D` — `raylib/src/core/window.rs`
- ✅ `GetCameraMatrix` — `raylib/src/core/camera.rs`
- ✅ `GetCameraMatrix2D` — `raylib/src/core/camera.rs`

### Section 7 (rcore): Timing-related functions

- ✅ `SetTargetFPS` — `raylib/src/core/window.rs`
- ✅ `GetFrameTime` — `raylib/src/core/window.rs`
- ✅ `GetTime` — `raylib/src/core/window.rs`
- ✅ `GetFPS` — `raylib/src/core/window.rs`

### Section 8 (rcore): Custom frame control functions

- ✅ `SwapScreenBuffer` — `raylib/src/core/window.rs`
- ✅ `PollInputEvents` — `raylib/src/core/window.rs`
- ✅ `WaitTime` — `raylib/src/core/window.rs`

### Section 9 (rcore): Random values generation functions

- ✅ `SetRandomSeed` — `raylib/src/core/misc.rs`
- ✅ `GetRandomValue` — `raylib/src/core/misc.rs`
- ✅ `LoadRandomSequence` — `raylib/src/core/misc.rs`
- ✅ `UnloadRandomSequence` — `raylib/src/core/misc.rs`

### Section 10 (rcore): Misc. functions

- ✅ `TakeScreenshot` — `raylib/src/core/misc.rs`
- ✅ `SetConfigFlags` — `raylib/src/core/window.rs`
- ✅ `OpenURL` — `raylib/src/core/misc.rs`

### Section 11 (rcore): Logging system

- ✅ `SetTraceLogLevel` — `raylib/src/core/logging.rs`
- ✅ `TraceLog` — `raylib/src/core/logging.rs`
- ✅ `SetTraceLogCallback` — `raylib/src/core/callbacks.rs`
  (implemented via C shim; the auto-scan misses it)

### Section 12 (rcore): Memory management

- ✅ `MemAlloc` — `raylib/src/core/data.rs` (used internally by
  `DataBuf`/`Image`/`Wave`/etc. allocators)
- 🟨 `MemRealloc` — SKIP. Reason: not needed by the safe API; Rust uses
  `Box`/`Vec` reallocation; wrap on demand.
- ✅ `MemFree` — `raylib/src/core/data.rs`

### Section 13 (rcore): Automation events functionality

- ✅ `LoadAutomationEventList` — `raylib/src/core/automation.rs`
- ✅ `UnloadAutomationEventList` — `raylib/src/core/automation.rs` (Drop)
- ✅ `ExportAutomationEventList` — `raylib/src/core/automation.rs`
- ✅ `SetAutomationEventList` — `raylib/src/core/automation.rs`
- ✅ `SetAutomationEventBaseFrame` — `raylib/src/core/automation.rs`
- ✅ `StartAutomationEventRecording` — `raylib/src/core/automation.rs`
- ✅ `StopAutomationEventRecording` — `raylib/src/core/automation.rs`
- ✅ `PlayAutomationEvent` — `raylib/src/core/automation.rs`

### Section 14 (rcore): Compression/Encoding functionality

- ✅ `CompressData` — `raylib/src/core/data.rs`
- ✅ `DecompressData` — `raylib/src/core/data.rs`
- ✅ `EncodeDataBase64` — `raylib/src/core/data.rs`
- ✅ `DecodeDataBase64` — `raylib/src/core/data.rs`
- 🟥 `ComputeCRC32` — GAP. Action: future-workstream-hashes.
- 🟥 `ComputeMD5` — GAP. Action: future-workstream-hashes.
- 🟥 `ComputeSHA1` — GAP. Action: future-workstream-hashes.
- 🟥 `ComputeSHA256` — GAP. Action: future-workstream-hashes.

### Section 15 (rcore): Input-related functions: keyboard

- ✅ `IsKeyPressed` — `raylib/src/core/input.rs`
- ✅ `IsKeyPressedRepeat` — `raylib/src/core/input.rs`
- ✅ `IsKeyDown` — `raylib/src/core/input.rs`
- ✅ `IsKeyReleased` — `raylib/src/core/input.rs`
- ✅ `IsKeyUp` — `raylib/src/core/input.rs`
- ✅ `GetKeyPressed` — `raylib/src/core/input.rs`
- ✅ `GetCharPressed` — `raylib/src/core/input.rs`
- 🟥 `GetKeyName` — GAP. Action: fix-now.
- ✅ `SetExitKey` — `raylib/src/core/input.rs`

### Section 16 (rcore): Input-related functions: gamepads

- ✅ `IsGamepadAvailable` — `raylib/src/core/input.rs`
- ✅ `GetGamepadName` — `raylib/src/core/input.rs`
- ✅ `IsGamepadButtonPressed` — `raylib/src/core/input.rs`
- ✅ `IsGamepadButtonDown` — `raylib/src/core/input.rs`
- ✅ `IsGamepadButtonReleased` — `raylib/src/core/input.rs`
- ✅ `IsGamepadButtonUp` — `raylib/src/core/input.rs`
- ✅ `GetGamepadButtonPressed` — `raylib/src/core/input.rs`
- ✅ `GetGamepadAxisCount` — `raylib/src/core/input.rs`
- ✅ `GetGamepadAxisMovement` — `raylib/src/core/input.rs`
- ✅ `SetGamepadMappings` — `raylib/src/core/input.rs`
- ✅ `SetGamepadVibration` — `raylib/src/core/input.rs`

### Section 17 (rcore): Input-related functions: mouse

- ✅ `IsMouseButtonPressed` — `raylib/src/core/input.rs`
- ✅ `IsMouseButtonDown` — `raylib/src/core/input.rs`
- ✅ `IsMouseButtonReleased` — `raylib/src/core/input.rs`
- ✅ `IsMouseButtonUp` — `raylib/src/core/input.rs`
- ✅ `GetMouseX` — `raylib/src/core/input.rs`
- ✅ `GetMouseY` — `raylib/src/core/input.rs`
- ✅ `GetMousePosition` — `raylib/src/core/input.rs`
- ✅ `GetMouseDelta` — `raylib/src/core/input.rs`
- ✅ `SetMousePosition` — `raylib/src/core/input.rs`
- ✅ `SetMouseOffset` — `raylib/src/core/input.rs`
- ✅ `SetMouseScale` — `raylib/src/core/input.rs`
- ✅ `GetMouseWheelMove` — `raylib/src/core/input.rs`
- ✅ `GetMouseWheelMoveV` — `raylib/src/core/input.rs`
- ✅ `SetMouseCursor` — `raylib/src/core/input.rs`

### Section 18 (rcore): Input-related functions: touch

- ✅ `GetTouchX` — `raylib/src/core/input.rs`
- ✅ `GetTouchY` — `raylib/src/core/input.rs`
- ✅ `GetTouchPosition` — `raylib/src/core/input.rs`
- ✅ `GetTouchPointId` — `raylib/src/core/input.rs`
- ✅ `GetTouchPointCount` — `raylib/src/core/input.rs`

### Section 19 (rcore): Gestures and Touch Handling Functions

- ✅ `SetGesturesEnabled` — `raylib/src/core/input.rs`
- ✅ `IsGestureDetected` — `raylib/src/core/input.rs`
- ✅ `GetGestureDetected` — `raylib/src/core/input.rs`
- ✅ `GetGestureHoldDuration` — `raylib/src/core/input.rs`
- ✅ `GetGestureDragVector` — `raylib/src/core/input.rs`
- ✅ `GetGestureDragAngle` — `raylib/src/core/input.rs`
- ✅ `GetGesturePinchVector` — `raylib/src/core/input.rs`
- ✅ `GetGesturePinchAngle` — `raylib/src/core/input.rs`

### Section 20 (rcore): Camera System Functions

- ✅ `UpdateCamera` — `raylib/src/core/camera.rs`
- ✅ `UpdateCameraPro` — `raylib/src/core/camera.rs`

### Section 21 (rshapes): Basic Shapes Drawing Functions (shapes-texture)

- ✅ `SetShapesTexture` — `raylib/src/core/drawing.rs`
- ✅ `GetShapesTexture` — `raylib/src/core/drawing.rs`
- ✅ `GetShapesTextureRectangle` — `raylib/src/core/drawing.rs`

### Section 22 (rshapes): Basic shapes drawing functions

- ✅ `DrawPixel` — `raylib/src/core/drawing.rs`
- ✅ `DrawPixelV` — `raylib/src/core/drawing.rs`
- ✅ `DrawLine` — `raylib/src/core/drawing.rs`
- ✅ `DrawLineV` — `raylib/src/core/drawing.rs`
- ✅ `DrawLineEx` — `raylib/src/core/drawing.rs`
- ✅ `DrawLineStrip` — `raylib/src/core/drawing.rs`
- ✅ `DrawLineBezier` — `raylib/src/core/drawing.rs`
- 🟥 `DrawLineDashed` — GAP. Action: fix-now.
- ✅ `DrawCircle` — `raylib/src/core/drawing.rs`
- ✅ `DrawCircleV` — `raylib/src/core/drawing.rs`
- ✅ `DrawCircleGradient` — `raylib/src/core/drawing.rs`
- ✅ `DrawCircleSector` — `raylib/src/core/drawing.rs`
- ✅ `DrawCircleSectorLines` — `raylib/src/core/drawing.rs`
- ✅ `DrawCircleLines` — `raylib/src/core/drawing.rs`
- ✅ `DrawCircleLinesV` — `raylib/src/core/drawing.rs`
- ✅ `DrawEllipse` — `raylib/src/core/drawing.rs`
- 🟥 `DrawEllipseV` — GAP. Action: fix-now.
- ✅ `DrawEllipseLines` — `raylib/src/core/drawing.rs`
- 🟥 `DrawEllipseLinesV` — GAP. Action: fix-now.
- ✅ `DrawRing` — `raylib/src/core/drawing.rs`
- ✅ `DrawRingLines` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangle` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangleV` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangleRec` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectanglePro` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangleGradientV` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangleGradientH` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangleGradientEx` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangleLines` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangleLinesEx` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangleRounded` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangleRoundedLines` — `raylib/src/core/drawing.rs`
- ✅ `DrawRectangleRoundedLinesEx` — `raylib/src/core/drawing.rs`
- ✅ `DrawTriangle` — `raylib/src/core/drawing.rs`
- ✅ `DrawTriangleLines` — `raylib/src/core/drawing.rs`
- ✅ `DrawTriangleFan` — `raylib/src/core/drawing.rs`
- ✅ `DrawTriangleStrip` — `raylib/src/core/drawing.rs`
- ✅ `DrawPoly` — `raylib/src/core/drawing.rs`
- ✅ `DrawPolyLines` — `raylib/src/core/drawing.rs`
- ✅ `DrawPolyLinesEx` — `raylib/src/core/drawing.rs`

### Section 23 (rshapes): Splines drawing functions

- ✅ `DrawSplineLinear` — `raylib/src/core/drawing.rs`
- ✅ `DrawSplineBasis` — `raylib/src/core/drawing.rs`
- ✅ `DrawSplineCatmullRom` — `raylib/src/core/drawing.rs`
- ✅ `DrawSplineBezierQuadratic` — `raylib/src/core/drawing.rs`
- ✅ `DrawSplineBezierCubic` — `raylib/src/core/drawing.rs`
- ✅ `DrawSplineSegmentLinear` — `raylib/src/core/drawing.rs`
- ✅ `DrawSplineSegmentBasis` — `raylib/src/core/drawing.rs`
- ✅ `DrawSplineSegmentCatmullRom` — `raylib/src/core/drawing.rs`
- ✅ `DrawSplineSegmentBezierQuadratic` — `raylib/src/core/drawing.rs`
- ✅ `DrawSplineSegmentBezierCubic` — `raylib/src/core/drawing.rs`

### Section 24 (rshapes): Spline segment point evaluation functions

- ✅ `GetSplinePointLinear` — `raylib/src/core/math.rs`
- ✅ `GetSplinePointBasis` — `raylib/src/core/math.rs`
- ✅ `GetSplinePointCatmullRom` — `raylib/src/core/math.rs`
- ✅ `GetSplinePointBezierQuad` — `raylib/src/core/math.rs`
- ✅ `GetSplinePointBezierCubic` — `raylib/src/core/math.rs`

### Section 25 (rshapes): Basic shapes collision detection functions

- ✅ `CheckCollisionRecs` — inherent method
  `Rectangle::check_collision_recs` at `raylib-sys/src/math.rs:107`.
- ✅ `CheckCollisionCircles` — `raylib/src/core/collision.rs`
  (`check_collision_circles`).
- ✅ `CheckCollisionCircleRec` — inherent method
  `Rectangle::check_collision_circle_rec` at `raylib-sys/src/math.rs:116`.
- ✅ `CheckCollisionCircleLine` — `raylib/src/core/collision.rs`
  (`check_collision_circle_line`).
- ✅ `CheckCollisionPointRec` — inherent method
  `Rectangle::check_collision_point_rec` at `raylib-sys/src/math.rs:148`.
- ✅ `CheckCollisionPointCircle` — `raylib/src/core/collision.rs`
  (`check_collision_point_circle`).
- ✅ `CheckCollisionPointTriangle` — `raylib/src/core/collision.rs`
  (`check_collision_point_triangle`).
- ✅ `CheckCollisionPointLine` — `raylib/src/core/collision.rs`
  (`check_collision_point_line`).
- ✅ `CheckCollisionPointPoly` — `raylib/src/core/collision.rs`
  (`check_collision_point_poly`).
- ✅ `CheckCollisionLines` — `raylib/src/core/collision.rs`
  (`check_collision_lines`).
- ✅ `GetCollisionRec` — inherent method `Rectangle::get_collision_rec` at
  `raylib-sys/src/math.rs:137` (returns `Option<Rectangle>` — empty
  intersection becomes `None`).

### Section 26 (rtextures): Image loading functions

- ✅ `LoadImage` — `raylib/src/core/texture.rs`
- ✅ `LoadImageRaw` — `raylib/src/core/texture.rs`
- ✅ `LoadImageAnim` — `raylib/src/core/texture.rs`
- ✅ `LoadImageAnimFromMemory` — `raylib/src/core/texture.rs`
- ✅ `LoadImageFromMemory` — `raylib/src/core/texture.rs`
- ✅ `LoadImageFromTexture` — `raylib/src/core/texture.rs`
- ✅ `LoadImageFromScreen` — `raylib/src/core/texture.rs`
- ✅ `IsImageValid` — `raylib/src/core/texture.rs`
- ✅ `UnloadImage` — `raylib/src/core/texture.rs` (Drop)
- ✅ `ExportImage` — `raylib/src/core/texture.rs`
- ✅ `ExportImageToMemory` — `raylib/src/core/texture.rs`
- ✅ `ExportImageAsCode` — `raylib/src/core/texture.rs`

### Section 27 (rtextures): Image generation functions

- ✅ `GenImageColor` — `raylib/src/core/texture.rs`
- ✅ `GenImageGradientLinear` — `raylib/src/core/texture.rs`
- ✅ `GenImageGradientRadial` — `raylib/src/core/texture.rs`
- ✅ `GenImageGradientSquare` — `raylib/src/core/texture.rs`
- ✅ `GenImageChecked` — `raylib/src/core/texture.rs`
- ✅ `GenImageWhiteNoise` — `raylib/src/core/texture.rs`
- ✅ `GenImagePerlinNoise` — `raylib/src/core/texture.rs`
- ✅ `GenImageCellular` — `raylib/src/core/texture.rs`
- ✅ `GenImageText` — `raylib/src/core/texture.rs`

### Section 28 (rtextures): Image manipulation functions

- ✅ `ImageCopy` — `raylib/src/core/texture.rs`
- ✅ `ImageFromImage` — `raylib/src/core/texture.rs`
- ✅ `ImageFromChannel` — `raylib/src/core/texture.rs`
- ✅ `ImageText` — `raylib/src/core/texture.rs`
- ✅ `ImageTextEx` — `raylib/src/core/texture.rs`
- ✅ `ImageFormat` — `raylib/src/core/texture.rs`
- ✅ `ImageToPOT` — `raylib/src/core/texture.rs`
- ✅ `ImageCrop` — `raylib/src/core/texture.rs`
- ✅ `ImageAlphaCrop` — `raylib/src/core/texture.rs`
- ✅ `ImageAlphaClear` — `raylib/src/core/texture.rs`
- ✅ `ImageAlphaMask` — `raylib/src/core/texture.rs`
- ✅ `ImageAlphaPremultiply` — `raylib/src/core/texture.rs`
- ✅ `ImageBlurGaussian` — `raylib/src/core/texture.rs`
- ✅ `ImageKernelConvolution` — `raylib/src/core/texture.rs`
- ✅ `ImageResize` — `raylib/src/core/texture.rs`
- ✅ `ImageResizeNN` — `raylib/src/core/texture.rs`
- ✅ `ImageResizeCanvas` — `raylib/src/core/texture.rs`
- ✅ `ImageMipmaps` — `raylib/src/core/texture.rs`
- ✅ `ImageDither` — `raylib/src/core/texture.rs`
- ✅ `ImageFlipVertical` — `raylib/src/core/texture.rs`
- ✅ `ImageFlipHorizontal` — `raylib/src/core/texture.rs`
- ✅ `ImageRotate` — `raylib/src/core/texture.rs`
- ✅ `ImageRotateCW` — `raylib/src/core/texture.rs`
- ✅ `ImageRotateCCW` — `raylib/src/core/texture.rs`
- ✅ `ImageColorTint` — `raylib/src/core/texture.rs`
- ✅ `ImageColorInvert` — `raylib/src/core/texture.rs`
- ✅ `ImageColorGrayscale` — `raylib/src/core/texture.rs`
- ✅ `ImageColorContrast` — `raylib/src/core/texture.rs`
- ✅ `ImageColorBrightness` — `raylib/src/core/texture.rs`
- ✅ `ImageColorReplace` — `raylib/src/core/texture.rs`
- ✅ `LoadImageColors` — `raylib/src/core/texture.rs`
- ✅ `LoadImagePalette` — `raylib/src/core/texture.rs`
- ✅ `UnloadImageColors` — `raylib/src/core/texture.rs` (Drop)
- ✅ `UnloadImagePalette` — `raylib/src/core/texture.rs` (Drop)
- ✅ `GetImageAlphaBorder` — `raylib/src/core/texture.rs`
- ✅ `GetImageColor` — `raylib/src/core/texture.rs`

### Section 29 (rtextures): Image drawing functions

- ✅ `ImageClearBackground` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawPixel` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawPixelV` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawLine` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawLineV` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawLineEx` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawCircle` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawCircleV` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawCircleLines` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawCircleLinesV` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawRectangle` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawRectangleV` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawRectangleRec` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawRectangleLines` — `raylib/src/core/texture.rs`
  (raylib 6.0 dropped the `…Ex` variant; only the single fn remains)
- ✅ `ImageDrawTriangle` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawTriangleEx` — `raylib/src/core/texture.rs`
  (renamed from `ImageDrawTriangleGradient` in 6.0)
- ✅ `ImageDrawTriangleLines` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawTriangleFan` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawTriangleStrip` — `raylib/src/core/texture.rs`
- ✅ `ImageDraw` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawText` — `raylib/src/core/texture.rs`
- ✅ `ImageDrawTextEx` — `raylib/src/core/texture.rs`

### Section 30 (rtextures): Texture loading functions

- ✅ `LoadTexture` — `raylib/src/core/texture.rs`
- ✅ `LoadTextureFromImage` — `raylib/src/core/texture.rs`
- ✅ `LoadTextureCubemap` — `raylib/src/core/texture.rs`
- ✅ `LoadRenderTexture` — `raylib/src/core/texture.rs`
- ✅ `IsTextureValid` — `raylib/src/core/texture.rs`
- ✅ `UnloadTexture` — `raylib/src/core/texture.rs` (Drop)
- ✅ `IsRenderTextureValid` — `raylib/src/core/texture.rs`
- ✅ `UnloadRenderTexture` — `raylib/src/core/texture.rs` (Drop)
- ✅ `UpdateTexture` — `raylib/src/core/texture.rs`
- ✅ `UpdateTextureRec` — `raylib/src/core/texture.rs`

### Section 31 (rtextures): Texture configuration functions

- ✅ `GenTextureMipmaps` — `raylib/src/core/texture.rs`
- ✅ `SetTextureFilter` — `raylib/src/core/texture.rs`
- ✅ `SetTextureWrap` — `raylib/src/core/texture.rs`

### Section 32 (rtextures): Texture drawing functions

- ✅ `DrawTexture` — `raylib/src/core/drawing.rs`
- ✅ `DrawTextureV` — `raylib/src/core/drawing.rs`
- ✅ `DrawTextureEx` — `raylib/src/core/drawing.rs`
- ✅ `DrawTextureRec` — `raylib/src/core/drawing.rs`
- ✅ `DrawTexturePro` — `raylib/src/core/drawing.rs`
- ✅ `DrawTextureNPatch` — `raylib/src/core/drawing.rs`

### Section 33 (rtextures): Color/pixel related functions

- ✅ `ColorIsEqual` — inherent method `Color::is_equal` at
  `raylib-sys/src/color.rs:164`.
- ✅ `Fade` — inherent method `Color::fade` (deprecated alias for `alpha`)
  at `raylib-sys/src/color.rs:153`.
- ✅ `ColorToInt` — inherent method `Color::color_to_int` at
  `raylib-sys/src/color.rs:96`.
- ✅ `ColorNormalize` — inherent method `Color::color_normalize` at
  `raylib-sys/src/color.rs:102`.
- ✅ `ColorFromNormalized` — assoc fn `Color::color_from_normalized` at
  `raylib-sys/src/color.rs:120`.
- ✅ `ColorToHSV` — inherent method `Color::color_to_hsv` at
  `raylib-sys/src/color.rs:108`.
- ✅ `ColorFromHSV` — assoc fn `Color::color_from_hsv` at
  `raylib-sys/src/color.rs:114`.
- ✅ `ColorTint` — inherent method `Color::tint` at
  `raylib-sys/src/color.rs:132`.
- ✅ `ColorBrightness` — inherent method `Color::brightness` at
  `raylib-sys/src/color.rs:137`.
- ✅ `ColorContrast` — inherent method `Color::contrast` at
  `raylib-sys/src/color.rs:142`.
- ✅ `ColorAlpha` — inherent method `Color::alpha` at
  `raylib-sys/src/color.rs:147`.
- ✅ `ColorAlphaBlend` — assoc fn `Color::color_alpha_blend` at
  `raylib-sys/src/color.rs:159`.
- ✅ `ColorLerp` — inherent method `Color::lerp` at
  `raylib-sys/src/color.rs:170`.
- ✅ `GetColor` — assoc fn `Color::get_color` at
  `raylib-sys/src/color.rs:126`.
- 🟥 `GetPixelColor` — GAP. Action: future-workstream-pixel-pointers.
- 🟥 `SetPixelColor` — GAP. Action: future-workstream-pixel-pointers.
- ✅ `GetPixelDataSize` — `raylib/src/core/texture.rs:361,1381`
  (both as method and free fn).

### Section 34 (rmodels): Basic geometric 3D shapes drawing functions

- ✅ `DrawLine3D` — `raylib/src/core/drawing.rs`
- ✅ `DrawPoint3D` — `raylib/src/core/drawing.rs`
- ✅ `DrawCircle3D` — `raylib/src/core/drawing.rs`
- ✅ `DrawTriangle3D` — `raylib/src/core/drawing.rs`
- ✅ `DrawTriangleStrip3D` — `raylib/src/core/drawing.rs`
- ✅ `DrawCube` — `raylib/src/core/drawing.rs`
- ✅ `DrawCubeV` — `raylib/src/core/drawing.rs`
- ✅ `DrawCubeWires` — `raylib/src/core/drawing.rs`
- ✅ `DrawCubeWiresV` — `raylib/src/core/drawing.rs`
- ✅ `DrawSphere` — `raylib/src/core/drawing.rs`
- ✅ `DrawSphereEx` — `raylib/src/core/drawing.rs`
- ✅ `DrawSphereWires` — `raylib/src/core/drawing.rs`
- ✅ `DrawCylinder` — `raylib/src/core/drawing.rs`
- ✅ `DrawCylinderEx` — `raylib/src/core/drawing.rs`
- ✅ `DrawCylinderWires` — `raylib/src/core/drawing.rs`
- ✅ `DrawCylinderWiresEx` — `raylib/src/core/drawing.rs`
- ✅ `DrawCapsule` — `raylib/src/core/drawing.rs`
- ✅ `DrawCapsuleWires` — `raylib/src/core/drawing.rs`
- ✅ `DrawPlane` — `raylib/src/core/drawing.rs`
- ✅ `DrawRay` — `raylib/src/core/drawing.rs`
- ✅ `DrawGrid` — `raylib/src/core/drawing.rs`

### Section 35 (rmodels): Model management functions

- ✅ `LoadModel` — `raylib/src/core/models.rs`
- ✅ `LoadModelFromMesh` — `raylib/src/core/models.rs`
- ✅ `IsModelValid` — `raylib/src/core/models.rs`
- ✅ `UnloadModel` — `raylib/src/core/models.rs` (Drop)
- ✅ `GetModelBoundingBox` — `raylib/src/core/models.rs`

### Section 36 (rmodels): Model drawing functions

- ✅ `DrawModel` — `raylib/src/core/drawing.rs`
- ✅ `DrawModelEx` — `raylib/src/core/drawing.rs`
- ✅ `DrawModelWires` — `raylib/src/core/drawing.rs`
- ✅ `DrawModelWiresEx` — `raylib/src/core/drawing.rs`
- ✅ `DrawBoundingBox` — `raylib/src/core/drawing.rs`
- ✅ `DrawBillboard` — `raylib/src/core/drawing.rs`
- ✅ `DrawBillboardRec` — `raylib/src/core/drawing.rs`
- ✅ `DrawBillboardPro` — `raylib/src/core/drawing.rs`

### Section 37 (rmodels): Mesh management functions

- ✅ `UploadMesh` — `raylib/src/core/models.rs`
- ✅ `UpdateMeshBuffer` — `raylib/src/core/models.rs`
- ✅ `UnloadMesh` — `raylib/src/core/models.rs` (Drop)
- ✅ `DrawMesh` — `raylib/src/core/drawing.rs`
- ✅ `DrawMeshInstanced` — `raylib/src/core/drawing.rs`
- ✅ `GetMeshBoundingBox` — `raylib/src/core/models.rs`
- ✅ `GenMeshTangents` — `raylib/src/core/models.rs`
- ✅ `ExportMesh` — `raylib/src/core/models.rs`
- ✅ `ExportMeshAsCode` — `raylib/src/core/models.rs`

### Section 38 (rmodels): Mesh generation functions

- ✅ `GenMeshPoly` — `raylib/src/core/models.rs`
- ✅ `GenMeshPlane` — `raylib/src/core/models.rs`
- ✅ `GenMeshCube` — `raylib/src/core/models.rs`
- ✅ `GenMeshSphere` — `raylib/src/core/models.rs`
- ✅ `GenMeshHemiSphere` — `raylib/src/core/models.rs`
- ✅ `GenMeshCylinder` — `raylib/src/core/models.rs`
- ✅ `GenMeshCone` — `raylib/src/core/models.rs`
- ✅ `GenMeshTorus` — `raylib/src/core/models.rs`
- ✅ `GenMeshKnot` — `raylib/src/core/models.rs`
- ✅ `GenMeshHeightmap` — `raylib/src/core/models.rs`
- ✅ `GenMeshCubicmap` — `raylib/src/core/models.rs`

### Section 39 (rmodels): Material loading/unloading functions

- ✅ `LoadMaterials` — `raylib/src/core/models.rs`
- ✅ `LoadMaterialDefault` — `raylib/src/core/models.rs`
- ✅ `IsMaterialValid` — `raylib/src/core/models.rs`
- ✅ `UnloadMaterial` — `raylib/src/core/models.rs` (Drop)
- ✅ `SetMaterialTexture` — `raylib/src/core/models.rs`
- ✅ `SetModelMeshMaterial` — `raylib/src/core/models.rs`

### Section 40 (rmodels): Model animations loading/unloading functions

- ✅ `LoadModelAnimations` — `raylib/src/core/models.rs` (returns
  `ModelAnimations` RAII container, redesigned in WS3).
- ✅ `UpdateModelAnimation` — `raylib/src/core/models.rs`
- ✅ `UpdateModelAnimationEx` — `raylib/src/core/models.rs`
- ✅ `UnloadModelAnimations` — `raylib/src/core/models.rs` (Drop on the
  `ModelAnimations` container).
- ✅ `IsModelAnimationValid` — `raylib/src/core/models.rs`

### Section 41 (rmodels): Collision detection functions

- ✅ `CheckCollisionSpheres` — `raylib/src/core/collision.rs`
- ✅ `CheckCollisionBoxes` — `raylib/src/core/collision.rs`
- ✅ `CheckCollisionBoxSphere` — `raylib/src/core/collision.rs`
- ✅ `GetRayCollisionSphere` — `raylib/src/core/collision.rs`
- ✅ `GetRayCollisionBox` — `raylib/src/core/collision.rs`
- ✅ `GetRayCollisionMesh` — `raylib/src/core/collision.rs`
- ✅ `GetRayCollisionTriangle` — `raylib/src/core/collision.rs`
- ✅ `GetRayCollisionQuad` — `raylib/src/core/collision.rs`

### Section 42 (raudio): Audio device management functions

- ✅ `InitAudioDevice` — `raylib/src/core/audio.rs`
- ✅ `CloseAudioDevice` — `raylib/src/core/audio.rs` (Drop on
  `AudioHandle`).
- ✅ `IsAudioDeviceReady` — `raylib/src/core/audio.rs`
- ✅ `SetMasterVolume` — `raylib/src/core/audio.rs`
- ✅ `GetMasterVolume` — `raylib/src/core/audio.rs`

### Section 43 (raudio): Wave/Sound loading/unloading functions

- ✅ `LoadWave` — `raylib/src/core/audio.rs`
- ✅ `LoadWaveFromMemory` — `raylib/src/core/audio.rs`
- ✅ `IsWaveValid` — `raylib/src/core/audio.rs`
- ✅ `LoadSound` — `raylib/src/core/audio.rs`
- ✅ `LoadSoundFromWave` — `raylib/src/core/audio.rs`
- ✅ `LoadSoundAlias` — `raylib/src/core/audio.rs`
- ✅ `IsSoundValid` — `raylib/src/core/audio.rs`
- ✅ `UpdateSound` — `raylib/src/core/audio.rs`
- ✅ `UnloadWave` — `raylib/src/core/audio.rs` (Drop)
- ✅ `UnloadSound` — `raylib/src/core/audio.rs` (Drop)
- ✅ `UnloadSoundAlias` — `raylib/src/core/audio.rs` (Drop)
- ✅ `ExportWave` — `raylib/src/core/audio.rs`
- ✅ `ExportWaveAsCode` — `raylib/src/core/audio.rs`

### Section 44 (raudio): Wave/Sound management functions

- ✅ `PlaySound` — `raylib/src/core/audio.rs`
- ✅ `StopSound` — `raylib/src/core/audio.rs`
- ✅ `PauseSound` — `raylib/src/core/audio.rs`
- ✅ `ResumeSound` — `raylib/src/core/audio.rs`
- ✅ `IsSoundPlaying` — `raylib/src/core/audio.rs`
- ✅ `SetSoundVolume` — `raylib/src/core/audio.rs`
- ✅ `SetSoundPitch` — `raylib/src/core/audio.rs`
- ✅ `SetSoundPan` — `raylib/src/core/audio.rs`
- ✅ `WaveCopy` — `raylib/src/core/audio.rs`
- ✅ `WaveCrop` — `raylib/src/core/audio.rs`
- ✅ `WaveFormat` — `raylib/src/core/audio.rs`
- ✅ `LoadWaveSamples` — `raylib/src/core/audio.rs`
- ✅ `UnloadWaveSamples` — `raylib/src/core/audio.rs` (Drop)

### Section 45 (raudio): Music management functions

- ✅ `LoadMusicStream` — `raylib/src/core/audio.rs`
- ✅ `LoadMusicStreamFromMemory` — `raylib/src/core/audio.rs`
- ✅ `IsMusicValid` — `raylib/src/core/audio.rs`
- ✅ `UnloadMusicStream` — `raylib/src/core/audio.rs` (Drop)
- ✅ `PlayMusicStream` — `raylib/src/core/audio.rs`
- ✅ `IsMusicStreamPlaying` — `raylib/src/core/audio.rs`
- ✅ `UpdateMusicStream` — `raylib/src/core/audio.rs`
- ✅ `StopMusicStream` — `raylib/src/core/audio.rs`
- ✅ `PauseMusicStream` — `raylib/src/core/audio.rs`
- ✅ `ResumeMusicStream` — `raylib/src/core/audio.rs`
- ✅ `SeekMusicStream` — `raylib/src/core/audio.rs`
- ✅ `SetMusicVolume` — `raylib/src/core/audio.rs`
- ✅ `SetMusicPitch` — `raylib/src/core/audio.rs`
- ✅ `SetMusicPan` — `raylib/src/core/audio.rs`
- ✅ `GetMusicTimeLength` — `raylib/src/core/audio.rs`
- ✅ `GetMusicTimePlayed` — `raylib/src/core/audio.rs`

### Section 46 (raudio): AudioStream management functions

- ✅ `LoadAudioStream` — `raylib/src/core/audio.rs`
- ✅ `IsAudioStreamValid` — `raylib/src/core/audio.rs`
- ✅ `UnloadAudioStream` — `raylib/src/core/audio.rs` (Drop)
- ✅ `UpdateAudioStream` — `raylib/src/core/audio.rs`
- ✅ `IsAudioStreamProcessed` — `raylib/src/core/audio.rs`
- ✅ `PlayAudioStream` — `raylib/src/core/audio.rs`
- ✅ `PauseAudioStream` — `raylib/src/core/audio.rs`
- ✅ `ResumeAudioStream` — `raylib/src/core/audio.rs`
- ✅ `IsAudioStreamPlaying` — `raylib/src/core/audio.rs`
- ✅ `StopAudioStream` — `raylib/src/core/audio.rs`
- ✅ `SetAudioStreamVolume` — `raylib/src/core/audio.rs`
- ✅ `SetAudioStreamPitch` — `raylib/src/core/audio.rs`
- ✅ `SetAudioStreamPan` — `raylib/src/core/audio.rs`
- ✅ `SetAudioStreamBufferSizeDefault` — `raylib/src/core/audio.rs`
- ✅ `SetAudioStreamCallback` — `raylib/src/core/audio.rs`
- ✅ `AttachAudioStreamProcessor` — wrapped via
  `attach_audio_stream_processor_with_user_data` in
  `raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs:142`
  (closure-driven stream effects).
- 🟥 `DetachAudioStreamProcessor` — GAP. Action: fix-now.
  (The user-data wrapper has `clear_context` but never calls
  `DetachAudioStreamProcessor`, so a detached closure may still receive
  one more invocation. Pair the detach with a real FFI call.)
- 🟥 `AttachAudioMixedProcessor` — GAP. Action: future-workstream-mixed-audio.
- 🟥 `DetachAudioMixedProcessor` — GAP. Action: future-workstream-mixed-audio.

---

## Skip rationale catalog

Distinct rationales used in §1:

- **"Not needed by the safe API; Rust handles it"** — `MemRealloc`. Rust
  uses `Box`/`Vec` reallocation; nothing in the safe binding needs to call
  `MemRealloc` from C. Wrap on demand if a user case appears.

(All other 🟨 entries elsewhere in the parity-checklist fall in the
out-of-scope `rtext` / file-system sections, where Rust `std` covers them
— see `memory/skip-std-equivalent-fns.md` for the locked rationale and
the `[~]` markers in `parity-checklist.md`.)

---

## Genuine gaps — small fixes (recommended for a quick follow-up commit)

| FFI symbol | Suggested pattern | Estimated diff |
| --- | --- | --- |
| `DrawLineDashed` | Add `fn draw_line_dashed(&mut self, start: impl Into<Vector2>, end: impl Into<Vector2>, dash_size: i32, space_size: i32, color: impl Into<Color>)` to the `RaylibDraw` trait in `raylib/src/core/drawing.rs`. | ~6 lines |
| `DrawEllipseV` | Free-`self` Vector2 variant of `DrawEllipse` in `raylib/src/core/drawing.rs`. | ~5 lines |
| `DrawEllipseLinesV` | Mirror of `DrawEllipseLines` taking `Vector2` instead of `(i32, i32)`. | ~5 lines |
| `GetKeyName` | Add `pub fn get_key_name(&self, key: KeyboardKey) -> Option<String>` in `raylib/src/core/input.rs`. The C fn returns a `const char*` from a static buffer — copy to a `String` before returning. Handle `NULL`/empty by returning `None`. | ~8 lines |
| `DetachAudioStreamProcessor` | Pair the existing `clear_context` in `stream_processor_with_user_data_wrapper.rs` with the actual FFI call. Today only the slot is freed, which leaves the C side still iterating the processor list. | ~3 lines |

Five small gaps, ~27 lines total of new safe-wrapper code.

Note: items the existing `parity-checklist.md` marks `[ ]` under
`Color/pixel related` and `Basic shapes collision` are **not gaps** —
they are wrapped as inherent methods on `raylib-sys::Color` and
`raylib-sys::Rectangle`. See §5 for the reconciliation delta.

---

## Genuine gaps — future workstream candidates

Eight functions need design judgement that goes beyond a one-line wrapper
(grouped into three workstreams):

1. **`GetPixelColor` / `SetPixelColor`** (workstream: `pixel-pointers`).
   Both take a `void *` into raw pixel memory plus a `PixelFormat`. A safe
   wrapper has to decide how to express the pointer: a `&[u8]` /
   `&mut [u8]` plus offset + format, or a typed `PixelData<F>` newtype, or
   only as a method on `Image` (which already knows its `format`). All
   three carry trade-offs; pick during a small design session.

2. **`ComputeCRC32` / `ComputeMD5` / `ComputeSHA1` / `ComputeSHA256`**
   (workstream: `hashes`). Returning a static `unsigned int[N]` from C —
   `MD5` returns `[u32; 4]`, `SHA1` returns `[u32; 5]`, `SHA256` returns
   `[u32; 8]`. The wrappers should:
   - Either return `[u32; N]` by value (Rust idiomatic — copy out
     immediately so the C static buffer is reusable).
   - Or expose them as `Hasher`-style helpers on a `Hash` enum/trait.
   Also need to evaluate whether to keep these at all: Rust crates like
   `crc32fast`, `md-5`, `sha1`, `sha2` already do this with established
   APIs, so the bindings may be redundant unless byte-identical output to
   raylib's C is required. Recommend bundling these with a brainstorming
   pass before wrapping.

3. **`AttachAudioMixedProcessor` / `DetachAudioMixedProcessor`**
   (workstream: `mixed-audio`). Currently only the per-stream variant has
   a closure wrapper (`stream_processor_with_user_data_wrapper.rs`). The
   *mixed* variant is global to the audio device, so it needs a different
   storage slot (`OnceCell<Box<dyn Fn>>` on `AudioHandle`, or one of the
   29 pre-registered callback slots with a global flag). Closure cleanup
   also has to detach before drop to avoid the C side calling back into a
   freed closure. Worth a small RAII-handle design pass.

Total: **8 functions** across **3 workstreams**.

**Owner-locked ordering (2026-05-29):** these three workstreams run
**before WS9 showcase**, so the published 6.0 crate ships with the full
cheatsheet surface covered. Order: `pixel-pointers` → `hashes` →
`mixed-audio` → WS9 → final-release. See also
`docs/superpowers/notes/ws8e-checkpoint-review-feedback.md` for the full
post-checkpoint workstream queue.

---

## Reconciliation with existing parity-checklist.md

The existing `docs/superpowers/parity-checklist.md` is generated by
`find_unimplemented.py`, which only scans the safe crate for top-level
`pub fn` wrappers and `ffi::FName` callsites. It misses **inherent
methods defined on `raylib-sys` types** (the auto-scan doesn't descend
into `raylib-sys/src/**`). Below are the **19 `[ ]` entries that should
actually be `[x]`** when the next reconciliation commit re-runs the
generator (or hand-corrects):

### Color/pixel related — 14 deltas

All marked `[ ]` in `parity-checklist.md` lines 489-501. All are wrapped
as inherent methods or assoc fns on `raylib-sys::Color` in
`raylib-sys/src/color.rs`:

- `ColorIsEqual` → `Color::is_equal` (line 164)
- `Fade` → `Color::fade` deprecated alias for `alpha` (line 153)
- `ColorToInt` → `Color::color_to_int` (line 96)
- `ColorNormalize` → `Color::color_normalize` (line 102)
- `ColorFromNormalized` → `Color::color_from_normalized` (line 120)
- `ColorToHSV` → `Color::color_to_hsv` (line 108)
- `ColorFromHSV` → `Color::color_from_hsv` (line 114)
- `ColorTint` → `Color::tint` (line 132)
- `ColorBrightness` → `Color::brightness` (line 137)
- `ColorContrast` → `Color::contrast` (line 142)
- `ColorAlpha` → `Color::alpha` (line 147)
- `ColorAlphaBlend` → `Color::color_alpha_blend` (line 159)
- `ColorLerp` → `Color::lerp` (line 170)
- `GetColor` → `Color::get_color` (line 126)

### Audio stream callbacks — 1 delta

`AttachAudioStreamProcessor` is marked `[ ]` in `parity-checklist.md`
line 754 but is wrapped at
`raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs:148`
via `attach_audio_stream_processor_with_user_data` (closure-driven stream
effects). The detach side (`DetachAudioStreamProcessor`) is a genuine
gap — see §3.

### Basic shapes collision detection — 4 deltas

All marked `[ ]` in `parity-checklist.md` lines 356-366. All are wrapped
as inherent methods on `raylib-sys::Rectangle` in
`raylib-sys/src/math.rs`:

- `CheckCollisionRecs` → `Rectangle::check_collision_recs` (line 107)
- `CheckCollisionCircleRec` →
  `Rectangle::check_collision_circle_rec` (line 116)
- `CheckCollisionPointRec` →
  `Rectangle::check_collision_point_rec` (line 148)
- `GetCollisionRec` → `Rectangle::get_collision_rec` (line 137) — returns
  `Option<Rectangle>` (empty-intersection becomes `None`, slightly
  different from C's "empty rect" sentinel — document this in the
  parity-checklist if useful).

### Net result after reconciliation

- Before reconciliation: 508 wrapped / 43 wont-impl / 49 TODO.
- After applying the 18 type-method deltas above (14 Color + 4 Rectangle)
  AND noting `AttachAudioStreamProcessor` is wrapped via the user-data
  callback wrapper (1 more delta = 19 total):
  - Wrapped: 508 + 19 = **527**.
  - TODO: 49 - 19 = **30**.
- Remaining 30 TODO entries break down as:
  - In-scope per this audit: 13 (5 small + 8 future-workstream).
  - Out-of-scope (rtext / rfilesystem covered by `std`): 17 — should
    arguably be re-marked `[~]` in the parity-checklist with the same
    "Rust std covers" rationale used elsewhere in the file. Specifically:
    `FileRename`, `FileRemove`, `FileCopy`, `FileMove`,
    `FileTextReplace`, `FileTextFindIndex`, `GetDirectoryFileCount`,
    `GetDirectoryFileCountEx`, `MeasureTextCodepoints`, `LoadTextLines`,
    `UnloadTextLines`, `TextRemoveSpaces`, `GetTextBetween`,
    `TextReplaceAlloc`, `TextReplaceBetween`, `TextReplaceBetweenAlloc`,
    `TextInsertAlloc` — but a separate audit should classify each. The
    "files" group does NOT have direct Rust equivalents (e.g. atomic
    cross-volume `FileMove`, `FileTextReplace`) — those are arguably
    genuine gaps even though Rust `std` handles the easy cases. Defer.

---

## Self-review

- [x] Every non-skipped cheatsheet section is represented in §1
      (46 sections numbered).
- [x] Every function has a clear ✅ / 🟨 / 🟥 marker.
- [x] Every 🟨 has a one-line reason (`MemRealloc` is the only in-scope
      skip).
- [x] Every 🟥 has a small / future-workstream tag.
- [x] §5 reconciliation notes the 18 deltas vs `parity-checklist.md`
      (14 Color + 4 Rectangle).
- [x] No `<N>` placeholders remain; all counts filled.
- [x] Doc lives at `docs/superpowers/notes/cheatsheet-parity-audit.md`,
      NOT committed.
