//! BRepOffset_Analyse at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod classify;
mod grouping;
mod mixed;
mod queries;
mod tangent_faces;
use super::BRepOffsetInterval;
use rustycad_tkbrep::{
    topods::{Edge, Shape},
    toptools::ShapeMapHasher,
};
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
    ancestor_order: Vec<Shape>,
    face_offsets: ShapeBindings<f64>,
    new_faces: Vec<Shape>,
    descendants: ShapeBindings<Vec<Shape>>,
    generated: ShapeBindings<Shape>,
    replacements: ShapeBindings<ShapeBindings<Edge>>,
}
impl BRepOffsetAnalyse {
    /// Create an idle analysis; `BRepOffsetAnalyse::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Analyse connectivity and artificial faces for differing offsets; `analysis.perform(&shape, 0.01)?`.
    pub fn perform(&mut self, shape: &Shape, angle: f64) -> Result<(), Failure> {
        self.shape = shape.clone();
        self.angle = angle;
        self.new_faces.clear();
        self.descendants.clear(false);
        self.generated.clear(false);
        self.replacements.clear(false);
        self.build_ancestors()?;
        let tangent_edges = self.classify_edges()?;
        self.treat_tangent_faces(&tangent_edges)?;
        self.rebuild_descendants()?;
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
    /// Copy per-face offsets used for artificial tangent faces; `analysis.set_face_offset_map(&offsets)`.
    pub fn set_face_offset_map(&mut self, offsets: &ShapeBindings<f64>) {
        self.face_offsets = offsets.clone();
    }
    /// Clear topology and results, retaining angle and offset; `analysis.clear()`.
    pub fn clear(&mut self) {
        self.done = false;
        self.shape = Shape::default();
        self.intervals.clear(false);
        self.ancestors.clear(false);
        self.ancestor_order.clear();
        self.face_offsets.clear(false);
        self.new_faces.clear();
        self.descendants.clear(false);
        self.generated.clear(false);
        self.replacements.clear(false);
    }
    /// Read artificial tangent faces between differently offset supports; `analysis.new_faces()`.
    pub fn new_faces(&self) -> &[Shape] {
        &self.new_faces
    }
    /// Check generated face or side-edge bindings; `analysis.has_generated(&edge)`.
    pub fn has_generated(&self, shape: &Shape) -> bool {
        self.generated.is_bound(shape)
    }
}
