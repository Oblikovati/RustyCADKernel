// Rust translation of the static Bubble, EvalPhase and EvalParameters
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimSurfaceTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersections of a bisector with the projection in the plane of an edge of a
//! face: (parameter on the bisector, on the edge, 0); for a degenerated edge, the angle
//! of the bisector start around the axis of the surface.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomGeometry, GeomPlane, GeomSurface, GeomSurfaceOfRevolution, GeomSweptSurface,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorCurve, GeomAdaptorSurface, GeomAdaptorSurfaceOfRevolution,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis3, Direction3, Point2, Point3, Vector3};
use std::f64::consts::PI;

/// `1.e-6`: the intersection tolerance (BRepFill_Precision natively commented out).
const TOL: f64 = 1.0e-6;

/// `Bubble(Seq)`: the points ordered by increasing X.
pub(super) fn bubble(seq: &mut [Point3]) {
    let mut invert = true;
    while invert {
        invert = false;
        for i in 1..seq.len() {
            if seq[i].x() < seq[i - 1].x() {
                seq.swap(i - 1, i);
                invert = true;
            }
        }
    }
}

/// `EvalPhase(Edge, Face, GAS, Axis)`: PI when the face is on the other side of the
/// axis than the point (0, V) of the surface, V the far end of another edge, else 0.
fn eval_phase(
    edge: &Edge,
    face: &Face,
    gas: &GeomAdaptorSurface,
    axis: &Axis3,
) -> Result<f64, Failure> {
    let (pe1, _) = BRepTool::uv_points_on_face(edge, face)?;
    let v_deg: f64 = pe1.y();
    let mut v: f64 = 0.;
    let mut exp = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while exp.more() {
        let other = topods::edge(exp.current())?;
        if !other.is_same(edge) {
            let (pf1, pf2) = BRepTool::uv_points_on_face(&other, face)?;
            v = if (pf1.y() - v_deg).abs() > (pf2.y() - v_deg).abs() {
                pf1.y()
            } else {
                pf2.y()
            };
            break;
        }
        exp.next()?;
    }
    let p: Point3 = gas.value(0., v)?;
    let side: f64 = Vector3::between(axis.location(), p).dot(Vector3::from(axis.x_direction()));
    Ok(if side < 0. { PI } else { 0. })
}

/// Whether the curve type can be extended (not a Bezier, B-spline, offset or other).
fn can_elongate(t: CurveType) -> bool {
    !matches!(
        t,
        CurveType::BezierCurve
            | CurveType::BSplineCurve
            | CurveType::OffsetCurve
            | CurveType::OtherCurve
    )
}

/// The edge projected in the XY plane.
fn projected_edge(edge: &Edge) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let ct = Handle::new(GeomTrimmedCurve::new(
        &found.curve,
        found.first,
        found.last,
        true,
        true,
    )?);
    ct.get()
        .ok_or_else(null_geometry)?
        .transform(found.location.transformation())?;
    let plane: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_coefficients(0., 0., 1., 0.)?).cast();
    geom_proj_lib::curve2d(&ct.cast(), &plane)
}

/// The intersections of the bisector and the projected edge; without point, both
/// extended by the tolerance (but splines and offsets) and intersected again with ten
/// times the tolerance. Returns the intersector and its count of points.
fn intersect(
    bis: &Handle<dyn Geom2dCurve>,
    c2d: &Handle<dyn Geom2dCurve>,
) -> Result<(GInter, i32), Failure> {
    let mut ac = Geom2dAdaptorCurve::with_curve(c2d)?;
    let mut a_bis = Geom2dAdaptorCurve::with_curve(bis)?;
    let intersector = done(GInter::between(&a_bis, &ac, 0., TOL)?)?;
    let nb_points: i32 = intersector.nb_points()?;
    if nb_points >= 1 {
        return Ok((intersector, nb_points));
    }
    // Try to elongate the curves and enlarge the tolerance (not from the beginning,
    // which would give extra solutions, an incoherent intersection).
    let (c_type, bis_type) = (ac.get_type()?, a_bis.get_type()?);
    if can_elongate(c_type) {
        ac = extended(c2d)?;
    }
    if can_elongate(bis_type) {
        a_bis = extended(bis)?;
    }
    let intersector = done(GInter::between(&a_bis, &ac, 0., TOL * 10.)?)?;
    let nb_points: i32 = intersector.nb_points()?;
    Ok((intersector, nb_points))
}

/// The trimmed curve extended by the tolerance at both ends (its trim changed in
/// place, as natively), as an adaptor.
fn extended(c: &Handle<dyn Geom2dCurve>) -> Result<Geom2dAdaptorCurve, Failure> {
    let trimmed: Handle<Geom2dTrimmedCurve> = c.cast();
    let t = trimmed.get().ok_or_else(null_geometry)?;
    t.set_trim(
        t.first_parameter() - TOL,
        t.last_parameter() + TOL,
        true,
        true,
    )?;
    Geom2dAdaptorCurve::with_curve(&trimmed.cast())
}

/// StdFail_NotDone of an intersection not done.
fn done(intersector: GInter) -> Result<GInter, Failure> {
    if intersector.is_done() {
        return Ok(intersector);
    }
    Err(Failure::with_type(
        "StdFail_NotDone",
        "BRepFill_TrimSurfaceTool::IntersectWith",
        "BRepFill_TrimSurfaceTool",
    ))
}

/// The points and the middles of the segments of the intersection, ordered on the
/// bisector, a point closer than the tolerance to the next removed (over the first
/// `nb_points` only, as natively).
fn intersection_points(intersector: &GInter, mut nb_points: i32) -> Result<Vec<Point3>, Failure> {
    let mut seq: Vec<Point3> = Vec::new();
    for i in 1..=nb_points {
        let p = intersector.point(i)?;
        seq.push(Point3::new(p.param_on_first(), p.param_on_second(), 0.));
    }
    for i in 1..=intersector.nb_segments()? {
        let seg = intersector.segment(i)?;
        let (fp, lp) = (seg.first_point()?, seg.last_point()?);
        seq.push(Point3::new(
            (fp.param_on_first() + lp.param_on_first()) / 2.,
            (fp.param_on_second() + lp.param_on_second()) / 2.,
            0.,
        ));
    }
    bubble(&mut seq);
    let mut i: i32 = 1;
    while i < nb_points {
        let (p1, p2) = (seq[(i - 1) as usize], seq[i as usize]);
        if p2.x() - p1.x() < TOL {
            seq.remove((i - 1) as usize);
            i -= 1;
            nb_points -= 1;
        }
        i += 1;
    }
    Ok(seq)
}

/// `EvalParameters(Edge, Face, Bis, Seq)`: the intersections of the bisector with the
/// projected edge, or for a degenerated edge at an end of the bisector the angle of
/// the bisector there.
pub(super) fn eval_parameters(
    edge: &Edge,
    face: &Face,
    bis: &Handle<dyn Geom2dCurve>,
) -> Result<Vec<Point3>, Failure> {
    if !BRepTool::degenerated(edge)? {
        let c2d = projected_edge(edge)?;
        let (intersector, nb_points) = intersect(bis, &c2d)?;
        return intersection_points(&intersector, nb_points);
    }
    degenerated_parameter(edge, face, bis)
}

/// The end of the bisector at the degenerated edge (the first, else the last), `None`
/// when neither is within the tolerance.
fn bisector_end(edge: &Edge, bis: &dyn Geom2dCurve) -> Result<Option<f64>, Failure> {
    let p3d = BRepTool::pnt(&topexp::first_vertex(edge, false))?;
    let p2d = Point2::new(p3d.x(), p3d.y());
    let u_first: f64 = bis.first_parameter();
    let p_bis: Point2 = bis.value(u_first)?;
    // An infinite point would overflow in gp_Pnt2d::Distance.
    let infinite: bool = precision::is_positive_infinite(p_bis.x().abs())
        || precision::is_positive_infinite(p_bis.y().abs());
    if !infinite && p_bis.distance(p2d) <= TOL {
        return Ok(Some(u_first));
    }
    let u_last: f64 = bis.last_parameter();
    if u_last >= precision::infinite() || bis.value(u_last)?.distance(p2d) > TOL {
        return Ok(None);
    }
    Ok(Some(u_last))
}

/// The axis of the surface and the phase of the face; Standard_NotImplemented for
/// another surface than a sphere, a cone, a torus, a cylinder or a revolution.
fn surface_axis(edge: &Edge, face: &Face) -> Result<(Axis3, f64), Failure> {
    let gs = BRepTool::surface(face)?;
    let gas = GeomAdaptorSurface::with_surface(&gs)?;
    Ok(match gas.get_type()? {
        SurfaceType::Sphere => (gas.sphere()?.position(), 0.),
        SurfaceType::Cone => {
            let axis = gas.cone()?.position();
            (axis, eval_phase(edge, face, &gas, &axis)?)
        }
        SurfaceType::Torus => (gas.torus()?.position(), 0.),
        SurfaceType::Cylinder => (gas.cylinder()?.position(), 0.),
        SurfaceType::SurfaceOfRevolution => {
            let revolution: Handle<GeomSurfaceOfRevolution> = gs.cast();
            let basis = revolution.get().ok_or_else(null_geometry)?.basis_curve();
            let hc: Handle<dyn Adaptor3dCurve> =
                Handle::new(GeomAdaptorCurve::with_curve(&basis)?).cast();
            let as_rev =
                GeomAdaptorSurfaceOfRevolution::with_curve_and_axis(&hc, gas.axe_of_revolution()?)?;
            let axis = as_rev.axis();
            (axis, eval_phase(edge, face, &gas, &axis)?)
        }
        _ => {
            return Err(Failure::with_type(
                "Standard_NotImplemented",
                " BRepFill_TrimSurfaceTool",
                "BRepFill_TrimSurfaceTool",
            ));
        }
    })
}

/// The parameter of a degenerated edge: the angle of the bisector tangent at its end
/// around the axis of the surface (plus the phase, in [0, 2 PI)).
fn degenerated_parameter(
    edge: &Edge,
    face: &Face,
    bis: &Handle<dyn Geom2dCurve>,
) -> Result<Vec<Point3>, Failure> {
    let curve = bis.get().ok_or_else(null_geometry)?;
    let Some(u_bis) = bisector_end(edge, curve)? else {
        return Ok(Vec::new());
    };
    let (axis, phase) = surface_axis(edge, face)?;
    let d12d = curve.dn(u_bis, 1)?;
    let d1 = Vector3::new(d12d.x(), d12d.y(), 0.);
    let reference: Direction3 = axis.x_direction().crossed(axis.y_direction())?;
    let mut u: f64 = axis
        .x_direction()
        .angle_with_ref(Direction3::from_vector(d1)?, reference);
    u += phase;
    if u < 0. {
        u += 2. * PI;
    }
    Ok(vec![Point3::new(curve.first_parameter(), u, 0.)])
}

/// Standard_NullObject of a null curve or surface (natively a crash).
fn null_geometry() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_TrimSurfaceTool: null curve or surface; expected a geometry",
        "BRepFill_TrimSurfaceTool",
    )
}
