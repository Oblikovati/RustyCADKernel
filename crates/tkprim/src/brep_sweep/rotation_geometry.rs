// Rust translation of BRepSweep_Rotation.cxx (ComputeTolerance, SetThePCurve,
// MakeEmptyDirectingEdge, MakeEmptyFace, SetPCurve, SetGeneratingPCurve,
// SetDirectingPCurve, DirectSolid and the shape tests) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges, faces and p-curves of the rotation sweep.
use super::BRepSweepRotation;
use super::sweep_geometry::{null_curve, set_the_pcurve};
use crate::sweep::SweepNumShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfRevolution, GeomToroidalSurface,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorCurve, GeomAdaptorSurface, GeomAdaptorSurfaceOfRevolution,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis22d, Circle2, Direction2, Direction3, Line2, Point2, Point3, Vector3, dx2d,
    dy2d, origin2d,
};
use std::f64::consts::PI;

/// Whether each shape of `shapes` has the matching type, tested in order and lazily
/// (a later shape is not read once one differs), as the native `&&` chains do.
pub(crate) fn kinds_are(shapes: [&Shape; 3], kinds: [ShapeEnum; 3]) -> Result<bool, Failure> {
    for (shape, kind) in shapes.iter().zip(kinds) {
        if shape.shape_type()? != kind {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The side face of a generating edge along the edge, with the new edge of one of its
/// vertices (the first test of `GGDShapeIsToAdd` and `SeparatedWires`).
pub(crate) fn is_side_face_of_vertex(
    new_shape: &Shape,
    new_sub_shape: &Shape,
    gen_s: &Shape,
    sub_gen_s: &Shape,
    dir_s: &SweepNumShape,
) -> Result<bool, Failure> {
    if !kinds_are(
        [new_shape, new_sub_shape, gen_s],
        [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Edge],
    )? {
        return Ok(false);
    }
    Ok(sub_gen_s.shape_type()? == ShapeEnum::Vertex && dir_s.kind() == ShapeEnum::Edge)
}

/// The kind of the raw surface of a new face (`GeomAdaptor_Surface(BRep_Tool::Surface(F,
/// L)).GetType()`).
pub(crate) fn raw_surface_type(face: &Shape) -> Result<SurfaceType, Failure> {
    raw_adaptor(face)?.get_type()
}

/// `GeomAdaptor_Surface AS(BRep_Tool::Surface(F, Loc))`.
fn raw_adaptor(face: &Shape) -> Result<GeomAdaptorSurface, Failure> {
    let (surface, _) = BRepTool::surface_with_location(&topods::face(face)?)?;
    GeomAdaptorSurface::with_surface(&surface)
}

/// The poles of the B-spline or Bezier curve of an adaptor.
pub(crate) fn curve_poles(
    curve: &BRepAdaptorCurve,
    kind: CurveType,
) -> Result<Vec<Point3>, Failure> {
    if kind == CurveType::BSplineCurve {
        let bspline = curve.bspline()?;
        let b = bspline.get().ok_or_else(|| null_curve("IsInvariant"))?;
        return (1..=b.nb_poles()).map(|i| b.pole(i)).collect();
    }
    let bezier = curve.bezier()?;
    let b = bezier.get().ok_or_else(|| null_curve("IsInvariant"))?;
    (1..=b.nb_poles()).map(|i| b.pole(i)).collect()
}

/// `ComputeTolerance(E, F, C)`: the tolerance of a degenerated edge, else 1.5 times the
/// largest distance between the 3D curve and the surface point of the p-curve at 24
/// parameters (infinite on an infinite point), at least 1e-7.
fn compute_tolerance(
    edge: &Edge,
    face: &Face,
    pcurve: &Handle<dyn Geom2dCurve>,
) -> Result<f64, Failure> {
    if BRepTool::degenerated(edge)? {
        return BRepTool::edge_tolerance(edge);
    }
    let surf = BRepTool::surface(face)?;
    let surf = surf
        .get()
        .ok_or_else(|| null_curve("ComputeTolerance (surface)"))?;
    let (c3d, first, last) = BRepTool::curve(edge)?;
    let c3d = c3d.get().ok_or_else(|| null_curve("ComputeTolerance"))?;
    let c = pcurve
        .get()
        .ok_or_else(|| null_curve("ComputeTolerance (p-curve)"))?;
    let mut d2 = 0.0;
    let nn = 23;
    let unsurnn = 1.0 / f64::from(nn);
    for i in 0..=nn {
        let t = unsurnn * f64::from(i);
        let u = first * (1.0 - t) + last * t;
        let pc3d = c3d.value(u)?;
        let uv = c.value(u)?;
        let pcons = surf.value(uv.x(), uv.y())?;
        if precision::is_infinite(pcons.x())
            || precision::is_infinite(pcons.y())
            || precision::is_infinite(pcons.z())
        {
            d2 = precision::infinite();
            break;
        }
        let temp = pc3d.square_distance(&pcons);
        if temp > d2 {
            d2 = temp;
        }
    }
    d2 = 1.5 * f64::sqrt(d2);
    Ok(if d2 < 1.0e-7 { 1.0e-7 } else { d2 })
}

/// The rotation's `SetThePCurve`, at `ComputeTolerance` of the new p-curve.
fn set_pcurve_computed(
    builder: BRepBuilder,
    edge: &Shape,
    face: &Shape,
    orientation: Orientation,
    pcurve: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    set_the_pcurve(builder, edge, face, orientation, pcurve, compute_tolerance)
}

/// `MakeEmptyDirectingEdge`: the circle of the vertex point around the axis (normal the
/// axis, X toward the point), or a degenerated edge on a circle of radius 0 when the
/// point is on the axis.
pub(crate) fn directing_circle_edge(
    builder: BRepBuilder,
    axe: Axis1,
    vertex: &Vertex,
) -> Result<Shape, Failure> {
    let point = BRepTool::pnt(vertex)?;
    let dirz = axe.direction();
    let v = Vector3::from(dirz);
    let mut o = axe.location();
    o.translate(v.dot(Vector3::between(o, point)) * v);
    let tol = BRepTool::vertex_tolerance(vertex)?;
    let mut edge = Edge::default();
    if o.is_equal(&point, precision::confusion()) {
        // A degenerated edge; the null circle is kept so that the parameters register.
        let circle: Handle<dyn GeomCurve> =
            Handle::new(GeomCircle::new(Axis2::new(o, dirz)?, 0.0)?).cast();
        builder.make_edge_with_curve(&mut edge, &circle, tol)?;
        builder.degenerated(&edge, true)?;
        return Ok(edge.into());
    }
    let axis = Axis2::with_x_direction(
        o,
        dirz,
        Direction3::from_vector(Vector3::between(o, point))?,
    )?;
    let circle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(axis, o.distance(&point))?).cast();
    builder.make_edge_with_curve(&mut edge, &circle, tol)?;
    Ok(edge.into())
}

/// `MakeEmptyFace`: along the edge, the revolution of the trimmed located curve of the
/// generating edge, canonized into an elementary surface when possible (jgv 9.12.03);
/// at a vertex, a copy of the located surface of the generating face, rotated at the
/// last vertex.
pub(crate) fn make_empty_face(
    r: &BRepSweepRotation,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
) -> Result<Shape, Failure> {
    let (surface, toler) = if gen_s.shape_type()? == ShapeEnum::Edge {
        revolution_surface(r.axe(), &topods::edge(gen_s)?)?
    } else {
        cap_surface(r, &topods::face(gen_s)?, dir_s)?
    };
    let mut face = Face::default();
    r.brep_builder()
        .make_face_on_surface(&mut face, &surface, toler)?;
    Ok(face.into())
}

/// The surface of revolution of a generating edge and its tolerance.
fn revolution_surface(axe: Axis1, edge: &Edge) -> Result<(Handle<dyn GeomSurface>, f64), Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let toler = BRepTool::edge_tolerance(edge)?;
    let source = found
        .curve
        .get()
        .ok_or_else(|| null_curve("MakeEmptyFace"))?;
    let copy: Handle<dyn GeomCurve> = source.copy()?.cast();
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(
        &copy,
        found.first,
        found.last,
        true,
        true,
    )?)
    .cast();
    curve
        .get()
        .ok_or_else(|| null_curve("MakeEmptyFace"))?
        .transform(found.location.transformation())?;
    let adapted = GeomAdaptorCurve::default();
    adapted.load_range(&curve, found.first, found.last)?;
    let adapted: Handle<dyn rustycad_tkg3d::adaptor3d::Adaptor3dCurve> =
        Handle::new(adapted).cast();
    let revolution = GeomAdaptorSurfaceOfRevolution::with_curve_and_axis(&adapted, axe)?;
    let surface: Handle<dyn GeomSurface> = match revolution.get_type()? {
        SurfaceType::Plane => Handle::new(GeomPlane::from_pln(revolution.plane()?)).cast(),
        SurfaceType::Cylinder => Handle::new(GeomCylindricalSurface::from_cylinder(
            revolution.cylinder()?,
        ))
        .cast(),
        SurfaceType::Sphere => {
            Handle::new(GeomSphericalSurface::from_sphere(revolution.sphere()?)).cast()
        }
        SurfaceType::Cone => Handle::new(GeomConicalSurface::from_cone(revolution.cone()?)).cast(),
        SurfaceType::Torus => {
            Handle::new(GeomToroidalSurface::from_torus(revolution.torus()?)).cast()
        }
        _ => Handle::new(GeomSurfaceOfRevolution::new(&curve, axe)?).cast(),
    };
    Ok((surface, toler))
}

/// The copied cap surface of a generating face and its tolerance.
fn cap_surface(
    r: &BRepSweepRotation,
    face: &Face,
    dir_s: &SweepNumShape,
) -> Result<(Handle<dyn GeomSurface>, f64), Failure> {
    let (raw, location) = BRepTool::surface_with_location(face)?;
    let toler = BRepTool::face_tolerance(face)?;
    let source = raw
        .get()
        .ok_or_else(|| null_curve("MakeEmptyFace (surface)"))?;
    let surface: Handle<dyn GeomSurface> = source.copy()?.cast();
    let copy = surface
        .get()
        .ok_or_else(|| null_curve("MakeEmptyFace (surface)"))?;
    copy.transform(location.transformation())?;
    if dir_s.index() == 2 {
        copy.transform(r.trsf_location().transformation())?;
    }
    Ok((surface, toler))
}

/// `SetPCurve`: the p-curve of the generating edge on the generating face, on the new
/// edge of the new cap face.
pub(crate) fn set_cap_pcurve(
    builder: BRepBuilder,
    new_face: &Shape,
    new_edge: &Shape,
    (gen_f, gen_e): (&Shape, &Shape),
    orien: Orientation,
) -> Result<(), Failure> {
    let pcurve = BRepTool::curve_on_face(&topods::edge(gen_e)?, &topods::face(gen_f)?)?.pcurve;
    set_pcurve_computed(builder, new_edge, new_face, orien, &pcurve)
}

/// `SetGeneratingPCurve`: the p-curve of a new generating edge on a new side face, a
/// line by the kind of the surface (the line of the edge in a plane, an iso-U line on a
/// torus, a sphere and any other surface).
pub(crate) fn set_generating_pcurve(
    r: &BRepSweepRotation,
    new_face: &Shape,
    new_edge: &Shape,
    dir_v: &SweepNumShape,
    orien: Orientation,
) -> Result<(), Failure> {
    let adaptor = raw_adaptor(new_face)?;
    let edge = topods::edge(new_edge)?;
    let line = match adaptor.get_type()? {
        SurfaceType::Plane => generating_line_in_plane(&adaptor, &edge)?,
        SurfaceType::Torus => generating_line_on_torus(r, &adaptor, &edge, dir_v)?,
        SurfaceType::Sphere => generating_line_on_sphere(r, &adaptor, &edge, dir_v)?,
        _ => {
            let angle = if dir_v.index() == 2 { r.angle() } else { 0.0 };
            Line2::new(Point2::new(angle, 0.0), dy2d())
        }
    };
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(line)).cast();
    set_pcurve_computed(r.brep_builder(), new_edge, new_face, orien, &pcurve)
}

/// The generating p-curve in a plane: the located line of the new edge (a line or a
/// trimmed line, else Standard_ConstructionError) in the plane's frame.
fn generating_line_in_plane(adaptor: &GeomAdaptorSurface, edge: &Edge) -> Result<Line2, Failure> {
    let ax3 = adaptor.plane()?.position();
    let found = BRepTool::curve_with_location(edge)?;
    let mut line: Handle<GeomLine> = found.curve.cast();
    if line.is_null() {
        let trimmed: Handle<GeomTrimmedCurve> = found.curve.cast();
        if let Some(trimmed) = trimmed.get() {
            line = trimmed.basis_curve().cast();
            if line.is_null() {
                return Err(Failure::with_message(
                    "Standard_ConstructionError",
                    "BRepSweep_Rotation::SetGeneratingPCurve: the trimmed curve of the new edge is not a line; expected a line in the planar face",
                ));
            }
        }
    }
    let line = line
        .get()
        .ok_or_else(|| null_curve("SetGeneratingPCurve"))?;
    let gl = line.lin().transformed(found.location.transformation());
    let (u, v) = elslib::plane_parameters(ax3, gl.location());
    let dir = gl.direction();
    Ok(Line2::new(
        Point2::new(u, v),
        Direction2::new(dir.dot(ax3.x_direction()), dir.dot(ax3.y_direction()))?,
    ))
}

/// The generating p-curve on a torus: the iso-U line through the start of the new
/// edge, with U adjusted into (-PI, PI) against the other end of the rotation
/// (NIZHNY-EAP Mar 1 2000).
fn generating_line_on_torus(
    r: &BRepSweepRotation,
    adaptor: &GeomAdaptorSurface,
    edge: &Edge,
    dir_v: &SweepNumShape,
) -> Result<Line2, Failure> {
    let tor = adaptor.torus()?;
    let bc = BRepAdaptorCurve::with_edge(edge)?;
    let first = bc.first_parameter()?;
    let point = bc.value(first)?;
    let (mut u, mut v) = if point.distance(&tor.location()) < precision::confusion() {
        (0.0, PI)
    } else {
        elslib::torus_parameters(
            tor.position(),
            tor.major_radius(),
            tor.minor_radius(),
            point,
        )
    };
    v = elclib::in_period(v, 0.0, 2.0 * PI);
    if (2.0 * PI - v) <= precision::p_confusion() {
        v -= 2.0 * PI;
    }
    if dir_v.index() == 2 {
        (_, u) = elclib::adjust_periodic(-PI, PI, precision::p_confusion(), u - r.angle(), u);
    } else {
        (u, _) = elclib::adjust_periodic(-PI, PI, precision::p_confusion(), u, u + r.angle());
    }
    Ok(Line2::new(Point2::new(u, v - first), dy2d()))
}

/// The generating p-curve on a sphere: the iso-U line at 0 (or the angle at the last
/// vertex) through the latitude of the start of the new edge.
fn generating_line_on_sphere(
    r: &BRepSweepRotation,
    adaptor: &GeomAdaptorSurface,
    edge: &Edge,
    dir_v: &SweepNumShape,
) -> Result<Line2, Failure> {
    let sph = adaptor.sphere()?;
    let bc = BRepAdaptorCurve::with_edge(edge)?;
    let first = bc.first_parameter()?;
    let point = bc.value(first)?;
    let (_, v) = elslib::sphere_parameters(sph.position(), sph.radius(), point);
    let u = if dir_v.index() == 2 { r.angle() } else { 0.0 };
    Ok(Line2::new(Point2::new(u, v - first), dy2d()))
}

/// `SetDirectingPCurve`: the p-curve of a new directing edge on a new side face, by the
/// kind of the surface.
pub(crate) fn set_directing_pcurve(
    r: &BRepSweepRotation,
    new_face: &Shape,
    new_edge: &Shape,
    (gen_e, gen_v): (&Shape, &Shape),
    orien: Orientation,
) -> Result<(), Failure> {
    let adaptor = raw_adaptor(new_face)?;
    let (edge, vertex) = (topods::edge(gen_e)?, topods::vertex(gen_v)?);
    let par = BRepTool::parameter(&vertex, &edge)?;
    let p2 = BRepTool::pnt(&vertex)?;
    let pcurve: Handle<dyn Geom2dCurve> = match adaptor.get_type()? {
        SurfaceType::Plane => {
            let pln = adaptor.plane()?;
            let radius = pln.location().distance(&p2);
            let (u, v) = elslib::plane_parameters(pln.position(), p2);
            let axe = Axis22d::with_directions(origin2d(), Direction2::new(u, v)?, dy2d())?;
            Handle::new(Geom2dCircle::from_circ2d(Circle2::new(axe, radius)?)).cast()
        }
        SurfaceType::Cone => {
            let cone = adaptor.cone()?;
            let (_, v) =
                elslib::cone_parameters(cone.position(), cone.ref_radius(), cone.semi_angle(), p2);
            iso_v_line(v)
        }
        SurfaceType::Sphere => {
            let sph = adaptor.sphere()?;
            iso_v_line(elslib::sphere_parameters(sph.position(), sph.radius(), p2).1)
        }
        SurfaceType::Torus => directing_line_on_torus(r, &adaptor, &edge, gen_v)?,
        _ => iso_v_line(par),
    };
    set_pcurve_computed(r.brep_builder(), new_edge, new_face, orien, &pcurve)
}

/// The iso-V line at `v` along U, from U = 0.
fn iso_v_line(v: f64) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_lin2d(Line2::new(
        Point2::new(0.0, v),
        dx2d(),
    )))
    .cast()
}

/// The directing p-curve on a torus: the iso-V line at the V of the start (FORWARD
/// vertex) or end of the generating edge, from its U adjusted into (-PI, PI)
/// (NIZHNY-EAP Mar 2 2000).
fn directing_line_on_torus(
    r: &BRepSweepRotation,
    adaptor: &GeomAdaptorSurface,
    gen_e: &Edge,
    gen_v: &Shape,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let tor = adaptor.torus()?;
    let bc = BRepAdaptorCurve::with_edge(gen_e)?;
    let (major, minor) = (tor.major_radius(), tor.minor_radius());
    let p1 = bc.value(bc.first_parameter()?)?;
    let (u1, v1) = if p1.distance(&tor.location()) < precision::confusion() {
        (0.0, PI)
    } else {
        elslib::torus_parameters(tor.position(), major, minor, p1)
    };
    let p2 = bc.value(bc.last_parameter()?)?;
    let v2 = if p2.distance(&tor.location()) < precision::confusion() {
        PI
    } else {
        elslib::torus_parameters(tor.position(), major, minor, p2).1
    };
    let (v1, v2) = elclib::adjust_periodic(0.0, 2.0 * PI, precision::p_confusion(), v1, v2);
    let (u1, _) = elclib::adjust_periodic(-PI, PI, precision::p_confusion(), u1, u1 + r.angle());
    let v = if gen_v.orientation() == Orientation::Forward {
        v1
    } else {
        v2
    };
    Ok(Handle::new(Geom2dLine::from_lin2d(Line2::new(
        Point2::new(u1, v),
        dx2d(),
    )))
    .cast())
}

/// `DirectSolid` (NIZNHY-PKV Jun 14 2011): FORWARD when the normal of the generating
/// face at the middle of its bounds turns with the rotation; a point on the axis is
/// replaced by the point at 0.43213918 of the bounds.
pub(crate) fn direct_solid(axe: Axis1, gen_s: &Shape) -> Result<Orientation, Failure> {
    let surf = BRepAdaptorSurface::with_face(&topods::face(gen_s)?, true)?;
    let tol2 = precision::confusion() * precision::confusion();
    let (loc, dir) = (axe.location(), Vector3::from(axe.direction()));
    let (u1, u2) = (surf.first_u_parameter()?, surf.last_u_parameter()?);
    let (v1, v2) = (surf.first_v_parameter()?, surf.last_v_parameter()?);
    let tx = 0.5;
    let mut d1 = surf.d1(tx * (u1 + u2), tx * (v1 + v2))?;
    let mut v = Vector3::between(loc, d1.point);
    v.cross(dir);
    if v.square_magnitude() < tol2 {
        let tx = 0.43213918;
        d1 = surf.d1(u1 * (1.0 - tx) + u2 * tx, v1 * (1.0 - tx) + v2 * tx)?;
        v = Vector3::from(d1.point.xyz() - loc.xyz());
        v.cross(dir);
    }
    let x = v.dot_cross(d1.d1u, d1.d1v);
    Ok(if x > 0.0 {
        Orientation::Forward
    } else {
        Orientation::Reversed
    })
}
