//! Selecting axis state, including OCCT 3d097a0 initialization assertions.
use super::{SelectMgrBaseIntersectorBase, SelectMgrSelectionType};
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Axis1, Direction3, GeneralTransform3, Point3, TransformForm, Xyz};

/// A ray selector; initialize its axis before geometric queries.
/// Example: `let volume = SelectMgrAxisIntersector::new(); volume.init(axis);`.
pub struct SelectMgrAxisIntersector {
    pub(super) transient: StandardTransient,
    /// Inherited camera and selection type; `volume.base.selection_type.read()`.
    pub base: SelectMgrBaseIntersectorBase,
    pub(super) axis: SharedState<Axis1>,
}
impl Default for SelectMgrAxisIntersector {
    fn default() -> Self {
        Self::new()
    }
}
impl SelectMgrAxisIntersector {
    /// Create an uninitialized selector with the native default axis; `SelectMgrAxisIntersector::new()`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: SelectMgrBaseIntersectorBase::default(),
            axis: SharedState::new(Axis1::default()),
        }
    }
    /// Set the axis and Point selection type; `volume.init(axis)`.
    pub fn init(&self, axis: Axis1) {
        *self.base.selection_type.write() = SelectMgrSelectionType::Point;
        *self.axis.write() = axis;
    }
    pub(super) fn initialized_axis(&self, operation: &str) -> Result<Axis1, Failure> {
        let selection = *self.base.selection_type.read();
        if selection != SelectMgrSelectionType::Point {
            return Err(Failure::with_message(
                "Standard_ProgramError",
                &format!(
                    "AxisIntersector::{operation}: selection_type={}; expected Point(0) after Init",
                    selection as i32
                ),
            ));
        }
        Ok(*self.axis.read())
    }
    pub(super) fn transformed_axis(&self, transform: &GeneralTransform3) -> Result<Self, Failure> {
        let axis = self.initialized_axis("ScaleAndTransform")?;
        let result = Self::new();
        // OCCT 3d097a0 deliberately returns an uninitialized default for Identity form.
        if transform.form() == TransformForm::Identity {
            return Ok(result);
        }
        let mut location = axis.location().xyz();
        transform.transforms(&mut location);
        let mut direction = axis.direction().xyz();
        let mut linear = *transform;
        linear.set_translation_part(Xyz::new(0., 0., 0.));
        linear.transforms(&mut direction);
        result.init(Axis1::new(
            Point3::from(location),
            Direction3::from_xyz(direction)?,
        ));
        Ok(result)
    }
}
