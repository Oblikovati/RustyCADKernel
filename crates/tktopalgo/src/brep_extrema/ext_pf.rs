// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ExtPF.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_ExtPF`: the extrema between a vertex and a face.
use super::extrema_capture::sequence_value;
use crate::brep_class::BRepClassFaceClassifier;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPS, POnSurf};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};

/// `BRepExtrema_ExtPF` (docs/topalgo-design.md section 13): `Extrema_ExtPS` on the
/// surface of a face over its UV bounds; the extrema whose point the face classifier
/// finds IN or ON the face are kept.
///
/// The surface adaptor is shared with the `Extrema_ExtPS` member (a handle here, the
/// member adaptor `mySurf` natively), so the member keeps its state from one call to
/// the next, as natively.
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaExtPF;
/// let ext = BRepExtremaExtPF::new();
/// assert!(!ext.is_done());
/// assert_eq!(ext.nb_ext(), 0);
/// ```
pub struct BRepExtremaExtPF {
    my_ext_ps: ExtPS<'static>,
    my_sq_dist: Vec<f64>,
    my_points: Vec<POnSurf>,
    my_surf: Handle<BRepAdaptorSurface>,
}

impl Default for BRepExtremaExtPF {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepExtremaExtPF {
    /// `BRepExtrema_ExtPF()`: no face; `BRepExtremaExtPF::new()`.
    pub fn new() -> Self {
        Self {
            my_ext_ps: ExtPS::new(),
            my_sq_dist: Vec::new(),
            my_points: Vec::new(),
            my_surf: Handle::new(BRepAdaptorSurface::default()),
        }
    }

    /// `BRepExtrema_ExtPF(TheVertex, TheFace, TheFlag, TheAlgo)` (the native defaults
    /// are MINMAX and Grad): `Initialize` then `Perform`;
    /// `BRepExtremaExtPF::with_vertex_face(&v, &f, ExtFlag::MinMax, ExtAlgo::Grad)?`.
    pub fn with_vertex_face(
        vertex: &Vertex,
        face: &Face,
        flag: ExtFlag,
        algo: ExtAlgo,
    ) -> Result<Self, Failure> {
        let mut ext: Self = Self::new();
        ext.initialize(face, flag, algo)?;
        ext.perform(vertex, face)?;
        Ok(ext)
    }

    /// `Initialize(TheFace, TheFlag, TheAlgo)`: the unrestricted adaptor of the face
    /// (kept even when the rest is skipped), then, for a geometric surface, the flag,
    /// the algorithm and `Extrema_ExtPS` on the UV bounds of the face with the U and V
    /// resolutions of min(face tolerance, Confusion), at least PConfusion;
    /// `ext.initialize(&face, ExtFlag::MinMax, ExtAlgo::Grad)?`.
    pub fn initialize(&mut self, face: &Face, flag: ExtFlag, algo: ExtAlgo) -> Result<(), Failure> {
        // The native member adaptor is re-initialized in place, as the shared one here.
        self.my_surf.initialize(face, false)?;
        if self.my_surf.get_type()? == SurfaceType::OtherSurface {
            return Ok(());
        }
        let tol: f64 = BRepTool::face_tolerance(face)?.min(precision::confusion());
        let tol_u: f64 = self
            .my_surf
            .u_resolution(tol)?
            .max(precision::p_confusion());
        let tol_v: f64 = self
            .my_surf
            .v_resolution(tol)?
            .max(precision::p_confusion());
        let (u1, u2, v1, v2) = brep_tools::uv_bounds(face)?;
        self.my_ext_ps.set_flag(flag);
        self.my_ext_ps.set_algo(algo);
        self.my_ext_ps.initialize_shared(
            &self.my_surf.clone().cast(),
            (u1, u2),
            (v1, v2),
            (tol_u, tol_v),
        )
    }

    /// `Perform(TheVertex, TheFace)`: the extrema from the vertex point, kept when the
    /// face classifier (with the face tolerance) finds their point IN or ON `face`
    /// (the face only classifies: the surface is the initialized one);
    /// `ext.perform(&vertex, &face)?`.
    pub fn perform(&mut self, vertex: &Vertex, face: &Face) -> Result<(), Failure> {
        self.my_sq_dist.clear();
        self.my_points.clear();
        let point: Point3 = BRepTool::pnt(vertex)?;
        if self.my_surf.get_type()? == SurfaceType::OtherSurface {
            return Ok(());
        }
        self.my_ext_ps.perform(point)?;
        if !self.my_ext_ps.is_done() {
            return Ok(());
        }
        let mut classifier = BRepClassFaceClassifier::new();
        let tol: f64 = BRepTool::face_tolerance(face)?;
        for i in 1..=self.my_ext_ps.nb_ext()? {
            let extremum: POnSurf = self.my_ext_ps.point(i)?;
            let (u, v) = extremum.parameter();
            classifier.perform(face, &Point2::new(u, v), tol, false, 0.1)?;
            if matches!(classifier.state(), State::On | State::In) {
                self.my_sq_dist.push(self.my_ext_ps.square_distance(i)?);
                self.my_points.push(self.my_ext_ps.point(i)?);
            }
        }
        Ok(())
    }

    /// `IsDone()`: the state of the `Extrema_ExtPS` member; `ext.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_ext_ps.is_done()
    }

    /// `NbExt()`: the number of kept extrema; `ext.nb_ext()`.
    pub fn nb_ext(&self) -> i32 {
        self.my_points.len() as i32
    }

    /// `SquareDistance(N)`: the `N`-th kept square distance (Standard_OutOfRange outside
    /// `[1, NbExt]`); `ext.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        sequence_value("BRepExtrema_ExtPF::mySqDist", &self.my_sq_dist, n)
    }

    /// `Parameter(N, U, V)`: the face parameters of the `N`-th kept extremum;
    /// `let (u, v) = ext.parameter(1)?;`.
    pub fn parameter(&self, n: i32) -> Result<(f64, f64), Failure> {
        Ok(sequence_value("BRepExtrema_ExtPF::myPoints", &self.my_points, n)?.parameter())
    }

    /// `Point(N)`: the point on the face of the `N`-th kept extremum; `ext.point(1)?`.
    pub fn point(&self, n: i32) -> Result<Point3, Failure> {
        Ok(*sequence_value("BRepExtrema_ExtPF::myPoints", &self.my_points, n)?.value())
    }

    /// `SetFlag(F)`: the flag of the member for the next `Perform`;
    /// `ext.set_flag(ExtFlag::Min)`.
    pub fn set_flag(&mut self, flag: ExtFlag) {
        self.my_ext_ps.set_flag(flag);
    }

    /// `SetAlgo(A)`: the algorithm of the member for the next `Perform`;
    /// `ext.set_algo(ExtAlgo::Tree)`.
    pub fn set_algo(&mut self, algo: ExtAlgo) {
        self.my_ext_ps.set_algo(algo);
    }
}
