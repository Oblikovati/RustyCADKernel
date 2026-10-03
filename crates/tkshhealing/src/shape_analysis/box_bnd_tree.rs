//! Stateful wire endpoint selector from ShapeAnalysis_BoxBndTree at 3d097a0328e71b8.
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::{
    ncollection::{FallibleTreeSelector, HArray1},
    standard::{Failure, Handle},
};
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
use std::collections::HashSet;
mod selection;

/// Select wire endpoints by identity or distance; `selector.accept(1)?`.
pub struct ShapeAnalysisBoxBndTreeSelector {
    sequence: Handle<HArray1<Shape>>,
    shared: bool,
    boxes: [BoundingBox3; 2],
    vertices: [Vertex; 2],
    points: [Point3; 2],
    visited: HashSet<i32>,
    tolerance: f64,
    minimum: f64,
    indices: [i32; 2],
    selected: i32,
    status: i32,
    stopped: bool,
}
impl ShapeAnalysisBoxBndTreeSelector {
    /// Keep the caller's shared shape array; `ShapeAnalysisBoxBndTreeSelector::new(&array,false)`.
    pub fn new(sequence: &Handle<HArray1<Shape>>, shared: bool) -> Self {
        Self {
            sequence: sequence.clone(),
            shared,
            boxes: std::array::from_fn(|_| BoundingBox3::default()),
            vertices: std::array::from_fn(|_| Vertex::default()),
            points: [Point3::default(); 2],
            visited: HashSet::new(),
            tolerance: 1e-7,
            minimum: 1e-7,
            indices: [0; 2],
            selected: 0,
            status: 0,
            stopped: false,
        }
    }
    /// Replace query boxes and clear only candidate indices; `selector.define_boxes(&first,&last)`.
    pub fn define_boxes(&mut self, first: &BoundingBox3, last: &BoundingBox3) {
        self.boxes = [first.clone(), last.clone()];
        self.indices = [0; 2];
    }
    /// Replace shared endpoint identities and reset status; `selector.define_vertexes(first,last)`.
    pub fn define_vertexes(&mut self, first: Vertex, last: Vertex) {
        self.vertices = [first, last];
        self.status = 0;
    }
    /// Replace geometric endpoints and reset status; `selector.define_pnt(first,last)`.
    pub fn define_pnt(&mut self, first: Point3, last: Point3) {
        self.points = [first, last];
        self.status = 0;
    }
    /// Read the selected wire index; `selector.nb()`.
    pub fn nb(&self) -> i32 {
        self.selected
    }
    /// Set the selected index without altering candidates; `selector.set_nb(3)`.
    pub fn set_nb(&mut self, index: i32) {
        self.selected = index;
    }
    /// Exclude an index from future acceptance; `selector.load_list(3)`.
    pub fn load_list(&mut self, index: i32) {
        self.visited.insert(index);
    }
    /// Query the exclusion set; `selector.cont_wire(3)`.
    pub fn cont_wire(&self, index: i32) -> bool {
        self.visited.contains(&index)
    }
    /// Reset the inherited stop flag (native SetStop); `selector.set_stop()`.
    pub fn set_stop(&mut self) {
        self.stopped = false;
    }
    /// Observe the inherited stop flag; `selector.stop()`.
    pub fn stop(&self) -> bool {
        self.stopped
    }
    /// Replace tolerance and nearest distance, retaining candidates; `selector.set_tolerance(0.01)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
        self.minimum = tolerance;
        self.status = 0;
    }
    /// Decode the latest status; `selector.last_check_status(ShapeExtendStatus::Done1)`.
    pub fn last_check_status(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.status, status)
    }
    /// Reject boxes outside both query endpoints; `selector.reject(&bounds)`.
    pub fn reject(&self, bounds: &BoundingBox3) -> bool {
        self.boxes[0].is_out_box(bounds) && self.boxes[1].is_out_box(bounds)
    }
    /// Consider a one-based wire index; `selector.accept(1)?`.
    pub fn accept(&mut self, index: i32) -> Result<bool, Failure> {
        selection::accept(self, index)
    }
}
impl FallibleTreeSelector<i32, BoundingBox3> for ShapeAnalysisBoxBndTreeSelector {
    type Error = Failure;
    fn reject(&self, bounds: &BoundingBox3) -> bool {
        self.reject(bounds)
    }
    fn accept(&mut self, index: &i32) -> Result<bool, Failure> {
        self.accept(*index)
    }
    fn stop(&self) -> bool {
        self.stopped
    }
}
