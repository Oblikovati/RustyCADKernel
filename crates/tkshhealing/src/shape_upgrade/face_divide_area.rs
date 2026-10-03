//! Area-based face subdivision from OCCT 3d097a0328e71b8.
use super::{
    FaceDivideData, FaceDivider, ShapeUpgradeFaceDivide, ShapeUpgradeSplitSurfaceArea,
    ShapeUpgradeTool,
};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::{
    impl_transient, precision,
    standard::{Failure, Handle, SharedState},
};
mod perform;

#[derive(Clone, Copy)]
struct AreaDivisionSettings {
    max_area: f64,
    parts: i32,
    u_splits: i32,
    v_splits: i32,
    by_number: bool,
}

/// Subdivide faces by maximum area; `ShapeUpgradeFaceDivideArea::new(face)`.
pub struct ShapeUpgradeFaceDivideArea {
    parent: ShapeUpgradeFaceDivide,
    settings: SharedState<AreaDivisionSettings>,
}
impl_transient!(ShapeUpgradeFaceDivideArea, "ShapeUpgrade_FaceDivideArea", ShapeUpgradeFaceDivide, parent, interfaces:[dyn FaceDivider]);
impl Default for ShapeUpgradeFaceDivideArea {
    fn default() -> Self {
        let parent = ShapeUpgradeFaceDivide::empty();
        parent.set_precision(1e-5);
        parent.set_split_surface_tool(&Handle::new(ShapeUpgradeSplitSurfaceArea::new()).cast());
        Self {
            parent,
            settings: SharedState::new(AreaDivisionSettings {
                max_area: precision::infinite(),
                parts: 0,
                u_splits: -1,
                v_splits: -1,
                by_number: false,
            }),
        }
    }
}
impl std::ops::Deref for ShapeUpgradeFaceDivideArea {
    type Target = ShapeUpgradeFaceDivide;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl FaceDivider for ShapeUpgradeFaceDivideArea {
    fn face_divide_data(&self) -> &FaceDivideData {
        self.parent.face_divide_data()
    }
    fn face_divide_base(&self) -> &ShapeUpgradeTool {
        self.parent.face_divide_base()
    }
    fn perform(&self, _area: f64) -> Result<bool, Failure> {
        self.perform_area_division()
    }
}
impl ShapeUpgradeFaceDivideArea {
    /// Construct without a face; `ShapeUpgradeFaceDivideArea::empty()`.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Initialize with a working face; `ShapeUpgradeFaceDivideArea::new(face)`.
    pub fn new(face: Face) -> Self {
        let divider = Self::empty();
        divider.init(&face);
        divider
    }
    /// Maximum accepted face area; `divider.max_area()`.
    pub fn max_area(&self) -> f64 {
        self.settings.read().max_area
    }
    /// Set the area limit, or -1 to derive it from the part count; `divider.set_max_area(2.)`.
    pub fn set_max_area(&self, area: f64) {
        self.settings.write().max_area = area;
    }
    /// Requested part count; `divider.nb_parts()`.
    pub fn nb_parts(&self) -> i32 {
        self.settings.read().parts
    }
    /// Set the part count used with area -1; `divider.set_nb_parts(4)`.
    pub fn set_nb_parts(&self, parts: i32) {
        self.settings.write().parts = parts;
    }
    /// Select square-grid subdivision; `divider.set_splitting_by_number(true)`.
    pub fn set_splitting_by_number(&self, enabled: bool) {
        self.settings.write().by_number = enabled;
    }
    /// Set fixed grid counts; `divider.set_numbers_uv_splits(2, 3)?`.
    pub fn set_numbers_uv_splits(&self, u: i32, v: i32) -> Result<(), Failure> {
        let mut settings = self.settings.write();
        settings.u_splits = u;
        settings.v_splits = v;
        if u > 0 && v > 0 {
            settings.parts = u.checked_mul(v).ok_or_else(|| {
                Failure::with_type(
                    "Standard_OutOfRange",
                    &format!(
                        "U/V split counts {u} and {v}; expected product within signed 32-bit range"
                    ),
                    "",
                )
            })?;
        }
        Ok(())
    }
    /// Perform subdivision; the argument is ignored as in OCCT; `divider.perform(0.)?`.
    pub fn perform(&self, area: f64) -> Result<bool, Failure> {
        FaceDivider::perform(self, area)
    }
}
