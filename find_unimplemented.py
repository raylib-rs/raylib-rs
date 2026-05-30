"""
find_unimplemented.py — raylib-rs parity checker.

Scans raylib.h for every RLAPI function, classifies it as:
  [x]  implemented (ffi::<Name> found in raylib/src/core/)
  [~]  intentionally skipped (Rust std / listed below)
  [ ]  TODO — not yet wrapped

Emits docs/superpowers/parity-checklist.md as the authoritative source
of truth for WS3+ parity work.

Raygui (RAYGUIAPI) is scanned separately and appended as its own section.

Usage:
    python find_unimplemented.py        # from repo root
"""

import os

# ---------------------------------------------------------------------------
# Functions intentionally not wrapped: Rust std / core idioms do these as
# well or better.  (Locked decision: std-only line — base64/DEFLATE/CRC/
# MD5/SHA1 are KEPT; they are not in std.)
# ---------------------------------------------------------------------------
wont_impl = {
    "SetTraceLogCallback": "implemented via C shim; not detected by the ffi:: scan",

    # UTF-8 — use Rust's native &str/String/char
    "GetCodepointNext": "Rust str/char iteration",
    "GetCodepointPrevious": "Rust str/char iteration",
    "CodepointToUTF8": "char::encode_utf8",
    "LoadUTF8": "String/str",
    "UnloadUTF8": "String/str",

    # Text — use Rust's str/String + format!/std
    "TextCopy": "String",
    "TextIsEqual": "str ==",
    "TextLength": "str::len",
    "TextFormat": "format!",
    "TextSubtext": "str slicing",
    "TextReplace": "str::replace",
    "TextInsert": "String::insert_str",
    "TextJoin": "[T]::join",
    "TextSplit": "str::split",
    "TextAppend": "String::push_str",
    "TextFindIndex": "str::find",
    "TextToUpper": "str::to_uppercase",
    "TextToLower": "str::to_lowercase",
    "TextToPascal": "Rust string ops",
    "TextToSnake": "Rust string ops",
    "TextToCamel": "Rust string ops",
    "TextToInteger": "str::parse",
    "TextToFloat": "str::parse",

    # Filesystem / paths — use std::fs / std::path
    "LoadFileData": "std::fs::read",
    "UnloadFileData": "Drop",
    "SaveFileData": "std::fs::write",
    "LoadFileText": "std::fs::read_to_string",
    "UnloadFileText": "Drop",
    "SaveFileText": "std::fs::write",
    "FileExists": "Path::exists",
    "DirectoryExists": "Path::is_dir",
    "GetFileExtension": "Path::extension",
    "GetFileName": "Path::file_name",
    "GetFileNameWithoutExt": "Path::file_stem",
    "GetDirectoryPath": "Path::parent",
    "GetPrevDirectoryPath": "Path ops",
    "GetWorkingDirectory": "std::env::current_dir",
    "MakeDirectory": "std::fs::create_dir_all",
    "ChangeDirectory": "std::env::set_current_dir",
    "IsFileNameValid": "Rust path validation",
    "GetFileModTime": "fs::Metadata::modified",

    # Misc — not needed
    "MemRealloc": "not needed",
}


def extract_func_name(line):
    """Parse a function name from an RLAPI / RAYGUIAPI declaration line."""
    # e.g. 'RLAPI void InitWindow(int width, ...)' → 'InitWindow'
    # or   '    RAYGUIAPI void GuiEnable(void)' → 'GuiEnable'
    try:
        tokens = line.split()
        # Find the token containing '('
        for tok in tokens:
            if "(" in tok:
                name = tok.split("(")[0].replace("*", "").strip()
                if name:
                    return name
    except Exception:
        pass
    return None


def load_src_files(directory):
    """Read all .rs files under directory (recursive) into a list of strings.

    Uses os.walk so that files in subdirectories (e.g. core/callbacks/) are
    included, preventing subdir-only wrappers from being falsely marked TODO.
    """
    src_files = []
    try:
        for dirpath, _dirnames, filenames in os.walk(directory):
            for filename in filenames:
                if not filename.endswith(".rs"):
                    continue
                filepath = os.path.join(dirpath, filename)
                try:
                    with open(filepath, encoding="utf-8", errors="replace") as f:
                        src_files.append(f.read())
                except OSError:
                    pass
    except OSError:
        pass
    return src_files


def classify(func_name, src_files):
    """Return ("x"|"~"|" ", reason_or_None)."""
    if func_name in wont_impl:
        return ("~", wont_impl[func_name])
    for content in src_files:
        if "ffi::" + func_name in content:
            return ("x", None)
    return (" ", None)


# ---------------------------------------------------------------------------
# Section-header detection
# A line is treated as a section header when it is a // comment that:
#   - is not a NOTE/WARNING continuation, and
#   - contains 'function' or 'module' (case-insensitive), and
#   - at least one RLAPI appears in the next 30 lines.
# ---------------------------------------------------------------------------
def is_section_header(line, lookahead_lines, opener):
    stripped = line.strip()
    if not stripped.startswith("//"):
        return False
    text = stripped[2:].strip()
    if not text:
        return False
    tl = text.lower()
    # Must look like a standalone header, not a NOTE/WARNING prefix
    if tl.startswith("note:") or tl.startswith("warning"):
        return False
    # "control" covers raygui's "// Controls" / "// Basic controls set" headings.
    # We restrict the "control" match to short lines (≤80 chars) so that longer
    # inline comments like "// To avoid that behaviour and control frame…" are
    # not mistakenly promoted to section headers.
    has_control = "control" in tl and len(text) <= 80
    if "function" not in tl and "module" not in tl and not has_control:
        return False
    # Confirm at least one API line follows (match exact opener prefix)
    return any(l.startswith(opener) for l in lookahead_lines)


def parse_header(h_path, opener):
    """
    Parse h_path for API lines grouped under section headers.

    opener is the exact prefix for API lines, e.g. "RLAPI" or "    RAYGUIAPI".
    Section-header detection uses "RLAPI" or the trimmed opener in the lookahead.

    Returns:
        sections: list of (section_name, [func_name, ...]) in order
    """
    with open(h_path, encoding="utf-8", errors="replace") as f:
        lines = f.readlines()

    # For lookahead/section detection we always match on the exact opener.
    sections = []
    current_section = "General"
    current_funcs = []

    for i, line in enumerate(lines):
        lookahead = lines[i : i + 30]
        if is_section_header(line, lookahead, opener):
            # Flush previous section
            if current_funcs:
                sections.append((current_section, current_funcs))
                current_funcs = []
            current_section = line.strip()[2:].strip()
        elif line.startswith(opener):
            # It's an API declaration
            name = extract_func_name(line)
            if name:
                current_funcs.append(name)

    # Flush last section
    if current_funcs:
        sections.append((current_section, current_funcs))

    return sections


# ---------------------------------------------------------------------------
# Build parity table for one header
# ---------------------------------------------------------------------------
def build_parity(sections, src_files):
    """
    Returns:
        rows: list of (section, func_name, status, reason)
        counts: dict with n_impl, n_skip, n_todo, n_total
    """
    rows = []
    n_impl = n_skip = n_todo = 0
    for section, funcs in sections:
        for fn in funcs:
            status, reason = classify(fn, src_files)
            rows.append((section, fn, status, reason))
            if status == "x":
                n_impl += 1
            elif status == "~":
                n_skip += 1
            else:
                n_todo += 1
    n_total = n_impl + n_skip + n_todo
    return rows, {"n_impl": n_impl, "n_skip": n_skip, "n_todo": n_todo, "n_total": n_total}


# ---------------------------------------------------------------------------
# Emit checklist section to an open file handle
# ---------------------------------------------------------------------------
def write_section_rows(out, rows):
    """Write grouped rows (must all be the same section within a caller loop)."""
    current_section = None
    for (section, fn, status, reason) in rows:
        if section != current_section:
            out.write(f"\n### {section}\n\n")
            current_section = section
        if status == "~":
            # NOTE: [~] is a visual convention meaning "intentionally skipped".
            # It is NOT standard GitHub task-list syntax (GitHub only renders
            # [x] and [ ] as checkboxes).  The legend in the generated file
            # explains this to human readers; this comment is for maintainers.
            out.write(f"- [~] `{fn}` — {reason}\n")
        elif status == "x":
            out.write(f"- [x] `{fn}`\n")
        else:
            out.write(f"- [ ] `{fn}`\n")


# ---------------------------------------------------------------------------
# Console summary (keeps original behaviour)
# ---------------------------------------------------------------------------
def print_todo(lib_name, rows):
    print(f"===== {lib_name} (TODO) =====")
    for (_, fn, status, _) in rows:
        if status == " ":
            print(f"- [ ] {fn}")
    print()


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
RAYLIB_H = "./raylib-sys/raylib/src/raylib.h"
RAYGUI_H = "./raylib-sys/binding/raygui.h"
CORE_DIR = "./raylib/src/core/"
RGUI_DIR = "./raylib/src/rgui"
OUT_PATH = "./docs/superpowers/parity-checklist.md"

# --- raylib ---
raylib_src = load_src_files(CORE_DIR)
raylib_sections = parse_header(RAYLIB_H, "RLAPI")
raylib_rows, raylib_counts = build_parity(raylib_sections, raylib_src)

# --- raygui ---
raygui_src = load_src_files(RGUI_DIR)
raygui_sections = parse_header(RAYGUI_H, "    RAYGUIAPI")
raygui_rows, raygui_counts = build_parity(raygui_sections, raygui_src)

# --- Console output ---
print_todo("Raylib", raylib_rows)
print_todo("Raygui", raygui_rows)

rc = raylib_counts
gc = raygui_counts
print(
    f"raylib  : {rc['n_impl']:3d} implemented · {rc['n_skip']:3d} wont-impl · "
    f"{rc['n_todo']:3d} TODO of {rc['n_total']} RLAPI fns"
)
print(
    f"raygui  : {gc['n_impl']:3d} implemented · {gc['n_skip']:3d} wont-impl · "
    f"{gc['n_todo']:3d} TODO of {gc['n_total']} RAYGUIAPI fns"
)

# --- Write checklist ---
os.makedirs(os.path.dirname(OUT_PATH), exist_ok=True)
with open(OUT_PATH, "w", encoding="utf-8") as out:
    # --- raylib section ---
    out.write("# raylib 6.0 ↔ safe-binding parity checklist\n\n")
    out.write(
        f"_Generated by `find_unimplemented.py`. "
        f"{rc['n_impl']} implemented · "
        f"{rc['n_skip']} wont-impl (std/shim) · "
        f"{rc['n_todo']} TODO of {rc['n_total']} RLAPI fns._\n\n"
    )
    out.write(
        "Legend: `[x]` wrapped · `[~]` intentionally skipped (Rust std / see reason) · `[ ]` TODO\n\n"
    )
    out.write("## raylib.h — RLAPI surface\n")
    write_section_rows(out, raylib_rows)

    # --- raygui section ---
    out.write(
        f"\n---\n\n"
        f"## raygui.h — RAYGUIAPI surface\n\n"
        f"_Raygui parity is tracked here for visibility; full raygui work is WS5._\n\n"
        f"_{gc['n_impl']} implemented · "
        f"{gc['n_skip']} wont-impl · "
        f"{gc['n_todo']} TODO of {gc['n_total']} RAYGUIAPI fns._\n"
    )
    write_section_rows(out, raygui_rows)

print(f"\nChecklist written -> {OUT_PATH}")
