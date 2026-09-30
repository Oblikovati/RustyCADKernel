// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ExtCF.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_ExtCF`: the extrema between an edge and a face.
use super::extrema_capture::{IndexedCapture, sequence_value};
use crate::brep_top_adaptor::BRepTopAdaptorFClass2d;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeombase::extrema::{ExtCS, POnCurv, POnSurf};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};

/// One call made on the native `Extrema_ExtCS` member.
#[derive(Clone)]
enum CurveSurfaceCall {
    /// `myExtCS.Initialize(*myHS, U1, U2, V1, V2, aTolC, aTolS)`.
    Initialize {
        surface: Handle<BRepAdaptorSurface>,
        bounds: (f64, f64, f64, f64),
        tolerances: (f64, f64),
    },
    /// `myExtCS.Perform(*HC, U1, U2)`.
    Perform {
        curve: Handle<BRepAdaptorCurve>,
        range: (f64, f64),
    },
}

impl CurveSurfaceCall {
    fn apply<'a>(&'a self, ext: &mut ExtCS<'a>) -> Result<(), Failure> {
        match self {
            Self::Initialize {
                surface,
                bounds: (u1, u2, v1, v2),
                tolerances,
            } => {
                let surface: &'a dyn Adaptor3dSurface = &**surface;
                ext.initialize_range(surface, (*u1, *u2), (*v1, *v2), *tolerances)
            }
            Self::Perform { curve, range } => ext.perform(&**curve, range.0, range.1),
        }
    }
}

/// The accessors of the `Extrema_ExtCS` member after the last call.
#[derive(Clone)]
struct CurveSurfaceResults {
    done: bool,
    is_parallel: Result<bool, Failure>,
    nb_ext: Result<i32, Failure>,
    square_distance: IndexedCapture<f64>,
    points: IndexedCapture<(POnCurv, POnSurf)>,
}

impl CurveSurfaceResults {
    fn read(ext: &ExtCS<'_>) -> Self {
        let count: i32 = ext.nb_ext().unwrap_or(0);
        Self {
            done: ext.is_done(),
            is_parallel: ext.is_parallel(),
            nb_ext: ext.nb_ext(),
            square_distance: IndexedCapture::capture(count, |n| ext.square_distance(n)),
            points: IndexedCapture::capture(count, |n| ext.points(n)),
        }
    }
}

/// `BRepExtrema_ExtCF` (docs/topalgo-design.md section 13): `Extrema_ExtCS` between
/// the curve of an edge over its range and the surface of a face over its UV bounds;
/// the extrema whose surface point `BRepTopAdaptor_FClass2d` finds IN or ON the face
/// are kept (only the square distance in the parallel case).
///
/// As [`super::BRepExtremaExtPC`], the calls made on the native `Extrema_ExtCS`
/// member are recorded and replayed, so the member keeps its native state (its
/// parallel flag in particular) from one call to the next.
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaExtCF;
/// let ext = BRepExtremaExtCF::new();
/// assert!(!ext.is_done());
/// assert_eq!(ext.nb_ext(), 0);
/// ```
#[derive(Clone)]
pub struct BRepExtremaExtCF {
    calls: Vec<CurveSurfaceCall>,
    results: CurveSurfaceResults,
    my_sq_dist: Vec<f64>,
    my_points_on_s: Vec<POnSurf>,
    my_points_on_c: Vec<POnCurv>,
    my_hs: Handle<BRepAdaptorSurface>,
}

impl Default for BRepExtremaExtCF {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepExtremaExtCF {
    /// `BRepExtrema_ExtCF()`: no face; `BRepExtremaExtCF::new()`.
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
            results: CurveSurfaceResults::read(&ExtCS::new()),
            my_sq_dist: Vec::new(),
            my_points_on_s: Vec::new(),
            my_points_on_c: Vec::new(),
            my_hs: Handle::default(),
        }
    }

    /// `BRepExtrema_ExtCF(E, F)`: `Initialize(E, F)` then `Perform(E, F)`;
    /// `BRepExtremaExtCF::with_edge_face(&edge, &face)?`.
    pub fn with_edge_face(edge: &Edge, face: &Face) -> Result<Self, Failure> {
        let mut ext: Self = Self::new();
        ext.initialize(edge, face)?;
        ext.perform(edge, face)?;
        Ok(ext)
    }

    /// `Initialize(E, F)`: `Extrema_ExtCS` on the restricted surface of the face over
    /// its UV bounds, with the curve resolution of min(edge tolerance, Confusion) and
    /// the smaller surface resolution of min(face tolerance, Confusion), both at least
    /// PConfusion; nothing changes for a non-geometric surface or edge;
    /// `ext.initialize(&edge, &face)?`.
    pub fn initialize(&mut self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        let surf = BRepAdaptorSurface::with_face(face, true)?;
        if surf.get_type()? == SurfaceType::OtherSurface || !BRepTool::is_geometric_edge(edge)? {
            return Ok(());
        }
        let curve = BRepAdaptorCurve::with_edge(edge)?;
        self.my_hs = Handle::new(surf);
        let tol_s: f64 = BRepTool::face_tolerance(face)?.min(precision::confusion());
        let tol_s: f64 = self.my_hs.u_resolution(tol_s)?.min(self.my_hs.v_resolution(tol_s)?);
        let tol_s: f64 = tol_s.max(precision::p_confusion());
        let tol_c: f64 = BRepTool::edge_tolerance(edge)?.min(precision::confusion());
        let tol_c: f64 = curve.resolution(tol_c)?.max(precision::p_confusion());
        let call = CurveSurfaceCall::Initialize {
            surface: self.my_hs.clone(),
            bounds: brep_tools::uv_bounds(face)?,
            tolerances: (tol_c, tol_s),
        };
        self.call(call)
    }

    /// `Perform(E, F)`: the extrema between `edge` (over its range) and the initialized
    /// surface; `face` only classifies the extrema points; `ext.perform(&edge, &face)?`.
    pub fn perform(&mut self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        self.my_sq_dist.clear();
        self.my_points_on_s.clear();
        self.my_points_on_c.clear();
        if self.my_hs.is_null() {
            return Ok(());
        }
        let range: (f64, f64) = BRepTool::range(edge)?;
        let curve = Handle::new(BRepAdaptorCurve::with_edge(edge)?);
        self.call(CurveSurfaceCall::Perform { curve, range })?;
        if !self.results.done {
            return Ok(());
        }
        if self.results.is_parallel.clone()? {
            let square_distance: f64 = self.results.square_distance.get(1)?;
            self.my_sq_dist.push(square_distance);
            return Ok(());
        }
        self.keep_classified(face)
    }

    /// The exploration of the extrema: those whose surface point is ON or IN `face`
    /// (classified in its UV space without periodic adjustment).
    fn keep_classified(&mut self, face: &Face) -> Result<(), Failure> {
        let tol: f64 = BRepTool::face_tolerance(face)?;
        let classifier = BRepTopAdaptorFClass2d::new(face, tol)?;
        // The surface of a periodic face gives its points within the period.
        let is_adjust_periodic: bool = false;
        for i in 1..=self.results.nb_ext.clone()? {
            let (p1, p2) = self.results.points.get(i)?;
            let (u, v) = p2.parameter();
            let state: State = classifier.perform(Point2::new(u, v), is_adjust_periodic)?;
            if matches!(state, State::On | State::In) {
                self.my_sq_dist.push(self.results.square_distance.get(i)?);
                self.my_points_on_c.push(p1);
                self.my_points_on_s.push(p2);
            }
        }
        Ok(())
    }

    /// Records `call`, replays every call on a new `Extrema_ExtCS` and reads it; the
    /// result is the one of `call`.
    fn call(&mut self, call: CurveSurfaceCall) -> Result<(), Failure> {
        self.calls.push(call);
        let mut ext: ExtCS<'_> = ExtCS::new();
        let mut last: Result<(), Failure> = Ok(());
        for call in &self.calls {
            last = call.apply(&mut ext);
        }
        self.results = CurveSurfaceResults::read(&ext);
        last
    }

    /// `IsDone()`: the state of the `Extrema_ExtCS` member; `ext.is_done()`.
    pub fn is_done(&self) -> bool {
        self.results.done
    }

    /// `NbExt()`: the number of kept square distances; `ext.nb_ext()`.
    pub fn nb_ext(&self) -> i32 {
        self.my_sq_dist.len() as i32
    }

    /// `SquareDistance(N)`: the `N`-th kept square distance; `ext.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        sequence_value("BRepExtrema_ExtCF::mySqDist", &self.my_sq_dist, n)
    }

    /// `IsParallel()`: the parallel flag of the `Extrema_ExtCS` member
    /// (StdFail_NotDone when not done); `ext.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        self.results.is_parallel.clone()
    }

    /// `ParameterOnEdge(N)`: the edge parameter of the `N`-th kept extremum;
    /// `ext.parameter_on_edge(1)?`.
    pub fn parameter_on_edge(&self, n: i32) -> Result<f64, Failure> {
        Ok(sequence_value("BRepExtrema_ExtCF::myPointsOnC", &self.my_points_on_c, n)?.parameter())
    }

    /// `ParameterOnFace(N, U, V)`: the face parameters of the `N`-th kept extremum;
    /// `let (u, v) = ext.parameter_on_face(1)?;`.
    pub fn parameter_on_face(&self, n: i32) -> Result<(f64, f64), Failure> {
        Ok(sequence_value("BRepExtrema_ExtCF::myPointsOnS", &self.my_points_on_s, n)?.parameter())
    }

    /// `PointOnEdge(N)`: the edge point of the `N`-th kept extremum;
    /// `ext.point_on_edge(1)?`.
    pub fn point_on_edge(&self, n: i32) -> Result<Point3, Failure> {
        Ok(*sequence_value("BRepExtrema_ExtCF::myPointsOnC", &self.my_points_on_c, n)?.value())
    }

    /// `PointOnFace(N)`: the face point of the `N`-th kept extremum;
    /// `ext.point_on_face(1)?`.
    pub fn point_on_face(&self, n: i32) -> Result<Point3, Failure> {
        Ok(*sequence_value("BRepExtrema_ExtCF::myPointsOnS", &self.my_points_on_s, n)?.value())
    }
}
