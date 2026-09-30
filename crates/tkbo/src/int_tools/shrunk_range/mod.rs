//! IntTools_ShrunkRange from pinned OCCT 3d097a0; reused boxes and failed-range writes are retained.
use super::Context;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve,
    topods::{Edge, Vertex},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::{bnd_lib::Add3dCurve, gcpnts::AbscissaPoint};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_lib::{RangeEnd, find_valid_range};
mod compute;
/// A working edge range outside both endpoint tolerance spheres.
///
/// `let range = ShrunkRange::new(); assert!(!range.is_done());`
pub struct ShrunkRange {
    edge: Edge,
    vertices: [Vertex; 2],
    parameters: [f64; 2],
    shrunk: [f64; 2],
    bounds: BoundingBox3,
    context: Handle<Context>,
    done: bool,
    splittable: bool,
    length: f64,
}
impl Default for ShrunkRange {
    fn default() -> Self {
        Self::new()
    }
}
impl ShrunkRange {
    /// Create the native sentinel range (-99, -99) and void box.
    ///
    /// `assert_eq!(ShrunkRange::new().shrunk_range(), (-99., -99.));`
    pub fn new() -> Self {
        Self {
            edge: Edge::default(),
            vertices: [Vertex::default(), Vertex::default()],
            parameters: [-99.; 2],
            shrunk: [-99.; 2],
            bounds: BoundingBox3::default(),
            context: Handle::default(),
            done: false,
            splittable: false,
            length: 0.,
        }
    }
    /// Select an edge interval, retaining the previous shrunk interval and box.
    ///
    /// `range.set_data(&edge, 0., 1., &first, &last);`
    pub fn set_data(&mut self, edge: &Edge, first: f64, last: f64, v1: &Vertex, v2: &Vertex) {
        self.edge = edge.clone();
        self.vertices = [v1.clone(), v2.clone()];
        self.parameters = [first, last];
        self.done = false;
        self.splittable = false;
        self.length = 0.;
    }
    /// Retain the supplied context handle even when null.
    ///
    /// `range.set_context(&context);`
    pub fn set_context(&mut self, context: &Handle<Context>) {
        self.context = context.clone();
    }
    /// Borrow the original context handle.
    ///
    /// `assert!(ShrunkRange::new().context().is_null());`
    pub fn context(&self) -> &Handle<Context> {
        &self.context
    }
    /// Borrow the selected edge with its original orientation and location.
    ///
    /// `let edge = range.edge();`
    pub fn edge(&self) -> &Edge {
        &self.edge
    }
    /// Return the last written shrunk parameters, including failed-search writes.
    ///
    /// `let (first,last) = range.shrunk_range();`
    pub fn shrunk_range(&self) -> (f64, f64) {
        (self.shrunk[0], self.shrunk[1])
    }
    /// Borrow the accumulated native box.
    ///
    /// `let bounds = range.bounding_box();`
    pub fn bounding_box(&self) -> &BoundingBox3 {
        &self.bounds
    }
    /// Set the parameters before adding the curve range to the retained box.
    ///
    /// `range.set_shrunk_range(0.2,0.8)?;`
    pub fn set_shrunk_range(&mut self, first: f64, last: f64) -> Result<(), Failure> {
        self.shrunk = [first, last];
        let curve = BRepAdaptorCurve::with_edge(&self.edge)?;
        Add3dCurve::add_range(&curve, first, last, 0., &mut self.bounds)
    }
    /// Test successful computation without requiring a splittable edge.
    ///
    /// `assert!(!ShrunkRange::new().is_done());`
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Test whether the computed interval can accommodate another tolerance sphere.
    ///
    /// `assert!(!ShrunkRange::new().is_splittable());`
    pub fn is_splittable(&self) -> bool {
        self.splittable
    }
    /// Return the most recently computed length, retained by a failing Perform.
    ///
    /// `assert_eq!(ShrunkRange::new().length(),0.);`
    pub fn length(&self) -> f64 {
        self.length
    }
}
