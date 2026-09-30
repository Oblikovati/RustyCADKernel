// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_HInterTool.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_HInterTool`: the surface sampling and the arc and vertex queries of the
//! surface-surface intersection.
use super::failures::null_object;
use crate::int_start::{TheArc, TheSOBTool, TheVertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeombase::extrema::EPCOfExtPC2d;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Point2, Point3};

/// `IntPatch_HInterTool`: static queries on surfaces, arcs and vertices, and the sample
/// points of a surface (the only members that keep state: the sampled range).
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchHInterTool;
/// let tool = IntPatchHInterTool::new();
/// assert_eq!(tool, IntPatchHInterTool::default());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IntPatchHInterTool {
    uinf: f64,
    vinf: f64,
    usup: f64,
    vsup: f64,
}

/// The adaptor behind an arc handle, or the Standard_NullObject of a null arc.
pub(crate) fn arc_of<'a>(
    arc: &'a TheArc,
    member: &str,
) -> Result<&'a dyn Adaptor2dCurve2d, Failure> {
    arc.get()
        .ok_or_else(|| null_object(member, "Adaptor2d_Curve2d"))
}

/// The sample count of a spline in one direction: knots times degree, doubled when not
/// rational, at least 4.
fn spline_samples(nb_knots: i32, degree: i32, rational: bool) -> i32 {
    let mut nbs: i32 = nb_knots * degree;
    if !rational {
        nbs *= 2;
    }
    if nbs < 4 {
        nbs = 4;
    }
    nbs
}

/// A range made finite for sampling: an infinite side is 2e5 from the other one, both
/// infinite sides give `[-1e5, 1e5]`.
fn finite_range(mut inf: f64, mut sup: f64) -> (f64, f64) {
    if sup < inf {
        (inf, sup) = (sup, inf);
    }
    if inf == real_first() && sup == real_last() {
        return (-1.0e5, 1.0e5);
    }
    if inf == real_first() {
        inf = sup - 2.0e5;
    } else if sup == real_last() {
        sup = inf + 2.0e5;
    }
    (inf, sup)
}

impl IntPatchHInterTool {
    /// `IntPatch_HInterTool()`: the sampled range is 0. `IntPatchHInterTool::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SingularOnUMin(S)`: always false. `IntPatchHInterTool::singular_on_u_min(&s)`.
    pub fn singular_on_u_min(_s: &Adaptor3dSurfaceHandle) -> bool {
        false
    }

    /// `SingularOnUMax(S)`: always false. `IntPatchHInterTool::singular_on_u_max(&s)`.
    pub fn singular_on_u_max(_s: &Adaptor3dSurfaceHandle) -> bool {
        false
    }

    /// `SingularOnVMin(S)`: always false. `IntPatchHInterTool::singular_on_v_min(&s)`.
    pub fn singular_on_v_min(_s: &Adaptor3dSurfaceHandle) -> bool {
        false
    }

    /// `SingularOnVMax(S)`: always false. `IntPatchHInterTool::singular_on_v_max(&s)`.
    pub fn singular_on_v_max(_s: &Adaptor3dSurfaceHandle) -> bool {
        false
    }

    /// `NbSamplesU(S, u1, u2)`: the U sample count by surface type (the range is not
    /// used). `IntPatchHInterTool::nb_samples_u(&s, 0.0, 1.0)?`.
    pub fn nb_samples_u(s: &Adaptor3dSurfaceHandle, _u1: f64, _u2: f64) -> Result<i32, Failure> {
        Ok(match s.get_type()? {
            SurfaceType::Plane => 2,
            SurfaceType::BezierSurface => 3 + s.nb_u_poles()?,
            SurfaceType::BSplineSurface => {
                spline_samples(s.nb_u_knots()?, s.u_degree()?, s.is_u_rational()?)
            }
            SurfaceType::Torus => 20,
            _ => 10,
        })
    }

    /// `NbSamplesV(S, v1, v2)`: the V sample count by surface type (the range is not
    /// used). `IntPatchHInterTool::nb_samples_v(&s, 0.0, 1.0)?`.
    pub fn nb_samples_v(s: &Adaptor3dSurfaceHandle, _v1: f64, _v2: f64) -> Result<i32, Failure> {
        Ok(match s.get_type()? {
            SurfaceType::Plane => 2,
            SurfaceType::BezierSurface => 3 + s.nb_v_poles()?,
            SurfaceType::BSplineSurface => {
                spline_samples(s.nb_v_knots()?, s.v_degree()?, s.is_v_rational()?)
            }
            SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
            | SurfaceType::SurfaceOfRevolution
            | SurfaceType::SurfaceOfExtrusion => 15,
            _ => 10,
        })
    }

    /// `NbSamplePoints(S)`: keeps the parametric range of `s` made finite and returns
    /// the number of sample points on it. `tool.nb_sample_points(&s)?`.
    pub fn nb_sample_points(&mut self, s: &Adaptor3dSurfaceHandle) -> Result<i32, Failure> {
        let (u0, u1) = (s.first_u_parameter()?, s.last_u_parameter()?);
        let (v0, v1) = (s.first_v_parameter()?, s.last_v_parameter()?);
        (self.uinf, self.usup) = finite_range(u0, u1);
        (self.vinf, self.vsup) = finite_range(v0, v1);
        let nu: i32 = Self::nb_samples_u(s, self.uinf, self.usup)?;
        let nv: i32 = Self::nb_samples_v(s, self.vinf, self.vsup)?;
        Ok((1 + nu / 2) * (1 + nv / 2))
    }

    /// `SamplePoint(S, Index, U, V)`: the sample point `index` of the range kept by
    /// `NbSamplePoints`, a slightly perturbed grid point (to avoid symmetry problems),
    /// or one of five fixed points when the grid has at most 5 points.
    /// `tool.sample_point(&s, 1)?`.
    pub fn sample_point(
        &self,
        s: &Adaptor3dSurfaceHandle,
        index: i32,
    ) -> Result<(f64, f64), Failure> {
        let nb_int_u: i32 = (1 + Self::nb_samples_u(s, self.uinf, self.usup)?) >> 1;
        let nb_int_v: i32 = (1 + Self::nb_samples_v(s, self.vinf, self.vsup)?) >> 1;
        if nb_int_u * nb_int_v > 5 {
            return Ok(self.grid_point(index, nb_int_u, nb_int_v));
        }
        Ok(self.sample_point_on_range(index))
    }

    fn grid_point(&self, index: i32, nb_int_u: i32, nb_int_v: i32) -> (f64, f64) {
        let nv: i32 = (index - 1) / nb_int_u;
        let nu: i32 = (index - 1) - nv * nb_int_u;
        let mut du: f64 = (self.usup - self.uinf) / f64::from(nb_int_u + 1);
        let mut dv: f64 = (self.vsup - self.vinf) / f64::from(nb_int_v + 1);
        match (nu + nv) & 3 {
            1 => {
                dv *= 1.001;
                dv *= 0.999;
            }
            2 => {
                du *= 1.001;
                dv *= 1.001;
            }
            3 => du *= 0.999,
            _ => {}
        }
        (
            self.uinf + du * f64::from(nu + 1),
            self.vinf + dv * f64::from(nv + 1),
        )
    }

    /// The five fixed sample points of `SamplePoint` on a small grid.
    fn sample_point_on_range(&self, index: i32) -> (f64, f64) {
        let (ui, us, vi, vs) = (self.uinf, self.usup, self.vinf, self.vsup);
        match index {
            1 => (0.76 * ui + 0.24 * us, 0.74 * vi + 0.26 * vs),
            2 => (0.73 * ui + 0.27 * us, 0.24 * vi + 0.76 * vs),
            3 => (0.25 * ui + 0.75 * us, 0.76 * vi + 0.24 * vs),
            4 => (0.26 * ui + 0.74 * us, 0.25 * vi + 0.75 * vs),
            _ => (0.51 * ui + 0.49 * us, 0.49 * vi + 0.51 * vs),
        }
    }

    /// `NbSamplesOnArc(A)`: the sample count of an arc by curve type.
    /// `IntPatchHInterTool::nb_samples_on_arc(&arc)?`.
    pub fn nb_samples_on_arc(a: &TheArc) -> Result<i32, Failure> {
        let arc: &dyn Adaptor2dCurve2d = arc_of(a, "IntPatch_HInterTool::NbSamplesOnArc")?;
        Ok(match arc.get_type()? {
            CurveType::Line => 2,
            CurveType::Circle | CurveType::Ellipse | CurveType::Hyperbola | CurveType::Parabola => {
                10
            }
            CurveType::BezierCurve => arc.nb_poles()?,
            CurveType::BSplineCurve => 2 + arc.nb_knots()? * arc.degree()?,
            _ => 10,
        })
    }

    /// `Bounds(C, Ufirst, Ulast)`: the parametric range of the arc.
    /// `IntPatchHInterTool::bounds(&arc)?`.
    pub fn bounds(c: &TheArc) -> Result<(f64, f64), Failure> {
        let arc: &dyn Adaptor2dCurve2d = arc_of(c, "IntPatch_HInterTool::Bounds")?;
        Ok((arc.first_parameter()?, arc.last_parameter()?))
    }

    /// `Project(C, P, Paramproj, Ptproj)`: the parameter and the point of the nearest
    /// extremum of `p` on the arc (20 samples), `None` when there is none.
    /// `IntPatchHInterTool::project(&arc, p)?`.
    pub fn project(c: &TheArc, p: Point2) -> Result<Option<(f64, Point2)>, Failure> {
        let arc: &dyn Adaptor2dCurve2d = arc_of(c, "IntPatch_HInterTool::Project")?;
        let (eps_x, nbu, tol) = (1.0e-8, 20, 1.0e-5);
        let extrema = EPCOfExtPC2d::with_curve(p, arc, nbu, eps_x, tol)?;
        if !extrema.is_done() {
            return Ok(None);
        }
        let nb_ext: i32 = extrema.nb_ext()?;
        if nb_ext == 0 {
            return Ok(None);
        }
        let mut indexmin: i32 = 1;
        let mut dist2: f64 = extrema.square_distance(1)?;
        for i in 2..=nb_ext {
            if extrema.square_distance(i)? < dist2 {
                indexmin = i;
                dist2 = extrema.square_distance(i)?;
            }
        }
        let point = extrema.point(indexmin)?;
        Ok(Some((point.parameter(), *point.value())))
    }

    /// `Tolerance(V, C)`: the parametric resolution of the vertex on the arc.
    /// `IntPatchHInterTool::tolerance(&vertex, &arc)?`.
    pub fn tolerance(v: &TheVertex, c: &TheArc) -> Result<f64, Failure> {
        let vertex = v
            .get()
            .ok_or_else(|| null_object("IntPatch_HInterTool::Tolerance", "Adaptor3d_HVertex"))?;
        vertex.resolution(c)
    }

    /// `Parameter(V, C)`: the parameter of the vertex on the arc.
    /// `IntPatchHInterTool::parameter(&vertex, &arc)?`.
    pub fn parameter(v: &TheVertex, c: &TheArc) -> Result<f64, Failure> {
        let vertex = v
            .get()
            .ok_or_else(|| null_object("IntPatch_HInterTool::Parameter", "Adaptor3d_HVertex"))?;
        vertex.parameter(c)
    }

    /// `HasBeenSeen(C)`: always false. `IntPatchHInterTool::has_been_seen(&arc)`.
    pub fn has_been_seen(_c: &TheArc) -> bool {
        false
    }

    /// `NbPoints(C)`: always 0. `IntPatchHInterTool::nb_points(&arc)`.
    pub fn nb_points(_c: &TheArc) -> i32 {
        0
    }

    /// `Value(C, Index, Pt, Tol, U)`: always Standard_OutOfRange (no known point).
    /// `IntPatchHInterTool::value(&arc, 1).is_err()`.
    pub fn value(_c: &TheArc, index: i32) -> Result<(Point3, f64, f64), Failure> {
        Err(no_known_item("Value", index))
    }

    /// `IsVertex(C, Index)`: always false. `IntPatchHInterTool::is_vertex(&arc, 1)`.
    pub fn is_vertex(_c: &TheArc, _index: i32) -> bool {
        false
    }

    /// `Vertex(C, Index, V)`: always Standard_OutOfRange.
    /// `IntPatchHInterTool::vertex(&arc, 1).is_err()`.
    pub fn vertex(_c: &TheArc, index: i32) -> Result<TheVertex, Failure> {
        Err(no_known_item("Vertex", index))
    }

    /// `NbSegments(C)`: always 0. `IntPatchHInterTool::nb_segments(&arc)`.
    pub fn nb_segments(_c: &TheArc) -> i32 {
        0
    }

    /// `HasFirstPoint(C, Index, IndFirst)`: always Standard_OutOfRange.
    /// `IntPatchHInterTool::has_first_point(&arc, 1).is_err()`.
    pub fn has_first_point(_c: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        Err(no_known_item("HasFirstPoint", index))
    }

    /// `HasLastPoint(C, Index, IndLast)`: always Standard_OutOfRange.
    /// `IntPatchHInterTool::has_last_point(&arc, 1).is_err()`.
    pub fn has_last_point(_c: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        Err(no_known_item("HasLastPoint", index))
    }

    /// `IsAllSolution(C)`: always false. `IntPatchHInterTool::is_all_solution(&arc)`.
    pub fn is_all_solution(_c: &TheArc) -> bool {
        false
    }
}

fn no_known_item(member: &str, index: i32) -> Failure {
    Failure::out_of_range(&format!(
        "IntPatch_HInterTool::{member}() - index {index}: an arc has no known point or segment"
    ))
}

/// `TheSOBTool` of `IntPatch_TheSOnBounds`: the static members above.
impl TheSOBTool for IntPatchHInterTool {
    fn parameter(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure> {
        IntPatchHInterTool::parameter(vertex, arc)
    }
    fn tolerance(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure> {
        IntPatchHInterTool::tolerance(vertex, arc)
    }
    fn has_been_seen(arc: &TheArc) -> Result<bool, Failure> {
        Ok(IntPatchHInterTool::has_been_seen(arc))
    }
    fn bounds(arc: &TheArc) -> Result<(f64, f64), Failure> {
        IntPatchHInterTool::bounds(arc)
    }
    fn nb_points(arc: &TheArc) -> Result<i32, Failure> {
        Ok(IntPatchHInterTool::nb_points(arc))
    }
    fn value(arc: &TheArc, index: i32) -> Result<(Point3, f64, f64), Failure> {
        IntPatchHInterTool::value(arc, index)
    }
    fn is_vertex(arc: &TheArc, index: i32) -> Result<bool, Failure> {
        Ok(IntPatchHInterTool::is_vertex(arc, index))
    }
    fn vertex(arc: &TheArc, index: i32) -> Result<TheVertex, Failure> {
        IntPatchHInterTool::vertex(arc, index)
    }
    fn nb_segments(arc: &TheArc) -> Result<i32, Failure> {
        Ok(IntPatchHInterTool::nb_segments(arc))
    }
    fn has_first_point(arc: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        IntPatchHInterTool::has_first_point(arc, index)
    }
    fn has_last_point(arc: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        IntPatchHInterTool::has_last_point(arc, index)
    }
    fn is_all_solution(arc: &TheArc) -> Result<bool, Failure> {
        Ok(IntPatchHInterTool::is_all_solution(arc))
    }
}
