# DataBuf + Mesh testing workstream — done-note

> Status: IN PROGRESS — sections marked (Task 11) are finalized at workstream close.

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

## Wrapper-fn × edge-case matrix (Task 11)

## Test census (Task 11)

## Memory-checking decision record (Task 11)

## Future-work list

> Note: Tasks 11 sections above are finalized at workstream close.
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
