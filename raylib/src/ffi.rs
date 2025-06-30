pub use raylib_sys::*;

/// Provides handwritten documentation comments for FFI functions via rustdoc
pub mod with_docs {

/// Document that a function accesses a static variable without locking, and advises
/// readers of the proper precautions associated with thread-unsafe static access.
///
/// # Examples
///
/// ## Write access
/// ```ignore
/// static mut UNLOCKED_STATIC: bool;
///
/// /// Does dangerous stuff without checking
/// ///
/// /// #Safety
/// ///
/// #[doc = static_access!(mut UNLOCKED_STATIC)]
/// unsafe fn dangerous() {
///     unsafe {
///         UNLOCKED_STATIC = !UNLOCKED_STATIC;
///     }
/// }
/// ```
///
/// ## Read access
/// ```ignore
/// static mut UNLOCKED_STATIC: bool;
///
/// /// Does dangerous stuff without checking
/// ///
/// /// #Safety
/// ///
/// #[doc = static_access!(const UNLOCKED_STATIC)]
/// unsafe fn dangerous() {
///     if unsafe { UNLOCKED_STATIC } {
///         ...
///     }
/// }
/// ```
macro_rules! static_access {
    ($access:ident $global:expr) => {
        concat!("This function may ", static_access!(@$access), " the static `", stringify!($global), "` without locking. \
        Either ensure this function is only called on the same thread that initialized RLGL, or use appropriate synchronization.")
    };
    (@const) => { "read" };
    (@mut) => { "assign" };
}

macro_rules! join {
    (#($sep:literal, $final_sep:literal) $a:expr $(,)?) => { $a };
    (#($sep:literal, $final_sep:literal) $a:expr, $b:expr $(,)?) => { concat!($a, $final_sep, $b) };
    (#($sep:literal, $final_sep:literal) $a:expr, $b:expr, $($rest:expr),+ $(,)?) => { concat!($a, $sep, join!(#($sep, $final_sep) $b, $($rest),+)) };
}

/// Document that an item dereferences a pointer, and advise readers of the proper precautions
/// associated with pointer dereferencing.
///
/// # Arguments
///
/// ## `$access`
///
/// - `const` - Read-only access
/// - `mut` - Write access
///
/// ## `$reqs`
///
/// Pointer requirements
///
/// Requirements are whitespace-separated (NOT comma-separated) within square brackets (ex: `[n z i]`).
///
/// - `n` - Pointer must be non-null
/// - `z` - Pointer must be nul-terminated (C-string) (incompatible with `[T; n]`)
/// - `[T; n]` - Pointer must be an array of type `T` with `n` elements (incompatible with `z`)
/// - `i` - Data pointed to must be initialized *prior to calling* (as opposed to uninitialized)
///
/// A literal string can be made into a requirement by enclosing it in curly braces (ex: `{"not a fat pointer"}`).
/// A requirement can be enclosed in parentheses to append a footnote (ex: `(z [^"my_footnote"])`).
///
/// Any requirement can be made an explicit non-requirement by putting it in an optional, second square
/// bracket list with a ! at the start. (ex: `[n z] ![i]`: must be non-null and nul-terminated, but does
/// not need to be initialized).
///
/// # Examples
///
/// Mutable, uninitialized C-string
/// ```ignore
/// /// Does dangerous stuff without checking
/// ///
/// /// #Safety
/// ///
/// #[doc = ptr_deref!(mut "`foo`" [z n])]
/// unsafe fn dangerous(foo: *mut c_void) {
///     unsafe {
///         let p = foo.cast::<u8>();
///         while *p != 0 {
///             *p = 255;
///             p = p.add(1);
///         }
///     }
/// }
/// ```
///
/// Immutable, initialized array with special case for null
/// ```ignore
/// /// Does dangerous stuff without checking
/// ///
/// /// #Safety
/// ///
/// #[doc = ptr_deref!(const "`foo`" [(n [^"null"]) [u16; "`len`"] i])]
/// ///
/// /// [^null]: If `foo` is null, truncates and returns `len` instead
/// unsafe fn dangerous(foo: *const c_void, len: usize) -> u32 {
///     if !foo.is_null() {
///         let mut hash = 0;
///         unsafe {
///             let p = foo.cast::<u16>();
///             for i in 0..len {
///                 hash ^= *p.add(i);
///             }
///         }
///         hash
///     } else {
///         len as u32
///     }
/// }
/// ```
/// The above example uses `(n [^"null"])` for demonstration. In practice, this would be
/// better expressed with
/// ```ignore
/// ptr_deref!(const "`foo`" [[u16; "`len`"] i] if "`foo` is non-null")
/// ```
macro_rules! ptr_deref {
    // conditional
    ($access:ident $($ptr:literal),+ $(,)? [$($reqs:tt)*] $(![$($nonreqs:tt)+])? if $cond:literal $(else $else:literal)?) => {
        concat!("This function may dereference ", join!(#(", ", " and/or ") $($ptr),+), " if ", $cond, ". \
        In this case, the caller must ", ptr_deref!(%$access $($ptr),+ [$($reqs)*] $(![$($nonreqs)+])?),
        $(" otherwise, ", $else)?)
    };
    // unconditional
    ($access:ident $($ptr:literal),+ $(,)? [$($reqs:tt)*] $(![$($nonreqs:tt)+])?) => {
        concat!("This function will dereference ", join!(#(", ", " and/or ") $($ptr),+), " unconditionally. \
        The caller must ", ptr_deref!(%$access $($ptr),+ [$($reqs)*] $(![$($nonreqs)+])?))
    };
    // shared
    (%$access:ident $($ptr:literal),+ $(,)? [$($reqs:tt)*] $(![$($nonreqs:tt)+])?) => {
        concat!("ensure ", join!(#(", ", " and/or ") $($ptr),+), " is/are ", join!(#(", ", " and ") $(ptr_deref!(#$reqs),)* concat!(" safe to dereference for ", ptr_deref!(@$access))), "."
        $(, " However there is no requirement for it/them to be ", join!(#(", ", " or ") $(ptr_deref!(#$nonreqs),)+), ".")?)
    };
    // arguments
    (@const) => { "reading" };
    (@mut) => { "writing" };
    (#n) => { "non-null" };
    (#z) => { "nul-terminated" };
    (#[$T:ty; $n:literal]) => { concat!("pointing to ", $n, " elements of ", stringify!($T)) };
    (#i) => { "initialized prior to calling" };
    (#{$lit:literal}) => { $lit };
    (#($token:tt [^$footnote:literal])) => { concat!(ptr_deref!(#$token), "[^", $footnote, "]") };
}

/// Document that a function calls a GL function without validating GL, and advises readers
/// of the proper precautions associated with calling possibly-unloaded external functions.
///
/// [`link_gl`] should also be used to create links to the documentation of the GL functions
/// referenced.
///
/// # Examples
///
/// ## Unconditional
/// ```ignore
/// /// Does dangerous stuff without checking
/// ///
/// /// #Safety
/// ///
/// #[doc = require_gl!(glBindTexture)]
/// ///
/// #[doc = link_gl!(4::glBindTexture)]
/// unsafe fn dangerous() {
///     unsafe {
///         glBindTexture(GL_TEXTURE_2D, 0);
///     }
/// }
/// ```
///
/// ## Multiple unconditional
/// ```ignore
/// /// Does dangerous stuff without checking
/// ///
/// /// #Safety
/// ///
/// #[doc = require_gl!(glBindTexture, glGenTextures, glTexImage2D)]
/// ///
/// #[doc = link_gl!(4::{glBindTexture, glGenTextures, glTexImage2D})]
/// unsafe fn dangerous() {
///     unsafe {
///         glBindTexture(GL_TEXTURE_2D, 0);
///         glGenTextures(GL_TEXTURE_2D, 0);
///         glTexImage2D(GL_TEXTURE_2D, 0);
///     }
/// }
/// ```
///
/// ## Multiple conditional
/// ```ignore
/// /// Does dangerous stuff without checking
/// ///
/// /// #Safety
/// ///
/// #[doc = require_gl!(glBindTexture, glGenTextures, glTexImage2D if "`x` is true")]
/// ///
/// #[doc = link_gl!(4::{glBindTexture, glGenTextures, glTexImage2D})]
/// unsafe fn dangerous(x: bool) {
///     if x {
///         unsafe {
///             glBindTexture(GL_TEXTURE_2D, 0);
///             glGenTextures(GL_TEXTURE_2D, 0);
///             glTexImage2D(GL_TEXTURE_2D, 0);
///         }
///     }
/// }
/// ```
macro_rules! require_gl {
    // unconditional call(s)
    ($($list:ident),+) => {
        concat!("This function will call ", join!(#(", ", " and/or ") $(concat!("[`", stringify!($list), "`][]")),+), " unconditionally. ", require_gl!())
    };
    // conditional call(s)
    ($($list:ident),+ if $cond:literal) => {
        concat!("This function may call ", join!(#(", ", " and/or ") $(concat!("[`", stringify!($list), "`][]")),+), " if ", $cond, ". In which case ", require_gl!())
    };
    // soundness
    () => { "GL must be loaded and ready to be called into." };
}

/// Create reusable (not inline) reference-style links to khronos.org OpenGL refpages.
///
/// Uses syntax inspired by Rust glob-import.
///
/// # Examples
///
/// ## Linking a single OpenGL 4 function
/// ```ignore
/// /// Calls [`glBindTexture`][]
/// ///
/// #[doc = link_gl!(4::glBindTexture)]
/// fn foo() { ... }
/// ```
///
/// ## Linking multiple OpenGL 4 functions
/// ```ignore
/// /// Calls [`glBindTexture`][], [`glGenTextures`][], and [`glTexImage2D`][]
/// ///
/// #[doc = link_gl!(4::{glBindTexture, glGenTextures, glTexImage2D})]
/// fn foo() { ... }
/// ```
///
/// ## Linking functions from both OpenGL 4 and 2.1
/// ```ignore
/// /// Calls [`glMatrixMode`][] and [`glPushMatrix`][] when using GL 2.1,
/// /// or [`glBindTexture`][] if using GL 4
/// ///
/// #[doc = link_gl!(2.1::{glMatrixMode, glPushMatrix}, 4::glBindTexture)]
/// fn foo() { ... }
/// ```
macro_rules! link_gl {
    // single item doesn't need braces
    ($version:tt::$fn:ident) => {
        link_gl!($version::{$fn})
    };

    // link gen
    ($version:tt::{$($fn:ident),+ $(,)?}) => {
        concat!($(
            // reference identifier
            "[`", stringify!($fn), "`]: ",

            // link
            link_gl!(@ $version $fn),

            // title
            " \"",
                stringify!($fn), " - OpenGL ", stringify!($version), " Reference Pages",
            "\"\n",
        )+)
    };

    // multiple versions into one item
    ($($version:tt::$fns:tt),+ $(,)?) => { concat!($(link_gl!($version::$fns)),+) };

    // version-specific links
    (@ 2.1 $fn:ident) => {
        concat!("https://registry.khronos.org/OpenGL-Refpages/gl", stringify!($version), "/xhtml/", stringify!($fn), ".xml")
    };
    (@ 4 $fn:ident) => {
        concat!("https://registry.khronos.org/OpenGL-Refpages/gl", stringify!($version), "/html/", stringify!($fn), ".xhtml")
    };
}

// /// Supplies the feature flags of a given graphics API
// ///
// /// GRAPHICS API NOTES:
// /// - `GRAPHICS_API_OPENGL_11` => `feature = "opengl_11"`
// /// - `GRAPHICS_API_OPENGL_21` => `feature = "opengl_21"`
// /// - `GRAPHICS_API_OPENGL_33` => `feature = "opengl_33"`
// /// - `GRAPHICS_API_OPENGL_43` => `feature = "opengl_43"`
// /// - `GRAPHICS_API_OPENGL_ES2` => `feature = "opengl_es_20"`
// /// - `GRAPHICS_API_OPENGL_ES3` => `feature = "opengl_es_30"`
// #[allow(unused, reason = "currently not compatible with cfg, but macro expansion is still useful for generating them")]
// macro_rules! graphics_api {
//     (gl_11) => {
//         feature = "opengl_11"
//     };

//     (gl_21) => {
//         all(
//             feature = "opengl_21",
//             // // Security check in case multiple GRAPHICS_API_OPENGL_* defined
//             // #if defined(GRAPHICS_API_OPENGL_11)
//             //     #if defined(GRAPHICS_API_OPENGL_21)
//             //         #undef GRAPHICS_API_OPENGL_21
//             //     #endif
//             //     // ...
//             // #endif
//             not(feature = "opengl_11")
//         )
//     };

//     (gl_33) => {
//         all(
//             any(
//                 feature = "opengl_33",
//                 // // Security check in case no GRAPHICS_API_OPENGL_* defined
//                 // #if !defined(GRAPHICS_API_OPENGL_11) && \
//                 //     !defined(GRAPHICS_API_OPENGL_21) && \
//                 //     !defined(GRAPHICS_API_OPENGL_33) && \
//                 //     !defined(GRAPHICS_API_OPENGL_43) && \
//                 //     !defined(GRAPHICS_API_OPENGL_ES2) && \
//                 //     !defined(GRAPHICS_API_OPENGL_ES3)
//                 //         #define GRAPHICS_API_OPENGL_33
//                 // #endif
//                 all(
//                     not(feature = "opengl_11"),
//                     not(feature = "opengl_21"),
//                     not(feature = "opengl_33"),
//                     not(feature = "opengl_43"),
//                     not(feature = "opengl_es_20"),
//                     not(feature = "opengl_es_30"),
//                 ),
//                 // // OpenGL 2.1 uses most of OpenGL 3.3 Core functionality
//                 // // WARNING: Specific parts are checked with #if defines
//                 // #if defined(GRAPHICS_API_OPENGL_21)
//                 //     #define GRAPHICS_API_OPENGL_33
//                 // #endif
//                 feature = "opengl_21",
//                 // // OpenGL 4.3 uses OpenGL 3.3 Core functionality
//                 // #if defined(GRAPHICS_API_OPENGL_43)
//                 //     #define GRAPHICS_API_OPENGL_33
//                 // #endif
//                 feature = "opengl_43",
//             ),
//             // // Security check in case multiple GRAPHICS_API_OPENGL_* defined
//             // #if defined(GRAPHICS_API_OPENGL_11)
//             //     // ...
//             //     #if defined(GRAPHICS_API_OPENGL_33)
//             //         #undef GRAPHICS_API_OPENGL_33
//             //     #endif
//             //     // ...
//             // #endif
//             not(feature = "opengl_11")
//         )
//     };

//     (gl_43) => {
//         all(
//             feature = "opengl_43",
//             // // Security check in case multiple GRAPHICS_API_OPENGL_* defined
//             // #if defined(GRAPHICS_API_OPENGL_11)
//             //     // ...
//             //     #if defined(GRAPHICS_API_OPENGL_43)
//             //         #undef GRAPHICS_API_OPENGL_43
//             //     #endif
//             //     // ...
//             // #endif
//             not(feature = "opengl_11")
//         )
//     };

//     (gl_es2) => {
//         all(
//             any(
//                 feature = "opengl_es_20",
//                 // // OpenGL ES 3.0 uses OpenGL ES 2.0 functionality (and more)
//                 // #if defined(GRAPHICS_API_OPENGL_ES3)
//                 //     #define GRAPHICS_API_OPENGL_ES2
//                 // #endif
//                 feature = "opengl_es_30",
//             ),
//             // // Security check in case multiple GRAPHICS_API_OPENGL_* defined
//             // #if defined(GRAPHICS_API_OPENGL_11)
//             //     // ...
//             //     #if defined(GRAPHICS_API_OPENGL_ES2)
//             //         #undef GRAPHICS_API_OPENGL_ES2
//             //     #endif
//             // #endif
//             not(feature = "opengl_11")
//         )
//     };

//     (gl_es3) => {
//         feature = "opengl_es_30" // Something tells me the lack of "not(opengl_11)" might be unintentional
//     };

//     // Combinations
//     (($($tokens:tt)+)) => {
//         graphics_api!($($tokens)+)
//     };
//     (!$item:tt) => {
//         not(graphics_api!($item))
//     };
//     ($first:tt|$($rest:tt)|+) => {
//         any(graphics_api!($first), $(graphics_api!($rest)),+)
//     };
//     ($first:tt&$($rest:tt)&+) => {
//         all(graphics_api!($first), $(graphics_api!($rest)),+)
//     };
// }

use {
    static_access,
    ptr_deref,
    require_gl,
    link_gl,
    join,
    // graphics_api,
};

/// Provide documentation for public reexports of [`raylib_sys`] items,
/// automatically generating Raylib Github links to the symbols.
///
/// # Example
/// ```ignore
/// provide_docs!{
///     ["rcore.c"] // the Raylib source file defining the upcoming symbols
///
///     /// Begins drawing
///     BeginDrawing
///
///     /// Ends drawing
///     EndDrawing
///
///     ["rshapes.c"]
///
///     /// Test if two rectangles are colliding
///     CheckCollisionRecs
/// }
/// ```
macro_rules! provide_docs {
    ($(
        [$file:literal]
        $(
            $(#[$m:meta])*
            $item:ident
        )*
    )*) => {$(
        $(
            $(#[$m])*
            #[doc = concat!("\n\nSee [", $file, "#`", stringify!($item), "`](https://github.com/search?q=repo:raysan5/raylib+src/", $file, "+", stringify!($item), "&type=code)")]
            pub use raylib_sys::$item;
        )*
    )*};
}

provide_docs!{

["rlgl.h"] // rlgl.h is header-only; we are still targeting the definitions not just declarations

/// Default internal render batch elements limits
///
/// ## `GRAPHICS_API_OPENGL_11` or `GRAPHICS_API_OPENGL_33`
///
/// This is the maximum amount of elements (quads) per batch
///
/// NOTE: Be careful with text, every letter maps to a quad
///
/// ## `GRAPHICS_API_OPENGL_ES2`
///
/// We reduce memory sizes for embedded systems (RPI and HTML5)
///
/// NOTE: On HTML5 (emscripten) this is allocated on heap,
/// by default it's only 16MB!...just take care...
RL_DEFAULT_BATCH_BUFFER_ELEMENTS

/// Default number of batch buffers (multi-buffering)
RL_DEFAULT_BATCH_BUFFERS

/// Default number of batch draw calls (by state changes: mode, texture)
RL_DEFAULT_BATCH_DRAWCALLS

/// Maximum number of textures units that can be activated on batch drawing ([`SetShaderValueTexture()`])
RL_DEFAULT_BATCH_MAX_TEXTURE_UNITS

// Internal Matrix stack

/// Maximum size of Matrix stack
RL_MAX_MATRIX_STACK_SIZE

// Shader limits

/// Maximum number of shader locations supported
RL_MAX_SHADER_LOCATIONS

// Projection matrix culling

/// Default near cull distance
RL_CULL_DISTANCE_NEAR

/// Default far cull distance
RL_CULL_DISTANCE_FAR

// Texture parameters (equivalent to OpenGL defines)

/// `GL_TEXTURE_WRAP_S`
RL_TEXTURE_WRAP_S
/// `GL_TEXTURE_WRAP_T`
RL_TEXTURE_WRAP_T
/// `GL_TEXTURE_MAG_FILTER`
RL_TEXTURE_MAG_FILTER
/// `GL_TEXTURE_MIN_FILTER`
RL_TEXTURE_MIN_FILTER
/// `GL_NEAREST`
RL_TEXTURE_FILTER_NEAREST
/// `GL_LINEAR`
RL_TEXTURE_FILTER_LINEAR
/// `GL_NEAREST_MIPMAP_NEAREST`
RL_TEXTURE_FILTER_MIP_NEAREST
/// `GL_NEAREST_MIPMAP_LINEAR`
RL_TEXTURE_FILTER_NEAREST_MIP_LINEAR
/// `GL_LINEAR_MIPMAP_NEAREST`
RL_TEXTURE_FILTER_LINEAR_MIP_NEAREST
/// `GL_LINEAR_MIPMAP_LINEAR`
RL_TEXTURE_FILTER_MIP_LINEAR
/// Anisotropic filter (custom identifier)
RL_TEXTURE_FILTER_ANISOTROPIC
/// Texture mipmap bias, percentage ratio (custom identifier)
RL_TEXTURE_MIPMAP_BIAS_RATIO
/// `GL_REPEAT`
RL_TEXTURE_WRAP_REPEAT
/// `GL_CLAMP_TO_EDGE`
RL_TEXTURE_WRAP_CLAMP
/// `GL_MIRRORED_REPEAT`
RL_TEXTURE_WRAP_MIRROR_REPEAT
/// `GL_MIRROR_CLAMP_EXT`
RL_TEXTURE_WRAP_MIRROR_CLAMP

// Matrix modes (equivalent to OpenGL)

/// `GL_MODELVIEW`
RL_MODELVIEW
/// `GL_PROJECTION`
RL_PROJECTION
/// `GL_TEXTURE`
RL_TEXTURE

// Primitive assembly draw modes

/// `GL_LINES`
RL_LINES
/// `GL_TRIANGLES`
RL_TRIANGLES
/// `GL_QUADS`
RL_QUADS

// GL equivalent data types

/// `GL_UNSIGNED_BYTE`
RL_UNSIGNED_BYTE
/// `GL_FLOAT`
RL_FLOAT

// GL buffer usage hint

/// `GL_STREAM_DRAW`
RL_STREAM_DRAW
/// `GL_STREAM_READ`
RL_STREAM_READ
/// `GL_STREAM_COPY`
RL_STREAM_COPY
/// `GL_STATIC_DRAW`
RL_STATIC_DRAW
/// `GL_STATIC_READ`
RL_STATIC_READ
/// `GL_STATIC_COPY`
RL_STATIC_COPY
/// `GL_DYNAMIC_DRAW`
RL_DYNAMIC_DRAW
/// `GL_DYNAMIC_READ`
RL_DYNAMIC_READ
/// `GL_DYNAMIC_COPY`
RL_DYNAMIC_COPY

// GL Shader type

/// `GL_FRAGMENT_SHADER`
RL_FRAGMENT_SHADER
/// `GL_VERTEX_SHADER`
RL_VERTEX_SHADER
/// `GL_COMPUTE_SHADER`
RL_COMPUTE_SHADER

// GL blending factors

/// `GL_ZERO`
RL_ZERO
/// `GL_ONE`
RL_ONE
/// `GL_SRC_COLOR`
RL_SRC_COLOR
/// `GL_ONE_MINUS_SRC_COLOR`
RL_ONE_MINUS_SRC_COLOR
/// `GL_SRC_ALPHA`
RL_SRC_ALPHA
/// `GL_ONE_MINUS_SRC_ALPHA`
RL_ONE_MINUS_SRC_ALPHA
/// `GL_DST_ALPHA`
RL_DST_ALPHA
/// `GL_ONE_MINUS_DST_ALPHA`
RL_ONE_MINUS_DST_ALPHA
/// `GL_DST_COLOR`
RL_DST_COLOR
/// `GL_ONE_MINUS_DST_COLOR`
RL_ONE_MINUS_DST_COLOR
/// `GL_SRC_ALPHA_SATURATE`
RL_SRC_ALPHA_SATURATE
/// `GL_CONSTANT_COLOR`
RL_CONSTANT_COLOR
/// `GL_ONE_MINUS_CONSTANT_COLOR`
RL_ONE_MINUS_CONSTANT_COLOR
/// `GL_CONSTANT_ALPHA`
RL_CONSTANT_ALPHA
/// `GL_ONE_MINUS_CONSTANT_ALPHA`
RL_ONE_MINUS_CONSTANT_ALPHA

// GL blending functions/equations

/// `GL_FUNC_ADD`
RL_FUNC_ADD
/// `GL_MIN`
RL_MIN
/// `GL_MAX`
RL_MAX
/// `GL_FUNC_SUBTRACT`
RL_FUNC_SUBTRACT
/// `GL_FUNC_REVERSE_SUBTRACT`
RL_FUNC_REVERSE_SUBTRACT
/// `GL_BLEND_EQUATION`
RL_BLEND_EQUATION
/// `GL_BLEND_EQUATION_RGB`
/// (Same as [`RL_BLEND_EQUATION`])
RL_BLEND_EQUATION_RGB
/// `GL_BLEND_EQUATION_ALPHA`
RL_BLEND_EQUATION_ALPHA
/// `GL_BLEND_DST_RGB`
RL_BLEND_DST_RGB
/// `GL_BLEND_SRC_RGB`
RL_BLEND_SRC_RGB
/// `GL_BLEND_DST_ALPHA`
RL_BLEND_DST_ALPHA
/// `GL_BLEND_SRC_ALPHA`
RL_BLEND_SRC_ALPHA
/// `GL_BLEND_COLOR`
RL_BLEND_COLOR

/// `GL_READ_FRAMEBUFFER`
RL_READ_FRAMEBUFFER
/// `GL_DRAW_FRAMEBUFFER`
RL_DRAW_FRAMEBUFFER


/// Choose the current matrix to be transformed
///
/// # Safety
///
/// ## `GRAPHICS_API_OPENGL_11`
///
#[doc = require_gl!(glMatrixMode if "`mode` is [`RL_PROJECTION`], [`RL_MODELVIEW`], or [`RL_TEXTURE`]")]
///
#[doc = link_gl!(2.1::{glMatrixMode})]
///
/// ## `GRAPHICS_API_OPENGL_33` or `GRAPHICS_API_OPENGL_ES2`
///
#[doc = static_access!(mut RLGL.State)]
///
/// [`RL_TEXTURE`] is not supported with `GRAPHICS_API_OPENGL_33` or `GRAPHICS_API_OPENGL_ES2`.
/// `RLGL.State.currentMatrixMode` is assigned with `mode` unchecked, but `RLGL.State.currentMatrix` is only updated if `mode` is valid for the graphics API.
/// This can cause unexpected (but not undefined) behavior. Please avoid it.
rlMatrixMode

/// Push the current matrix to stack
///
/// # Safety
///
/// ## `GRAPHICS_API_OPENGL_11`
///
#[doc = require_gl!(glPushMatrix)]
///
#[doc = link_gl!(2.1::{glPushMatrix})]
///
/// ## `GRAPHICS_API_OPENGL_33` or `GRAPHICS_API_OPENGL_ES2`
///
#[doc = static_access!(mut RLGL.State)]
///
/// If `RLGL.State.stackCounter` is [`RL_MAX_MATRIX_STACK_SIZE`] or more, `RLGL.State.stack` will be assigned out of bounds.
/// Every [`rlPushMatrix`] call increments `RLGL.State.stackCounter`.
///
/// Make sure each [`rlPushMatrix`] call is paired with a corresponding [`rlPopMatrix`] call.
///
/// If `RLGL.State.currentMatrixMode` is [`RL_MODELVIEW`], this function will assign `RLGL.State.currentMatrix` with a pointer guaranteed to be non-null and dereferenceable.
#[doc = ptr_deref!(mut "`RLGL.State.currentMatrix`" [n])]
rlPushMatrix

/// Pop latest inserted matrix from stack
///
/// # Safety
///
/// ## `GRAPHICS_API_OPENGL_11`
///
#[doc = require_gl!(glPushMatrix)]
///
#[doc = link_gl!(2.1::{glPushMatrix})]
///
/// ## `GRAPHICS_API_OPENGL_33` or `GRAPHICS_API_OPENGL_ES2`
///
#[doc = static_access!(mut RLGL.State)]
///
#[doc = ptr_deref!(mut "`RLGL.State.currentMatrix`" [n] if "the stack is not empty")]
///
/// If the stack becomes empty **after popping**, and `RLGL.State.currentMatrixMode` is [`RL_MODELVIEW`], this function will assign `RLGL.State.currentMatrix` with a pointer guaranteed to be non-null and dereferenceable.
rlPopMatrix

/// Reset current matrix to identity matrix
/// # Safety
/// TODO
rlLoadIdentity

/// Multiply the current matrix by a translation matrix
/// # Safety
/// TODO
rlTranslatef

/// Multiply the current matrix by a rotation matrix
/// # Safety
/// TODO
rlRotatef

/// Multiply the current matrix by a scaling matrix
/// # Safety
/// TODO
rlScalef

/// Multiply the current matrix by another matrix
/// # Safety
/// TODO
rlMultMatrixf

/// TBD
/// # Safety
/// TODO
rlFrustum

/// TBD
/// # Safety
/// TODO
rlOrtho

/// Set the viewport area
/// # Safety
/// TODO
rlViewport

/// Set clip planes distances
/// # Safety
/// TODO
rlSetClipPlanes

/// Get cull plane distance near
/// # Safety
/// TODO
rlGetCullDistanceNear

/// Get cull plane distance far
/// # Safety
/// TODO
rlGetCullDistanceFar

//------------------------------------------------------------------------------------
// Functions Declaration - Vertex level operations
//------------------------------------------------------------------------------------

/// Initialize drawing mode (how to organize vertex)
/// # Safety
/// TODO
rlBegin

/// Finish vertex providing
/// # Safety
/// TODO
rlEnd

/// Define one vertex (position) - 2 int
/// # Safety
/// TODO
rlVertex2i

/// Define one vertex (position) - 2 float
/// # Safety
/// TODO
rlVertex2f

/// Define one vertex (position) - 3 float
/// # Safety
/// TODO
rlVertex3f

/// Define one vertex (texture coordinate) - 2 float
/// # Safety
/// TODO
rlTexCoord2f

/// Define one vertex (normal) - 3 float
/// # Safety
/// TODO
rlNormal3f

/// Define one vertex (color) - 4 byte
/// # Safety
/// TODO
rlColor4ub

/// Define one vertex (color) - 3 float
/// # Safety
/// TODO
rlColor3f

/// Define one vertex (color) - 4 float
/// # Safety
/// TODO
rlColor4f

//------------------------------------------------------------------------------------
// Functions Declaration - OpenGL style functions (common to 1.1, 3.3+, ES2)
// NOTE: This functions are used to completely abstract raylib code from OpenGL layer,
// some of them are direct wrappers over OpenGL calls, some others are custom
//------------------------------------------------------------------------------------

// Vertex buffers state

/// Enable vertex array (VAO, if supported)
/// # Safety
/// TODO
rlEnableVertexArray

/// Disable vertex array (VAO, if supported)
/// # Safety
/// TODO
rlDisableVertexArray

/// Enable vertex buffer (VBO)
/// # Safety
/// TODO
rlEnableVertexBuffer

/// Disable vertex buffer (VBO)
/// # Safety
/// TODO
rlDisableVertexBuffer

/// Enable vertex buffer element (VBO element)
/// # Safety
/// TODO
rlEnableVertexBufferElement

/// Disable vertex buffer element (VBO element)
/// # Safety
/// TODO
rlDisableVertexBufferElement

/// Enable vertex attribute index
/// # Safety
/// TODO
rlEnableVertexAttribute

/// Disable vertex attribute index
/// # Safety
/// TODO
rlDisableVertexAttribute

// Textures state

/// Select and active a texture slot
/// # Safety
/// TODO
rlActiveTextureSlot

/// Enable texture
/// # Safety
/// TODO
rlEnableTexture

/// Disable texture
/// # Safety
/// TODO
rlDisableTexture

/// Enable texture cubemap
/// # Safety
/// TODO
rlEnableTextureCubemap

/// Disable texture cubemap
/// # Safety
/// TODO
rlDisableTextureCubemap

/// Set texture parameters (filter, wrap)
/// # Safety
/// TODO
rlTextureParameters

/// Set cubemap parameters (filter, wrap)
/// # Safety
/// TODO
rlCubemapParameters

// Shader state

/// Enable shader program
/// # Safety
/// TODO
rlEnableShader

/// Disable shader program
/// # Safety
/// TODO
rlDisableShader

// Framebuffer state

/// Enable render texture (fbo)
/// # Safety
/// TODO
rlEnableFramebuffer

/// Disable render texture (fbo), return to default framebuffer
/// # Safety
/// TODO
rlDisableFramebuffer

/// Get the currently active render texture (fbo), 0 for default framebuffer
/// # Safety
/// TODO
rlGetActiveFramebuffer

/// Activate multiple draw color buffers
/// # Safety
/// TODO
rlActiveDrawBuffers

/// Blit active framebuffer to main framebuffer
/// # Safety
/// TODO
rlBlitFramebuffer

/// Bind framebuffer (FBO)
/// # Safety
/// TODO
rlBindFramebuffer

// General render state

/// Enable color blending
/// # Safety
/// TODO
rlEnableColorBlend

/// Disable color blending
/// # Safety
/// TODO
rlDisableColorBlend

/// Enable depth test
/// # Safety
/// TODO
rlEnableDepthTest

/// Disable depth test
/// # Safety
/// TODO
rlDisableDepthTest

/// Enable depth write
/// # Safety
/// TODO
rlEnableDepthMask

/// Disable depth write
/// # Safety
/// TODO
rlDisableDepthMask

/// Enable backface culling
/// # Safety
/// TODO
rlEnableBackfaceCulling

/// Disable backface culling
/// # Safety
/// TODO
rlDisableBackfaceCulling

/// Color mask control
/// # Safety
/// TODO
rlColorMask

/// Set face culling mode
/// # Safety
/// TODO
rlSetCullFace

/// Enable scissor test
/// # Safety
/// TODO
rlEnableScissorTest

/// Disable scissor test
/// # Safety
/// TODO
rlDisableScissorTest

/// Scissor test
/// # Safety
/// TODO
rlScissor

/// Enable wire mode
/// # Safety
/// TODO
rlEnableWireMode

/// Enable point mode
/// # Safety
/// TODO
rlEnablePointMode

/// Disable wire (and point) mode
/// # Safety
/// TODO
rlDisableWireMode

/// Set the line drawing width
/// # Safety
/// TODO
rlSetLineWidth

/// Get the line drawing width
/// # Safety
/// TODO
rlGetLineWidth

/// Enable line aliasing
/// # Safety
/// TODO
rlEnableSmoothLines

/// Disable line aliasing
/// # Safety
/// TODO
rlDisableSmoothLines

/// Enable stereo rendering
/// # Safety
/// TODO
rlEnableStereoRender

/// Disable stereo rendering
/// # Safety
/// TODO
rlDisableStereoRender

/// Check if stereo render is enabled
/// # Safety
/// TODO
rlIsStereoRenderEnabled

/// Clear color buffer with color
/// # Safety
/// TODO
rlClearColor

/// Clear used screen buffers (color and depth)
/// # Safety
/// TODO
rlClearScreenBuffers

/// Check and log OpenGL error codes
/// # Safety
/// TODO
rlCheckErrors

/// Set blending mode
/// # Safety
/// TODO
rlSetBlendMode

/// Set blending mode factor and equation (using OpenGL factors)
/// # Safety
/// TODO
rlSetBlendFactors

/// Set blending mode factors and equations separately (using OpenGL factors)
/// # Safety
/// TODO
rlSetBlendFactorsSeparate

//------------------------------------------------------------------------------------
// Functions Declaration - rlgl functionality
//------------------------------------------------------------------------------------
// rlgl initialization functions

/// Initialize rlgl (buffers, shaders, textures, states)
/// # Safety
/// - if GRAPHICS_API_OPENGL_33 or GRAPHICS_API_OPENGL_ES2
///   - calls rlLoadTexture and assigns RLGL.State.defaultTextureId with return (possibly 0)
///   - calls rlLoadShaderDefault, assigning
///     - RLGL.State.defaultShaderLocs
///     - RLGL.State.defaultVShaderId
///     - RLGL.State.defaultFShaderId
///     - RLGL.State.defaultShaderId (possibly 0)
/// TODO
rlglInit

/// De-initialize rlgl (buffers, shaders, textures)
/// # Safety
/// TODO
rlglClose

/// Load OpenGL extensions (loader function required)
/// # Safety
/// TODO
rlLoadExtensions

/// Get current OpenGL version
/// # Safety
/// TODO
rlGetVersion

/// Set current framebuffer width
/// # Safety
/// TODO
rlSetFramebufferWidth

/// Get default framebuffer width
/// # Safety
/// TODO
rlGetFramebufferWidth

/// Set current framebuffer height
/// # Safety
/// TODO
rlSetFramebufferHeight

/// Get default framebuffer height
/// # Safety
/// TODO
rlGetFramebufferHeight

/// Get default texture id
/// # Safety
/// TODO
rlGetTextureIdDefault

/// Get default shader id
/// # Safety
/// TODO
rlGetShaderIdDefault

/// Get default shader locations
/// # Safety
/// TODO
rlGetShaderLocsDefault

// Render batch management
// NOTE: rlgl provides a default render batch to behave like OpenGL 1.1 immediate mode
// but this render batch API is exposed in case of custom batches are required

/// Load a render batch system
/// # Safety
/// TODO
rlLoadRenderBatch

/// Unload render batch system
/// # Safety
/// TODO
rlUnloadRenderBatch

/// Draw render batch data (Update->Draw->Reset)
/// # Safety
/// TODO
rlDrawRenderBatch

/// Set the active render batch for rlgl (NULL for default internal)
/// # Safety
/// TODO
rlSetRenderBatchActive

/// Update and draw internal render batch
/// # Safety
/// TODO
rlDrawRenderBatchActive

/// Check internal buffer overflow for a given number of vertex
/// # Safety
/// TODO
rlCheckRenderBatchLimit

/// Set current texture for render batch and check buffers limits
/// # Safety
/// TODO
rlSetTexture

//------------------------------------------------------------------------------------------------------------------------

// Vertex buffers management

/// Load vertex array (vao) if supported
/// # Safety
/// TODO
rlLoadVertexArray

/// Load a vertex buffer object
/// # Safety
/// TODO
rlLoadVertexBuffer

/// Load vertex buffer elements object
/// # Safety
/// TODO
rlLoadVertexBufferElement

/// Update vertex buffer object data on GPU buffer
/// # Safety
/// TODO
rlUpdateVertexBuffer

/// Update vertex buffer elements data on GPU buffer
/// # Safety
/// TODO
rlUpdateVertexBufferElements

/// Unload vertex array (vao)
/// # Safety
/// TODO
rlUnloadVertexArray

/// Unload vertex buffer object
/// # Safety
/// TODO
rlUnloadVertexBuffer

/// Set vertex attribute data configuration
/// # Safety
/// TODO
rlSetVertexAttribute

/// Set vertex attribute data divisor
/// # Safety
/// TODO
rlSetVertexAttributeDivisor

/// Set vertex attribute default value, when attribute to provided
/// # Safety
/// TODO
rlSetVertexAttributeDefault

/// Draw vertex array (currently active vao)
/// # Safety
/// TODO
rlDrawVertexArray

/// Draw vertex array elements
/// # Safety
/// TODO
rlDrawVertexArrayElements

/// Draw vertex array (currently active vao) with instancing
/// # Safety
/// TODO
rlDrawVertexArrayInstanced

/// Draw vertex array elements with instancing
/// # Safety
/// TODO
rlDrawVertexArrayElementsInstanced

// Textures management

/// Load texture data
///
/// # Errors
///
/// This function will error if `format` is not supported by the graphics API or if [`glGenTextures`][] fails.
/// The id returned will be zero if an error has occurred.
///
/// `width` and `height` should be >= 0 to avoid an error.
///
/// # Safety
///
#[doc = require_gl!(glBindTexture)]
///
/// If `format` is valid, `data` is passed to either [`glTexImage2D`][] or [`glCompressedTexImage2D`][] as bytes.
/// In this case, `data` must be either null[^nulldata] *or* initialized and valid. `width`, `height`, `mipmapCount`, and `format` must accurately describe the memory `data` points to.
///
#[doc = static_access!(mut RLGL.ExtSupported)]
///
/// [^nulldata]: "`data` may be a null pointer. In this case, texture memory is allocated to accommodate a texture of width `width` and height `height`.
/// You can then download subtextures to initialize this texture memory.
/// The image is undefined if the user tries to apply an uninitialized portion of the texture image to a primitive."
/// &mdash; [`glTexImage2D`][]
///
#[doc = link_gl!(4::{glBindTexture, glGenTextures, glTexImage2D, glCompressedTexImage2D})]
rlLoadTexture

/// Load depth texture/renderbuffer (to be attached to fbo)
/// # Safety
/// TODO
rlLoadTextureDepth

/// Load texture cubemap data
/// # Safety
/// TODO
rlLoadTextureCubemap

/// Update texture with new data on GPU
/// # Safety
/// TODO
rlUpdateTexture

/// Get OpenGL internal formats
///
/// # Errors
///
/// This function will error if `format` is not supported by the graphics API.
/// After returning, `glInternalFormat`, `glFormat`, and `glType` are zero if an error has occurred.
///
/// # Safety
///
/// RLGL must be initialized.
///
#[doc = static_access!(mut RLGL.ExtSupported)]
///
#[doc = ptr_deref!(mut "`glInternalFormat`", "`glFormat`", "`glType`" [n] ![i] if "the stack is not empty")]
rlGetGlTextureFormats

/// Get name string for pixel format
/// # Safety
/// TODO
rlGetPixelFormatName

/// Unload texture from GPU memory
/// # Safety
/// TODO
rlUnloadTexture

/// Generate mipmap data for selected texture
/// # Safety
/// TODO
rlGenTextureMipmaps

/// Read texture pixel data
/// # Safety
/// TODO
rlReadTexturePixels

/// Read screen pixel data (color buffer)
/// # Safety
/// TODO
rlReadScreenPixels

// Framebuffer management (fbo)

/// Load an empty framebuffer
/// # Safety
/// TODO
rlLoadFramebuffer

/// Attach texture/renderbuffer to a framebuffer
/// # Safety
/// TODO
rlFramebufferAttach

/// Verify framebuffer is complete
/// # Safety
/// TODO
rlFramebufferComplete

/// Delete framebuffer from GPU
/// # Safety
/// TODO
rlUnloadFramebuffer

// Shaders management

/// Load shader from code strings
/// # Safety
/// TODO
rlLoadShaderCode

/// Compile custom shader and return shader id (type: [`RL_VERTEX_SHADER`], [`RL_FRAGMENT_SHADER`], [`RL_COMPUTE_SHADER`])
///
/// # Notes
///
/// `shaderCode` does not need to outlive the compiled shader.
/// &mdash; [`glShaderSource`][]
///
/// # Errors
///
/// This function will error either if shaders are not supported by the graphics API[^shaderapi] or if either [`glCreateShader`][] or [`glCompileShader`][] fails.
/// The id returned will be zero if an error has occurred.
///
/// # Safety
///
#[doc = require_gl!(glCreateShader, glShaderSource, glCompileShader if "shaders are supported[^shaderapi]")]
///
#[doc = ptr_deref!(const "`shaderCode`" [n z i])]
///
#[doc = link_gl!(4::{glCreateShader, glShaderSource, glCompileShader})]
///
/// [^shaderapi]: Either `GRAPHICS_API_OPENGL_33` or `GRAPHICS_API_OPENGL_ES2` is required for shader support.
rlCompileShader

/// Load custom shader program
/// # Safety
/// TODO
rlLoadShaderProgram

/// Unload shader program
/// # Safety
/// TODO
rlUnloadShaderProgram

/// Get shader location uniform
/// # Safety
/// TODO
rlGetLocationUniform

/// Get shader location attribute
/// # Safety
/// TODO
rlGetLocationAttrib

/// Set shader value uniform
/// # Safety
/// TODO
rlSetUniform

/// Set shader value matrix
/// # Safety
/// TODO
rlSetUniformMatrix

/// Set shader value matrices
/// # Safety
/// TODO
rlSetUniformMatrices

/// Set shader value sampler
/// # Safety
/// TODO
rlSetUniformSampler

/// Set shader currently active (id and locations)
/// # Safety
/// TODO
rlSetShader

// Compute shader management

/// Load compute shader program
/// # Safety
/// TODO
rlLoadComputeShaderProgram

/// Dispatch compute shader (equivalent to *draw* for graphics pipeline)
/// # Safety
/// TODO
rlComputeShaderDispatch

// Shader buffer storage object management (ssbo)

/// Load shader storage buffer object (SSBO)
/// # Safety
/// TODO
rlLoadShaderBuffer

/// Unload shader storage buffer object (SSBO)
/// # Safety
/// TODO
rlUnloadShaderBuffer

/// Update SSBO buffer data
/// # Safety
/// TODO
rlUpdateShaderBuffer

/// Bind SSBO buffer
/// # Safety
/// TODO
rlBindShaderBuffer

/// Read SSBO buffer data (GPU->CPU)
/// # Safety
/// TODO
rlReadShaderBuffer

/// Copy SSBO data between buffers
/// # Safety
/// TODO
rlCopyShaderBuffer

/// Get SSBO buffer size
/// # Safety
/// TODO
rlGetShaderBufferSize

// Buffer management

/// Bind image texture
/// # Safety
/// TODO
rlBindImageTexture

// Matrix state management

/// Get internal modelview matrix
/// # Safety
/// TODO
rlGetMatrixModelview

/// Get internal projection matrix
/// # Safety
/// TODO
rlGetMatrixProjection

/// Get internal accumulated transform matrix
/// # Safety
/// TODO
rlGetMatrixTransform

/// Get internal projection matrix for stereo render (selected eye)
/// # Safety
/// TODO
rlGetMatrixProjectionStereo

/// Get internal view offset matrix for stereo render (selected eye)
/// # Safety
/// TODO
rlGetMatrixViewOffsetStereo

/// Set a custom projection matrix (replaces internal projection matrix)
/// # Safety
/// TODO
rlSetMatrixProjection

/// Set a custom modelview matrix (replaces internal modelview matrix)
/// # Safety
/// TODO
rlSetMatrixModelview

/// Set eyes projection matrices for stereo rendering
/// # Safety
/// TODO
rlSetMatrixProjectionStereo

/// Set eyes view offsets matrices for stereo rendering
/// # Safety
/// TODO
rlSetMatrixViewOffsetStereo

// Quick and dirty cube/quad buffers load->draw->unload

/// Load and draw a cube
/// # Safety
/// TODO
rlLoadDrawCube

/// Load and draw a quad
/// # Safety
/// TODO
rlLoadDrawQuad

["rcore.c"]

// Window-related functions

/// Initialize window and OpenGL context
/// # Safety
/// - Window must not currently be open
/// - assigns CORE
/// - assigns isGpuReady
/// - calls InitPlatform
/// - calls rlglInit
/// - calls SetupViewport
/// - if SUPPORT_MODULE_RTEXT
///   - if SUPPORT_DEFAULT_FONT
///     - calls LoadFontDefault
///     - if SUPPORT_MODULE_RSHAPES
///       - calls GetFontDefault (unchecked)
///       - calls SetShapesTexture
/// - else
///   - if SUPPORT_MODULE_RSHAPES
///     - calls rlGetTextureIdDefault (unchecked)
///     - calls SetShapesTexture
/// - calls SetRandomSeed
/// - calls GetWorkingDirectory
/// TODO
InitWindow

/// Close window and unload OpenGL context
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
CloseWindow

/// Check if application should close (KEY_ESCAPE pressed or windows close icon clicked)
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
WindowShouldClose

/// Check if window has been initialized successfully
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
IsWindowReady

/// Check if window is currently fullscreen
/// # Safety
/// TODO
IsWindowFullscreen

/// Check if window is currently hidden
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
IsWindowHidden

/// Check if window is currently minimized
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
IsWindowMinimized

/// Check if window is currently maximized
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
IsWindowMaximized

/// Check if window is currently focused
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
IsWindowFocused

/// Check if window has been resized last frame
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
IsWindowResized

/// Check if one specific window flag is enabled
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
IsWindowState

/// Set window configuration state using flags
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowState

/// Clear window configuration state flags
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
ClearWindowState

/// Toggle window state: fullscreen/windowed, resizes monitor to match window resolution
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
ToggleFullscreen

/// Toggle window state: borderless windowed, resizes window to match monitor resolution
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
ToggleBorderlessWindowed

/// Set window state: maximized, if resizable
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
MaximizeWindow

/// Set window state: minimized, if resizable
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
MinimizeWindow

/// Set window state: not minimized/maximized
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
RestoreWindow

/// Set icon for window (single image, RGBA 32bit)
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowIcon

/// Set icon for window (multiple images, RGBA 32bit)
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowIcons

/// Set title for window
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowTitle

/// Set window position on screen
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowPosition

/// Set monitor for the current window
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowMonitor

/// Set window minimum dimensions (for FLAG_WINDOW_RESIZABLE)
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowMinSize

/// Set window maximum dimensions (for FLAG_WINDOW_RESIZABLE)
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowMaxSize

/// Set window dimensions
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowSize

/// Set window opacity [0.0f..1.0f]
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowOpacity

/// Set window focused
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetWindowFocused

/// Get native window handle
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetWindowHandle

/// Get current screen width
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetScreenWidth

/// Get current screen height
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetScreenHeight

/// Get current render width (it considers HiDPI)
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetRenderWidth

/// Get current render height (it considers HiDPI)
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetRenderHeight

/// Get number of connected monitors
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetMonitorCount

/// Get current monitor where window is placed
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetCurrentMonitor

/// Get specified monitor position
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetMonitorPosition

/// Get specified monitor width (current video mode used by monitor)
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetMonitorWidth

/// Get specified monitor height (current video mode used by monitor)
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetMonitorHeight

/// Get specified monitor physical width in millimetres
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetMonitorPhysicalWidth

/// Get specified monitor physical height in millimetres
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetMonitorPhysicalHeight

/// Get specified monitor refresh rate
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetMonitorRefreshRate

/// Get window position XY on monitor
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetWindowPosition

/// Get window scale DPI factor
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetWindowScaleDPI

/// Get the human-readable, UTF-8 encoded name of the specified monitor
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetMonitorName

/// Set clipboard text content
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
SetClipboardText

/// Get clipboard text content
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetClipboardText

/// Get clipboard image content
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
GetClipboardImage

/// Enable waiting for events on EndDrawing(), no automatic event polling
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
EnableEventWaiting

/// Disable waiting for events on EndDrawing(), automatic events polling
/// # Safety
/// - Window must currently be open
/// - Must be called on same thread window was opened
/// TODO
DisableEventWaiting

// Cursor-related functions

/// Shows cursor
/// # Safety
/// TODO
ShowCursor

/// Hides cursor
/// # Safety
/// TODO
HideCursor

/// Check if cursor is not visible
/// # Safety
/// TODO
IsCursorHidden

/// Enables cursor (unlock cursor)
/// # Safety
/// TODO
EnableCursor

/// Disables cursor (lock cursor)
/// # Safety
/// TODO
DisableCursor

/// Check if cursor is on the screen
/// # Safety
/// TODO
IsCursorOnScreen

// Drawing-related functions

/// Set background color (framebuffer clear color)
/// # Safety
/// TODO
ClearBackground

/// Setup canvas (framebuffer) to start drawing
/// # Safety
/// TODO
BeginDrawing

/// End canvas drawing and swap buffers (double buffering)
/// # Safety
/// TODO
EndDrawing

/// Begin 2D mode with custom camera (2D)
/// # Safety
/// TODO
BeginMode2D

/// Ends 2D mode with custom camera
/// # Safety
/// TODO
EndMode2D

/// Begin 3D mode with custom camera (3D)
/// # Safety
/// TODO
BeginMode3D

/// Ends 3D mode and returns to default 2D orthographic mode
/// # Safety
/// TODO
EndMode3D

/// Begin drawing to render texture
/// # Safety
/// TODO
BeginTextureMode

/// Ends drawing to render texture
/// # Safety
/// TODO
EndTextureMode

/// Begin custom shader drawing
/// # Safety
/// TODO
BeginShaderMode

/// End custom shader drawing (use default shader)
/// # Safety
/// TODO
EndShaderMode

/// Begin blending mode (alpha, additive, multiplied, subtract, custom)
/// # Safety
/// TODO
BeginBlendMode

/// End blending mode (reset to default: alpha blending)
/// # Safety
/// TODO
EndBlendMode

/// Begin scissor mode (define screen area for following drawing)
/// # Safety
/// TODO
BeginScissorMode

/// End scissor mode
/// # Safety
/// TODO
EndScissorMode

/// Begin stereo rendering (requires VR simulator)
/// # Safety
/// TODO
BeginVrStereoMode

/// End stereo rendering (requires VR simulator)
/// # Safety
/// TODO
EndVrStereoMode

// VR stereo config functions for VR simulator

/// Load VR stereo config for VR simulator device parameters
/// # Safety
/// TODO
LoadVrStereoConfig

/// Unload VR stereo config
/// # Safety
/// TODO
UnloadVrStereoConfig

// Shader management functions
// NOTE: Shader functionality is not available on OpenGL 1.1

/// Load shader from files and bind default locations
/// # Safety
/// TODO
LoadShader

/// Load shader from code strings and bind default locations
/// # Safety
/// TODO
LoadShaderFromMemory

/// Check if a shader is valid (loaded on GPU)
/// # Safety
/// TODO
IsShaderValid

/// Get shader uniform location
/// # Safety
/// TODO
GetShaderLocation

/// Get shader attribute location
/// # Safety
/// TODO
GetShaderLocationAttrib

/// Set shader uniform value
/// # Safety
/// TODO
SetShaderValue

/// Set shader uniform value vector
/// # Safety
/// TODO
SetShaderValueV

/// Set shader uniform value (matrix 4x4)
/// # Safety
/// TODO
SetShaderValueMatrix

/// Set shader uniform value for texture (sampler2d)
/// # Safety
/// TODO
SetShaderValueTexture

/// Unload shader from GPU memory (VRAM)
/// # Safety
/// TODO
UnloadShader

// Screen-space-related functions

/// Get a ray trace from screen position (i.e mouse)
/// # Safety
/// TODO
GetScreenToWorldRay

/// Get a ray trace from screen position (i.e mouse) in a viewport
/// # Safety
/// TODO
GetScreenToWorldRayEx

/// Get the screen space position for a 3d world space position
/// # Safety
/// TODO
GetWorldToScreen

/// Get size position for a 3d world space position
/// # Safety
/// TODO
GetWorldToScreenEx

/// Get the screen space position for a 2d camera world space position
/// # Safety
/// TODO
GetWorldToScreen2D

/// Get the world space position for a 2d camera screen space position
/// # Safety
/// TODO
GetScreenToWorld2D

/// Get camera transform matrix (view matrix)
/// # Safety
/// TODO
GetCameraMatrix

/// Get camera 2d transform matrix
/// # Safety
/// TODO
GetCameraMatrix2D

// Timing-related functions

/// Set target FPS (maximum)
/// # Safety
/// TODO
SetTargetFPS

/// Get time in seconds for last frame drawn (delta time)
/// # Safety
/// TODO
GetFrameTime

/// Get elapsed time in seconds since InitWindow()
/// # Safety
/// TODO
GetTime

/// Get current FPS
/// # Safety
/// TODO
GetFPS

// Custom frame control functions
// NOTE: Those functions are intended for advanced users that want full control over the frame processing
// By default EndDrawing() does this job: draws everything + SwapScreenBuffer() + manage frame timing + PollInputEvents()
// To avoid that behaviour and control frame processes manually, enable in config.h: SUPPORT_CUSTOM_FRAME_CONTROL

/// Swap back buffer with front buffer (screen drawing)
/// # Safety
/// TODO
SwapScreenBuffer

/// Register all input events
/// # Safety
/// TODO
PollInputEvents

/// Wait for some time (halt program execution)
/// # Safety
/// TODO
WaitTime

// Random values generation functions

/// Set the seed for the random number generator
/// # Safety
/// TODO
SetRandomSeed

/// Get a random value between min and max (both included)
/// # Safety
/// TODO
GetRandomValue

/// Load random values sequence, no values repeated
/// # Safety
/// TODO
LoadRandomSequence

/// Unload random values sequence
/// # Safety
/// TODO
UnloadRandomSequence

// Misc. functions

/// Takes a screenshot of current screen (filename extension defines format)
/// # Safety
/// TODO
TakeScreenshot

/// Setup init configuration flags (view FLAGS)
/// # Safety
/// TODO
SetConfigFlags

/// Open URL with default system browser (if available)
/// # Safety
/// TODO
OpenURL

// NOTE: Following functions implemented in module [utils]
//------------------------------------------------------------------

/// Show trace log messages (LOG_DEBUG, LOG_INFO, LOG_WARNING, LOG_ERROR...)
/// # Safety
/// TODO
TraceLog

/// Set the current threshold (minimum) log level
/// # Safety
/// TODO
SetTraceLogLevel

/// Internal memory allocator
/// # Safety
/// TODO
MemAlloc

/// Internal memory reallocator
/// # Safety
/// TODO
MemRealloc

/// Internal memory free
/// # Safety
/// TODO
MemFree

// Set custom callbacks
// WARNING: Callbacks setup is intended for advanced users

/// Set custom trace log
/// # Safety
/// TODO
SetTraceLogCallback

/// Set custom file binary data loader
/// # Safety
/// TODO
SetLoadFileDataCallback

/// Set custom file binary data saver
/// # Safety
/// TODO
SetSaveFileDataCallback

/// Set custom file text data loader
/// # Safety
/// TODO
SetLoadFileTextCallback

/// Set custom file text data saver
/// # Safety
/// TODO
SetSaveFileTextCallback

// Files management functions

/// Load file data as byte array (read)
/// # Safety
/// TODO
LoadFileData

/// Unload file data allocated by LoadFileData()
/// # Safety
/// TODO
UnloadFileData

/// Save data to file from byte array (write), returns true on success
/// # Safety
/// TODO
SaveFileData

/// Export data to code (.h), returns true on success
/// # Safety
/// TODO
ExportDataAsCode

/// Load text data from file (read), returns a '\0' terminated string
/// # Safety
/// TODO
LoadFileText

/// Unload file text data allocated by LoadFileText()
/// # Safety
/// TODO
UnloadFileText

/// Save text data to file (write), string must be '\0' terminated, returns true on success
/// # Safety
/// TODO
SaveFileText

//------------------------------------------------------------------

// File system functions

/// Check if file exists
/// # Safety
/// TODO
FileExists

/// Check if a directory path exists
/// # Safety
/// TODO
DirectoryExists

/// Check file extension (including point: .png, .wav)
/// # Safety
/// TODO
IsFileExtension

/// Get file length in bytes (NOTE: GetFileSize() conflicts with windows.h)
/// # Safety
/// TODO
GetFileLength

/// Get pointer to extension for a filename string (includes dot: '.png')
/// # Safety
/// TODO
GetFileExtension

/// Get pointer to filename for a path string
/// # Safety
/// TODO
GetFileName

/// Get filename string without extension (uses static string)
/// # Safety
/// TODO
GetFileNameWithoutExt

/// Get full path for a given fileName with path (uses static string)
/// # Safety
/// TODO
GetDirectoryPath

/// Get previous directory path for a given path (uses static string)
/// # Safety
/// TODO
GetPrevDirectoryPath

/// Get current working directory (uses static string)
/// # Safety
/// TODO
GetWorkingDirectory

/// Get the directory of the running application (uses static string)
/// # Safety
/// TODO
GetApplicationDirectory

/// Create directories (including full path requested), returns 0 on success
/// # Safety
/// TODO
MakeDirectory

/// Change working directory, return true on success
/// # Safety
/// TODO
ChangeDirectory

/// Check if a given path is a file or a directory
/// # Safety
/// TODO
IsPathFile

/// Check if fileName is valid for the platform/OS
/// # Safety
/// TODO
IsFileNameValid

/// Load directory filepaths
/// # Safety
/// TODO
LoadDirectoryFiles

/// Load directory filepaths with extension filtering and recursive directory scan. Use 'DIR' in the filter string to include directories in the result
/// # Safety
/// TODO
LoadDirectoryFilesEx

/// Unload filepaths
/// # Safety
/// TODO
UnloadDirectoryFiles

/// Check if a file has been dropped into window
/// # Safety
/// TODO
IsFileDropped

/// Load dropped filepaths
/// # Safety
/// TODO
LoadDroppedFiles

/// Unload dropped filepaths
/// # Safety
/// TODO
UnloadDroppedFiles

/// Get file modification time (last write time)
/// # Safety
/// TODO
GetFileModTime

// Compression/Encoding functionality

/// Compress data (DEFLATE algorithm), memory must be MemFree()
/// # Safety
/// TODO
CompressData

/// Decompress data (DEFLATE algorithm), memory must be MemFree()
/// # Safety
/// TODO
DecompressData

/// Encode data to Base64 string, memory must be MemFree()
/// # Safety
/// TODO
EncodeDataBase64

/// Decode Base64 string data, memory must be MemFree()
/// # Safety
/// TODO
DecodeDataBase64

/// Compute CRC32 hash code
/// # Safety
/// TODO
ComputeCRC32

/// Compute MD5 hash code, returns static int[4] (16 bytes)
/// # Safety
/// TODO
ComputeMD5

/// Compute SHA1 hash code, returns static int[5] (20 bytes)
/// # Safety
/// TODO
ComputeSHA1


// Automation events functionality

/// Load automation events list from file, NULL for empty list, capacity = MAX_AUTOMATION_EVENTS
/// # Safety
/// TODO
LoadAutomationEventList

/// Unload automation events list from file
/// # Safety
/// TODO
UnloadAutomationEventList

/// Export automation events list as text file
/// # Safety
/// TODO
ExportAutomationEventList

/// Set automation event list to record to
/// # Safety
/// TODO
SetAutomationEventList

/// Set automation event internal base frame to start recording
/// # Safety
/// TODO
SetAutomationEventBaseFrame

/// Start recording automation events (AutomationEventList must be set)
/// # Safety
/// TODO
StartAutomationEventRecording

/// Stop recording automation events
/// # Safety
/// TODO
StopAutomationEventRecording

/// Play a recorded automation event
/// # Safety
/// TODO
PlayAutomationEvent

//------------------------------------------------------------------------------------
// Input Handling Functions (Module: core)
//------------------------------------------------------------------------------------

// Input-related functions: keyboard

/// Check if a key has been pressed once
/// # Safety
/// TODO
IsKeyPressed

/// Check if a key has been pressed again
/// # Safety
/// TODO
IsKeyPressedRepeat

/// Check if a key is being pressed
/// # Safety
/// TODO
IsKeyDown

/// Check if a key has been released once
/// # Safety
/// TODO
IsKeyReleased

/// Check if a key is NOT being pressed
/// # Safety
/// TODO
IsKeyUp

/// Get key pressed (keycode), call it multiple times for keys queued, returns 0 when the queue is empty
/// # Safety
/// TODO
GetKeyPressed

/// Get char pressed (unicode), call it multiple times for chars queued, returns 0 when the queue is empty
/// # Safety
/// TODO
GetCharPressed

/// Get name of a QWERTY key on the current keyboard layout (eg returns string 'q' for KEY_A on an AZERTY keyboard)
/// # Safety
/// TODO
GetKeyName

/// Set a custom key to exit program (default is ESC)
/// # Safety
/// TODO
SetExitKey

// Input-related functions: gamepads

/// Check if a gamepad is available
/// # Safety
/// TODO
IsGamepadAvailable

/// Get gamepad internal name id
/// # Safety
/// TODO
GetGamepadName

/// Check if a gamepad button has been pressed once
/// # Safety
/// TODO
IsGamepadButtonPressed

/// Check if a gamepad button is being pressed
/// # Safety
/// TODO
IsGamepadButtonDown

/// Check if a gamepad button has been released once
/// # Safety
/// TODO
IsGamepadButtonReleased

/// Check if a gamepad button is NOT being pressed
/// # Safety
/// TODO
IsGamepadButtonUp

/// Get the last gamepad button pressed
/// # Safety
/// TODO
GetGamepadButtonPressed

/// Get gamepad axis count for a gamepad
/// # Safety
/// TODO
GetGamepadAxisCount

/// Get axis movement value for a gamepad axis
/// # Safety
/// TODO
GetGamepadAxisMovement

/// Set internal gamepad mappings (SDL_GameControllerDB)
/// # Safety
/// TODO
SetGamepadMappings

/// Set gamepad vibration for both motors (duration in seconds)
/// # Safety
/// TODO
SetGamepadVibration

// Input-related functions: mouse

/// Check if a mouse button has been pressed once
/// # Safety
/// TODO
IsMouseButtonPressed

/// Check if a mouse button is being pressed
/// # Safety
/// TODO
IsMouseButtonDown

/// Check if a mouse button has been released once
/// # Safety
/// TODO
IsMouseButtonReleased

/// Check if a mouse button is NOT being pressed
/// # Safety
/// TODO
IsMouseButtonUp

/// Get mouse position X
/// # Safety
/// TODO
GetMouseX

/// Get mouse position Y
/// # Safety
/// TODO
GetMouseY

/// Get mouse position XY
/// # Safety
/// TODO
GetMousePosition

/// Get mouse delta between frames
/// # Safety
/// TODO
GetMouseDelta

/// Set mouse position XY
/// # Safety
/// TODO
SetMousePosition

/// Set mouse offset
/// # Safety
/// TODO
SetMouseOffset

/// Set mouse scaling
/// # Safety
/// TODO
SetMouseScale

/// Get mouse wheel movement for X or Y, whichever is larger
/// # Safety
/// TODO
GetMouseWheelMove

/// Get mouse wheel movement for both X and Y
/// # Safety
/// TODO
GetMouseWheelMoveV

/// Set mouse cursor
/// # Safety
/// TODO
SetMouseCursor

// Input-related functions: touch

/// Get touch position X for touch point 0 (relative to screen size)
/// # Safety
/// TODO
GetTouchX

/// Get touch position Y for touch point 0 (relative to screen size)
/// # Safety
/// TODO
GetTouchY

/// Get touch position XY for a touch point index (relative to screen size)
/// # Safety
/// TODO
GetTouchPosition

/// Get touch point identifier for given index
/// # Safety
/// TODO
GetTouchPointId

/// Get number of touch points
/// # Safety
/// TODO
GetTouchPointCount

//------------------------------------------------------------------------------------
// Gestures and Touch Handling Functions (Module: rgestures)
//------------------------------------------------------------------------------------

/// Enable a set of gestures using flags
/// # Safety
/// TODO
SetGesturesEnabled

/// Check if a gesture have been detected
/// # Safety
/// TODO
IsGestureDetected

/// Get latest detected gesture
/// # Safety
/// TODO
GetGestureDetected

/// Get gesture hold time in seconds
/// # Safety
/// TODO
GetGestureHoldDuration

/// Get gesture drag vector
/// # Safety
/// TODO
GetGestureDragVector

/// Get gesture drag angle
/// # Safety
/// TODO
GetGestureDragAngle

/// Get gesture pinch delta
/// # Safety
/// TODO
GetGesturePinchVector

/// Get gesture pinch angle
/// # Safety
/// TODO
GetGesturePinchAngle

//------------------------------------------------------------------------------------
// Camera System Functions (Module: rcamera)
//------------------------------------------------------------------------------------

/// Update camera position for selected mode
/// # Safety
/// TODO
UpdateCamera

/// Update camera movement/rotation
/// # Safety
/// TODO
UpdateCameraPro

//------------------------------------------------------------------------------------
// Basic Shapes Drawing Functions (Module: shapes)
//------------------------------------------------------------------------------------
// Set texture and rectangle to be used on shapes drawing
// NOTE: It can be useful when using basic shapes and one single font,
// defining a font char white rectangle would allow drawing everything in a single draw call

/// Set texture and rectangle to be used on shapes drawing
/// # Safety
/// TODO
SetShapesTexture

/// Get texture that is used for shapes drawing
/// # Safety
/// TODO
GetShapesTexture

/// Get texture source rectangle that is used for shapes drawing
/// # Safety
/// TODO
GetShapesTextureRectangle

// Basic shapes drawing functions

/// Draw a pixel using geometry [Can be slow, use with care]
/// # Safety
/// TODO
DrawPixel

/// Draw a pixel using geometry (Vector version) [Can be slow, use with care]
/// # Safety
/// TODO
DrawPixelV

/// Draw a line
/// # Safety
/// TODO
DrawLine

/// Draw a line (using gl lines)
/// # Safety
/// TODO
DrawLineV

/// Draw a line (using triangles/quads)
/// # Safety
/// TODO
DrawLineEx

/// Draw lines sequence (using gl lines)
/// # Safety
/// TODO
DrawLineStrip

/// Draw line segment cubic-bezier in-out interpolation
/// # Safety
/// TODO
DrawLineBezier

/// Draw a color-filled circle
/// # Safety
/// TODO
DrawCircle

/// Draw a piece of a circle
/// # Safety
/// TODO
DrawCircleSector

/// Draw circle sector outline
/// # Safety
/// TODO
DrawCircleSectorLines

/// Draw a gradient-filled circle
/// # Safety
/// TODO
DrawCircleGradient

/// Draw a color-filled circle (Vector version)
/// # Safety
/// TODO
DrawCircleV

/// Draw circle outline
/// # Safety
/// TODO
DrawCircleLines

/// Draw circle outline (Vector version)
/// # Safety
/// TODO
DrawCircleLinesV

/// Draw ellipse
/// # Safety
/// TODO
DrawEllipse

/// Draw ellipse outline
/// # Safety
/// TODO
DrawEllipseLines

/// Draw ring
/// # Safety
/// TODO
DrawRing

/// Draw ring outline
/// # Safety
/// TODO
DrawRingLines

/// Draw a color-filled rectangle
/// # Safety
/// TODO
DrawRectangle

/// Draw a color-filled rectangle (Vector version)
/// # Safety
/// TODO
DrawRectangleV

/// Draw a color-filled rectangle
/// # Safety
/// TODO
DrawRectangleRec

/// Draw a color-filled rectangle with pro parameters
/// # Safety
/// TODO
DrawRectanglePro

/// Draw a vertical-gradient-filled rectangle
/// # Safety
/// TODO
DrawRectangleGradientV

/// Draw a horizontal-gradient-filled rectangle
/// # Safety
/// TODO
DrawRectangleGradientH

/// Draw a gradient-filled rectangle with custom vertex colors
/// # Safety
/// TODO
DrawRectangleGradientEx

/// Draw rectangle outline
/// # Safety
/// TODO
DrawRectangleLines

/// Draw rectangle outline with extended parameters
/// # Safety
/// TODO
DrawRectangleLinesEx

/// Draw rectangle with rounded edges
/// # Safety
/// TODO
DrawRectangleRounded

/// Draw rectangle lines with rounded edges
/// # Safety
/// TODO
DrawRectangleRoundedLines

/// Draw rectangle with rounded edges outline
/// # Safety
/// TODO
DrawRectangleRoundedLinesEx

/// Draw a color-filled triangle (vertex in counter-clockwise order!)
/// # Safety
/// TODO
DrawTriangle

/// Draw triangle outline (vertex in counter-clockwise order!)
/// # Safety
/// TODO
DrawTriangleLines

/// Draw a triangle fan defined by points (first vertex is the center)
/// # Safety
/// TODO
DrawTriangleFan

/// Draw a triangle strip defined by points
/// # Safety
/// TODO
DrawTriangleStrip

/// Draw a regular polygon (Vector version)
/// # Safety
/// TODO
DrawPoly

/// Draw a polygon outline of n sides
/// # Safety
/// TODO
DrawPolyLines

/// Draw a polygon outline of n sides with extended parameters
/// # Safety
/// TODO
DrawPolyLinesEx

// Splines drawing functions

/// Draw spline: Linear, minimum 2 points
/// # Safety
/// TODO
DrawSplineLinear

/// Draw spline: B-Spline, minimum 4 points
/// # Safety
/// TODO
DrawSplineBasis

/// Draw spline: Catmull-Rom, minimum 4 points
/// # Safety
/// TODO
DrawSplineCatmullRom

/// Draw spline: Quadratic Bezier, minimum 3 points (1 control point): [p1, c2, p3, c4...]
/// # Safety
/// TODO
DrawSplineBezierQuadratic

/// Draw spline: Cubic Bezier, minimum 4 points (2 control points): [p1, c2, c3, p4, c5, c6...]
/// # Safety
/// TODO
DrawSplineBezierCubic

/// Draw spline segment: Linear, 2 points
/// # Safety
/// TODO
DrawSplineSegmentLinear

/// Draw spline segment: B-Spline, 4 points
/// # Safety
/// TODO
DrawSplineSegmentBasis

/// Draw spline segment: Catmull-Rom, 4 points
/// # Safety
/// TODO
DrawSplineSegmentCatmullRom

/// Draw spline segment: Quadratic Bezier, 2 points, 1 control point
/// # Safety
/// TODO
DrawSplineSegmentBezierQuadratic

/// Draw spline segment: Cubic Bezier, 2 points, 2 control points
/// # Safety
/// TODO
DrawSplineSegmentBezierCubic

// Spline segment point evaluation functions, for a given t [0.0f .. 1.0f]

/// Get (evaluate) spline point: Linear
/// # Safety
/// TODO
GetSplinePointLinear

/// Get (evaluate) spline point: B-Spline
/// # Safety
/// TODO
GetSplinePointBasis

/// Get (evaluate) spline point: Catmull-Rom
/// # Safety
/// TODO
GetSplinePointCatmullRom

/// Get (evaluate) spline point: Quadratic Bezier
/// # Safety
/// TODO
GetSplinePointBezierQuad

/// Get (evaluate) spline point: Cubic Bezier
/// # Safety
/// TODO
GetSplinePointBezierCubic

// Basic shapes collision detection functions

/// Check collision between two rectangles
/// # Safety
/// TODO
CheckCollisionRecs

/// Check collision between two circles
/// # Safety
/// TODO
CheckCollisionCircles

/// Check collision between circle and rectangle
/// # Safety
/// TODO
CheckCollisionCircleRec

/// Check if circle collides with a line created betweeen two points [p1] and [p2]
/// # Safety
/// TODO
CheckCollisionCircleLine

/// Check if point is inside rectangle
/// # Safety
/// TODO
CheckCollisionPointRec

/// Check if point is inside circle
/// # Safety
/// TODO
CheckCollisionPointCircle

/// Check if point is inside a triangle
/// # Safety
/// TODO
CheckCollisionPointTriangle

/// Check if point belongs to line created between two points [p1] and [p2] with defined margin in pixels [threshold]
/// # Safety
/// TODO
CheckCollisionPointLine

/// Check if point is within a polygon described by array of vertices
/// # Safety
/// TODO
CheckCollisionPointPoly

/// Check the collision between two lines defined by two points each, returns collision point by reference
/// # Safety
/// TODO
CheckCollisionLines

/// Get collision rectangle for two rectangles collision
/// # Safety
/// TODO
GetCollisionRec

//------------------------------------------------------------------------------------
// Texture Loading and Drawing Functions (Module: textures)
//------------------------------------------------------------------------------------

// Image loading functions
// NOTE: These functions do not require GPU access

/// Load image from file into CPU memory (RAM)
/// # Safety
/// TODO
LoadImage

/// Load image from RAW file data
/// # Safety
/// TODO
LoadImageRaw

/// Load image sequence from file (frames appended to image.data)
/// # Safety
/// TODO
LoadImageAnim

/// Load image sequence from memory buffer
/// # Safety
/// TODO
LoadImageAnimFromMemory

/// Load image from memory buffer, fileType refers to extension: i.e. '.png'
/// # Safety
/// TODO
LoadImageFromMemory

/// Load image from GPU texture data
/// # Safety
/// TODO
LoadImageFromTexture

/// Load image from screen buffer and (screenshot)
/// # Safety
/// TODO
LoadImageFromScreen

/// Check if an image is valid (data and parameters)
/// # Safety
/// TODO
IsImageValid

/// Unload image from CPU memory (RAM)
/// # Safety
/// TODO
UnloadImage

/// Export image data to file, returns true on success
/// # Safety
/// TODO
ExportImage

/// Export image to memory buffer
/// # Safety
/// TODO
ExportImageToMemory

/// Export image as code file defining an array of bytes, returns true on success
/// # Safety
/// TODO
ExportImageAsCode

// Image generation functions

/// Generate image: plain color
/// # Safety
/// TODO
GenImageColor

/// Generate image: linear gradient, direction in degrees [0..360], 0=Vertical gradient
/// # Safety
/// TODO
GenImageGradientLinear

/// Generate image: radial gradient
/// # Safety
/// TODO
GenImageGradientRadial

/// Generate image: square gradient
/// # Safety
/// TODO
GenImageGradientSquare

/// Generate image: checked
/// # Safety
/// TODO
GenImageChecked

/// Generate image: white noise
/// # Safety
/// TODO
GenImageWhiteNoise

/// Generate image: perlin noise
/// # Safety
/// TODO
GenImagePerlinNoise

/// Generate image: cellular algorithm, bigger tileSize means bigger cells
/// # Safety
/// TODO
GenImageCellular

/// Generate image: grayscale image from text data
/// # Safety
/// TODO
GenImageText

// Image manipulation functions

/// Create an image duplicate (useful for transformations)
/// # Safety
/// TODO
ImageCopy

/// Create an image from another image piece
/// # Safety
/// TODO
ImageFromImage

/// Create an image from a selected channel of another image (GRAYSCALE)
/// # Safety
/// TODO
ImageFromChannel

/// Create an image from text (default font)
/// # Safety
/// TODO
ImageText

/// Create an image from text (custom sprite font)
/// # Safety
/// TODO
ImageTextEx

/// Convert image data to desired format
/// # Safety
/// TODO
ImageFormat

/// Convert image to POT (power-of-two)
/// # Safety
/// TODO
ImageToPOT

/// Crop an image to a defined rectangle
/// # Safety
/// TODO
ImageCrop

/// Crop image depending on alpha value
/// # Safety
/// TODO
ImageAlphaCrop

/// Clear alpha channel to desired color
/// # Safety
/// TODO
ImageAlphaClear

/// Apply alpha mask to image
/// # Safety
/// TODO
ImageAlphaMask

/// Premultiply alpha channel
/// # Safety
/// TODO
ImageAlphaPremultiply

/// Apply Gaussian blur using a box blur approximation
/// # Safety
/// TODO
ImageBlurGaussian

/// Apply custom square convolution kernel to image
/// # Safety
/// TODO
ImageKernelConvolution

/// Resize image (Bicubic scaling algorithm)
/// # Safety
/// TODO
ImageResize

/// Resize image (Nearest-Neighbor scaling algorithm)
/// # Safety
/// TODO
ImageResizeNN

/// Resize canvas and fill with color
/// # Safety
/// TODO
ImageResizeCanvas

/// Compute all mipmap levels for a provided image
/// # Safety
/// TODO
ImageMipmaps

/// Dither image data to 16bpp or lower (Floyd-Steinberg dithering)
/// # Safety
/// TODO
ImageDither

/// Flip image vertically
/// # Safety
/// TODO
ImageFlipVertical

/// Flip image horizontally
/// # Safety
/// TODO
ImageFlipHorizontal

/// Rotate image by input angle in degrees (-359 to 359)
/// # Safety
/// TODO
ImageRotate

/// Rotate image clockwise 90deg
/// # Safety
/// TODO
ImageRotateCW

/// Rotate image counter-clockwise 90deg
/// # Safety
/// TODO
ImageRotateCCW

/// Modify image color: tint
/// # Safety
/// TODO
ImageColorTint

/// Modify image color: invert
/// # Safety
/// TODO
ImageColorInvert

/// Modify image color: grayscale
/// # Safety
/// TODO
ImageColorGrayscale

/// Modify image color: contrast (-100 to 100)
/// # Safety
/// TODO
ImageColorContrast

/// Modify image color: brightness (-255 to 255)
/// # Safety
/// TODO
ImageColorBrightness

/// Modify image color: replace color
/// # Safety
/// TODO
ImageColorReplace

/// Load color data from image as a Color array (RGBA - 32bit)
/// # Safety
/// TODO
LoadImageColors

/// Load colors palette from image as a Color array (RGBA - 32bit)
/// # Safety
/// TODO
LoadImagePalette

/// Unload color data loaded with LoadImageColors()
/// # Safety
/// TODO
UnloadImageColors

/// Unload colors palette loaded with LoadImagePalette()
/// # Safety
/// TODO
UnloadImagePalette

/// Get image alpha border rectangle
/// # Safety
/// TODO
GetImageAlphaBorder

/// Get image pixel color at (x, y) position
/// # Safety
/// TODO
GetImageColor

// Image drawing functions
// NOTE: Image software-rendering functions (CPU)

/// Clear image background with given color
/// # Safety
/// TODO
ImageClearBackground

/// Draw pixel within an image
/// # Safety
/// TODO
ImageDrawPixel

/// Draw pixel within an image (Vector version)
/// # Safety
/// TODO
ImageDrawPixelV

/// Draw line within an image
/// # Safety
/// TODO
ImageDrawLine

/// Draw line within an image (Vector version)
/// # Safety
/// TODO
ImageDrawLineV

/// Draw a line defining thickness within an image
/// # Safety
/// TODO
ImageDrawLineEx

/// Draw a filled circle within an image
/// # Safety
/// TODO
ImageDrawCircle

/// Draw a filled circle within an image (Vector version)
/// # Safety
/// TODO
ImageDrawCircleV

/// Draw circle outline within an image
/// # Safety
/// TODO
ImageDrawCircleLines

/// Draw circle outline within an image (Vector version)
/// # Safety
/// TODO
ImageDrawCircleLinesV

/// Draw rectangle within an image
/// # Safety
/// TODO
ImageDrawRectangle

/// Draw rectangle within an image (Vector version)
/// # Safety
/// TODO
ImageDrawRectangleV

/// Draw rectangle within an image
/// # Safety
/// TODO
ImageDrawRectangleRec

/// Draw rectangle lines within an image
/// # Safety
/// TODO
ImageDrawRectangleLines

/// Draw triangle within an image
/// # Safety
/// TODO
ImageDrawTriangle

/// Draw triangle with interpolated colors within an image
/// # Safety
/// TODO
ImageDrawTriangleEx

/// Draw triangle outline within an image
/// # Safety
/// TODO
ImageDrawTriangleLines

/// Draw a triangle fan defined by points within an image (first vertex is the center)
/// # Safety
/// TODO
ImageDrawTriangleFan

/// Draw a triangle strip defined by points within an image
/// # Safety
/// TODO
ImageDrawTriangleStrip

/// Draw a source image within a destination image (tint applied to source)
/// # Safety
/// TODO
ImageDraw

/// Draw text (using default font) within an image (destination)
/// # Safety
/// TODO
ImageDrawText

/// Draw text (custom sprite font) within an image (destination)
/// # Safety
/// TODO
ImageDrawTextEx

// Texture loading functions
// NOTE: These functions require GPU access

/// Load texture from file into GPU memory (VRAM)
/// # Safety
/// TODO
LoadTexture

/// Load texture from image data
/// # Safety
/// TODO
LoadTextureFromImage

/// Load cubemap from image, multiple image cubemap layouts supported
/// # Safety
/// TODO
LoadTextureCubemap

/// Load texture for rendering (framebuffer)
/// # Safety
/// TODO
LoadRenderTexture

/// Check if a texture is valid (loaded in GPU)
/// # Safety
/// TODO
IsTextureValid

/// Unload texture from GPU memory (VRAM)
/// # Safety
/// TODO
UnloadTexture

/// Check if a render texture is valid (loaded in GPU)
/// # Safety
/// TODO
IsRenderTextureValid

/// Unload render texture from GPU memory (VRAM)
/// # Safety
/// TODO
UnloadRenderTexture

/// Update GPU texture with new data
/// # Safety
/// TODO
UpdateTexture

/// Update GPU texture rectangle with new data
/// # Safety
/// TODO
UpdateTextureRec

// Texture configuration functions

/// Generate GPU mipmaps for a texture
/// # Safety
/// TODO
GenTextureMipmaps

/// Set texture scaling filter mode
/// # Safety
/// TODO
SetTextureFilter

/// Set texture wrapping mode
/// # Safety
/// TODO
SetTextureWrap

// Texture drawing functions

/// Draw a Texture2D
/// # Safety
/// TODO
DrawTexture

/// Draw a Texture2D with position defined as Vector2
/// # Safety
/// TODO
DrawTextureV

/// Draw a Texture2D with extended parameters
/// # Safety
/// TODO
DrawTextureEx

/// Draw a part of a texture defined by a rectangle
/// # Safety
/// TODO
DrawTextureRec

/// Draw a part of a texture defined by a rectangle with 'pro' parameters
/// # Safety
/// TODO
DrawTexturePro

/// Draws a texture (or part of it) that stretches or shrinks nicely
/// # Safety
/// TODO
DrawTextureNPatch

// Color/pixel related functions

/// Check if two colors are equal
/// # Safety
/// TODO
ColorIsEqual

/// Get color with alpha applied, alpha goes from 0.0f to 1.0f
/// # Safety
/// TODO
Fade

/// Get hexadecimal value for a Color (0xRRGGBBAA)
/// # Safety
/// TODO
ColorToInt

/// Get Color normalized as float [0..1]
/// # Safety
/// TODO
ColorNormalize

/// Get Color from normalized values [0..1]
/// # Safety
/// TODO
ColorFromNormalized

/// Get HSV values for a Color, hue [0..360], saturation/value [0..1]
/// # Safety
/// TODO
ColorToHSV

/// Get a Color from HSV values, hue [0..360], saturation/value [0..1]
/// # Safety
/// TODO
ColorFromHSV

/// Get color multiplied with another color
/// # Safety
/// TODO
ColorTint

/// Get color with brightness correction, brightness factor goes from -1.0f to 1.0f
/// # Safety
/// TODO
ColorBrightness

/// Get color with contrast correction, contrast values between -1.0f and 1.0f
/// # Safety
/// TODO
ColorContrast

/// Get color with alpha applied, alpha goes from 0.0f to 1.0f
/// # Safety
/// TODO
ColorAlpha

/// Get src alpha-blended into dst color with tint
/// # Safety
/// TODO
ColorAlphaBlend

/// Get color lerp interpolation between two colors, factor [0.0f..1.0f]
/// # Safety
/// TODO
ColorLerp

/// Get Color structure from hexadecimal value
/// # Safety
/// TODO
GetColor

/// Get Color from a source pixel pointer of certain format
/// # Safety
/// TODO
GetPixelColor

/// Set color formatted into destination pixel pointer
/// # Safety
/// TODO
SetPixelColor

/// Get pixel data size in bytes for certain format
/// # Safety
/// TODO
GetPixelDataSize

//------------------------------------------------------------------------------------
// Font Loading and Text Drawing Functions (Module: text)
//------------------------------------------------------------------------------------

// Font loading/unloading functions

/// Get the default Font
/// # Safety
/// TODO
GetFontDefault

/// Load font from file into GPU memory (VRAM)
/// # Safety
/// TODO
LoadFont

/// Load font from file with extended parameters, use NULL for codepoints and 0 for codepointCount to load the default character set, font size is provided in pixels height
/// # Safety
/// TODO
LoadFontEx

/// Load font from Image (XNA style)
/// # Safety
/// TODO
LoadFontFromImage

/// Load font from memory buffer, fileType refers to extension: i.e. '.ttf'
/// # Safety
/// TODO
LoadFontFromMemory

/// Check if a font is valid (font data loaded, WARNING: GPU texture not checked)
/// # Safety
/// TODO
IsFontValid

/// Load font data for further use
/// # Safety
/// TODO
LoadFontData

/// Generate image font atlas using chars info
/// # Safety
/// TODO
GenImageFontAtlas

/// Unload font chars info data (RAM)
/// # Safety
/// TODO
UnloadFontData

/// Unload font from GPU memory (VRAM)
/// # Safety
/// TODO
UnloadFont

/// Export font as code file, returns true on success
/// # Safety
/// TODO
ExportFontAsCode

// Text drawing functions

/// Draw current FPS
/// # Safety
/// TODO
DrawFPS

/// Draw text (using default font)
/// # Safety
/// TODO
DrawText

/// Draw text using font and additional parameters
/// # Safety
/// TODO
DrawTextEx

/// Draw text using Font and pro parameters (rotation)
/// # Safety
/// TODO
DrawTextPro

/// Draw one character (codepoint)
/// # Safety
/// TODO
DrawTextCodepoint

/// Draw multiple character (codepoint)
/// # Safety
/// TODO
DrawTextCodepoints

// Text font info functions

/// Set vertical line spacing when drawing with line-breaks
/// # Safety
/// TODO
SetTextLineSpacing

/// Measure string width for default font
/// # Safety
/// TODO
MeasureText

/// Measure string size for Font
/// # Safety
/// TODO
MeasureTextEx

/// Get glyph index position in font for a codepoint (unicode character), fallback to '?' if not found
/// # Safety
/// TODO
GetGlyphIndex

/// Get glyph font info data for a codepoint (unicode character), fallback to '?' if not found
/// # Safety
/// TODO
GetGlyphInfo

/// Get glyph rectangle in font atlas for a codepoint (unicode character), fallback to '?' if not found
/// # Safety
/// TODO
GetGlyphAtlasRec

// Text codepoints management functions (unicode characters)

/// Load UTF-8 text encoded from codepoints array
/// # Safety
/// TODO
LoadUTF8

/// Unload UTF-8 text encoded from codepoints array
/// # Safety
/// TODO
UnloadUTF8

/// Load all codepoints from a UTF-8 text string, codepoints count returned by parameter
/// # Safety
/// TODO
LoadCodepoints

/// Unload codepoints data from memory
/// # Safety
/// TODO
UnloadCodepoints

/// Get total number of codepoints in a UTF-8 encoded string
/// # Safety
/// TODO
GetCodepointCount

/// Get next codepoint in a UTF-8 encoded string, 0x3f('?') is returned on failure
/// # Safety
/// TODO
GetCodepoint

/// Get next codepoint in a UTF-8 encoded string, 0x3f('?') is returned on failure
/// # Safety
/// TODO
GetCodepointNext

/// Get previous codepoint in a UTF-8 encoded string, 0x3f('?') is returned on failure
/// # Safety
/// TODO
GetCodepointPrevious

/// Encode one codepoint into UTF-8 byte array (array length returned as parameter)
/// # Safety
/// TODO
CodepointToUTF8

// Text strings management functions (no UTF-8 strings, only byte chars)
// NOTE: Some strings allocate memory internally for returned strings, just be careful!

/// Copy one string to another, returns bytes copied
/// # Safety
/// TODO
TextCopy

/// Check if two text string are equal
/// # Safety
/// TODO
TextIsEqual

/// Get text length, checks for '\0' ending
/// # Safety
/// TODO
TextLength

/// Text formatting with variables (sprintf() style)
/// # Safety
/// TODO
TextFormat

/// Get a piece of a text string
/// # Safety
/// TODO
TextSubtext

/// Replace text string (WARNING: memory must be freed!)
/// # Safety
/// TODO
TextReplace

/// Insert text in a position (WARNING: memory must be freed!)
/// # Safety
/// TODO
TextInsert

/// Join text strings with delimiter
/// # Safety
/// TODO
TextJoin

/// Split text into multiple strings
/// # Safety
/// TODO
TextSplit

/// Append text at specific position and move cursor!
/// # Safety
/// TODO
TextAppend

/// Find first text occurrence within a string
/// # Safety
/// TODO
TextFindIndex

/// Get upper case version of provided string
/// # Safety
/// TODO
TextToUpper

/// Get lower case version of provided string
/// # Safety
/// TODO
TextToLower

/// Get Pascal case notation version of provided string
/// # Safety
/// TODO
TextToPascal

/// Get Snake case notation version of provided string
/// # Safety
/// TODO
TextToSnake

/// Get Camel case notation version of provided string
/// # Safety
/// TODO
TextToCamel

/// Get integer value from text (negative values not supported)
/// # Safety
/// TODO
TextToInteger

/// Get float value from text (negative values not supported)
/// # Safety
/// TODO
TextToFloat

//------------------------------------------------------------------------------------
// Basic 3d Shapes Drawing Functions (Module: models)
//------------------------------------------------------------------------------------

// Basic geometric 3D shapes drawing functions

/// Draw a line in 3D world space
/// # Safety
/// TODO
DrawLine3D

/// Draw a point in 3D space, actually a small line
/// # Safety
/// TODO
DrawPoint3D

/// Draw a circle in 3D world space
/// # Safety
/// TODO
DrawCircle3D

/// Draw a color-filled triangle (vertex in counter-clockwise order!)
/// # Safety
/// TODO
DrawTriangle3D

/// Draw a triangle strip defined by points
/// # Safety
/// TODO
DrawTriangleStrip3D

/// Draw cube
/// # Safety
/// TODO
DrawCube

/// Draw cube (Vector version)
/// # Safety
/// TODO
DrawCubeV

/// Draw cube wires
/// # Safety
/// TODO
DrawCubeWires

/// Draw cube wires (Vector version)
/// # Safety
/// TODO
DrawCubeWiresV

/// Draw sphere
/// # Safety
/// TODO
DrawSphere

/// Draw sphere with extended parameters
/// # Safety
/// TODO
DrawSphereEx

/// Draw sphere wires
/// # Safety
/// TODO
DrawSphereWires

/// Draw a cylinder/cone
/// # Safety
/// TODO
DrawCylinder

/// Draw a cylinder with base at startPos and top at endPos
/// # Safety
/// TODO
DrawCylinderEx

/// Draw a cylinder/cone wires
/// # Safety
/// TODO
DrawCylinderWires

/// Draw a cylinder wires with base at startPos and top at endPos
/// # Safety
/// TODO
DrawCylinderWiresEx

/// Draw a capsule with the center of its sphere caps at startPos and endPos
/// # Safety
/// TODO
DrawCapsule

/// Draw capsule wireframe with the center of its sphere caps at startPos and endPos
/// # Safety
/// TODO
DrawCapsuleWires

/// Draw a plane XZ
/// # Safety
/// TODO
DrawPlane

/// Draw a ray line
/// # Safety
/// TODO
DrawRay

/// Draw a grid (centered at (0, 0, 0))
/// # Safety
/// TODO
DrawGrid

//------------------------------------------------------------------------------------
// Model 3d Loading and Drawing Functions (Module: models)
//------------------------------------------------------------------------------------

// Model management functions

/// Load model from files (meshes and materials)
/// # Safety
/// TODO
LoadModel

/// Load model from generated mesh (default material)
/// # Safety
/// TODO
LoadModelFromMesh

/// Check if a model is valid (loaded in GPU, VAO/VBOs)
/// # Safety
/// TODO
IsModelValid

/// Unload model (including meshes) from memory (RAM and/or VRAM)
/// # Safety
/// TODO
UnloadModel

/// Compute model bounding box limits (considers all meshes)
/// # Safety
/// TODO
GetModelBoundingBox

// Model drawing functions

/// Draw a model (with texture if set)
/// # Safety
/// TODO
DrawModel

/// Draw a model with extended parameters
/// # Safety
/// TODO
DrawModelEx

/// Draw a model wires (with texture if set)
/// # Safety
/// TODO
DrawModelWires

/// Draw a model wires (with texture if set) with extended parameters
/// # Safety
/// TODO
DrawModelWiresEx

/// Draw a model as points
/// # Safety
/// TODO
DrawModelPoints

/// Draw a model as points with extended parameters
/// # Safety
/// TODO
DrawModelPointsEx

/// Draw bounding box (wires)
/// # Safety
/// TODO
DrawBoundingBox

/// Draw a billboard texture
/// # Safety
/// TODO
DrawBillboard

/// Draw a billboard texture defined by source
/// # Safety
/// TODO
DrawBillboardRec

/// Draw a billboard texture defined by source and rotation
/// # Safety
/// TODO
DrawBillboardPro

// Mesh management functions

/// Upload mesh vertex data in GPU and provide VAO/VBO ids
/// # Safety
/// TODO
UploadMesh

/// Update mesh vertex data in GPU for a specific buffer index
/// # Safety
/// TODO
UpdateMeshBuffer

/// Unload mesh data from CPU and GPU
/// # Safety
/// TODO
UnloadMesh

/// Draw a 3d mesh with material and transform
/// # Safety
/// TODO
DrawMesh

/// Draw multiple mesh instances with material and different transforms
/// # Safety
/// TODO
DrawMeshInstanced

/// Compute mesh bounding box limits
/// # Safety
/// TODO
GetMeshBoundingBox

/// Compute mesh tangents
/// # Safety
/// TODO
GenMeshTangents

/// Export mesh data to file, returns true on success
/// # Safety
/// TODO
ExportMesh

/// Export mesh as code file (.h) defining multiple arrays of vertex attributes
/// # Safety
/// TODO
ExportMeshAsCode

// Mesh generation functions

/// Generate polygonal mesh
/// # Safety
/// TODO
GenMeshPoly

/// Generate plane mesh (with subdivisions)
/// # Safety
/// TODO
GenMeshPlane

/// Generate cuboid mesh
/// # Safety
/// TODO
GenMeshCube

/// Generate sphere mesh (standard sphere)
/// # Safety
/// TODO
GenMeshSphere

/// Generate half-sphere mesh (no bottom cap)
/// # Safety
/// TODO
GenMeshHemiSphere

/// Generate cylinder mesh
/// # Safety
/// TODO
GenMeshCylinder

/// Generate cone/pyramid mesh
/// # Safety
/// TODO
GenMeshCone

/// Generate torus mesh
/// # Safety
/// TODO
GenMeshTorus

/// Generate trefoil knot mesh
/// # Safety
/// TODO
GenMeshKnot

/// Generate heightmap mesh from image data
/// # Safety
/// TODO
GenMeshHeightmap

/// Generate cubes-based map mesh from image data
/// # Safety
/// TODO
GenMeshCubicmap

// Material loading/unloading functions

/// Load materials from model file
/// # Safety
/// TODO
LoadMaterials

/// Load default material (Supports: DIFFUSE, SPECULAR, NORMAL maps)
/// # Safety
/// TODO
LoadMaterialDefault

/// Check if a material is valid (shader assigned, map textures loaded in GPU)
/// # Safety
/// TODO
IsMaterialValid

/// Unload material from GPU memory (VRAM)
/// # Safety
/// TODO
UnloadMaterial

/// Set texture for a material map type (MATERIAL_MAP_DIFFUSE, MATERIAL_MAP_SPECULAR...)
/// # Safety
/// TODO
SetMaterialTexture

/// Set material for a mesh
/// # Safety
/// TODO
SetModelMeshMaterial

// Model animations loading/unloading functions

/// Load model animations from file
/// # Safety
/// TODO
LoadModelAnimations

/// Update model animation pose (CPU)
/// # Safety
/// TODO
UpdateModelAnimation

/// Update model animation mesh bone matrices (GPU skinning)
/// # Safety
/// TODO
UpdateModelAnimationBones

/// Unload animation data
/// # Safety
/// TODO
UnloadModelAnimation

/// Unload animation array data
/// # Safety
/// TODO
UnloadModelAnimations

/// Check model animation skeleton match
/// # Safety
/// TODO
IsModelAnimationValid

// Collision detection functions

/// Check collision between two spheres
/// # Safety
/// TODO
CheckCollisionSpheres

/// Check collision between two bounding boxes
/// # Safety
/// TODO
CheckCollisionBoxes

/// Check collision between box and sphere
/// # Safety
/// TODO
CheckCollisionBoxSphere

/// Get collision info between ray and sphere
/// # Safety
/// TODO
GetRayCollisionSphere

/// Get collision info between ray and box
/// # Safety
/// TODO
GetRayCollisionBox

/// Get collision info between ray and mesh
/// # Safety
/// TODO
GetRayCollisionMesh

/// Get collision info between ray and triangle
/// # Safety
/// TODO
GetRayCollisionTriangle

/// Get collision info between ray and quad
/// # Safety
/// TODO
GetRayCollisionQuad

//------------------------------------------------------------------------------------
// Audio Loading and Playing Functions (Module: audio)
//------------------------------------------------------------------------------------

// Audio device management functions

/// Initialize audio device and context
/// # Safety
/// TODO
InitAudioDevice

/// Close the audio device and context
/// # Safety
/// TODO
CloseAudioDevice

/// Check if audio device has been initialized successfully
/// # Safety
/// TODO
IsAudioDeviceReady

/// Set master volume (listener)
/// # Safety
/// TODO
SetMasterVolume

/// Get master volume (listener)
/// # Safety
/// TODO
GetMasterVolume

// Wave/Sound loading/unloading functions

/// Load wave data from file
/// # Safety
/// TODO
LoadWave

/// Load wave from memory buffer, fileType refers to extension: i.e. '.wav'
/// # Safety
/// TODO
LoadWaveFromMemory

/// Checks if wave data is valid (data loaded and parameters)
/// # Safety
/// TODO
IsWaveValid

/// Load sound from file
/// # Safety
/// TODO
LoadSound

/// Load sound from wave data
/// # Safety
/// TODO
LoadSoundFromWave

/// Create a new sound that shares the same sample data as the source sound, does not own the sound data
/// # Safety
/// TODO
LoadSoundAlias

/// Checks if a sound is valid (data loaded and buffers initialized)
/// # Safety
/// TODO
IsSoundValid

/// Update sound buffer with new data
/// # Safety
/// TODO
UpdateSound

/// Unload wave data
/// # Safety
/// TODO
UnloadWave

/// Unload sound
/// # Safety
/// TODO
UnloadSound

/// Unload a sound alias (does not deallocate sample data)
/// # Safety
/// TODO
UnloadSoundAlias

/// Export wave data to file, returns true on success
/// # Safety
/// TODO
ExportWave

/// Export wave sample data to code (.h), returns true on success
/// # Safety
/// TODO
ExportWaveAsCode

// Wave/Sound management functions

/// Play a sound
/// # Safety
/// TODO
PlaySound

/// Stop playing a sound
/// # Safety
/// TODO
StopSound

/// Pause a sound
/// # Safety
/// TODO
PauseSound

/// Resume a paused sound
/// # Safety
/// TODO
ResumeSound

/// Check if a sound is currently playing
/// # Safety
/// TODO
IsSoundPlaying

/// Set volume for a sound (1.0 is max level)
/// # Safety
/// TODO
SetSoundVolume

/// Set pitch for a sound (1.0 is base level)
/// # Safety
/// TODO
SetSoundPitch

/// Set pan for a sound (0.5 is center)
/// # Safety
/// TODO
SetSoundPan

/// Copy a wave to a new wave
/// # Safety
/// TODO
WaveCopy

/// Crop a wave to defined frames range
/// # Safety
/// TODO
WaveCrop

/// Convert wave data to desired format
/// # Safety
/// TODO
WaveFormat

/// Load samples data from wave as a 32bit float data array
/// # Safety
/// TODO
LoadWaveSamples

/// Unload samples data loaded with LoadWaveSamples()
/// # Safety
/// TODO
UnloadWaveSamples

// Music management functions

/// Load music stream from file
/// # Safety
/// TODO
LoadMusicStream

/// Load music stream from data
/// # Safety
/// TODO
LoadMusicStreamFromMemory

/// Checks if a music stream is valid (context and buffers initialized)
/// # Safety
/// TODO
IsMusicValid

/// Unload music stream
/// # Safety
/// TODO
UnloadMusicStream

/// Start music playing
/// # Safety
/// TODO
PlayMusicStream

/// Check if music is playing
/// # Safety
/// TODO
IsMusicStreamPlaying

/// Updates buffers for music streaming
/// # Safety
/// TODO
UpdateMusicStream

/// Stop music playing
/// # Safety
/// TODO
StopMusicStream

/// Pause music playing
/// # Safety
/// TODO
PauseMusicStream

/// Resume playing paused music
/// # Safety
/// TODO
ResumeMusicStream

/// Seek music to a position (in seconds)
/// # Safety
/// TODO
SeekMusicStream

/// Set volume for music (1.0 is max level)
/// # Safety
/// TODO
SetMusicVolume

/// Set pitch for a music (1.0 is base level)
/// # Safety
/// TODO
SetMusicPitch

/// Set pan for a music (0.5 is center)
/// # Safety
/// TODO
SetMusicPan

/// Get music time length (in seconds)
/// # Safety
/// TODO
GetMusicTimeLength

/// Get current music time played (in seconds)
/// # Safety
/// TODO
GetMusicTimePlayed

// AudioStream management functions

/// Load audio stream (to stream raw audio pcm data)
/// # Safety
/// TODO
LoadAudioStream

/// Checks if an audio stream is valid (buffers initialized)
/// # Safety
/// TODO
IsAudioStreamValid

/// Unload audio stream and free memory
/// # Safety
/// TODO
UnloadAudioStream

/// Update audio stream buffers with data
/// # Safety
/// TODO
UpdateAudioStream

/// Check if any audio stream buffers requires refill
/// # Safety
/// TODO
IsAudioStreamProcessed

/// Play audio stream
/// # Safety
/// TODO
PlayAudioStream

/// Pause audio stream
/// # Safety
/// TODO
PauseAudioStream

/// Resume audio stream
/// # Safety
/// TODO
ResumeAudioStream

/// Check if audio stream is playing
/// # Safety
/// TODO
IsAudioStreamPlaying

/// Stop audio stream
/// # Safety
/// TODO
StopAudioStream

/// Set volume for audio stream (1.0 is max level)
/// # Safety
/// TODO
SetAudioStreamVolume

/// Set pitch for audio stream (1.0 is base level)
/// # Safety
/// TODO
SetAudioStreamPitch

/// Set pan for audio stream (0.5 is centered)
/// # Safety
/// TODO
SetAudioStreamPan

/// Default size for new audio streams
/// # Safety
/// TODO
SetAudioStreamBufferSizeDefault

/// Audio thread callback to request new data
/// # Safety
/// TODO
SetAudioStreamCallback

/// Attach audio stream processor to stream, receives the samples as 'float'
/// # Safety
/// TODO
AttachAudioStreamProcessor

/// Detach audio stream processor from stream
/// # Safety
/// TODO
DetachAudioStreamProcessor

/// Attach audio stream processor to the entire audio pipeline, receives the samples as 'float'
/// # Safety
/// TODO
AttachAudioMixedProcessor

/// Detach audio stream processor from the entire audio pipeline
/// # Safety
/// TODO
DetachAudioMixedProcessor

}

}