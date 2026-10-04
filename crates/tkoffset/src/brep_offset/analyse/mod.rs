//! BRepOffset_Analyse at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod classify;
mod mixed;
mod queries;
use super::BRepOffsetInterval;
use rustycad_tkbrep::{topods::Shape, toptools::ShapeMapHasher};
use rustycad_tkernel::{ncollection::DataMap, standard::Failure};
type ShapeBindings<T> = DataMap<Shape, T, ShapeMapHasher>;
/// Offset connectivity and identity-preserving incidence; `BRepOffsetAnalyse::new()`.
#[derive(Default)]
pub struct BRepOffsetAnalyse {
    shape: Shape,
    angle: f64,
    offset: f64,
    done: bool,
    intervals: ShapeBindings<Vec<BRepOffsetInterval>>,
    ancestors: ShapeBindings<Vec<Shape>>,
    new_faces: Vec<Shape>,
}
impl BRepOffsetAnalyse {
    /// Create an idle analysis; `BRepOffsetAnalyse::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Analyse uniform-offset connectivity; `analysis.perform(&shape, 0.01)?`.
    pub fn perform(&mut self, shape: &Shape, angle: f64) -> Result<(), Failure> {
        self.shape = shape.clone();
        self.angle = angle;
        self.new_faces.clear();
        self.build_ancestors()?;
        self.classify_edges()?;
        self.done = true;
        Ok(())
    }
    /// Construct and perform analysis; `BRepOffsetAnalyse::with_shape(&shape, 0.01)?`.
    pub fn with_shape(shape: &Shape, angle: f64) -> Result<Self, Failure> {
        let mut analysis = Self::new();
        analysis.perform(shape, angle)?;
        Ok(analysis)
    }
    /// Read completion state; `analysis.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Store the global offset; `analysis.set_offset_value(2.)`.
    pub fn set_offset_value(&mut self, offset: f64) {
        self.offset = offset;
    }
    /// Clear topology and results, retaining angle and offset; `analysis.clear()`.
    pub fn clear(&mut self) {
        self.done = false;
        self.shape = Shape::default();
        self.intervals.clear(false);
        self.ancestors.clear(false);
        self.new_faces.clear();
    }
    /// Read artificial tangent faces; uniform offsets create none; `analysis.new_faces()`.
    pub fn new_faces(&self) -> &[Shape] {
        &self.new_faces
    }
    /// Check generated-face bindings; uniform offsets create none; `analysis.has_generated(&edge)`.
    pub fn has_generated(&self, _shape: &Shape) -> bool {
        false
    }
}
