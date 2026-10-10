//! Standard selection construction helpers.
mod brep_selection_tool;
pub use brep_selection_tool::StdSelectBrepSelectionTool;
/// Native legacy alias; the underlying selector retains its current implementation scope.
/// Example: `let selector = StdSelectViewerSelector3d::new()`.
pub type StdSelectViewerSelector3d = crate::select_mgr::SelectMgrViewerSelector;

mod selection_image;
pub use selection_image::StdSelectTypeOfSelectionImage;
