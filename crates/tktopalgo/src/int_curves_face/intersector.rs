// Rust translation of IntCurvesFace_Intersector.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurvesFace_Intersector`: the intersection points of a line or a curve with a
//! face (IN or ON its boundary), sorted by their parameter on the curve.
use super::sample_pars::compute_sample_pars;
use crate::brep_top_adaptor::BRepTopAdaptorTopolTool;
use crate::brep_top_adaptor::errors::c_int_cast;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeomalgo::int_curve_surface::{
    IntersectionPoint, ThePolyhedronOfHInter, TransitionOnCurve,
};
use rustycad_tkmath::bnd::{BoundSortBox, BoundingBox3};
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{Point2, Point3};

/// `IntCurvesFace_Intersector` (docs/topalgo-design.md section 4): a face, its
/// surface adaptor and topology tool, and a polyhedron of the surface for the
/// non-elementary surfaces; each Perform keeps the intersection points whose UV point
/// is IN or ON the face, sorted by their parameter on the curve.
///
/// ```
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tktopalgo::int_curves_face::IntCurvesFaceIntersector;
/// // A null face gives an intersector that is never ready.
/// let intersector = IntCurvesFaceIntersector::new(&Face::default(), 1e-7, true, true);
/// assert!(intersector.is_err() || !intersector.unwrap().is_done());
/// ```
pub struct IntCurvesFaceIntersector {
    parent: StandardTransient,
    pub(super) my_topol_tool: Handle<BRepTopAdaptorTopolTool>,
    pub(super) hsurface: Handle<dyn Adaptor3dSurface>,
    pub(super) tol: f64,
    pub(super) seq_pnt: Vec<IntersectionPoint>,
    pub(super) my_seq_state: Vec<i32>,
    pub(super) done: bool,
    pub(super) my_ready: bool,
    pub(super) nbpnt: i32,
    pub(super) face: Face,
    pub(super) my_polyhedron: Option<ThePolyhedronOfHInter>,
    pub(super) my_bnd_bounding: Option<BoundSortBox>,
    pub(super) my_use_bound_tol: bool,
    pub(super) my_is_parallel: bool,
}
impl_transient!(
    IntCurvesFaceIntersector,
    "IntCurvesFace_Intersector",
    StandardTransient,
    parent
);

/// The polyhedron sample limits of the constructor.
const THRESHOLD: f64 = 100.0;
const MAX_SAMPLES: i32 = 40;

impl IntCurvesFaceIntersector {
    /// `IntCurvesFace_Intersector(F, aTol, aRestr, UseBToler)`: loads the face (its
    /// surface restricted to the face UV bounds with `restr`); `tol` lets a point
    /// just before the curve start count; with `use_b_toler` a UV point is classified
    /// with the tolerances of the face edges, else with a null tolerance. A surface
    /// other than a plane or a quadric gets a polyhedron of 20 to 40 samples per
    /// direction (rebalanced when one direction is 100 times longer); a surface
    /// without extension in a direction leaves the intersector never done;
    /// `IntCurvesFaceIntersector::new(&face, 1e-7, true, true)?`.
    pub fn new(face: &Face, tol: f64, restr: bool, use_b_toler: bool) -> Result<Self, Failure> {
        let surface = BRepAdaptorSurface::with_face(face, restr)?;
        let hsurface: Handle<dyn Adaptor3dSurface> = Handle::new(surface).cast();
        let topol_tool = BRepTopAdaptorTopolTool::with_surface(&hsurface)?;
        let mut intersector = Self {
            parent: StandardTransient::default(),
            my_topol_tool: Handle::new(topol_tool),
            hsurface,
            tol,
            seq_pnt: Vec::new(),
            my_seq_state: Vec::new(),
            done: false,
            my_ready: false,
            nbpnt: 0,
            face: face.clone(),
            my_polyhedron: None,
            my_bnd_bounding: None,
            my_use_bound_tol: use_b_toler,
            my_is_parallel: false,
        };
        intersector.my_ready = intersector.build_polyhedron()?;
        Ok(intersector)
    }

    /// The polyhedron of a surface that is not a plane or a quadric; false when the
    /// surface has no extension along a direction (the intersector is not ready).
    fn build_polyhedron(&mut self) -> Result<bool, Failure> {
        let kind: SurfaceType = self.hsurface.get_type()?;
        if matches!(
            kind,
            SurfaceType::Plane
                | SurfaceType::Cylinder
                | SurfaceType::Cone
                | SurfaceType::Sphere
                | SurfaceType::Torus
        ) {
            return Ok(true);
        }
        let s: &dyn Adaptor3dSurface = &*self.hsurface;
        let (u0, u1) = (s.first_u_parameter()?, s.last_u_parameter()?);
        let (v0, v1) = (s.first_v_parameter()?, s.last_v_parameter()?);
        let tool: &BRepTopAdaptorTopolTool = &self.my_topol_tool;
        let samples = (tool.nb_samples_u()?, tool.nb_samples_v()?);
        let (a_u_res, a_v_res) = (s.u_resolution(1.0)?, s.v_resolution(1.0)?);
        let lengths = ((u1 - u0) / a_u_res, (v1 - v0) / a_v_res);
        let Some((nbsu, nbsv)) = balanced_samples(samples, lengths) else {
            return Ok(false); // surface has no extension along one of directions
        };
        let polyhedron = if s.nb_u_intervals(Shape::C2)? > 1 || s.nb_v_intervals(Shape::C2)? > 1 {
            let (upars, vpars) = compute_sample_pars(s, nbsu, nbsv)?;
            ThePolyhedronOfHInter::with_parameters(&self.hsurface, &upars, &vpars)?
        } else {
            ThePolyhedronOfHInter::new(&self.hsurface, nbsu, nbsv, u0, v0, u1, v1)?
        };
        self.my_polyhedron = Some(polyhedron);
        Ok(true)
    }

    /// `SurfaceType()`: the kind of the face surface; `intersector.surface_type()?`.
    pub fn surface_type(&self) -> Result<SurfaceType, Failure> {
        self.hsurface.get_type()
    }

    /// `IsDone()`: whether the last Perform ran (the intersector is ready);
    /// `intersector.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `NbPnt()`; `intersector.nb_pnt()`.
    pub fn nb_pnt(&self) -> i32 {
        self.nbpnt
    }

    // The i-th point (1-based); Standard_OutOfRange outside [1, NbPnt].
    fn point_at(&self, index: i32) -> Result<&IntersectionPoint, Failure> {
        self.seq_pnt
            .get(sequence_slot(index, self.seq_pnt.len())?)
            .ok_or_else(|| out_of_range(index, self.seq_pnt.len()))
    }

    /// `UParameter(I)`: the U parameter of the i-th point on the surface;
    /// Standard_OutOfRange outside `[1, NbPnt]`; `intersector.u_parameter(1)?`.
    pub fn u_parameter(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.point_at(index)?.u())
    }

    /// `VParameter(I)`; `intersector.v_parameter(1)?`.
    pub fn v_parameter(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.point_at(index)?.v())
    }

    /// `WParameter(I)`: the parameter of the i-th point on the curve;
    /// `intersector.w_parameter(1)?`.
    pub fn w_parameter(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.point_at(index)?.w())
    }

    /// `Pnt(I)`: the i-th point; `intersector.pnt(1)?`.
    pub fn pnt(&self, index: i32) -> Result<Point3, Failure> {
        Ok(self.point_at(index)?.pnt())
    }

    /// `Transition(I)`: the transition of the curve at the i-th point (IN and OUT
    /// swapped on a REVERSED face); `intersector.transition(1)?`.
    pub fn transition(&self, index: i32) -> Result<TransitionOnCurve, Failure> {
        Ok(self.point_at(index)?.transition())
    }

    /// `State(I)`: IN or ON (on a boundary of the face) for the i-th point;
    /// `intersector.state(1)?`.
    pub fn state(&self, index: i32) -> Result<State, Failure> {
        let slot: usize = sequence_slot(index, self.my_seq_state.len())?;
        Ok(if self.my_seq_state[slot] == 0 {
            State::In
        } else {
            State::On
        })
    }

    /// `IsParallel()`: whether a Perform found the curve parallel to (or on) the
    /// surface (never reset by a later Perform); `intersector.is_parallel()`.
    pub fn is_parallel(&self) -> bool {
        self.my_is_parallel
    }

    /// `Face()`; `intersector.face()`.
    pub fn face(&self) -> &Face {
        &self.face
    }

    /// `ClassifyUVPoint(Puv)`: the state of a UV point with the tolerance 1e-7 (the
    /// classifier keeps the first tolerance it was built with);
    /// `intersector.classify_uv_point(Point2::new(0.5, 0.5))?`.
    pub fn classify_uv_point(&self, point: Point2) -> Result<State, Failure> {
        self.my_topol_tool.classify(point, 1e-7, true)
    }

    /// `Bounding()`: the box of the polyhedron, void without one;
    /// `intersector.bounding()`.
    pub fn bounding(&self) -> BoundingBox3 {
        self.my_polyhedron
            .as_ref()
            .map_or_else(BoundingBox3::default, |polyhedron| {
                polyhedron.bounding().clone()
            })
    }

    /// `SetUseBoundToler(UseBToler)`; `intersector.set_use_bound_toler(false)`.
    pub fn set_use_bound_toler(&mut self, use_b_toler: bool) {
        self.my_use_bound_tol = use_b_toler;
    }

    /// `GetUseBoundToler()`; `intersector.get_use_bound_toler()`.
    pub fn get_use_bound_toler(&self) -> bool {
        self.my_use_bound_tol
    }
}

// The sample counts of the polyhedron: the tool's counts clamped to [20, 40], then,
// when one direction is 100 times longer, sqrt(dU / dV) * 40 in U (at least 10) and
// 1600 / nbsu in V (at least 10, then U is 160); None without extension.
fn balanced_samples((mut nbsu, mut nbsv): (i32, i32), (du, dv): (f64, f64)) -> Option<(i32, i32)> {
    let mut min_samples: i32 = 20;
    let max_samples2: i32 = MAX_SAMPLES * MAX_SAMPLES;
    nbsu = nbsu.max(min_samples).min(MAX_SAMPLES);
    nbsv = nbsv.max(min_samples).min(MAX_SAMPLES);
    // Checking correlation between number of samples and length of the face along each axis
    if !(du > precision::confusion() && dv > precision::confusion()) {
        return None;
    }
    let (longest, shortest) = if du < dv { (dv, du) } else { (du, dv) };
    if longest > shortest * THRESHOLD {
        min_samples = 10;
        nbsu = c_int_cast((du / dv).sqrt() * f64::from(MAX_SAMPLES));
        nbsu = nbsu.max(min_samples);
        nbsv = max_samples2 / nbsu;
        if nbsv < min_samples {
            nbsv = min_samples;
            nbsu = max_samples2 / min_samples;
        }
    }
    Some((nbsu, nbsv))
}

/// The 0-based slot of the 1-based `index` in a sequence of `len` items;
/// Standard_OutOfRange outside `[1, len]`.
pub(super) fn sequence_slot(index: i32, len: usize) -> Result<usize, Failure> {
    if index < 1 || index as usize > len {
        return Err(out_of_range(index, len));
    }
    Ok(index as usize - 1)
}

fn out_of_range(index: i32, len: usize) -> Failure {
    Failure::out_of_range(&format!(
        "NCollection_Sequence::Value: index {index} outside [1, {len}]"
    ))
}
