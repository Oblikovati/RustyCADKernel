// Rust translation of ShapeFix/ShapeFix_Shape.cxx dispatch at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Native shape dispatch and shared repair tools; wire dispatch remains incomplete.
mod configuration;
use configuration::ShapeFixModes;
mod perform;
use super::{FixRoot, FixRootData};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};

/// Shape-level repair dispatch with shared tools and context handling.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkshhealing::shape_fix::ShapeFixShape;
/// let fixer = ShapeFixShape::new(Shape::default());
/// assert!(!fixer.perform().unwrap());
/// assert!(fixer.shape().is_null());
/// ```
pub struct ShapeFixShape {
    parent: super::ShapeFixRoot,
    result: SharedState<Shape>,
    status: SharedState<i32>,
    solid_tool: Handle<super::ShapeFixSolid>,
    modes: SharedState<ShapeFixModes>,
    visited: SharedState<MapOfShape>,
}

impl_transient!(ShapeFixShape, "ShapeFix_Shape", super::ShapeFixRoot, parent, interfaces: [dyn FixRoot]);

impl ShapeFixShape {
    /// Construct an unloaded shape repair tool; `ShapeFixShape::empty()`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// `ShapeFix_Shape(shape)`: initialize the input and result to `shape`;
    /// `ShapeFixShape::new(Shape::default())`.
    pub fn new(shape: Shape) -> Self {
        let fixer = Self::default();
        fixer.init(shape);
        fixer
    }

    /// `Init(shape)`: replace the input and the result with `shape`;
    /// `fixer.init(Shape::default())`.
    ///
    /// ```
    /// use rustycad_tkbrep::topods::Shape;
    /// use rustycad_tkshhealing::shape_fix::ShapeFixShape;
    /// let fixer = ShapeFixShape::default();
    /// fixer.init(Shape::default());
    /// ```
    pub fn init(&self, shape: Shape) {
        self.parent.fix_root_data().with_mut(|fields| {
            fields.shape = shape.clone();
            if fields.context.is_null() {
                let context = Handle::new(ShapeBuildReShape::new());
                context.set_mode_consider_location(true);
                fields.context = context;
            }
        });
        *self.result.write() = shape;
    }

    /// `Perform()`: dispatch the loaded shape to its repair tool;
    /// `assert!(!fixer.perform()?)`.
    ///
    /// ```
    /// use rustycad_tkbrep::topods::Shape;
    /// use rustycad_tkshhealing::shape_fix::ShapeFixShape;
    /// let fixer = ShapeFixShape::new(Shape::default());
    /// assert!(!fixer.perform().unwrap());
    /// ```
    pub fn perform(&self) -> Result<bool, Failure> {
        self.perform_with_progress(&rustycad_tkernel::message::ProgressRange::default())
    }

    /// `Shape()`: return the result of the latest `Perform()` or `Init()`;
    /// `let result = fixer.shape()`.
    ///
    /// ```
    /// use rustycad_tkbrep::topods::Shape;
    /// use rustycad_tkshhealing::shape_fix::ShapeFixShape;
    /// let fixer = ShapeFixShape::new(Shape::default());
    /// assert!(fixer.shape().is_null());
    /// ```
    pub fn shape(&self) -> Shape {
        self.result.read().clone()
    }

    /// `Status(status)`: query the aggregate status of the last `Perform()`;
    /// `!fixer.status(ShapeExtendStatus::Fail)`.
    pub fn status(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(*self.status.read(), status)
    }

    /// `SetPrecision(preci)`: delegate the root precision update;
    /// `fixer.set_precision(0.01)`.
    pub fn set_precision(&self, precision: f64) {
        FixRoot::set_precision(self, precision);
    }

    /// `Precision()`: return the root precision;
    /// `fixer.precision()`.
    pub fn precision(&self) -> f64 {
        FixRoot::precision(self)
    }

    /// `SetMinTolerance(tol)`: update the minimum root tolerance;
    /// `fixer.set_min_tolerance(0.001)`.
    pub fn set_min_tolerance(&self, tolerance: f64) {
        FixRoot::set_min_tolerance(self, tolerance);
    }

    /// `MinTolerance()`: return the minimum root tolerance;
    /// `fixer.min_tolerance()`.
    pub fn min_tolerance(&self) -> f64 {
        FixRoot::min_tolerance(self)
    }

    /// `SetMaxTolerance(tol)`: update the maximum root tolerance;
    /// `fixer.set_max_tolerance(0.1)`.
    pub fn set_max_tolerance(&self, tolerance: f64) {
        FixRoot::set_max_tolerance(self, tolerance);
    }

    /// `MaxTolerance()`: return the maximum root tolerance;
    /// `fixer.max_tolerance()`.
    pub fn max_tolerance(&self) -> f64 {
        FixRoot::max_tolerance(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{FixRoot, ShapeFixShape};
    use rustycad_tkbrep::topods::Shape;

    #[test]
    fn perform_preserves_the_initialized_shape() {
        let input = Shape::default();
        let fixer = ShapeFixShape::new(input.clone());
        assert!(!fixer.perform().unwrap());
        assert_eq!(fixer.shape().tshape_address(), input.tshape_address());
        assert!(!fixer.context().is_null());
    }

    #[test]
    fn init_replaces_the_input_and_result() {
        let fixer = ShapeFixShape::default();
        let input = Shape::default();
        fixer.init(input.clone());
        assert_eq!(fixer.shape().tshape_address(), input.tshape_address());
        assert!(!fixer.context().is_null());
    }

    #[test]
    fn perform_requires_shape_initialization() {
        let fixer = ShapeFixShape::default();
        assert!(
            fixer
                .perform()
                .unwrap_err()
                .message()
                .contains("Init(shape)")
        );
    }
}
