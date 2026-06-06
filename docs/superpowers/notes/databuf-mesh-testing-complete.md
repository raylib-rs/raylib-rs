# DataBuf + Mesh testing workstream — done-note

> Status: COMPLETE — workstream closed 2026-06-05.

## rlgl coverage census (census of record)

Regenerate after any raylib bump:

```bash
BINDINGS=$(ls /c/Users/miaay/rl-dbmt-target/debug/build/raylib-sys-*/out/bindings.rs | head -1)
grep -ohE 'pub fn rl[A-Z][A-Za-z0-9_]*' "$BINDINGS" | sed 's/pub fn //' | sort -u > /tmp/rl_ffi.txt
grep -rohE 'ffi::rl[A-Z][A-Za-z0-9_]*' raylib/src/rlgl/ | sed 's/ffi:://' | sort -u > /tmp/rl_safe.txt
echo "total: $(wc -l < /tmp/rl_ffi.txt)  wrapped: $(wc -l < /tmp/rl_safe.txt)"
comm -23 /tmp/rl_ffi.txt /tmp/rl_safe.txt
```

Counts at audit time (2026-06-04): **161 rl\* FFI fns, 30 wrapped in raylib/src/rlgl/, 131 uncovered.**

rl\* fns used elsewhere in the safe crate (internal use, exercised via other safe wrappers):
`rlGetMatrixModelview`, `rlGetMatrixProjection`, `rlGetShaderIdDefault`, `rlGetShaderLocsDefault`,
`rlSetMatrixModelview` (wrapped in rlgl/), `rlSetMatrixProjection` (wrapped in rlgl/).

Note: `rlSetMatrixModelview` and `rlSetMatrixProjection` are already in the wrapped-30 set (appear
in `/tmp/rl_safe.txt`), so only 4 of the 6 internally-used fns are uncovered.

## Disposition table

Category tallies: **wrap-now: 1 / escape-hatch: 93 / future-work: 37.** Total: 131 == uncovered count.

Escape-hatch subcategories: texture-lifecycle: 14, shader-lifecycle/uniforms/compute: 16,
SSBO: 7, framebuffer: 16, vertex-array/buffer: 22, render-batch: 6, stereo/VR: 7, platform/init: 5.

| rl\* fn | Disposition | Rationale |
|---|---|---|
| `rlVertex2i` | **wrap (PR-B)** | completes the immediate-mode vertex family (`rlVertex2f`/`rlVertex3f` already wrapped) |
| `rlLoadTexture` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture lifecycle; module-doc policy |
| `rlLoadTextureCubemap` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture lifecycle |
| `rlLoadTextureDepth` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture lifecycle |
| `rlUnloadTexture` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture lifecycle |
| `rlUpdateTexture` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture lifecycle |
| `rlGenTextureMipmaps` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture lifecycle |
| `rlReadTexturePixels` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture lifecycle |
| `rlGetTextureIdDefault` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture lifecycle |
| `rlGetGlTextureFormats` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture lifecycle |
| `rlTextureParameters` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture params |
| `rlCubemapParameters` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture params |
| `rlEnableTextureCubemap` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture bind |
| `rlDisableTextureCubemap` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | texture bind |
| `rlBindImageTexture` | escape-hatch — GL-object lifecycle stays with `Texture2D` + raw FFI | compute image binding |
| `rlLoadShader` | escape-hatch — via safe `Shader`; compute has no safe-surface story yet | shader lifecycle |
| `rlLoadShaderProgram` | escape-hatch — via safe `Shader` | shader lifecycle |
| `rlLoadShaderProgramCompute` | escape-hatch — via safe `Shader`; compute has no safe-surface story yet | shader lifecycle / compute |
| `rlLoadShaderProgramEx` | escape-hatch — via safe `Shader` | shader lifecycle |
| `rlUnloadShader` | escape-hatch — via safe `Shader` | shader lifecycle |
| `rlUnloadShaderProgram` | escape-hatch — via safe `Shader` | shader lifecycle |
| `rlDisableShader` | escape-hatch — via safe `Shader` | shader bind |
| `rlSetUniform` | escape-hatch — via safe `Shader` | shader uniforms |
| `rlSetUniformMatrix` | escape-hatch — via safe `Shader` | shader uniforms |
| `rlSetUniformMatrices` | escape-hatch — via safe `Shader` | shader uniforms |
| `rlSetUniformSampler` | escape-hatch — via safe `Shader` | shader uniforms |
| `rlGetLocationUniform` | escape-hatch — via safe `Shader` | shader uniforms |
| `rlGetLocationAttrib` | escape-hatch — via safe `Shader` | shader uniforms |
| `rlGetShaderIdDefault` | escape-hatch — via safe `Shader` **(used internally by core/)** | shader default id; internal use |
| `rlGetShaderLocsDefault` | escape-hatch — via safe `Shader` **(used internally by core/)** | shader default locs; internal use |
| `rlComputeShaderDispatch` | escape-hatch — compute has no safe-surface story yet | compute dispatch |
| `rlLoadShaderBuffer` | escape-hatch — power-user GPU plumbing | SSBO |
| `rlUnloadShaderBuffer` | escape-hatch — power-user GPU plumbing | SSBO |
| `rlBindShaderBuffer` | escape-hatch — power-user GPU plumbing | SSBO |
| `rlUpdateShaderBuffer` | escape-hatch — power-user GPU plumbing | SSBO |
| `rlCopyShaderBuffer` | escape-hatch — power-user GPU plumbing | SSBO |
| `rlReadShaderBuffer` | escape-hatch — power-user GPU plumbing | SSBO |
| `rlGetShaderBufferSize` | escape-hatch — power-user GPU plumbing | SSBO |
| `rlLoadFramebuffer` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer |
| `rlUnloadFramebuffer` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer |
| `rlBindFramebuffer` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer |
| `rlEnableFramebuffer` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer |
| `rlDisableFramebuffer` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer |
| `rlFramebufferAttach` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer |
| `rlFramebufferComplete` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer |
| `rlBlitFramebuffer` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer |
| `rlCopyFramebuffer` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer |
| `rlActiveDrawBuffers` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer draw buffers |
| `rlGetFramebufferHeight` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer getter |
| `rlGetFramebufferWidth` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer getter |
| `rlSetFramebufferHeight` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer setter |
| `rlSetFramebufferWidth` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer setter |
| `rlResizeFramebuffer` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer resize |
| `rlGetActiveFramebuffer` | escape-hatch — lifecycle owned by `RenderTexture2D` | framebuffer query |
| `rlLoadVertexArray` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlUnloadVertexArray` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlEnableVertexArray` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlDisableVertexArray` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlLoadVertexBuffer` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlUnloadVertexBuffer` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlEnableVertexBuffer` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlDisableVertexBuffer` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlLoadVertexBufferElement` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlEnableVertexBufferElement` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlDisableVertexBufferElement` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlUpdateVertexBuffer` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlUpdateVertexBufferElements` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlSetVertexAttribute` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlSetVertexAttributeDefault` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlSetVertexAttributeDivisor` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlEnableVertexAttribute` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlDisableVertexAttribute` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlDrawVertexArray` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlDrawVertexArrayElements` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlDrawVertexArrayInstanced` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlDrawVertexArrayElementsInstanced` | escape-hatch — raw geometry pipeline; safe `Mesh` covers the supported path | vertex array/buffer |
| `rlLoadRenderBatch` | escape-hatch — internal batching control | render batch |
| `rlUnloadRenderBatch` | escape-hatch — internal batching control | render batch |
| `rlDrawRenderBatch` | escape-hatch — internal batching control | render batch |
| `rlDrawRenderBatchActive` | escape-hatch — internal batching control | render batch |
| `rlSetRenderBatchActive` | escape-hatch — internal batching control | render batch |
| `rlCheckRenderBatchLimit` | escape-hatch — internal batching control | render batch |
| `rlEnableStereoRender` | escape-hatch — VR out of 6.0 scope | stereo/VR |
| `rlDisableStereoRender` | escape-hatch — VR out of 6.0 scope | stereo/VR |
| `rlIsStereoRenderEnabled` | escape-hatch — VR out of 6.0 scope | stereo/VR |
| `rlGetMatrixProjectionStereo` | escape-hatch — VR out of 6.0 scope | stereo/VR |
| `rlGetMatrixViewOffsetStereo` | escape-hatch — VR out of 6.0 scope | stereo/VR |
| `rlSetMatrixProjectionStereo` | escape-hatch — VR out of 6.0 scope | stereo/VR |
| `rlSetMatrixViewOffsetStereo` | escape-hatch — VR out of 6.0 scope | stereo/VR |
| `rlLoadExtensions` | escape-hatch — init-time / GL1.1-only | platform/init |
| `rlGetProcAddress` | escape-hatch — init-time / GL1.1-only | platform/init |
| `rlGetVersion` | escape-hatch — init-time / GL1.1-only | platform/init |
| `rlEnableStatePointer` | escape-hatch — GL1.1-only | platform/init |
| `rlDisableStatePointer` | escape-hatch — GL1.1-only | platform/init |
| `rlViewport` | future-work — same-shape candidate for a safe-state tier follow-up (maintainer decision: trivial-only this workstream) | state toggle/param |
| `rlScissor` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlEnableScissorTest` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlDisableScissorTest` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlEnableColorBlend` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlDisableColorBlend` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlEnableDepthMask` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlDisableDepthMask` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlColorMask` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlEnableWireMode` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlDisableWireMode` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlEnablePointMode` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlDisablePointMode` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlEnableSmoothLines` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlDisableSmoothLines` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlSetBlendMode` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlSetBlendFactors` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlSetBlendFactorsSeparate` | future-work — same-shape candidate for a safe-state tier follow-up | state toggle/param |
| `rlSetLineWidth` | future-work — same-shape candidate for a safe-state tier follow-up | state getter/setter |
| `rlGetLineWidth` | future-work — same-shape candidate for a safe-state tier follow-up | state getter/setter |
| `rlSetPointSize` | future-work — same-shape candidate for a safe-state tier follow-up | state getter/setter |
| `rlGetPointSize` | future-work — same-shape candidate for a safe-state tier follow-up | state getter/setter |
| `rlSetCullFace` | future-work — same-shape candidate for a safe-state tier follow-up | state getter/setter |
| `rlSetClipPlanes` | future-work — same-shape candidate for a safe-state tier follow-up | state getter/setter |
| `rlGetCullDistanceNear` | future-work — same-shape candidate for a safe-state tier follow-up | state getter/setter |
| `rlGetCullDistanceFar` | future-work — same-shape candidate for a safe-state tier follow-up | state getter/setter |
| `rlClearColor` | future-work — same-shape candidate for a safe-state tier follow-up | state helper |
| `rlClearScreenBuffers` | future-work — same-shape candidate for a safe-state tier follow-up | state helper |
| `rlCheckErrors` | future-work — same-shape candidate for a safe-state tier follow-up | state helper |
| `rlGetPixelFormatName` | future-work — same-shape candidate for a safe-state tier follow-up | state helper |
| `rlFrustum` | future-work — same-shape candidate for a safe-state tier follow-up | matrix helper |
| `rlGetMatrixModelview` | future-work — same-shape candidate for a safe-state tier follow-up **(used internally by core/)** | matrix getter; internal use |
| `rlGetMatrixProjection` | future-work — same-shape candidate for a safe-state tier follow-up **(used internally by core/)** | matrix getter; internal use |
| `rlGetMatrixTransform` | future-work — same-shape candidate for a safe-state tier follow-up | matrix getter |
| `rlLoadDrawCube` | future-work — same-shape candidate for a safe-state tier follow-up | geometry helper |
| `rlLoadDrawQuad` | future-work — same-shape candidate for a safe-state tier follow-up | geometry helper |
| `rlReadScreenPixels` | future-work — same-shape candidate for a safe-state tier follow-up | pixel readback helper |

## Wrapper-fn × edge-case matrix (as executed)

The table below is adapted from the spec's design matrix and records what was
actually implemented. Tests that don't exist are marked with their documented
untestable rationale.

### `DataBuf<T>` / scalar operations (`raylib/src/core/databuf.rs`, in-file tests)

| Edge case | Test fn | File |
|---|---|---|
| `alloc` ZST → `AllocationError::ZeroBytes` | `test_alloc_zst_errors` | `databuf.rs` |
| `alloc_from` error returns value intact (destructure tuple) | `test_alloc_from_error_returns_value` | `databuf.rs` |
| `write` → value readable; `assume_init` roundtrip | `test_drop_value`, `test_from_raw` | `databuf.rs` |
| `alloc_from_clone`: non-`Copy` type, clone count == len, drop count == len | `test_alloc_from_clone_non_copy` | `databuf.rs` |
| `alloc_from_clone` panic-safety: panicking clone drops cloned prefix, frees allocation | `test_alloc_from_clone_panic_drops_prefix` | `databuf.rs` |
| `into_inner` suppresses drop | `test_into_inner_suppresses_content_drop` | `databuf.rs` |

### `DataBuf<[T]>` / slice operations (`raylib/src/core/databuf.rs`, in-file tests)

| Edge case | Test fn | File |
|---|---|---|
| `alloc` count == 0 → `ZeroBytes` | `test_alloc_slice_zero_len_errors` | `databuf.rs` |
| `alloc` `Layout::array` overflow → `IntoUIntFailed` | `test_alloc_slice_layout_overflow_errors` | `databuf.rs` |
| `alloc` byte-size > `u32::MAX` → `IntoUIntFailed` (cfg 64-bit only) | `test_alloc_slice_over_u32_max_errors` | `databuf.rs` |
| `alloc` large-but-valid (≥1 MB) succeeds | `test_alloc_large_but_valid_succeeds` | `databuf.rs` |
| `realloc` grow preserves prefix | `test_realloc_grow_preserves_prefix` | `databuf.rs` |
| `realloc` shrink keeps prefix | `test_realloc_shrink_keeps_prefix` | `databuf.rs` |
| `realloc` new_count == 0 → error tuple; original buffer still usable | `test_realloc_zero_returns_original_usable` | `databuf.rs` |
| Slice drop runs element `Drop` exactly `len` times | `test_slice_drop_count` | `databuf.rs` |
| `Deref`/`DerefMut`/`AsRef`/`AsMut` agree on contents | `test_view_parity` | `databuf.rs` |
| `slice_from_raw` count < 1 → panic (`pub(crate)`, `#[should_panic]`) | `test_slice_from_raw_zero_count_panics` | `databuf.rs` |
| `slice_from_raw` normal path | `test_slice_from_raw` | `databuf.rs` |

### `DataBuf` alloc/realloc/free under initialized raylib (Tier-2)

| Edge case | Test fn | File |
|---|---|---|
| alloc/realloc/free with live `RaylibHandle` (Memory platform) | `databuf_alloc_cycle_under_initialized_raylib` | `render_alloc_lifetimes.rs` |

### `DataBuf` alloc/cycle (Tier-1 integration)

| Edge case | Test fn | File |
|---|---|---|
| alloc → write → read roundtrip (windowless) | `databuf_alloc_write_read_roundtrip` | `databuf_lifetimes.rs` |
| alloc_from_copy → realloc grow → assume_init | `databuf_slice_alloc_cycle` | `databuf_lifetimes.rs` |

### `compress_data` / `decompress_data` / `encode_data_base64` / `decode_data_base64`

The "count == 0 fix" fired during probing: all four fns can return a **non-null
buffer with count 0** (raylib's sinfl/base64 decoders do this for empty or
garbage input). The fix in `raylib/src/core/data.rs` detects non-null + count
< 1, frees the buffer via `ffi::MemFree`, and returns `Err`.
`encode_data_base64(b"")` returns `Ok([0])` (NUL terminator only) — pinned, not
changed (the count is 1, which satisfies `slice_from_raw`'s assert).

| Edge case | Test fn | File |
|---|---|---|
| compress→decompress roundtrip at 1 B / 4 KiB / 1 MiB | `compression::compress_roundtrip_sizes` | `databuf_lifetimes.rs` |
| `decompress_data` garbage input → `Err(CompressionFailed)` (pinned 2026-06-03) | `compression::decompress_garbage_is_an_error_not_a_panic` | `databuf_lifetimes.rs` |
| `compress_data(b"")` → `Err(CompressionFailed)` (pinned 2026-06-03) | `compression::compress_empty_input_pinned` | `databuf_lifetimes.rs` |
| `decompress_data(b"")` → `Err(CompressionFailed)` (pinned 2026-06-03) | `compression::decompress_empty_input_pinned` | `databuf_lifetimes.rs` |
| `encode_data_base64(b"")` → `Ok([0])` NUL terminator (pinned 2026-06-03) | `base64::base64_encode_empty_input_pinned` | `databuf_lifetimes.rs` |
| `decode_data_base64(b"")` → `Err(DecodeFailed)` (pinned 2026-06-03) | `base64::base64_decode_empty_input_pinned` | `databuf_lifetimes.rs` |
| base64 roundtrip (encode→decode) | `base64::base64_roundtrip` | `databuf_lifetimes.rs` |
| base64 decode does NOT validate alphabet (pinned 2026-06-03: garbage → Ok with garbage bytes) | `base64::base64_decode_invalid_pinned` | `databuf_lifetimes.rs` |

### `ImageColors` / `ImagePalette` (Tier-2)

| Edge case | Test fn | File |
|---|---|---|
| `gen_image_color` → `get_image_data` len == w×h; all pixels red | `image_colors_and_palette_lifetimes` | `render_alloc_lifetimes.rs` |
| `extract_palette` on single-color image → 1 entry; correct color | `image_colors_and_palette_lifetimes` | `render_alloc_lifetimes.rs` |
| `render_frame` readback → blue pixel probe | `image_colors_and_palette_lifetimes` | `render_alloc_lifetimes.rs` |
| Drop runs via `UnloadImageColors` / `UnloadImagePalette` (ASAN validates free) | `image_colors_and_palette_lifetimes` | `render_alloc_lifetimes.rs` |

### `FilePathList` / `DroppedFilePathList`

| Edge case | Test fn | File |
|---|---|---|
| in-file fake-list iter `count`/`len` parity | `test_len`, `test_len_double_ended` | `file.rs` (in-file) |
| in-file null item / double-ended with null | `test_null_item`, `test_null_item_double_ended` | `file.rs` (in-file) |
| in-file null list | `test_null_list` | `file.rs` (in-file) |
| `load_directory_files` on tempdir with 3 files → count == 3, paths match | `file_path_list_real_directory` | `render_alloc_lifetimes.rs` |
| `ExactSizeIterator` / `DoubleEndedIterator` / `nth` parity on a real list | `file_path_list_real_directory` | `render_alloc_lifetimes.rs` |
| empty dir → count == 0 (pinned: non-null paths array even for empty dir) | `file_path_list_real_directory` | `render_alloc_lifetimes.rs` |
| Drop via `UnloadDirectoryFiles` (ASAN validates free) | `file_path_list_real_directory` | `render_alloc_lifetimes.rs` |

### Mesh slice accessors (8 accessor pairs)

| Edge case | Test fn | File |
|---|---|---|
| null-pointer Mesh: all 7 read accessors return empty slice | `null_field_accessors_are_empty_not_ub` | `models.rs` (in-file) |
| null-pointer Mesh: all 7 `_mut` accessors return empty slice | `null_field_accessors_are_empty_not_ub` | `models.rs` (in-file) |
| `gen_mesh_cube` → `vertices()` len == vertexCount | `mesh_accessors_match_counts` | `render_alloc_lifetimes.rs` |
| `gen_mesh_cube` → `normals()` len == vertexCount | `mesh_accessors_match_counts` | `render_alloc_lifetimes.rs` |
| `gen_mesh_cube` → optional attrs absent for cube (pin: texcoords2/tangents/colors empty) | `mesh_accessors_match_counts` | `render_alloc_lifetimes.rs` |
| `gen_mesh_cube` → `indices()` empty or len == triangleCount×3 | `mesh_accessors_match_counts` | `render_alloc_lifetimes.rs` |
| `gen_mesh_plane` → mutate via `vertices_mut`, read back via `vertices` | `mesh_accessor_mut_roundtrip` | `render_alloc_lifetimes.rs` |
| `gen_mesh_plane` → mutate via `normals_mut`, read back | `mesh_accessor_mut_roundtrip` | `render_alloc_lifetimes.rs` |

### `RSliceGlyphInfo` (in-file)

| Edge case | Test fn | File |
|---|---|---|
| Drop routes through `UnloadFontData` (ASAN validates both frees: each glyph image + the array) | `rslice_glyphinfo_drop_routes_through_unload_font_data` | `text.rs` (in-file) |

### `ModelAnimations` (Tier-2, `integration_model_animations.rs`)

| Edge case | Test fn | File |
|---|---|---|
| Load and drop (exercises `UnloadModelAnimations` once) | `model_animations_load_and_drop` | `integration_model_animations.rs` |
| Drop after partial index/borrow | `model_animations_partial_index_then_drop` | `integration_model_animations.rs` |
| Slice views in-bounds: `as_slice`/`as_mut_slice` len; all-elements access | `model_animations_slice_views_in_bounds` | `integration_model_animations.rs` |
| Drop order vs model: anims-before-model and model-before-anims | `model_animations_drop_order_vs_model` | `integration_model_animations.rs` |

### Untestable — documented rationale

- **`AllocationError::NullAlloc` on the alloc path**: requires `ffi::MemAlloc`
  to return null — needs near-OOM conditions or allocator interposition (fault
  injection). Not reproducible deterministically; do not fake it.
- **`AllocationError::IntoUIntFailed` via > u32::MAX byte count**: covered by
  `test_alloc_slice_over_u32_max_errors` on 64-bit platforms
  (`#[cfg(target_pointer_width = "64")]`) — the checked `usize → u32`
  conversion path is exercised there. Unavailable on 32-bit (usize == u32).

## Test census

### Before this workstream

- Tier-1 (`cargo nextest run -p raylib`): **77 tests** (pre-workstream baseline).
  - `databuf.rs` in-file: 3 tests.
  - Integration tests touching wrapper family: 0.
- Doctests: **144** (unchanged by this workstream).

### After this workstream (2026-06-05)

- Tier-1 (`cargo nextest run -p raylib`): **103 tests** (+26).
- Tier-2 (`cargo nextest run -p raylib --no-default-features --features software_renderer,...`): **119 tests** (+numerous from `render_alloc_lifetimes` and extended model-animation tests).
- Doctests: **144** (unchanged).

#### New tests by file

**`raylib/src/core/databuf.rs`** (in-file, 18 tests total; was 3):
- `test_drop_value` (pre-existing)
- `test_from_raw` (pre-existing)
- `test_slice_from_raw` (pre-existing)
- `test_alloc_from_clone_non_copy` (new)
- `test_alloc_from_clone_panic_drops_prefix` (new)
- `test_alloc_zst_errors` (new)
- `test_alloc_slice_zero_len_errors` (new)
- `test_alloc_slice_layout_overflow_errors` (new)
- `test_alloc_slice_over_u32_max_errors` (new, `#[cfg(target_pointer_width = "64")]`)
- `test_alloc_large_but_valid_succeeds` (new)
- `test_alloc_from_error_returns_value` (new)
- `test_realloc_grow_preserves_prefix` (new)
- `test_realloc_shrink_keeps_prefix` (new)
- `test_realloc_zero_returns_original_usable` (new)
- `test_slice_drop_count` (new)
- `test_into_inner_suppresses_content_drop` (new)
- `test_view_parity` (new)
- `test_slice_from_raw_zero_count_panics` (new, `#[should_panic]`)

**`raylib/src/core/models.rs`** (in-file, `mesh_soundness` module):
- `null_field_accessors_are_empty_not_ub` (extended to all 8 accessor pairs — was single-case)

**`raylib/src/core/text.rs`** (in-file, `rslice_tests` module):
- `rslice_glyphinfo_drop_routes_through_unload_font_data` (new)

**`raylib/tests/databuf_lifetimes.rs`** (new file, 10 tests; ASAN+LSAN CI target):
- `databuf_alloc_write_read_roundtrip`
- `databuf_slice_alloc_cycle`
- `compression::compress_roundtrip_sizes`
- `compression::decompress_garbage_is_an_error_not_a_panic`
- `compression::compress_empty_input_pinned`
- `compression::decompress_empty_input_pinned`
- `base64::base64_roundtrip`
- `base64::base64_decode_invalid_pinned`
- `base64::base64_encode_empty_input_pinned`
- `base64::base64_decode_empty_input_pinned`

**`raylib/tests/render_alloc_lifetimes.rs`** (new file, 4+1 = 5 tests when `SUPPORT_MESH_GENERATION` present, otherwise 4 render + 1 counted once):
- `mesh_accessors_match_counts` (Tier-2, `#[cfg(feature = "SUPPORT_MESH_GENERATION")]`)
- `mesh_accessor_mut_roundtrip` (Tier-2, `#[cfg(feature = "SUPPORT_MESH_GENERATION")]`)
- `image_colors_and_palette_lifetimes` (Tier-2, `#[cfg(feature = "SUPPORT_IMAGE_GENERATION")]`)
- `file_path_list_real_directory` (Tier-2)
- `databuf_alloc_cycle_under_initialized_raylib` (Tier-2)

**`raylib/tests/integration_model_animations.rs`** (extended, +3 tests beyond the pre-existing `model_animations_load_and_drop`):
- `model_animations_partial_index_then_drop` (new)
- `model_animations_slice_views_in_bounds` (new)
- `model_animations_drop_order_vs_model` (new)

**`raylib/src/rlgl/` — `render_rlgl.rs` probe** (Tier-2, PR-B):
- `rlgl_vertex2i_renders` (new — probes the wrapped `rlVertex2i`)

## Memory-checking decision record

### Decision: ASAN+LSAN, no valgrind

This workstream closes WS8e review comments 8 and 10 (owner's suggestions):

> **Comment 8:** "There need to be a lot more tests for edge cases and various
> uses of databuf. We can use the software renderer mode here to ensure
> allocations. Maybe this is where we pull in valgrind."
>
> **Comment 10:** "Double check that no rlgl functions are missing."

**Valgrind — declined.** Rationale: valgrind would duplicate the error classes
ASAN already covers (double-free, use-after-free, heap-buffer-overflow) at
higher CI cost (slower instrumentation, Linux-only availability, noisier
suppressions). The CI already runs ASAN under `sanitizers.yml` (D8: sanitizers
are informational, `continue-on-error`). Adding a valgrind job would widen the
per-PR feedback gap without providing coverage ASAN does not.

**ASAN+LSAN — chosen.** The existing ASAN leg in `sanitizers.yml` is extended
with a new windowless step that runs `--test databuf_lifetimes` with
`ASAN_OPTIONS=detect_leaks=1` (LeakSanitizer). This gives real evidence for the
drop-exactly-once / no-leak invariants on the wrapper family's critical path
(`MemAlloc`/`MemRealloc`/`MemFree`) without any false-positive noise from
GPU/window-adjacent code. The existing render-test ASAN steps keep
`detect_leaks=0` (window + driver-adjacent allocations generate known-benign
noise that is hard to suppress portably).

**CI file:** `.github/workflows/sanitizers.yml` — one new step
"ASAN+LSAN — Tier-1 wrapper lifetime tests" (nightly, `-Zsanitizer=address`,
`ASAN_OPTIONS=detect_leaks=1`, `--test databuf_lifetimes`). The new Tier-2
files were added to the existing ASAN/UBSAN render-test steps' `--test` list.

### LSAN findings — first canonical run

First canonical run: **pending** — the sanitizers workflow triggers on push to
`unstable`; PR #320 (PR-A, wrapper-family tests + alloc_from_clone fix + CI
sanitizers extension) is green and awaiting merge as of 2026-06-05. To be
updated with the run's findings before this PR (PR-B, rlgl coverage + disposition
table) merges.

## Dead-code disposition

**`RSliceGlyphInfo`** — producer-less from the safe API's perspective. No public
`fn` returns `RSliceGlyphInfo`; `load_font_data` returns `Option<GlyphInfo>`
(the single-item version). The type exists in `text.rs` for completeness (it
is the safe RAII shell over a raylib-allocated glyph array). Future work: wire
it to a glyph-loading API that returns a slice, or remove it on the next
breaking cycle. Seeded in future-work below.

**`Codepoints`** — `pub(crate)`, covered indirectly via `load_font_data`. No
gap.

## Future-work list

> The sub-list below was seeded during the rlgl coverage audit (Task 8) for the maintainer checkpoint.

**Seeded during audit:**

- The IQM-survives-`-DSUPPORT_FILEFORMAT_IQM=OFF` cmake mystery (from
  `raylib/tests/integration_model_animations.rs`'s NOTE — investigate why format-disabling doesn't
  compile loaders out under `CUSTOMIZE_BUILD`).
- `RSliceGlyphInfo` is producer-less (dead code from a caller's perspective) — wire to a
  glyph-loading API or remove next breaking cycle. (The type is produced internally in tests via
  `MemAlloc` to verify `Drop`, but no public `fn` returns `RSliceGlyphInfo`; `load_font_data`
  returns `Option<GlyphInfo>` instead.)

**Future-work rlgl fns (37 — safe-state tier follow-up candidates):**

`rlCheckErrors`, `rlClearColor`, `rlClearScreenBuffers`, `rlColorMask`,
`rlDisableColorBlend`, `rlDisableDepthMask`, `rlDisablePointMode`, `rlDisableScissorTest`,
`rlDisableSmoothLines`, `rlDisableWireMode`, `rlEnableColorBlend`, `rlEnableDepthMask`,
`rlEnablePointMode`, `rlEnableScissorTest`, `rlEnableSmoothLines`, `rlEnableWireMode`,
`rlFrustum`, `rlGetCullDistanceFar`, `rlGetCullDistanceNear`, `rlGetLineWidth`,
`rlGetMatrixModelview` (also used internally), `rlGetMatrixProjection` (also used internally),
`rlGetMatrixTransform`, `rlGetPixelFormatName`, `rlGetPointSize`,
`rlLoadDrawCube`, `rlLoadDrawQuad`, `rlReadScreenPixels`,
`rlScissor`, `rlSetBlendFactors`, `rlSetBlendFactorsSeparate`, `rlSetBlendMode`,
`rlSetClipPlanes`, `rlSetCullFace`, `rlSetLineWidth`, `rlSetPointSize`, `rlViewport`.

**Seeded during audit:**

- The IQM-survives-`-DSUPPORT_FILEFORMAT_IQM=OFF` cmake mystery (from
  `raylib/tests/integration_model_animations.rs`'s NOTE — investigate why format-disabling doesn't
  compile loaders out under `CUSTOMIZE_BUILD`).
- `RSliceGlyphInfo` is producer-less (dead code from a caller's perspective) — wire to a
  glyph-loading API or remove next breaking cycle. (The type is produced internally in tests via
  `MemAlloc` to verify `Drop`, but no public `fn` returns `RSliceGlyphInfo`; `load_font_data`
  returns `Option<GlyphInfo>` instead.)

**Future-work rlgl fns (37 — safe-state tier follow-up candidates):**

`rlCheckErrors`, `rlClearColor`, `rlClearScreenBuffers`, `rlColorMask`,
`rlDisableColorBlend`, `rlDisableDepthMask`, `rlDisablePointMode`, `rlDisableScissorTest`,
`rlDisableSmoothLines`, `rlDisableWireMode`, `rlEnableColorBlend`, `rlEnableDepthMask`,
`rlEnablePointMode`, `rlEnableScissorTest`, `rlEnableSmoothLines`, `rlEnableWireMode`,
`rlFrustum`, `rlGetCullDistanceFar`, `rlGetCullDistanceNear`, `rlGetLineWidth`,
`rlGetMatrixModelview` (also used internally), `rlGetMatrixProjection` (also used internally),
`rlGetMatrixTransform`, `rlGetPixelFormatName`, `rlGetPointSize`,
`rlLoadDrawCube`, `rlLoadDrawQuad`, `rlReadScreenPixels`,
`rlScissor`, `rlSetBlendFactors`, `rlSetBlendFactorsSeparate`, `rlSetBlendMode`,
`rlSetClipPlanes`, `rlSetCullFace`, `rlSetLineWidth`, `rlSetPointSize`, `rlViewport`.
