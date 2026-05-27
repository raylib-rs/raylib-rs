//! Safe raygui bindings (behind the `raygui` feature). Controls are grouped into
//! sub-traits, all blanket-implemented for the draw-handle types and re-exported
//! here; global-state controls are also implemented for `RaylibHandle`.

mod advanced;
mod containers;
mod controls;
mod icons;
mod scratch;
mod state;

pub use advanced::RaylibGuiAdvanced;
pub use containers::RaylibGuiContainers;
pub use controls::RaylibGuiControls;
pub use icons::RaylibGuiIcons;
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
