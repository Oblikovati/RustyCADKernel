//! Edge/face common-part construction, with unsupported intersection families explicit.
mod classify;
mod perform;
mod quick_coincidence;
mod touch;

use super::{CommonPrt, Context, Range};
use rustycad_tkbrep::{
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface},
    topods::{Edge, Face},
};
use rustycad_tkernel::{ncollection::Sequence, standard::Handle};

/// `IntTools_EdgeFace`; configure shapes/range, then call `perform()`.
pub struct EdgeFace {
    edge: Edge,
    face: Face,
    fuzzy_value: f64,
    curve: Handle<BRepAdaptorCurve>,
    surface: Handle<BRepAdaptorSurface>,
    criteria: f64,
    done: bool,
    error_status: i32,
    context: Handle<Context>,
    common_parts: Sequence<CommonPrt>,
    range: Range,
    quick_coincidence_check: bool,
    minimum_distance: f64,
}

impl Default for EdgeFace {
    fn default() -> Self {
        Self {
            edge: Edge::default(),
            face: Face::default(),
            fuzzy_value: rustycad_tkernel::precision::confusion(),
            curve: Handle::new(BRepAdaptorCurve::default()),
            surface: Handle::new(BRepAdaptorSurface::default()),
            criteria: 0.0,
            done: false,
            error_status: 1,
            context: Handle::default(),
            common_parts: Sequence::default(),
            range: Range::default(),
            quick_coincidence_check: false,
            minimum_distance: rustycad_tkernel::standard::real::real_last(),
        }
    }
}

impl EdgeFace {
    /// Creates the native initial state; `EdgeFace::new().error_status() == 1`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the edge while retaining prior results; `intersector.set_edge(&edge)`.
    pub fn set_edge(&mut self, edge: &Edge) {
        self.edge = edge.clone();
    }
    /// Borrows the configured edge; `intersector.edge().is_same(&edge)`.
    pub fn edge(&self) -> &Edge {
        &self.edge
    }
    /// Sets the face while retaining prior results; `intersector.set_face(&face)`.
    pub fn set_face(&mut self, face: &Face) {
        self.face = face.clone();
    }
    /// Borrows the configured face; `intersector.face().is_same(&face)`.
    pub fn face(&self) -> &Face {
        &self.face
    }
    /// Assigns the interval without sorting; `intersector.set_range_bounds(0.0, 1.0)`.
    pub fn set_range_bounds(&mut self, first: f64, last: f64) {
        self.range = Range::new(first, last);
    }
    /// Copies an interval; `intersector.set_range(&Range::new(0.0, 1.0))`.
    pub fn set_range(&mut self, range: &Range) {
        self.range = *range;
    }
    /// Borrows the current interval; `intersector.range().range()`.
    pub fn range(&self) -> &Range {
        &self.range
    }
    /// Applies native max(fuzz, Confusion) semantics; `intersector.set_fuzzy_value(1e-6)`.
    pub fn set_fuzzy_value(&mut self, fuzz: f64) {
        self.fuzzy_value = if fuzz < rustycad_tkernel::precision::confusion() {
            rustycad_tkernel::precision::confusion()
        } else {
            fuzz
        };
    }
    /// Reads the effective fuzzy value; `intersector.fuzzy_value()`.
    pub fn fuzzy_value(&self) -> f64 {
        self.fuzzy_value
    }
    /// Selects the quick coincidence predicate; `intersector.use_quick_coincidence_check(true)`.
    pub fn use_quick_coincidence_check(&mut self, enabled: bool) {
        self.quick_coincidence_check = enabled;
    }
    /// Reads whether the quick predicate is enabled; `intersector.is_coincidence_checked_quickly()`.
    pub fn is_coincidence_checked_quickly(&self) -> bool {
        self.quick_coincidence_check
    }
    /// Sets the shared context, including a null handle; `intersector.set_context(&context)`.
    pub fn set_context(&mut self, context: &Handle<Context>) {
        self.context = context.clone();
    }
    /// Borrows the current context handle; `intersector.context().is_null()`.
    pub fn context(&self) -> &Handle<Context> {
        &self.context
    }
    /// Reads completion; `intersector.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reads native error status; `intersector.error_status()`.
    pub fn error_status(&self) -> i32 {
        self.error_status
    }
    /// Borrows accumulated common parts; `intersector.common_parts().length()`.
    pub fn common_parts(&self) -> &Sequence<CommonPrt> {
        &self.common_parts
    }
    /// Reads the least distance reported by a completed BeanFace pass.
    /// `intersector.minimal_distance()`.
    pub fn minimal_distance(&self) -> f64 {
        self.minimum_distance
    }
}
