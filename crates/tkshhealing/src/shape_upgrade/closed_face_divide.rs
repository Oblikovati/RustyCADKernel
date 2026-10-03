//! Divide closed faces at a seam or a geometric closure.
use super::{FaceDivideData, FaceDivider, ShapeUpgradeFaceDivide, ShapeUpgradeTool};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, SharedState},
};
mod algorithm;
mod closure;
mod seam;

/// A face divider that opens closed faces with parameter-space cuts.
///
/// ```
/// use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis};
/// use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeClosedFaceDivide;
/// let face = BRepPrimAPIMakeCylinder::new(2.0, 5.0).face()?;
/// let divider = ShapeUpgradeClosedFaceDivide::new(face);
/// divider.set_nb_split_points(2);
/// assert!(divider.perform(0.0)?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct ShapeUpgradeClosedFaceDivide {
    parent: ShapeUpgradeFaceDivide,
    split_points: SharedState<i32>,
}
impl_transient!(ShapeUpgradeClosedFaceDivide, "ShapeUpgrade_ClosedFaceDivide", ShapeUpgradeFaceDivide, parent, interfaces:[dyn FaceDivider]);

impl Default for ShapeUpgradeClosedFaceDivide {
    fn default() -> Self {
        Self {
            parent: ShapeUpgradeFaceDivide::empty(),
            split_points: SharedState::new(1),
        }
    }
}

impl std::ops::Deref for ShapeUpgradeClosedFaceDivide {
    type Target = ShapeUpgradeFaceDivide;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl FaceDivider for ShapeUpgradeClosedFaceDivide {
    fn face_divide_data(&self) -> &FaceDivideData {
        self.parent.face_divide_data()
    }
    fn face_divide_base(&self) -> &ShapeUpgradeTool {
        self.parent.face_divide_base()
    }
    fn split_surface(&self, area: f64) -> Result<bool, Failure> {
        algorithm::split_surface(self, area)
    }
}

impl ShapeUpgradeClosedFaceDivide {
    /// Construct the inherited divider without an input face; `ShapeUpgradeClosedFaceDivide::empty()`.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Initialize from a working face; `ShapeUpgradeClosedFaceDivide::new(face)`.
    pub fn new(face: rustycad_tkbrep::topods::Face) -> Self {
        let divider = Self::empty();
        divider.init(&face);
        divider
    }
    /// Set the number of interior cuts; nonpositive values preserve the current value;
    /// `divider.set_nb_split_points(2)`.
    pub fn set_nb_split_points(&self, count: i32) {
        if count > 0 {
            *self.split_points.write() = count;
        }
    }
    /// Return the configured interior cut count; `divider.nb_split_points()`.
    pub fn nb_split_points(&self) -> i32 {
        *self.split_points.read()
    }
    /// Perform closed-face division; `divider.perform(0.0)?`.
    pub fn perform(&self, area: f64) -> Result<bool, Failure> {
        FaceDivider::perform(self, area)
    }
    /// Split the current face by its seam or geometric closure; `divider.split_surface(0.0)?`.
    pub fn split_surface(&self, area: f64) -> Result<bool, Failure> {
        FaceDivider::split_surface(self, area)
    }
}
