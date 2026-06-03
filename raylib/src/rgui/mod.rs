//! Safe raygui bindings (behind the `raygui` feature).
//!
//! raygui is the immediate-mode GUI library bundled with raylib. raylib-rs
//! exposes it at **6.0 parity** (57 functions). Controls are split into grouped
//! sub-traits — [`RaylibGuiState`], [`RaylibGuiContainers`],
//! [`RaylibGuiControls`], [`RaylibGuiAdvanced`], [`RaylibGuiIcons`] — all
//! blanket-implemented for the draw-handle types and re-exported here.
//! Global-state controls are also implemented for [`RaylibHandle`].
//!
//! String arguments accept any `impl AsRef<str>`; the bindings convert to
//! `CStr` via a thread-local scratch buffer to avoid per-call heap
//! allocations during drawing.
//!
//! All raygui calls must happen inside a `begin_drawing` frame: call them on
//! the draw handle returned by [`RaylibHandle::begin_drawing`]. There is no
//! separate gui begin/end; every widget call draws immediately.

mod advanced;
mod containers;
mod controls;
mod icons;
mod scratch;
mod state;

pub use advanced::RaylibGuiAdvanced;
pub use containers::RaylibGuiContainers;
pub use controls::RaylibGuiControls;
pub use icons::{RAYGUI_ICON_DATA_ELEMENTS, RAYGUI_ICON_MAX_ICONS, RaylibGuiIcons};
pub use state::{GuiProperty, RaylibGuiState};

use crate::core::RaylibHandle;
use crate::core::drawing::RaylibDraw;

/// Umbrella trait covering every raygui control group. Retained for source
/// compatibility — `use raylib::prelude::*` brings all gui methods into scope.
pub trait RaylibDrawGui:
    RaylibGuiState + RaylibGuiContainers + RaylibGuiControls + RaylibGuiAdvanced + RaylibGuiIcons
{
}

// Draw handles (during drawing) get every gui group.
impl<D: RaylibDraw> RaylibGuiState for D {}
impl<D: RaylibDraw> RaylibGuiContainers for D {}
impl<D: RaylibDraw> RaylibGuiControls for D {}
impl<D: RaylibDraw> RaylibGuiAdvanced for D {}
impl<D: RaylibDraw> RaylibGuiIcons for D {}
impl<D: RaylibDraw> RaylibDrawGui for D {}

// RaylibHandle (during setup) gets the global-state group only. (RaylibHandle is
// not RaylibDraw, so this does not conflict with the blanket impl above.)
impl RaylibGuiState for RaylibHandle {}
impl RaylibGuiIcons for RaylibHandle {}
