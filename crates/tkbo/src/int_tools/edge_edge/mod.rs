//! IntTools_EdgeEdge at OCCT 3d097a0, including persistent errors and accumulated results.
use crate::int_tools::{CommonPrt, Range};
use rustycad_tkbrep::{brep_adaptor::BRepAdaptorCurve, topods::Edge};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::GeomCurve;
mod distance;
mod distance_search;
#[path = "best_solution.rs"]
mod edge_best;
#[path = "coincidence.rs"]
mod edge_coincidence;
#[path = "intersection.rs"]
mod edge_intersection;
#[path = "line_line.rs"]
mod edge_lines;
#[path = "math.rs"]
mod edge_math;
#[path = "parameters.rs"]
mod edge_parameters;
#[path = "resolution.rs"]
mod edge_resolution;
#[path = "solutions.rs"]
mod edge_solutions;
mod merge;
mod perform;
mod prepare;
/// Edge intersection state; `assert!(EdgeEdge::new().is_done())` reflects the native initial status.
pub struct EdgeEdge {
    edges: [Edge; 2],
    curves: [BRepAdaptorCurve; 2],
    geometry: [Handle<dyn GeomCurve>; 2],
    ranges: [Range; 2],
    tolerances: [f64; 2],
    tolerance: f64,
    resolutions: [f64; 2],
    coefficients: [f64; 2],
    precisions: [f64; 2],
    fuzzy: f64,
    swapped: bool,
    error: i32,
    quick: bool,
    parts: Vec<CommonPrt>,
}
impl Default for EdgeEdge {
    fn default() -> Self {
        Self {
            edges: Default::default(),
            curves: Default::default(),
            geometry: Default::default(),
            ranges: [Range::new(0., 0.); 2],
            tolerances: [0.; 2],
            tolerance: 0.,
            resolutions: [0.; 2],
            coefficients: [0.; 2],
            precisions: [0.; 2],
            fuzzy: precision::confusion(),
            swapped: false,
            error: 0,
            quick: false,
            parts: Vec::new(),
        }
    }
}
impl EdgeEdge {
    /// Creates empty native state; `assert!(EdgeEdge::new().is_done())`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Retains both edges without computing; `EdgeEdge::with_edges(&first,&second)`.
    pub fn with_edges(first: &Edge, second: &Edge) -> Self {
        Self {
            edges: [first.clone(), second.clone()],
            ..Self::default()
        }
    }
    /// Retains explicit parameter bounds; `EdgeEdge::with_ranges(&first,0.,1.,&second,0.,1.)`.
    pub fn with_ranges(first: &Edge, a: f64, b: f64, second: &Edge, c: f64, d: f64) -> Self {
        Self {
            ranges: [Range::new(a, b), Range::new(c, d)],
            ..Self::with_edges(first, second)
        }
    }
    /// Replaces the first edge, retaining prior result state; `intersection.set_edge1(&edge)`.
    pub fn set_edge1(&mut self, edge: &Edge) {
        self.edges[0] = edge.clone();
    }
    /// Replaces the second edge; `intersection.set_edge2(&edge)`.
    pub fn set_edge2(&mut self, edge: &Edge) {
        self.edges[1] = edge.clone();
    }
    /// Replaces the first range; `intersection.set_range1(Range::new(0.,1.))`.
    pub fn set_range1(&mut self, range: Range) {
        self.ranges[0] = range;
    }
    /// Replaces the second range; `intersection.set_range2(Range::new(0.,1.))`.
    pub fn set_range2(&mut self, range: Range) {
        self.ranges[1] = range;
    }
    /// Replaces first bounds; `intersection.set_range1_bounds(0.,1.)`.
    pub fn set_range1_bounds(&mut self, first: f64, last: f64) {
        self.set_range1(Range::new(first, last));
    }
    /// Replaces second bounds; `intersection.set_range2_bounds(0.,1.)`.
    pub fn set_range2_bounds(&mut self, first: f64, last: f64) {
        self.set_range2(Range::new(first, last));
    }
    /// Replaces the first edge and bounds; `intersection.set_edge1_range(&edge,0.,1.)`.
    pub fn set_edge1_range(&mut self, edge: &Edge, first: f64, last: f64) {
        self.set_edge1(edge);
        self.set_range1_bounds(first, last);
    }
    /// Replaces the second edge and bounds; `intersection.set_edge2_range(&edge,0.,1.)`.
    pub fn set_edge2_range(&mut self, edge: &Edge, first: f64, last: f64) {
        self.set_edge2(edge);
        self.set_range2_bounds(first, last);
    }
    /// Retains native maximum ordering, including NaN; `intersection.set_fuzzy_value(1e-6)`.
    pub fn set_fuzzy_value(&mut self, fuzzy: f64) {
        self.fuzzy = std_max(fuzzy, precision::confusion());
    }
    /// Reads fuzzy tolerance; `intersection.fuzzy_value()`.
    pub fn fuzzy_value(&self) -> f64 {
        self.fuzzy
    }
    /// Reports the retained native error status; `intersection.is_done()`.
    pub fn is_done(&self) -> bool {
        self.error == 0
    }
    /// Borrows all accumulated common parts; `intersection.common_parts()`.
    pub fn common_parts(&self) -> &[CommonPrt] {
        &self.parts
    }
    /// Enables native 24-point coincidence sampling; `intersection.use_quick_coincidence_check(true)`.
    pub fn use_quick_coincidence_check(&mut self, quick: bool) {
        self.quick = quick;
    }
    /// Reads the quick-check flag; `intersection.is_coincidence_checked_quickly()`.
    pub fn is_coincidence_checked_quickly(&self) -> bool {
        self.quick
    }
    /// Computes intersections without resetting retained errors or results; `intersection.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.perform_native()
    }
}
