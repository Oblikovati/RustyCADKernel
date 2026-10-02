// Rust translation of the unchanged-shape path in ShapeFix/ShapeFix_Shape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The no-change path of `ShapeFix_Shape` for already valid shapes.
use super::{FixRoot, FixRootData};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The unchanged-shape path and context handling of `ShapeFix_Shape`.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkshhealing::shape_fix::ShapeFixShape;
/// let fixer = ShapeFixShape::new(Shape::default());
/// assert!(!fixer.perform().unwrap());
/// assert!(fixer.shape().is_null());
/// ```
#[derive(Default)]
pub struct ShapeFixShape {
    parent: StandardTransient,
    root: FixRootData,
    result: SharedState<Shape>,
    status: SharedState<i32>,
}

impl_transient!(ShapeFixShape, "ShapeFix_Shape", StandardTransient, parent, interfaces: [dyn FixRoot]);

impl ShapeFixShape {
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
        self.root.with_mut(|fields| {
            fields.shape = shape.clone();
            if fields.context.is_null() {
                let context = Handle::new(ShapeBuildReShape::new());
                context.set_mode_consider_location(true);
                fields.context = context;
            }
        });
        *self.result.write() = shape;
        *self.status.write() = 0;
    }

    /// `Perform()`: apply the context and report no internal fix in this supported path;
    /// `assert!(!fixer.perform()?)`.
    ///
    /// ```
    /// use rustycad_tkbrep::topods::Shape;
    /// use rustycad_tkshhealing::shape_fix::ShapeFixShape;
    /// let fixer = ShapeFixShape::new(Shape::default());
    /// assert!(!fixer.perform().unwrap());
    /// ```
    pub fn perform(&self) -> Result<bool, Failure> {
        *self.status.write() = 0;
        let (input, context) = self
            .root
            .with(|fields| (fields.shape.clone(), fields.context.clone()));
        if context.get().is_none() {
            return Err(Failure::new(
                "ShapeFix_Shape::Perform requires Init(shape) before Perform()",
            ));
        }
        if input.is_null() {
            *self.result.write() = input;
            return Ok(false);
        }
        let did_fix = fix_descendant_faces(&input, &context)?;
        let reshaped = context.apply(&input, ShapeEnum::Shape)?;
        *self.result.write() = super::shell_orientation::repair_shell_orientations(&reshaped)?;
        if did_fix {
            *self.status.write() = ShapeExtend::encode_status(ShapeExtendStatus::Done3)
                | ShapeExtend::encode_status(ShapeExtendStatus::Done4);
        }
        Ok(did_fix)
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

/// Runs the face fixer over every face before applying replacements to the whole shape.
fn fix_descendant_faces(
    input: &Shape,
    context: &Handle<ShapeBuildReShape>,
) -> Result<bool, Failure> {
    let mut explorer = Explorer::new(input, ShapeEnum::Face, ShapeEnum::Shape)?;
    let mut did_fix = false;
    while explorer.more() {
        let face = topods::face(explorer.current())?;
        let fixer = super::face::ShapeFixFace::new(face.into_shape());
        fixer.set_context(context);
        did_fix |= fixer.perform()?;
        explorer.next()?;
    }
    Ok(did_fix)
}

impl FixRoot for ShapeFixShape {
    fn fix_root_data(&self) -> &FixRootData {
        &self.root
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
