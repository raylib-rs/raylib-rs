//! raylib-rs showcase library crate.
//!
//! Each example under `showcase/examples/` is a self-contained `fn main()`
//! that depends on this crate for the source-viewer overlay and the
//! source-pair registry. See `docs/superpowers/specs/2026-05-31-ws9-showcase-design.md`.

pub mod registry;
pub mod viewer;

pub use registry::{ExampleMeta, EXAMPLES, lookup};
pub use viewer::SourceViewer;
