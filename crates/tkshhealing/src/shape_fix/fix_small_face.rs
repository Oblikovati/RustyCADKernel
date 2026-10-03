//! Spot-face healing from `ShapeFix_FixSmallFace`.
mod spot;

use super::{FixRoot, FixRootData, ShapeFixRoot};
use crate::shape_analysis::ShapeAnalysisCheckSmallFace;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};

/// The context and spot-face path of `ShapeFix_FixSmallFace`.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkshhealing::shape_fix::ShapeFixFixSmallFace;
/// let fixer = ShapeFixFixSmallFace::new(Shape::default());
/// assert!(fixer.shape().is_null());
/// ```
pub struct ShapeFixFixSmallFace {
    parent: ShapeFixRoot,
    input: SharedState<Shape>,
    result: SharedState<Shape>,
    status: SharedState<i32>,
    analyzer: SharedState<ShapeAnalysisCheckSmallFace>,
}

impl_transient!(ShapeFixFixSmallFace, "ShapeFix_FixSmallFace", ShapeFixRoot, parent, interfaces: [dyn FixRoot]);

impl Default for ShapeFixFixSmallFace {
    fn default() -> Self {
        let fixer = Self {
            parent: ShapeFixRoot::new(),
            input: SharedState::new(Shape::default()),
            result: SharedState::new(Shape::default()),
            status: SharedState::new(ShapeExtend::encode_status(ShapeExtendStatus::Ok)),
            analyzer: SharedState::new(ShapeAnalysisCheckSmallFace::new()),
        };
        FixRoot::set_precision(&fixer, rustycad_tkernel::precision::confusion());
        fixer
    }
}

impl ShapeFixFixSmallFace {
    /// `ShapeFix_FixSmallFace(shape)` followed by `Init`; `ShapeFixFixSmallFace::new(shape)`.
    pub fn new(shape: Shape) -> Self {
        let fixer = Self::default();
        fixer.init(shape);
        fixer
    }

    /// Initializes input/result and creates the reshape context when absent; `fixer.init(shape)`.
    pub fn init(&self, shape: Shape) {
        *self.input.write() = shape.clone();
        *self.result.write() = shape;
        self.ensure_context();
    }

    /// Returns `myShape`; `assert_eq!(fixer.shape().is_null(), shape.is_null())`.
    pub fn shape(&self) -> Shape {
        self.input.read().clone()
    }

    /// Decodes the aggregate native status; `fixer.status(ShapeExtendStatus::Done1)`.
    pub fn status(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(*self.status.read(), status)
    }

    /// Applies the native spot-face checker to each face and updates the reshape context.
    /// Example: `let result = fixer.fix_spot_face()?`.
    pub fn fix_spot_face(&self) -> Result<Shape, Failure> {
        spot::fix_spot_face(self)
    }

    fn ensure_context(&self) {
        if self.context().get().is_some() {
            return;
        }
        let context = Handle::new(ShapeBuildReShape::new());
        FixRoot::set_context(self, &context);
    }
}

impl FixRoot for ShapeFixFixSmallFace {
    fn fix_root_data(&self) -> &FixRootData {
        self.parent.fix_root_data()
    }
}

impl std::ops::Deref for ShapeFixFixSmallFace {
    type Target = ShapeFixRoot;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
