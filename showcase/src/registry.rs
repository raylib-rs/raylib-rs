//! Source-pair registry. The build script emits the inner `REGISTRY` map
//! into `$OUT_DIR/source_registry.rs`; this module includes it and exposes
//! a safe lookup API.

include!(concat!(env!("OUT_DIR"), "/source_registry.rs"));

/// Returns the (C, Rust) source pair for an example by its `[[example]] name`.
/// Returns `None` if the example isn't registered (the build script would
/// normally have errored out before we get here, but the API is total).
#[must_use]
pub fn lookup(name: &str) -> Option<&'static SourcePair> {
    REGISTRY.get(name)
}

/// Per-example metadata for the gallery index. The actual `EXAMPLES` slice
/// is populated at build time via `examples_meta.json`; for runtime the
/// registry is sufficient (the slice is only used by `xtask_build_pages`).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExampleMeta {
    pub name: String,
    pub category: String,
    pub wasm_excluded: bool,
}

/// The `EXAMPLES` slice is read at gallery-build time, not at example runtime.
/// `xtask_build_pages` consumes `$OUT_DIR/examples_meta.json` directly; this
/// public re-export is reserved for downstream tooling (post-release).
pub const EXAMPLES: &[ExampleMeta] = &[];
