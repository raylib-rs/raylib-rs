//! Assembles `showcase/_site/` from the build outputs.
//!
//! WS9 Phase 4 (Task 4.1): emits a categorized gallery indexing every
//! example with a thumbnail tile, a "desktop only" badge for wasm-excluded
//! entries, and per-tile deep links to the upstream C source + the Rust
//! port. Wasm-excluded examples get a placeholder `<cat>/<name>.html`
//! (rather than being missing) so the gallery tile still has a working
//! click target.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct ExampleMeta {
    name: String,
    category: String,
    wasm_excluded: bool,
    c_url: String,
    rust_url: String,
}

#[derive(Deserialize)]
struct ThumbnailEntry {
    name: String,
    #[allow(dead_code)]
    category: String,
    thumbnail: Option<String>,
    #[allow(dead_code)]
    reason: Option<String>,
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().to_path_buf();
    let site = manifest_dir.join("_site");
    let _ = fs::remove_dir_all(&site);
    fs::create_dir_all(site.join("examples")).unwrap();

    // Cargo creates multiple `raylib-showcase-<hash>` build dirs
    // (sometimes one for build-script output, one for run-script). Only
    // some contain the OUT_DIR/examples_meta.json our build.rs emits;
    // iterate and pick the one that actually has the file.
    let meta_path = find_examples_meta(&workspace_root)
        .expect("examples_meta.json not found; run `cargo build -p raylib-showcase --release --examples` first");
    let metas: Vec<ExampleMeta> =
        serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();

    // Optional thumbnail manifest from `gen_thumbnails`. If absent we
    // still emit tiles with a CSS placeholder block, so the gallery is
    // useful even before thumbnails have been captured.
    let thumbs_manifest = workspace_root
        .join("target")
        .join("thumbnails")
        .join("_manifest.json");
    let thumbs: BTreeMap<String, Option<String>> = if thumbs_manifest.exists() {
        let parsed: Vec<ThumbnailEntry> =
            serde_json::from_str(&fs::read_to_string(&thumbs_manifest).unwrap())
                .unwrap_or_default();
        parsed.into_iter().map(|t| (t.name, t.thumbnail)).collect()
    } else {
        BTreeMap::new()
    };

    // Group examples by category, sorted by category name then by example name.
    let mut by_category: BTreeMap<String, Vec<&ExampleMeta>> = BTreeMap::new();
    for m in &metas {
        by_category.entry(m.category.clone()).or_default().push(m);
    }
    for v in by_category.values_mut() {
        v.sort_by(|a, b| a.name.cmp(&b.name));
    }

    // Render the per-category sections that get spliced into the template.
    // Category ordering: alphabetical, but with `audio` pinned to the end.
    // Audio examples don't currently run in the web (no audio context wired
    // through to emscripten yet) and their software_renderer thumbnails fail,
    // so we push them down so the gallery doesn't lead with broken tiles.
    let mut ordered_cats: Vec<&String> = by_category.keys().collect();
    ordered_cats.sort_by(|a, b| match (a.as_str(), b.as_str()) {
        ("audio", "audio") => std::cmp::Ordering::Equal,
        ("audio", _) => std::cmp::Ordering::Greater,
        (_, "audio") => std::cmp::Ordering::Less,
        _ => a.cmp(b),
    });
    let mut categories_body = String::new();
    let mut total_tiles = 0usize;
    for cat in &ordered_cats {
        let entries = &by_category[*cat];
        categories_body.push_str(&format!(
            "    <section class=\"category\" id=\"cat-{}\">\n      <h2>{}</h2>\n      <div class=\"grid\">\n",
            cat, cat,
        ));
        for m in entries {
            total_tiles += 1;
            let thumb_html = match thumbs.get(&m.name).and_then(|t| t.clone()) {
                Some(file) => format!(
                    "<div class=\"thumb\"><img src=\"thumbnails/{}\" alt=\"{}\" loading=\"lazy\" /></div>",
                    file, m.name,
                ),
                // No thumbnail captured (gen-thumbnails failed for this
                // example). Show the name inside the placeholder so the
                // tile is still identifiable at a glance instead of being
                // a blank gray box.
                None => format!(
                    "<div class=\"thumb placeholder\"><span>{}</span></div>",
                    m.name,
                ),
            };
            let badge = if m.wasm_excluded {
                " <span class=\"badge\">desktop only</span>"
            } else {
                ""
            };
            categories_body.push_str(&format!(
                "        <a class=\"tile\" href=\"examples/{cat}/{name}.html\" data-name=\"{name}\">\n          {thumb}\n          <p>{name}{badge}</p>\n          <p class=\"links\"><a href=\"{c_url}\">C</a> &middot; <a href=\"{rust_url}\">Rust</a></p>\n        </a>\n",
                cat = cat,
                name = m.name,
                thumb = thumb_html,
                badge = badge,
                c_url = m.c_url,
                rust_url = m.rust_url,
            ));
        }
        categories_body.push_str("      </div>\n    </section>\n");
    }

    let index_template = fs::read_to_string(manifest_dir.join("index/template.html")).unwrap();
    let index_html = index_template.replace("{{CATEGORIES}}", &categories_body);
    fs::write(site.join("index.html"), index_html).unwrap();
    fs::copy(manifest_dir.join("index/style.css"), site.join("style.css")).unwrap();
    fs::copy(manifest_dir.join("index/script.js"), site.join("script.js")).unwrap();

    // Per-example HTML pages: copy wasm artifacts for buildable examples,
    // or write a "desktop only" placeholder for wasm-excluded ones.
    //
    // The shell template (`index/example_shell.html`) is loaded once and
    // used to synthesize the per-example HTML wrapper around the emscripten
    // `.js` loader. We don't rely on emcc emitting HTML itself, because
    // `cargo build --target wasm32-unknown-emscripten` invokes emcc with a
    // `.js` output path, so emcc never enters HTML-emit mode and the
    // `EMCC_CFLAGS --shell-file ...` is silently ignored. Synthesizing the
    // wrapper here keeps the substitution deterministic and avoids a second
    // emcc invocation.
    let wasm_dir = workspace_root
        .join("target")
        .join("wasm32-unknown-emscripten")
        .join("release")
        .join("examples");
    let shell_template = fs::read_to_string(manifest_dir.join("index/example_shell.html"))
        .expect("index/example_shell.html not found; required to synthesize per-example pages");
    for m in &metas {
        let out_dir = site.join("examples").join(&m.category);
        fs::create_dir_all(&out_dir).unwrap();

        if m.wasm_excluded {
            // Placeholder page so the gallery tile still has a destination.
            let html = render_desktop_only_placeholder(m);
            fs::write(out_dir.join(format!("{}.html", m.name)), html).unwrap();
            continue;
        }

        // Copy the runtime artifacts emcc produced. We deliberately do NOT
        // copy `.html` (emcc doesn't emit one — see comment above) and
        // instead synthesize it from the shell template below.
        let mut js_copied = false;
        for ext in &["js", "wasm", "data"] {
            let src = wasm_dir.join(format!("{}.{}", m.name, ext));
            if src.exists() {
                let dst = out_dir.join(format!("{}.{}", m.name, ext));
                let _ = fs::copy(src, dst);
                if *ext == "js" {
                    js_copied = true;
                }
            }
        }
        if js_copied {
            // Substitute placeholders in the shell. `{{{ SCRIPT }}}` is the
            // emscripten loader script tag (defaulted to async to match
            // emscripten's own default HTML output).
            let script_tag = format!(
                "<script async type=\"text/javascript\" src=\"{}.js\"></script>",
                m.name
            );
            let html = shell_template
                .replace("{{{ EXAMPLE_NAME }}}", &m.name)
                .replace("{{{ SCRIPT }}}", &script_tag);
            fs::write(out_dir.join(format!("{}.html", m.name)), html).unwrap();
        } else {
            // Fall back to a placeholder noting that the wasm artifacts
            // haven't been built yet, so the gallery tile still resolves
            // to a real page.
            let html = render_unbuilt_wasm_placeholder(m);
            fs::write(out_dir.join(format!("{}.html", m.name)), html).unwrap();
        }
    }

    // Copy thumbnail PNGs into _site/thumbnails/ so the gallery <img>
    // src="thumbnails/..." references resolve.
    let thumbs_src = workspace_root.join("target").join("thumbnails");
    if thumbs_src.exists() {
        let thumbs_dst = site.join("thumbnails");
        fs::create_dir_all(&thumbs_dst).unwrap();
        for e in fs::read_dir(thumbs_src).unwrap().flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("png") {
                let _ = fs::copy(&p, thumbs_dst.join(p.file_name().unwrap()));
            }
        }
    }

    eprintln!(
        "xtask_build_pages: wrote {} tiles across {} categories to {:?}",
        total_tiles,
        by_category.len(),
        site,
    );
}

fn render_desktop_only_placeholder(m: &ExampleMeta) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{name} &mdash; desktop only</title>
  <link rel="stylesheet" href="../../style.css" />
</head>
<body>
  <nav><a href="../../index.html">&larr; back</a></nav>
  <main>
    <h1>{name}</h1>
    <p class="banner">This example is <strong>desktop-only</strong> and is not built for the web.</p>
    <p>Run it locally with:</p>
    <pre><code>cargo run -p raylib-showcase --example {name}</code></pre>
    <h2>Source</h2>
    <ul>
      <li><a href="{c_url}">Original C source</a> (upstream raylib examples)</li>
      <li><a href="{rust_url}">Rust port</a> (this repository)</li>
    </ul>
  </main>
</body>
</html>
"#,
        name = m.name,
        c_url = m.c_url,
        rust_url = m.rust_url,
    )
}

fn render_unbuilt_wasm_placeholder(m: &ExampleMeta) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{name}</title>
  <link rel="stylesheet" href="../../style.css" />
</head>
<body>
  <nav><a href="../../index.html">&larr; back</a></nav>
  <main>
    <h1>{name}</h1>
    <p class="banner">The web build for this example was not available at gallery-build time.</p>
    <p>Run it locally with:</p>
    <pre><code>cargo run -p raylib-showcase --example {name}</code></pre>
    <h2>Source</h2>
    <ul>
      <li><a href="{c_url}">Original C source</a> (upstream raylib examples)</li>
      <li><a href="{rust_url}">Rust port</a> (this repository)</li>
    </ul>
  </main>
</body>
</html>
"#,
        name = m.name,
        c_url = m.c_url,
        rust_url = m.rust_url,
    )
}

fn find_examples_meta(workspace_root: &Path) -> Option<PathBuf> {
    for profile in &["release", "debug"] {
        let build_dir = workspace_root.join("target").join(profile).join("build");
        if !build_dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&build_dir).ok()?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("raylib-showcase-") {
                continue;
            }
            let candidate = entry.path().join("out").join("examples_meta.json");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}
