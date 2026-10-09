// Rust translation of CreateKPart (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Generator.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CreateKPart`: the analytic surface ruled between two section edges.
use super::kpart_detect::{KPART_CONE, KPART_CONE_APEX_FIRST, KPART_CYLINDER, KPART_PLANE};
use super::kpart_edge::{KPartEdge, oriented_curve};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSurface,
};
use rustycad_tkmath::gp::{Axis3, Circle3, Direction3, Line3, Point3, Vector3};

/// One side of the ruled face: the oriented curve (none for a degenerated edge) and
/// the first vertex in the orientation of the edge.
struct KPartSide {
    curve: Option<KPartEdge>,
    first_vertex: Vertex,
}

/// `CreateKPart(Edge1, Edge2, IType, Surf)`: the cylinder, cone or plane of a
/// particular case found by `detect_kpart`. `Ok(None)` is the native `false` (an edge
/// without a 3d curve); an unknown `IType` gives a null surface, as natively.
/// `let surface = create_kpart(&edge1, &edge2, itype)?;`
pub(crate) fn create_kpart(
    edge1: &Edge,
    edge2: &Edge,
    itype: i32,
) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    let Some(side1) = side(edge1)? else {
        return Ok(None);
    };
    let Some(side2) = side(edge2)? else {
        return Ok(None);
    };
    let (aa, bb) = trim_range(&side1, &side2);
    let surface: Handle<dyn GeomSurface> = match itype {
        KPART_CYLINDER => cylinder(&side1, &side2, aa, bb)?,
        KPART_CONE => cone(&side1, &side2, aa, bb)?,
        KPART_CONE_APEX_FIRST => cone_from_apex(&side1, &side2, aa, bb)?,
        KPART_PLANE => plane(&side1, &side2)?,
        // The torus (3), the extrusion (5) and any other code: no surface.
        _ => Handle::default(),
    };
    Ok(Some(surface))
}

/// The curve and first vertex of a section edge; `None` for a missing 3d curve.
fn side(edge: &Edge) -> Result<Option<KPartSide>, Failure> {
    if BRepTool::degenerated(edge)? {
        // Cone with the degenerated edge at the top.
        let (first_vertex, _) = topexp::vertices(edge, false);
        return Ok(Some(KPartSide {
            curve: None,
            first_vertex,
        }));
    }
    let Some(curve) = oriented_curve(edge)? else {
        return Ok(None);
    };
    let (vf, vl) = topexp::vertices(edge, false);
    let first_vertex: Vertex =
        if edge.orientation() == rustycad_tkg3d::top_abs::Orientation::Reversed {
            vl
        } else {
            vf
        };
    Ok(Some(KPartSide {
        curve: Some(curve),
        first_vertex,
    }))
}

/// The U range of the surface: the oriented range of the first edge; with a
/// degenerated first edge the range of the second edge as stored (not reversed, as
/// natively), and 0 for two degenerated edges.
fn trim_range(side1: &KPartSide, side2: &KPartSide) -> (f64, f64) {
    if let Some(curve) = &side1.curve {
        return (curve.first, curve.last);
    }
    match &side2.curve {
        Some(curve) => curve.raw,
        None => (0., 0.),
    }
}

/// The circle of a side; Standard_NoSuchObject natively when the side has no curve.
fn circle_of(side: &KPartSide) -> Result<Circle3, Failure> {
    match &side.curve {
        Some(curve) => curve.adaptor.circle(),
        None => Err(missing_curve("Circle")),
    }
}

/// The error of an analytic query on a degenerated side.
fn missing_curve(query: &str) -> Failure {
    let message = format!(
        "BRepFill_Generator CreateKPart: {query} of a side without a 3d curve; expected a curve"
    );
    Failure::with_type("Standard_NoSuchObject", &message, "BRepFill_Generator")
}

/// The surface trimmed to `[aa, bb] x [min(0, v), max(0, v)]`.
fn trimmed(
    basis: Handle<dyn GeomSurface>,
    aa: f64,
    bb: f64,
    v: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let surface = GeomRectangularTrimmedSurface::new(
        &basis,
        (aa, bb),
        (0f64.min(v), 0f64.max(v)),
        (true, true),
    )?;
    Ok(Handle::new(surface).cast::<dyn GeomSurface>())
}

/// The cylinder on the first circle, its axis turned towards the second circle.
fn cylinder(
    side1: &KPartSide,
    side2: &KPartSide,
    aa: f64,
    bb: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let c1: Circle3 = circle_of(side1)?;
    let c2: Circle3 = circle_of(side2)?;
    let mut ac1: Axis3 = Axis3::from_axis2(c1.position());
    let mut v: f64 =
        Vector3::between(c1.location(), c2.location()).dot(Vector3::from(ac1.direction()));
    if v < 0. {
        ac1.z_reverse();
        v = -v;
    }
    let cyl = GeomCylindricalSurface::new(ac1, c1.radius())?;
    trimmed(Handle::new(cyl).cast::<dyn GeomSurface>(), aa, bb, v)
}

/// The cone on the first circle towards the second circle or the apex vertex.
fn cone(
    side1: &KPartSide,
    side2: &KPartSide,
    aa: f64,
    bb: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let k1: Circle3 = circle_of(side1)?;
    let mut ak1: Axis3 = Axis3::from_axis2(k1.position());
    let (mut v, rad): (f64, f64) = match &side2.curve {
        None => {
            let top: Point3 = BRepTool::pnt(&side2.first_vertex)?;
            (
                Vector3::between(k1.location(), top).dot(Vector3::from(ak1.direction())),
                -k1.radius(),
            )
        }
        Some(curve) => {
            let k2: Circle3 = curve.adaptor.circle()?;
            let v =
                Vector3::between(k1.location(), k2.location()).dot(Vector3::from(ak1.direction()));
            (v, k2.radius() - k1.radius())
        }
    };
    if v < 0. {
        ak1.z_reverse();
        v = -v;
    }
    let ang: f64 = (rad / v).atan();
    let cone = GeomConicalSurface::new(ak1, ang, k1.radius())?;
    v /= ang.cos();
    trimmed(Handle::new(cone).cast::<dyn GeomSurface>(), aa, bb, v)
}

/// The cone whose apex is the degenerated first edge.
fn cone_from_apex(
    side1: &KPartSide,
    side2: &KPartSide,
    aa: f64,
    bb: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let k2: Circle3 = circle_of(side2)?;
    let mut ak2: Axis3 = Axis3::from_axis2(k2.position());
    let apex: Point3 = BRepTool::pnt(&side1.first_vertex)?;
    ak2.set_location(apex);
    let mut v: f64 = Vector3::between(apex, k2.location()).dot(Vector3::from(ak2.direction()));
    let rad: f64 = k2.radius();
    if v < 0. {
        ak2.z_reverse();
        v = -v;
    }
    let ang: f64 = (rad / v).atan();
    let cone = GeomConicalSurface::new(ak2, ang, 0.)?;
    v /= ang.cos();
    trimmed(Handle::new(cone).cast::<dyn GeomSurface>(), aa, bb, v)
}

/// The plane through the segment (of the second edge when both have one) and the
/// start of the other side; not trimmed, as natively.
fn plane(side1: &KPartSide, side2: &KPartSide) -> Result<Handle<dyn GeomSurface>, Failure> {
    // gp_Lin() natively: the Z axis through the origin.
    let mut a_line = Line3::default();
    let mut l1 = Line3::default();
    let mut l2 = Line3::default();
    if let Some(curve) = &side1.curve {
        l1 = curve.adaptor.line()?;
        a_line = l1;
    }
    if let Some(curve) = &side2.curve {
        l2 = curve.adaptor.line()?;
        a_line = l2;
    }
    let p1: Point3 = match side1.curve {
        None => BRepTool::pnt(&side1.first_vertex)?,
        Some(_) => l1.location(),
    };
    let p2: Point3 = match side2.curve {
        None => BRepTool::pnt(&side2.first_vertex)?,
        Some(_) => l2.location(),
    };
    let p1p2 = Vector3::between(p1, p2);
    let d1: Direction3 = a_line.direction();
    // `D1.Crossed(P1P2)` converts the vector to a gp_Dir first (gp_Dir(const gp_Vec&) is
    // implicit), so a null P1P2 raises Standard_ConstructionError there.
    let normal: Direction3 = d1.crossed(Direction3::from_vector(p1p2)?)?;
    let ax = Axis3::new(a_line.location(), normal, d1)?;
    Ok(Handle::new(GeomPlane::new(ax)).cast::<dyn GeomSurface>())
}
