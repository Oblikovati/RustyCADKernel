//! Martensite view gestures retain OCCT's public enum values and event records.
use super::AisSelectionScheme;
use crate::v3d::V3dTypeOfOrientation;
use rustycad_tkernel::ncollection::{Sequence, Vec2};
/// Selection gesture; `AisViewSelectionTool::RubberBand` uses a rectangle.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AisViewSelectionTool {
    /// Point picking.
    #[default]
    Picking = 0,
    /// Rectangular selection.
    RubberBand = 1,
    /// Polygon selection.
    Polygon = 2,
    /// Window zoom without selection.
    ZoomWindow = 3,
}
/// Upstream UI/render buffer role; `AisViewInputBufferType::Gl` names the rendering thread.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AisViewInputBufferType {
    /// Filled by the user interface thread.
    #[default]
    Ui = 0,
    /// Consumed by the rendering thread (the upstream API name is GL).
    Gl = 1,
}
/// Pending fit/orientation requests; `AisViewInputOrientation::default()` starts at Xpos.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AisViewInputOrientation {
    /// Request FitAll.
    pub to_fit_all: bool,
    /// Request a predefined orientation.
    pub to_set_view_orient: bool,
    /// Retained requested orientation.
    pub view_orient: V3dTypeOfOrientation,
}
impl Default for AisViewInputOrientation {
    fn default() -> Self {
        Self {
            to_fit_all: false,
            to_set_view_orient: false,
            view_orient: V3dTypeOfOrientation::Xpos,
        }
    }
}
/// Dynamic highlight request; `AisViewInputHighlighting::default()` has no pending request.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AisViewInputHighlighting {
    /// Request dynamic highlighting.
    pub to_hilight: bool,
    /// Retained highlight pixel.
    pub point: Vec2<i32>,
}
/// Pending selection tool and persistent points; `AisViewInputSelection::default()` has an unknown scheme.
#[derive(Clone)]
pub struct AisViewInputSelection {
    /// Current gesture tool.
    pub tool: AisViewSelectionTool,
    /// How the gesture affects selection.
    pub scheme: AisSelectionScheme,
    /// One-based retained polygon or rectangle coordinates.
    pub points: Sequence<Vec2<i32>>,
    /// Request applying the selected gesture.
    pub to_apply_tool: bool,
}
impl Default for AisViewInputSelection {
    fn default() -> Self {
        Self {
            tool: AisViewSelectionTool::Picking,
            scheme: AisSelectionScheme::Unknown,
            points: Sequence::default(),
            to_apply_tool: false,
        }
    }
}
