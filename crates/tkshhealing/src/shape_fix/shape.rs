// Rust translation of the unchanged-shape path in ShapeFix/ShapeFix_Shape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The no-change path of `ShapeFix_Shape` for already valid shapes.
use super::{FixRoot, FixRootData};
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::Shape;
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
        let (input, context) = self
            .root
            .with(|fields| (fields.shape.clone(), fields.context.clone()));
        let Some(context) = context.get() else {
            return Err(Failure::new(
                "ShapeFix_Shape::Perform requires Init(shape) before Perform()",
            ));
        };
        *self.result.write() = context.apply(&input, ShapeEnum::Shape)?;
        Ok(false)
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
