// Rust translation of BRepBuilderAPI_FindPlane.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_FindPlane`: the plane of the 3d curves of the edges of a shape, when
//! they lie in one.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve, GeomConic, GeomCurve, GeomPlane};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Vector3};

/// `BRepBuilderAPI_FindPlane` (docs/topalgo-design.md section 10): the plane of the
/// first conic among the edge curves (else a plane through three sampled points), kept
/// when every sampled point of every curve lies within the tolerance of it. The
/// tolerance is the largest of the given one and the edge tolerances.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIFindPlane, BRepBuilderAPIMakePolygon};
/// let wire = BRepBuilderAPIMakePolygon::from_three_points(
///     Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0), true)?.wire()?;
/// let finder = BRepBuilderAPIFindPlane::from_shape(&wire, -1.0)?;
/// assert!(finder.found());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct BRepBuilderAPIFindPlane {
    my_plane: Handle<GeomPlane>,
}

impl BRepBuilderAPIFindPlane {
    /// `BRepBuilderAPI_FindPlane()`: no plane; `BRepBuilderAPIFindPlane::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepBuilderAPI_FindPlane(S, Tol)`: then [`Self::init`] (the native default `Tol`
    /// is -1); `BRepBuilderAPIFindPlane::from_shape(&s, -1.0)?`.
    pub fn from_shape(shape: &Shape, tol: f64) -> Result<Self, Failure> {
        let mut finder = Self::new();
        finder.init(shape, tol)?;
        Ok(finder)
    }

    /// `Init(S, Tol)`: searches the plane of the edges of `S` (the plane is cleared
    /// first); `finder.init(&s, -1.0)?`.
    pub fn init(&mut self, shape: &Shape, tol: f64) -> Result<(), Failure> {
        self.my_plane.nullify();
        let tolerance: f64 = shape_tolerance(shape, tol)?;
        let tol2: f64 = tolerance * tolerance;
        let (conic_plane, points) = sampled_points(shape)?;
        let candidate = match conic_plane {
            Some(plane) => Some(plane),
            None => plane_through_points(&points, tol2)?,
        };
        let Some(plane) = candidate else {
            return Ok(());
        };
        // test if all points are on the plane
        let pln = plane.get().map(GeomPlane::pln);
        if pln.is_some_and(|pln| !points.iter().any(|p| pln.square_distance(*p) > tol2)) {
            self.my_plane = plane;
        }
        Ok(())
    }

    /// `Found()`: whether a plane was found; `finder.found()`.
    pub fn found(&self) -> bool {
        !self.my_plane.is_null()
    }

    /// `Plane()`: the plane found (null when none); `finder.plane()`.
    pub fn plane(&self) -> Handle<GeomPlane> {
        self.my_plane.clone()
    }
}

/// The given tolerance raised to the tolerance of each edge.
fn shape_tolerance(shape: &Shape, tol: f64) -> Result<f64, Failure> {
    let mut tolerance: f64 = tol;
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        let t: f64 = BRepTool::edge_tolerance(&topods::edge(explorer.current())?)?;
        if t > tolerance {
            tolerance = t;
        }
        explorer.next()?;
    }
    Ok(tolerance)
}

/// The points sampled on the 3d curve of each edge (located), and the plane of the
/// first conic met.
fn sampled_points(shape: &Shape) -> Result<(Option<Handle<GeomPlane>>, Vec<Point3>), Failure> {
    let mut plane: Option<Handle<GeomPlane>> = None;
    let mut points: Vec<Point3> = Vec::new();
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        let found = BRepTool::curve_with_location(&topods::edge(explorer.current())?)?;
        explorer.next()?;
        let Some(c3d) = found.curve.get() else {
            continue;
        };
        let located = c3d.transformed(found.location.transformation())?;
        let curve: Handle<dyn GeomCurve> = located.cast();
        let nb_pnts: i32 = sample_count(&curve, &mut plane)?;
        sample_curve(&curve, (found.first, found.last), nb_pnts, &mut points)?;
    }
    Ok((plane, points))
}

/// The number of points to sample on the curve, by its exact type: 3 on a line, 4 on a
/// conic (the first conic gives the plane of its position), the poles of a Bezier or
/// B-spline curve, else 10.
fn sample_count(
    curve: &Handle<dyn GeomCurve>,
    plane: &mut Option<Handle<GeomPlane>>,
) -> Result<i32, Failure> {
    if curve.is_instance_name("Geom_Line") {
        return Ok(3);
    }
    let conic_types = [
        "Geom_Circle",
        "Geom_Ellipse",
        "Geom_Parabola",
        "Geom_Hyperbola",
    ];
    if conic_types.iter().any(|name| curve.is_instance_name(name)) {
        if plane.is_none() {
            let conic: Handle<dyn GeomConic> = curve.cast();
            let position = conic
                .get()
                .map(GeomConic::position)
                .ok_or_else(null_curve)?;
            *plane = Some(Handle::new(GeomPlane::new(Axis3::from_axis2(position))));
        }
        return Ok(4);
    }
    if curve.is_instance_name("Geom_BezierCurve") {
        let bezier: Handle<GeomBezierCurve> = curve.cast();
        return bezier
            .get()
            .map(GeomBezierCurve::nb_poles)
            .ok_or_else(null_curve);
    }
    if curve.is_instance_name("Geom_BSplineCurve") {
        let bspline: Handle<GeomBSplineCurve> = curve.cast();
        return bspline
            .get()
            .map(GeomBSplineCurve::nb_poles)
            .ok_or_else(null_curve);
    }
    Ok(10)
}

/// The Standard_NullObject of a curve handle that does not cast to its dynamic type
/// (never, natively).
fn null_curve() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "BRepBuilderAPI_FindPlane: null curve",
    )
}

/// `nb_pnts` points: the ends and evenly spaced parameters between them.
fn sample_curve(
    curve: &Handle<dyn GeomCurve>,
    (first, last): (f64, f64),
    nb_pnts: i32,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    let c = curve.get().ok_or_else(null_curve)?;
    for i in 1..=nb_pnts {
        let parameter: f64 = if i == 1 {
            first
        } else if i == nb_pnts {
            last
        } else {
            first + (last - first) / f64::from(nb_pnts - 1) * f64::from(i - 1)
        };
        points.push(c.d0(parameter)?);
    }
    Ok(())
}

/// The plane through the first point, the point farthest from it and the point
/// farthest from their line, when those are distinct and not aligned.
fn plane_through_points(
    points: &[Point3],
    tol2: f64,
) -> Result<Option<Handle<GeomPlane>>, Failure> {
    if points.len() <= 2 {
        return Ok(None);
    }
    let p0: Point3 = points[0];
    let mut dis_max: f64 = 0.0;
    let mut p1 = Point3::default();
    for point in &points[1..] {
        let dist: f64 = p0.square_distance(point);
        if dist > dis_max {
            dis_max = dist;
            p1 = *point;
        }
    }
    // A NaN distance or tolerance fails every test below, as natively.
    let v1 = Vector3::between(p0, p1);
    let p2: Point3 = farthest_from_line(points, p0, v1);
    let spread: bool = dis_max > tol2 && p0.square_distance(&p2) > tol2;
    if !spread {
        return Ok(None);
    }
    let d1 = Direction3::from_vector(v1)?;
    let d2 = Direction3::from_vector(Vector3::between(p0, p2))?;
    if d1.is_parallel(d2, precision::angular()) {
        return Ok(None);
    }
    let frame = Axis3::new(p0, d1.crossed(d2)?, d1)?;
    Ok(Some(Handle::new(GeomPlane::new(frame))))
}

/// The point with the largest `|V1 ^ P0P|` (the first point when none is above 0).
fn farthest_from_line(points: &[Point3], p0: Point3, v1: Vector3) -> Point3 {
    let mut pro_max: f64 = 0.0;
    let mut p2: Point3 = p0;
    for point in &points[1..] {
        let v3 = v1.crossed(Vector3::between(p0, *point));
        let pro: f64 = v3.square_magnitude();
        if pro > pro_max {
            pro_max = pro;
            p2 = *point;
        }
    }
    p2
}
