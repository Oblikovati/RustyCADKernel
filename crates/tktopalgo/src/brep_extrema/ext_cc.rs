// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ExtCC.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_ExtCC`: the extrema between two edges.
use super::extrema_capture::IndexedCapture;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::extrema::{ExtCC, POnCurv};
use rustycad_tkmath::gp::Point3;

/// One edge given to the native `Extrema_ExtCC` member: its adaptor, range and
/// parameter tolerance.
#[derive(Clone)]
struct EdgeCurve {
    curve: Handle<BRepAdaptorCurve>,
    range: (f64, f64),
    tol: f64,
}

impl EdgeCurve {
    /// The adaptor of a geometric edge, its range and the curve resolution of
    /// min(edge tolerance, Confusion), at least PConfusion.
    fn of(edge: &Edge) -> Result<Self, Failure> {
        let curve = BRepAdaptorCurve::with_edge(edge)?;
        let tol: f64 = BRepTool::edge_tolerance(edge)?.min(precision::confusion());
        let tol: f64 = curve.resolution(tol)?.max(precision::p_confusion());
        Ok(Self {
            curve: Handle::new(curve),
            range: BRepTool::range(edge)?,
            tol,
        })
    }

    /// `SetCurve(rank, C, Uinf, Usup)` then `SetTolerance(rank, Tol)`.
    fn load<'a>(&'a self, ext: &mut ExtCC<'a>, rank: i32) -> Result<(), Failure> {
        let curve: &'a dyn Adaptor3dCurve = &*self.curve;
        ext.set_curve_range(rank, curve, self.range.0, self.range.1)?;
        ext.set_tolerance(rank, self.tol)
    }
}

/// One call made on the native `Extrema_ExtCC` member.
#[derive(Clone)]
enum CurveCurveCall {
    /// `Initialize(E2)`: the second curve.
    Second(EdgeCurve),
    /// `Perform(E1)`: the first curve, then `Perform()`.
    Perform(EdgeCurve),
}

/// The accessors of the `Extrema_ExtCC` member after the last call.
#[derive(Clone)]
struct CurveCurveResults {
    done: bool,
    is_parallel: Result<bool, Failure>,
    nb_ext: Result<i32, Failure>,
    square_distance: IndexedCapture<f64>,
    points: IndexedCapture<(POnCurv, POnCurv)>,
    trimmed: ([f64; 4], [Point3; 4]),
}

impl CurveCurveResults {
    fn read(ext: &ExtCC<'_>) -> Self {
        let count: i32 = ext.nb_ext().unwrap_or(0);
        Self {
            done: ext.is_done(),
            is_parallel: ext.is_parallel(),
            nb_ext: ext.nb_ext(),
            square_distance: IndexedCapture::capture(count, |n| ext.square_distance(n)),
            points: IndexedCapture::capture(count, |n| ext.points(n)),
            trimmed: ext.trimmed_square_distances(),
        }
    }
}

/// The `Extrema_ExtCC()` of the native member: tolerances 1e-10.
fn default_extrema<'a>() -> ExtCC<'a> {
    ExtCC::new(1.0e-10, 1.0e-10)
}

/// `BRepExtrema_ExtCC` (docs/topalgo-design.md section 13): `Extrema_ExtCC` between
/// the curves of two edges over their ranges, with their parameter tolerances.
///
/// As [`super::BRepExtremaExtPC`], this object records the calls made on the native
/// `Extrema_ExtCC` member and replays them on a new Extrema object at each call, so
/// the member keeps its native state from one call to the next (the solutions of its
/// general algorithm accumulate, the end points of an infinite range are kept).
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaExtCC;
/// let ext = BRepExtremaExtCC::new();
/// assert!(!ext.is_done());
/// assert!(ext.is_parallel().is_err());
/// ```
#[derive(Clone)]
pub struct BRepExtremaExtCC {
    // The adaptors of `myHC` and of the local `HC` of each Perform live in the calls.
    calls: Vec<CurveCurveCall>,
    results: CurveCurveResults,
}

impl Default for BRepExtremaExtCC {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepExtremaExtCC {
    /// `BRepExtrema_ExtCC()`: no edge; `BRepExtremaExtCC::new()`.
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
            results: CurveCurveResults::read(&default_extrema()),
        }
    }

    /// `BRepExtrema_ExtCC(E1, E2)`: `Initialize(E2)` then `Perform(E1)`;
    /// `BRepExtremaExtCC::with_edges(&e1, &e2)?`.
    pub fn with_edges(e1: &Edge, e2: &Edge) -> Result<Self, Failure> {
        let mut ext: Self = Self::new();
        ext.initialize(e2)?;
        ext.perform(e1)?;
        Ok(ext)
    }

    /// `Initialize(E2)`: the second edge of the extrema; an edge without a 3D curve or
    /// curve on surface changes nothing; `ext.initialize(&e2)?`.
    pub fn initialize(&mut self, e2: &Edge) -> Result<(), Failure> {
        if !BRepTool::is_geometric_edge(e2)? {
            return Ok(());
        }
        let second: EdgeCurve = EdgeCurve::of(e2)?;
        self.call(CurveCurveCall::Second(second))
    }

    /// `Perform(E1)`: the extrema between `e1` and the initialized edge; an edge without
    /// a 3D curve or curve on surface changes nothing; Standard_NullObject when no
    /// second edge was initialized; `ext.perform(&e1)?`.
    pub fn perform(&mut self, e1: &Edge) -> Result<(), Failure> {
        if !BRepTool::is_geometric_edge(e1)? {
            return Ok(());
        }
        let first: EdgeCurve = EdgeCurve::of(e1)?;
        self.call(CurveCurveCall::Perform(first))
    }

    /// Records `call`, replays every call on a new `Extrema_ExtCC` and reads it; the
    /// result is the one of `call`.
    fn call(&mut self, call: CurveCurveCall) -> Result<(), Failure> {
        self.calls.push(call);
        let mut ext: ExtCC<'_> = default_extrema();
        let mut last: Result<(), Failure> = Ok(());
        for call in &self.calls {
            last = match call {
                CurveCurveCall::Second(second) => second.load(&mut ext, 2),
                CurveCurveCall::Perform(first) => {
                    first.load(&mut ext, 1).and_then(|()| ext.perform())
                }
            };
        }
        self.results = CurveCurveResults::read(&ext);
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

    /// `IsParallel()`: true when the edges are parallel (StdFail_NotDone when not
    /// done); `ext.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        self.results.is_parallel.clone()
    }

    /// `SquareDistance(N)`: the `N`-th extremum square distance;
    /// `ext.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        self.results.square_distance.get(n)
    }

    /// `ParameterOnE1(N)`: the parameter on the first edge of the `N`-th extremum;
    /// `ext.parameter_on_e1(1)?`.
    pub fn parameter_on_e1(&self, n: i32) -> Result<f64, Failure> {
        Ok(self.results.points.get(n)?.0.parameter())
    }

    /// `PointOnE1(N)`: the point on the first edge of the `N`-th extremum;
    /// `ext.point_on_e1(1)?`.
    pub fn point_on_e1(&self, n: i32) -> Result<Point3, Failure> {
        Ok(*self.results.points.get(n)?.0.value())
    }

    /// `ParameterOnE2(N)`: the parameter on the second edge of the `N`-th extremum;
    /// `ext.parameter_on_e2(1)?`.
    pub fn parameter_on_e2(&self, n: i32) -> Result<f64, Failure> {
        Ok(self.results.points.get(n)?.1.parameter())
    }

    /// `PointOnE2(N)`: the point on the second edge of the `N`-th extremum;
    /// `ext.point_on_e2(1)?`.
    pub fn point_on_e2(&self, n: i32) -> Result<Point3, Failure> {
        Ok(*self.results.points.get(n)?.1.value())
    }

    /// `TrimmedSquareDistances(dist11, distP12, distP21, distP22, P11, P12, P21, P22)`:
    /// the square distances between the end points of the two ranges and those points,
    /// as `([d11, d12, d21, d22], [P11, P12, P21, P22])`;
    /// `let (distances, points) = ext.trimmed_square_distances();`.
    pub fn trimmed_square_distances(&self) -> ([f64; 4], [Point3; 4]) {
        self.results.trimmed
    }
}
