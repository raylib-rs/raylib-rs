# Resources

This directory mirrors the example resource subtrees vendored from
upstream so that example load paths match the C originals one-for-one
(per WS9 spec D7 — path-level visual parity).

| Source                                                  | Destination                  |
|---------------------------------------------------------|------------------------------|
| `raylib-sys/raylib/examples/<category>/resources/`      | `resources/<category>/`      |
| `raylib-sys/raygui-examples/examples/<program>/resources/` | `resources/raygui/<program>/` |
| `raylib-sys/raygui-examples/examples/resources/`        | `resources/raygui/`          |

Maintained by `cargo run -p raylib-showcase --bin xtask-vendor-resources`.
Re-run on raylib bumps to refresh.

## License

All vendored files inherit their upstream license, which for raylib's
standard example bundle is **zlib/libpng** — the same license raylib
itself uses. raygui's example resources are likewise zlib/libpng.

If you redistribute the showcase, preserve this README and the
`LICENSE` file at the repo root.

## Per-file attribution

The bundled resources are predominantly Ramon Santamaria's
(`@raysan5`) own work plus a handful of community contributions
already covered by the standard raylib zlib/libpng grant. Specific
attribution callouts for individual assets (CC-BY or CC0) are recorded
inline in the example `.c` files that load them — search
`raylib-sys/raylib/examples/<cat>/<example>.c` for `Resource by:` or
`Asset by:` comments when porting an example. When an example surfaces
a non-zlib attribution requirement during the P2/P3 porting waves, add
a row to the table below.

| File                                                | License | Attribution             |
|-----------------------------------------------------|---------|-------------------------|
| _(populated incrementally during P2/P3 porting)_    |         |                         |
