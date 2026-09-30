// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ExtPC.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_ExtPC`: the extrema between a vertex and an edge.
use super::extrema_capture::IndexedCapture;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::extrema::{ExtPC, POnCurv};
use rustycad_tkmath::gp::Point3;

/// One call made on the native `Extrema_ExtPC` member.
#[derive(Clone)]
enum PointCurveCall {
    /// `myExtPC.Initialize(*myHC, U1, U2, Tol)`.
    Initialize {
        curve: Handle<BRepAdaptorCurve>,
        range: (f64, f64),
        tol: f64,
    },
    /// `myExtPC.Perform(P)`.
    Perform(Point3),
}

/// The accessors of the `Extrema_ExtPC` member after the last call.
#[derive(Clone)]
struct PointCurveResults {
    done: bool,
    nb_ext: Result<i32, Failure>,
    square_distance: IndexedCapture<f64>,
    is_min: IndexedCapture<bool>,
    point: IndexedCapture<POnCurv>,
    trimmed: (f64, f64, Point3, Point3),
}

impl PointCurveResults {
    fn read(ext: &ExtPC<'_>) -> Self {
        let count: i32 = ext.nb_ext().unwrap_or(0);
        Self {
            done: ext.is_done(),
            nb_ext: ext.nb_ext(),
            square_distance: IndexedCapture::capture(count, |n| ext.square_distance(n)),
            is_min: IndexedCapture::capture(count, |n| ext.is_min(n)),
            point: IndexedCapture::capture(count, |n| ext.point(n)),
            trimmed: ext.trimmed_square_distances(),
        }
    }
}

/// `BRepExtrema_ExtPC` (docs/topalgo-design.md section 13): `Extrema_ExtPC` on the 3D
/// curve of an edge over its range, with the parameter tolerance of the edge.
///
/// Natively the `Extrema_ExtPC` member keeps a pointer to the curve adaptor this
/// object owns, and keeps its state from one call to the next. The port's Extrema
/// object borrows its adaptor, so this object records the calls made on the member and
/// replays them on a new Extrema object at each call, then reads its accessors; the
/// member thus ends in the native state.
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaExtPC;
/// let ext = BRepExtremaExtPC::new();
/// assert!(!ext.is_done());
/// assert!(ext.nb_ext().is_err());
/// ```
#[derive(Clone)]
pub struct BRepExtremaExtPC {
    my_hc: Handle<BRepAdaptorCurve>,
    calls: Vec<PointCurveCall>,
    results: PointCurveResults,
}

impl Default for BRepExtremaExtPC {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepExtremaExtPC {
    /// `BRepExtrema_ExtPC()`: no edge; `BRepExtremaExtPC::new()`.
    pub fn new() -> Self {
        Self {
            my_hc: Handle::default(),
            calls: Vec::new(),
            results: PointCurveResults::read(&ExtPC::new()),
        }
    }

    /// `BRepExtrema_ExtPC(V, E)`: `Initialize(E)` then `Perform(V)`;
    /// `BRepExtremaExtPC::with_vertex_edge(&vertex, &edge)?`.
    pub fn with_vertex_edge(vertex: &Vertex, edge: &Edge) -> Result<Self, Failure> {
        let mut ext: Self = Self::new();
        ext.initialize(edge)?;
        ext.perform(vertex)?;
        Ok(ext)
    }

    /// `Initialize(E)`: the adaptor of the edge, the parameter tolerance (the curve
    /// resolution of min(edge tolerance, Confusion), at least PConfusion) and the edge
    /// range; an edge without a 3D curve or curve on surface changes nothing;
    /// `ext.initialize(&edge)?`.
    pub fn initialize(&mut self, edge: &Edge) -> Result<(), Failure> {
        if !BRepTool::is_geometric_edge(edge)? {
            return Ok(());
        }
        self.my_hc = Handle::new(BRepAdaptorCurve::with_edge(edge)?);
        let tol: f64 = BRepTool::edge_tolerance(edge)?.min(precision::confusion());
        let tol: f64 = self.my_hc.resolution(tol)?.max(precision::p_confusion());
        let range: (f64, f64) = BRepTool::range(edge)?;
        let call = PointCurveCall::Initialize {
            curve: self.my_hc.clone(),
            range,
            tol,
        };
        self.call(call)
    }

    /// `Perform(V)`: the extrema from the vertex point, when an edge was initialized;
    /// `ext.perform(&vertex)?`.
    pub fn perform(&mut self, vertex: &Vertex) -> Result<(), Failure> {
        if self.my_hc.is_null() {
            return Ok(());
        }
        let point: Point3 = BRepTool::pnt(vertex)?;
        self.call(PointCurveCall::Perform(point))
    }

    /// Records `call`, replays every call on a new `Extrema_ExtPC` and reads it; the
    /// result is the one of `call` (earlier calls raised when they were made).
    fn call(&mut self, call: PointCurveCall) -> Result<(), Failure> {
        self.calls.push(call);
        let mut ext: ExtPC<'_> = ExtPC::new();
        let mut last: Result<(), Failure> = Ok(());
        for call in &self.calls {
            last = match call {
                PointCurveCall::Initialize { curve, range, tol } => {
                    ext.initialize(&**curve as &dyn Adaptor3dCurve, range.0, range.1, *tol)
                }
                PointCurveCall::Perform(point) => ext.perform(*point),
            };
        }
        self.results = PointCurveResults::read(&ext);
        last
    }

    /// `IsDone()`: true when the distances are found; `ext.is_done()`.
    pub fn is_done(&self) -> bool {
        self.results.done
    }

    /// `NbExt()`: the number of extremum distances (StdFail_NotDone when not done);
    /// `ext.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        self.results.nb_ext.clone()
    }

    /// `IsMin(N)`: whether the `N`-th extremum distance is a minimum; `ext.is_min(1)?`.
    pub fn is_min(&self, n: i32) -> Result<bool, Failure> {
        self.results.is_min.get(n)
    }

    /// `SquareDistance(N)`: the `N`-th extremum square distance;
    /// `ext.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        self.results.square_distance.get(n)
    }

    /// `Parameter(N)`: the parameter on the edge of the `N`-th extremum;
    /// `ext.parameter(1)?`.
    pub fn parameter(&self, n: i32) -> Result<f64, Failure> {
        Ok(self.results.point.get(n)?.parameter())
    }

    /// `Point(N)`: the point on the edge of the `N`-th extremum; `ext.point(1)?`.
    pub fn point(&self, n: i32) -> Result<Point3, Failure> {
        Ok(*self.results.point.get(n)?.value())
    }

    /// `TrimmedSquareDistances(dist1, dist2, pnt1, pnt2)`: the square distances from
    /// the vertex to the points of the first and last parameters, and those points;
    /// `let (d1, d2, p1, p2) = ext.trimmed_square_distances();`.
    pub fn trimmed_square_distances(&self) -> (f64, f64, Point3, Point3) {
        self.results.trimmed
    }
}
