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

#[cfg(test)]
mod tests {
    use super::*;

    // Tier-1 sanity: the reference port lands in the registry under its
    // [[example]] name with non-empty source strings.
    #[test]
    fn core_basic_window_is_registered() {
        let pair = lookup("core_basic_window").expect("reference port must be registered");
        assert_eq!(pair.category, "core");
        assert!(pair.c.contains("InitWindow"), "C source must reference InitWindow");
        assert!(
            pair.rust.contains("SourceViewer::for_current_example"),
            "Rust port must wire the SourceViewer",
        );
    }

    #[test]
    fn lookup_missing_returns_none() {
        assert!(lookup("__definitely_not_an_example__").is_none());
    }
}
