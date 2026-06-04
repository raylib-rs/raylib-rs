//! Tier-2 lifetime tests for raylib-allocated wrappers that need an
//! initialized raylib: Mesh accessors (GPU upload), ImageColors/ImagePalette,
//! FilePathList (RaylibHandle methods), and DataBuf under a live context.
//!
//! Run with the verbatim Tier-2 command from CLAUDE.md (all five
//! SUPPORT_MODULE features + SUPPORT_IMAGE_GENERATION are link requirements),
//! plus SUPPORT_MESH_GENERATION for the mesh accessor tests.
#![cfg(feature = "software_renderer")]

use raylib::prelude::*;
use raylib::test_harness::{render_frame, with_headless};

#[cfg(feature = "SUPPORT_MESH_GENERATION")]
#[test]
fn mesh_accessors_match_counts() {
    with_headless(64, 64, |_rl, thread| {
        let mesh = Mesh::gen_mesh_cube(thread, 1.0, 1.0, 1.0);
        let vc = mesh.as_ref().vertexCount as usize;
        let tc = mesh.as_ref().triangleCount as usize;
        assert!(vc > 0, "gen_mesh_cube must produce vertices");
        assert!(tc > 0, "gen_mesh_cube must produce triangles");
        // Mandatory attributes: present and sized by vertexCount.
        assert_eq!(mesh.vertices().len(), vc);
        assert_eq!(mesh.normals().len(), vc);
        assert_eq!(mesh.texcoords().len(), vc);
        // Optional attributes: either absent (empty) or sized by the count.
        // gen_mesh_cube does not generate these; pin emptiness so a future
        // raylib bump that starts generating them is noticed.
        assert!(mesh.texcoords2().is_empty());
        assert!(mesh.tangents().is_empty());
        assert!(mesh.colors().is_empty());
        // Indices: sized by triangleCount * 3 when present.
        let idx = mesh.indices();
        assert!(
            idx.is_empty() || idx.len() == tc * 3,
            "indices must be empty or triangleCount*3, got {}",
            idx.len()
        );
    });
}

#[cfg(feature = "SUPPORT_MESH_GENERATION")]
#[test]
fn mesh_accessor_mut_roundtrip() {
    with_headless(64, 64, |_rl, thread| {
        let mut mesh = Mesh::gen_mesh_plane(thread, 1.0, 1.0, 1, 1);
        let v0 = mesh.vertices()[0];
        let moved = Vector3::new(v0.x + 1.0, v0.y + 2.0, v0.z + 3.0);
        mesh.vertices_mut()[0] = moved;
        assert_eq!(
            mesh.vertices()[0],
            moved,
            "write through _mut must be visible"
        );
        let n = mesh.normals().len();
        if n > 0 {
            let flipped = Vector3::new(0.0, -1.0, 0.0);
            mesh.normals_mut()[0] = flipped;
            assert_eq!(mesh.normals()[0], flipped);
        }
    });
}

#[cfg(feature = "SUPPORT_IMAGE_GENERATION")]
#[test]
fn image_colors_and_palette_lifetimes() {
    with_headless(64, 64, |rl, thread| {
        // CPU-generated image → ImageColors / ImagePalette.
        let img = Image::gen_image_color(8, 4, Color::RED);
        let colors = img.get_image_data();
        assert_eq!(colors.len(), 8 * 4);
        assert!(
            colors.iter().all(|c| c.r == 255 && c.g == 0 && c.b == 0),
            "all pixels must be red"
        );
        let palette = img.extract_palette(16);
        assert_eq!(palette.len(), 1, "single-color image → 1 palette entry");
        assert_eq!((palette[0].r, palette[0].g, palette[0].b), (255, 0, 0));
        drop(palette); // UnloadImagePalette — ASAN validates the free path
        drop(colors); // UnloadImageColors — ASAN validates the free path

        // Rendered-frame readback → ImageColors (the owner's "use software
        // renderer mode to ensure allocations" case).
        let frame = render_frame(rl, thread, |d| d.clear_background(Color::BLUE));
        let frame_colors = frame.get_image_data();
        assert_eq!(frame_colors.len(), 64 * 64);
        let p = frame_colors[0];
        assert!(
            p.b > 150 && p.r < 90 && p.g < 90,
            "cleared-to-blue frame must read back blue, got ({}, {}, {})",
            p.r,
            p.g,
            p.b
        );
    });
}

#[test]
fn file_path_list_real_directory() {
    with_headless(32, 32, |rl, _thread| {
        let dir = std::env::temp_dir().join(format!("raylib_fpl_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["a.txt", "b.txt", "c.txt"] {
            std::fs::write(dir.join(name), name.as_bytes()).unwrap();
        }

        let list = rl.load_directory_files(dir.clone().into_os_string());
        let paths: Vec<&str> = list.iter().collect();
        assert_eq!(paths.len(), 3, "expected 3 files, got {paths:?}");
        for name in ["a.txt", "b.txt", "c.txt"] {
            assert!(
                paths.iter().any(|p| p.ends_with(name)),
                "missing {name} in {paths:?}"
            );
        }
        // ExactSizeIterator / DoubleEndedIterator / nth parity on a real list.
        assert_eq!(list.iter().len(), 3);
        assert_eq!(list.iter().rev().count(), 3);
        let mut it = list.iter();
        it.next();
        assert_eq!(it.len(), 2, "len must shrink as the iterator advances");
        assert!(list.iter().nth(2).is_some(), "nth(2) of 3 must exist");
        assert!(list.iter().nth(3).is_none(), "nth(3) of 3 must not exist");
        drop(list); // UnloadDirectoryFiles — ASAN validates the free path

        // Empty directory behavior.
        // NOTE (pinned): raylib's LoadDirectoryFiles returns a non-null paths
        // array even for empty directories (count=0), so iter() is safe.
        // FilePathIter::new asserts non-null; if a future raylib version
        // returns null for an empty dir this test will need to be adjusted.
        let empty = dir.join("empty_sub");
        std::fs::create_dir_all(&empty).unwrap();
        let empty_list = rl.load_directory_files(empty.into_os_string());
        assert_eq!(empty_list.iter().count(), 0, "empty dir → 0 paths");
        drop(empty_list);

        std::fs::remove_dir_all(&dir).ok();
    });
}

#[test]
fn databuf_alloc_cycle_under_initialized_raylib() {
    // Same allocator paths as the Tier-1 tests, but with raylib fully
    // initialized (rlsw Memory platform) — exercises MemAlloc/MemRealloc/
    // MemFree in the state real programs use them in.
    with_headless(32, 32, |_rl, _thread| {
        let buf = DataBuf::<[u32]>::alloc_from_copy(&[10, 20, 30]).expect("alloc");
        let mut grown = buf.realloc(5).map_err(|(e, _)| e).expect("realloc");
        grown[3].write(40);
        grown[4].write(50);
        // SAFETY: 0..3 initialized by alloc_from_copy + preserved by realloc;
        // 3..5 just written.
        let grown = unsafe { grown.assume_init() };
        assert_eq!(&*grown, &[10, 20, 30, 40, 50]);
    });
}
