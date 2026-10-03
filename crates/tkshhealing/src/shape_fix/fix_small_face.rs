//! Spot-face healing from `ShapeFix_FixSmallFace`.
mod spot;
mod strip;

use super::{FixRoot, FixRootData, ShapeFixRoot};
use crate::shape_analysis::ShapeAnalysisCheckSmallFace;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::topods::{Face, Shape};
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

    /// Replaces every face vertex by the native mean-point vertex; `fixer.replace_vertices_in_case_of_spot(&mut face, -1.0)?`.
    pub fn replace_vertices_in_case_of_spot(
        &self,
        face: &mut Face,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        spot::replace_vertices_in_case_of_spot(self, face, tolerance)
    }

    /// Removes the face and all its edges from the context; `fixer.remove_faces_in_case_of_spot(&face)?`.
    pub fn remove_faces_in_case_of_spot(&self, face: &Face) -> Result<bool, Failure> {
        spot::remove_faces_in_case_of_spot(self, face)
    }

    /// Applies `FixFace` to each face in the initialized shape; `let repaired = fixer.fix_shape()?`.
    pub fn fix_shape(&self) -> Result<Shape, Failure> {
        spot::fix_shape(self)
    }

    /// Copies and repairs one face; `let repaired = fixer.fix_face(&face)?`.
    pub fn fix_face(&self, face: &Face) -> Result<Face, Failure> {
        spot::fix_face(self, face)
    }

    /// Native `FixPinFace` is currently a no-op returning true; `assert!(fixer.fix_pin_face(&mut face))`.
    pub fn fix_pin_face(&self, _face: &mut Face) -> bool {
        true
    }

    /// Replaces recognized strip faces and applies the shared reshape context; `fixer.fix_strip_face(false)?`.
    pub fn fix_strip_face(&self, was_done: bool) -> Result<Shape, Failure> {
        strip::fix_strip_face(self, was_done)
    }

    /// Joins the two long strip edges through their adjacent support; `fixer.replace_in_case_of_strip(&mut face, &mut first, &mut second, 1e-7)?`.
    pub fn replace_in_case_of_strip(
        &self,
        face: &mut Face,
        first: &mut rustycad_tkbrep::topods::Edge,
        second: &mut rustycad_tkbrep::topods::Edge,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        strip::replace_in_case_of_strip(self, face, first, second, tolerance)
    }

    /// Removes a strip face from the reshape context and records its warning.
    /// Example: `fixer.remove_faces_in_case_of_strip(&face)?`.
    pub fn remove_faces_in_case_of_strip(&self, face: &Face) -> Result<bool, Failure> {
        strip::remove_faces_in_case_of_strip(self, face)
    }

    /// Builds the shared strip edge from matched endpoints and the support-face p-curve.
    /// Example: `let edge = fixer.compute_shared_edge_for_strip_face(&face, &a, &b, &neighbor, 1e-7)?`.
    pub fn compute_shared_edge_for_strip_face(
        &self,
        _face: &Face,
        first: &rustycad_tkbrep::topods::Edge,
        second: &rustycad_tkbrep::topods::Edge,
        support: &Face,
        tolerance: f64,
    ) -> Result<rustycad_tkbrep::topods::Edge, Failure> {
        strip::compute_shared_edge_for_strip_face(self, first, second, support, tolerance)
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
