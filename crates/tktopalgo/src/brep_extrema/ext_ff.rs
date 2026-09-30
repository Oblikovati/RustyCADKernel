// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ExtFF.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_ExtFF`: the extrema between two faces.
use super::extrema_capture::{IndexedCapture, sequence_value};
use crate::brep_class::BRepClassFaceClassifier;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeombase::extrema::{ExtSS, POnSurf};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};

/// One face given to the native `Extrema_ExtSS` member: its adaptor, its UV bounds
/// `[U1, U2, V1, V2]` and its parameter tolerance.
#[derive(Clone)]
struct FaceSurface {
    surface: Handle<BRepAdaptorSurface>,
    bounds: [f64; 4],
    tol: f64,
}

impl FaceSurface {
    /// The adaptor of a face (already checked geometric), its UV bounds and the smaller
    /// resolution of min(face tolerance, Confusion), at least PConfusion.
    fn of(surface: BRepAdaptorSurface, face: &Face) -> Result<Self, Failure> {
        let tol: f64 = BRepTool::face_tolerance(face)?.min(precision::confusion());
        let tol: f64 = surface.u_resolution(tol)?.min(surface.v_resolution(tol)?);
        let tol: f64 = tol.max(precision::p_confusion());
        let (u1, u2, v1, v2) = brep_tools::uv_bounds(face)?;
        Ok(Self {
            surface: Handle::new(surface),
            bounds: [u1, u2, v1, v2],
            tol,
        })
    }

    fn adaptor(&self) -> &dyn Adaptor3dSurface {
        &*self.surface
    }
}

/// One call made on the native `Extrema_ExtSS` member.
#[derive(Clone)]
enum SurfaceSurfaceCall {
    /// `myExtSS.Initialize(*myHS, U1, U2, V1, V2, Tol)`.
    Initialize(FaceSurface),
    /// `myExtSS.Perform(*HS1, U1, U2, V1, V2, Tol1)`.
    Perform(FaceSurface),
}

impl SurfaceSurfaceCall {
    fn apply<'a>(&'a self, ext: &mut ExtSS<'a>) -> Result<(), Failure> {
        match self {
            Self::Initialize(second) => ext.initialize(second.adaptor(), second.bounds, second.tol),
            Self::Perform(first) => ext.perform(first.adaptor(), first.bounds, first.tol),
        }
    }
}

/// The accessors of the `Extrema_ExtSS` member after the last call.
#[derive(Clone)]
struct SurfaceSurfaceResults {
    done: bool,
    is_parallel: Result<bool, Failure>,
    nb_ext: Result<i32, Failure>,
    square_distance: IndexedCapture<f64>,
    points: IndexedCapture<(POnSurf, POnSurf)>,
}

impl SurfaceSurfaceResults {
    fn read(ext: &ExtSS<'_>) -> Self {
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

/// `BRepExtrema_ExtFF` (docs/topalgo-design.md section 13): `Extrema_ExtSS` between the
/// restricted surfaces of two faces over their UV bounds; the extrema whose points the
/// face classifier finds IN or ON both faces are kept (only the square distance in the
/// parallel case).
///
/// As [`super::BRepExtremaExtPC`], the calls made on the native `Extrema_ExtSS`
/// member are recorded and replayed, so the member keeps its native state (its
/// parallel flag in particular) from one call to the next.
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaExtFF;
/// let ext = BRepExtremaExtFF::new();
/// assert!(!ext.is_done());
/// assert!(ext.is_parallel().is_err());
/// ```
#[derive(Clone)]
pub struct BRepExtremaExtFF {
    calls: Vec<SurfaceSurfaceCall>,
    results: SurfaceSurfaceResults,
    my_sq_dist: Vec<f64>,
    my_points_on_s1: Vec<POnSurf>,
    my_points_on_s2: Vec<POnSurf>,
    my_hs: Handle<BRepAdaptorSurface>,
}

impl Default for BRepExtremaExtFF {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepExtremaExtFF {
    /// `BRepExtrema_ExtFF()`: no face; `BRepExtremaExtFF::new()`.
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
            results: SurfaceSurfaceResults::read(&ExtSS::new()),
            my_sq_dist: Vec::new(),
            my_points_on_s1: Vec::new(),
            my_points_on_s2: Vec::new(),
            my_hs: Handle::default(),
        }
    }

    /// `BRepExtrema_ExtFF(F1, F2)`: `Initialize(F2)` then `Perform(F1, F2)`;
    /// `BRepExtremaExtFF::with_faces(&f1, &f2)?`.
    pub fn with_faces(f1: &Face, f2: &Face) -> Result<Self, Failure> {
        let mut ext: Self = Self::new();
        ext.initialize(f2)?;
        ext.perform(f1, f2)?;
        Ok(ext)
    }

    /// `Initialize(F2)`: `Extrema_ExtSS` on the second face; nothing changes for a
    /// non-geometric surface; `ext.initialize(&f2)?`.
    pub fn initialize(&mut self, f2: &Face) -> Result<(), Failure> {
        let surf = BRepAdaptorSurface::with_face(f2, true)?;
        if surf.get_type()? == SurfaceType::OtherSurface {
            return Ok(());
        }
        let second: FaceSurface = FaceSurface::of(surf, f2)?;
        self.my_hs = second.surface.clone();
        self.call(SurfaceSurfaceCall::Initialize(second))
    }

    /// `Perform(F1, F2)`: the extrema between the first face and the initialized one;
    /// `f2` only classifies the points on the second surface;
    /// `ext.perform(&f1, &f2)?`.
    pub fn perform(&mut self, f1: &Face, f2: &Face) -> Result<(), Failure> {
        self.my_sq_dist.clear();
        self.my_points_on_s1.clear();
        self.my_points_on_s2.clear();
        let surf1 = BRepAdaptorSurface::with_face(f1, true)?;
        if self.my_hs.is_null() || surf1.get_type()? == SurfaceType::OtherSurface {
            return Ok(());
        }
        let first: FaceSurface = FaceSurface::of(surf1, f1)?;
        let tol1: f64 = first.tol;
        self.call(SurfaceSurfaceCall::Perform(first))?;
        if !self.results.done {
            return Ok(());
        }
        if self.results.is_parallel.clone()? {
            let square_distance: f64 = self.results.square_distance.get(1)?;
            self.my_sq_dist.push(square_distance);
            return Ok(());
        }
        self.keep_classified((f1, tol1), f2)
    }

    /// The exploration of the extrema: those whose points are ON or IN both faces. The
    /// first face is classified with its parametric tolerance `Tol1`, as natively.
    fn keep_classified(&mut self, (f1, tol1): (&Face, f64), f2: &Face) -> Result<(), Failure> {
        let mut classifier = BRepClassFaceClassifier::new();
        let tol2: f64 = BRepTool::face_tolerance(f2)?;
        for i in 1..=self.results.nb_ext.clone()? {
            let (p1, p2) = self.results.points.get(i)?;
            let (u1, v1) = p1.parameter();
            classifier.perform(f1, &Point2::new(u1, v1), tol1, false, 0.1)?;
            if !matches!(classifier.state(), State::On | State::In) {
                continue;
            }
            let (u2, v2) = p2.parameter();
            classifier.perform(f2, &Point2::new(u2, v2), tol2, false, 0.1)?;
            if matches!(classifier.state(), State::On | State::In) {
                self.my_sq_dist.push(self.results.square_distance.get(i)?);
                self.my_points_on_s1.push(p1);
                self.my_points_on_s2.push(p2);
            }
        }
        Ok(())
    }

    /// Records `call`, replays every call on a new `Extrema_ExtSS` and reads it; the
    /// result is the one of `call`.
    fn call(&mut self, call: SurfaceSurfaceCall) -> Result<(), Failure> {
        self.calls.push(call);
        let mut ext: ExtSS<'_> = ExtSS::new();
        let mut last: Result<(), Failure> = Ok(());
        for call in &self.calls {
            last = call.apply(&mut ext);
        }
        self.results = SurfaceSurfaceResults::read(&ext);
        last
    }

    /// `IsDone()`: the state of the `Extrema_ExtSS` member; `ext.is_done()`.
    pub fn is_done(&self) -> bool {
        self.results.done
    }

    /// `IsParallel()`: the parallel flag of the `Extrema_ExtSS` member
    /// (StdFail_NotDone when not done); `ext.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        self.results.is_parallel.clone()
    }

    /// `NbExt()`: the number of kept square distances; `ext.nb_ext()`.
    pub fn nb_ext(&self) -> i32 {
        self.my_sq_dist.len() as i32
    }

    /// `SquareDistance(N)`: the `N`-th kept square distance; `ext.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        sequence_value("BRepExtrema_ExtFF::mySqDist", &self.my_sq_dist, n)
    }

    /// `ParameterOnFace1(N, U, V)`: the first face parameters of the `N`-th kept
    /// extremum; `let (u, v) = ext.parameter_on_face1(1)?;`.
    pub fn parameter_on_face1(&self, n: i32) -> Result<(f64, f64), Failure> {
        Ok(sequence_value("BRepExtrema_ExtFF::myPointsOnS1", &self.my_points_on_s1, n)?.parameter())
    }

    /// `ParameterOnFace2(N, U, V)`: the second face parameters of the `N`-th kept
    /// extremum; `let (u, v) = ext.parameter_on_face2(1)?;`.
    pub fn parameter_on_face2(&self, n: i32) -> Result<(f64, f64), Failure> {
        Ok(sequence_value("BRepExtrema_ExtFF::myPointsOnS2", &self.my_points_on_s2, n)?.parameter())
    }

    /// `PointOnFace1(N)`: the first face point of the `N`-th kept extremum;
    /// `ext.point_on_face1(1)?`.
    pub fn point_on_face1(&self, n: i32) -> Result<Point3, Failure> {
        Ok(*sequence_value("BRepExtrema_ExtFF::myPointsOnS1", &self.my_points_on_s1, n)?.value())
    }

    /// `PointOnFace2(N)`: the second face point of the `N`-th kept extremum;
    /// `ext.point_on_face2(1)?`.
    pub fn point_on_face2(&self, n: i32) -> Result<Point3, Failure> {
        Ok(*sequence_value("BRepExtrema_ExtFF::myPointsOnS2", &self.my_points_on_s2, n)?.value())
    }
}
