// Rust translation of DetectKPart (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Generator.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DetectKPart`: the analytic surface (cylinder, cone, plane) that rules two
//! consecutive section edges, when one exists.
use super::kpart_edge::{KPartEdge, oriented_curve};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};

/// No particular case: the general ruled surface.
pub(crate) const KPART_NONE: i32 = 0;
/// A cylinder (or a piece of one) between two coaxial circles of equal radius.
pub(crate) const KPART_CYLINDER: i32 = 1;
/// A cone between two coaxial circles, or a circle and an apex vertex.
pub(crate) const KPART_CONE: i32 = 2;
/// A plane between two parallel segments, or a segment and a vertex.
pub(crate) const KPART_PLANE: i32 = 4;
/// A cone whose apex is the degenerated first edge.
pub(crate) const KPART_CONE_APEX_FIRST: i32 = -2;
/// An edge without a 3d curve (an error, `IType == -1` natively).
pub(crate) const KPART_NULL_CURVE: i32 = -1;

/// The characteristics of the first edge: its type code, a reference point, a
/// distance (radius or length) and an axis.
struct FirstSection {
    itype: i32,
    pos1: Point3,
    dist1: f64,
    axe1: Axis1,
    edge: KPartEdge,
}

/// `DetectKPart(Edge1, Edge2)`: 1 cylinder, 2 cone, -2 cone with the apex first, 4
/// plane, 0 no particular case and -1 when an edge has no 3d curve; the torus (3)
/// and the extrusion (5) are found but returned as 0, as natively.
/// `let itype = detect_kpart(&edge1, &edge2)?;`
pub(crate) fn detect_kpart(edge1: &Edge, edge2: &Edge) -> Result<i32, Failure> {
    let Some(first) = first_section(edge1)? else {
        return Ok(KPART_NULL_CURVE);
    };
    if first.itype == KPART_NONE {
        return Ok(KPART_NONE);
    }
    let itype: i32 = if BRepTool::degenerated(edge2)? {
        with_degenerated_second(&first, edge2)?
    } else {
        let Some(second) = oriented_curve(edge2)? else {
            return Ok(KPART_NULL_CURVE);
        };
        with_second_curve(&first, &second)?
    };
    // The torus and the extrusion are not particular cases.
    if itype == 3 || itype == 5 {
        return Ok(KPART_NONE);
    }
    Ok(itype)
}

/// The first edge: a degenerated edge (-2) at its first vertex, a circle (1), a
/// segment (4) or any curve (0); `None` for a missing 3d curve.
fn first_section(edge1: &Edge) -> Result<Option<FirstSection>, Failure> {
    if BRepTool::degenerated(edge1)? {
        let (v1, _) = topexp::vertices(edge1, false);
        return Ok(Some(FirstSection {
            itype: KPART_CONE_APEX_FIRST,
            pos1: BRepTool::pnt(&v1)?,
            dist1: 0.,
            axe1: Axis1::default(),
            edge: KPartEdge::default(),
        }));
    }
    let Some(edge) = oriented_curve(edge1)? else {
        return Ok(None);
    };
    let adaptor: &GeomAdaptorCurve = &edge.adaptor;
    let mut first = FirstSection {
        itype: KPART_NONE,
        pos1: Point3::default(),
        dist1: 0.,
        axe1: Axis1::default(),
        edge: KPartEdge::default(),
    };
    match adaptor.get_type()? {
        CurveType::Circle => {
            let circle = adaptor.circle()?;
            first.itype = KPART_CYLINDER;
            first.pos1 = circle.location();
            first.dist1 = circle.radius();
            first.axe1 = circle.axis();
        }
        CurveType::Line => {
            first.itype = KPART_PLANE;
            first.pos1 = adaptor.line()?.location();
            let (start, end) = (adaptor.value(edge.first)?, adaptor.value(edge.last)?);
            first.dist1 = start.distance(&end);
            first.axe1 = Axis1::new(
                start,
                Direction3::from_vector(Vector3::between(start, end))?,
            );
        }
        _ => {}
    }
    first.edge = edge;
    Ok(Some(first))
}

/// The second edge is degenerated: a circle and a vertex on its axis give a cone,
/// a segment and a vertex a plane.
fn with_degenerated_second(first: &FirstSection, edge2: &Edge) -> Result<i32, Failure> {
    let (v1, _) = topexp::vertices(edge2, false);
    let pos: Point3 = BRepTool::pnt(&v1)?;
    if first.itype == KPART_CYLINDER {
        // The only particular case with a degenerated edge at the end: the cone.
        if pos.is_equal(&first.pos1, precision::confusion()) {
            // The top is mixed with the center of the circle.
            return Ok(KPART_NONE);
        }
        let axe = Axis1::new(
            first.pos1,
            Direction3::from_vector(Vector3::between(first.pos1, pos))?,
        );
        // The top must be on the axis of the circle.
        return Ok(if axe.is_parallel(first.axe1, precision::angular()) {
            KPART_CONE
        } else {
            KPART_NONE
        });
    }
    if first.itype != KPART_PLANE {
        return Ok(KPART_NONE);
    }
    Ok(first.itype)
}

/// The second edge has a 3d curve: dispatch on the type of the first section.
fn with_second_curve(first: &FirstSection, second: &KPartEdge) -> Result<i32, Failure> {
    if first.itype > 0 && first.itype < 4 {
        return circular_pair(first, second);
    }
    if first.itype >= 4 {
        return straight_pair(first, second);
    }
    apex_then_curve(first, second)
}

/// A circle then a curve: cylinder (same axis and radius), cone (same axis), torus
/// (other axis, same radius), else nothing.
fn circular_pair(first: &FirstSection, second: &KPartEdge) -> Result<i32, Failure> {
    let adaptor: &GeomAdaptorCurve = &second.adaptor;
    if adaptor.get_type()? != CurveType::Circle {
        // Section not circular: no particular case.
        return Ok(KPART_NONE);
    }
    let circle = adaptor.circle()?;
    let coaxial: bool =
        circle
            .axis()
            .is_coaxial(first.axe1, precision::angular(), precision::confusion());
    if !coaxial {
        // Different axis: a torus when the radius is the same (exact comparison, as natively).
        return Ok(if circle.radius() == first.dist1 {
            3
        } else {
            KPART_NONE
        });
    }
    let same_radius: bool = (circle.radius() - first.dist1).abs() < precision::confusion();
    let kind: i32 = if same_radius {
        KPART_CYLINDER
    } else {
        KPART_CONE
    };
    let mut itype: i32 = if same_span(&first.edge, second)? {
        kind
    } else {
        KPART_NONE
    };
    if circle
        .location()
        .is_equal(&first.pos1, precision::confusion())
    {
        // The centers are mixed.
        itype = KPART_NONE;
    }
    Ok(itype)
}

/// The two arcs have the same parametric length and the chord between their middle
/// points is normal to the first arc there (the interval of definition is correct).
fn same_span(first: &KPartEdge, second: &KPartEdge) -> Result<bool, Failure> {
    let h1: f64 = (first.last - first.first).abs();
    let h2: f64 = (second.last - second.first).abs();
    let same_parametric_length: bool = (h1 - h2).abs() < precision::p_confusion();
    let m1: f64 = (first.first + first.last) / 2.;
    let m2: f64 = (second.first + second.last) / 2.;
    let (p1, du) = first.adaptor.d1(m1)?;
    let p2: Point3 = second.adaptor.d0(m2)?;
    // `&&` short-circuits natively, so IsNormal (which may raise) runs only on equal lengths.
    if !same_parametric_length {
        return Ok(false);
    }
    Vector3::between(p1, p2).is_normal(du, precision::angular())
}

/// A segment then a curve: a plane between two parallel segments of the same length
/// whose start chord is normal to them, an extrusion when it is not.
fn straight_pair(first: &FirstSection, second: &KPartEdge) -> Result<i32, Failure> {
    let adaptor: &GeomAdaptorCurve = &second.adaptor;
    if adaptor.get_type()? != CurveType::Line {
        // Not a straight line section: no particular case.
        return Ok(KPART_NONE);
    }
    let (start, end) = (adaptor.value(second.first)?, adaptor.value(second.last)?);
    let dist: f64 = start.distance(&end);
    let vec: Vector3 = Vector3::between(start, end);
    let axe = Axis1::new(start, Direction3::from_vector(vec)?);
    if !axe.is_parallel(first.axe1, precision::angular()) {
        return Ok(KPART_NONE);
    }
    if (dist - first.dist1).abs() >= precision::confusion() {
        // Different length: no particular case.
        return Ok(KPART_NONE);
    }
    let start1: Point3 = first.edge.adaptor.value(first.edge.first)?;
    let dir = Direction3::from_vector(Vector3::between(start1, start))?;
    Ok(
        if dir.is_normal(Direction3::from_vector(vec)?, precision::angular()) {
            KPART_PLANE
        } else {
            5
        },
    )
}

/// A degenerated first edge then a curve: a plane for a segment, a cone for a
/// circle whose axis passes through the apex.
fn apex_then_curve(first: &FirstSection, second: &KPartEdge) -> Result<i32, Failure> {
    let adaptor: &GeomAdaptorCurve = &second.adaptor;
    match adaptor.get_type()? {
        CurveType::Line => Ok(KPART_PLANE),
        CurveType::Circle => {
            let circle = adaptor.circle()?;
            let pos: Point3 = circle.location();
            if first.pos1.is_equal(&pos, precision::confusion()) {
                // The top is mixed with the center of the circle.
                return Ok(KPART_NONE);
            }
            let axe1 = Axis1::new(
                first.pos1,
                Direction3::from_vector(Vector3::between(first.pos1, pos))?,
            );
            Ok(if circle.axis().is_parallel(axe1, precision::angular()) {
                KPART_CONE_APEX_FIRST
            } else {
                KPART_NONE
            })
        }
        _ => Ok(KPART_NONE),
    }
}
