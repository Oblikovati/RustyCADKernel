//! `PrsDim` package functions extracting the measured geometry: nearest and farthest
//! vertices, the curves of edges (optionally projected on a plane) and point projections.
use super::PrsDim;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomEllipse, GeomLine, GeomPlane, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeombase::geom_proj_lib::project_on_plane;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Axis3, Line3, Plane3, Point3, Vector3};

/// `SquareTolerance`: `Precision::SquareConfusion()`.
pub(super) const SQUARE_TOLERANCE: f64 = 1e-14;

/// The first and last points of a curve: `(first, last)`.
pub type PrsDimEnds = (Point3, Point3);

/// A curve with its ends.
pub type PrsDimCurveEnds = (Handle<dyn GeomCurve>, PrsDimEnds);

/// A curve with its ends and whether it is infinite.
pub type PrsDimCurveInfinite = (Handle<dyn GeomCurve>, PrsDimEnds, bool);

/// The basis of a trimmed curve, else the curve.
pub(super) fn untrimmed(curve: Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    match curve.cast::<GeomTrimmedCurve>().get() {
        Some(trimmed) if curve.is_instance(GeomTrimmedCurve::static_type()) => {
            trimmed.basis_curve()
        }
        _ => curve,
    }
}

/// The curve of an adaptor, transformed by its location.
pub(super) fn adaptor_curve(adaptor: &BRepAdaptorCurve) -> Result<Handle<dyn GeomCurve>, Failure> {
    let transformed = adaptor.transformed_curve();
    let curve = transformed
        .curve()
        .get()
        .map(|c| c.curve())
        .unwrap_or_default();
    let Some(value) = curve.get() else {
        return Ok(Handle::default());
    };
    Ok(value
        .transformed(transformed.trsf())?
        .cast::<dyn GeomCurve>())
}

/// The ends of the line, circle or ellipse of a curve at two parameters.
pub(super) fn conic_ends(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<Option<PrsDimEnds>, Failure> {
    if let Some(line) = curve.cast::<GeomLine>().get() {
        let lin = line.lin();
        return Ok(Some((
            elclib::value(first, &lin),
            elclib::value(last, &lin),
        )));
    }
    if let Some(circle) = curve.cast::<GeomCircle>().get() {
        let circ = circle.circ()?;
        return Ok(Some((
            elclib::value(first, &circ),
            elclib::value(last, &circ),
        )));
    }
    if let Some(ellipse) = curve.cast::<GeomEllipse>().get() {
        let elips = ellipse.elips()?;
        return Ok(Some((
            elclib::value(first, &elips),
            elclib::value(last, &elips),
        )));
    }
    Ok(None)
}

fn vertices(shape: &Shape) -> Result<Vec<Point3>, Failure> {
    let mut points = Vec::new();
    for vertex in Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        points.push(BRepTool::pnt(&topods::vertex(&vertex)?)?);
    }
    Ok(points)
}

impl PrsDim {
    /// `Nearest(shape, point)`: the vertex of a shape nearest to a point (the origin for none).
    /// Example: `PrsDim::nearest_vertex(&shape, &point)?`.
    pub fn nearest_vertex(shape: &Shape, point: &Point3) -> Result<Point3, Failure> {
        let mut best = (f64::MAX, Point3::new(0.0, 0.0, 0.0));
        for vertex in vertices(shape)? {
            let distance = point.square_distance(&vertex);
            if distance < best.0 {
                best = (distance, vertex);
            }
        }
        Ok(best.1)
    }

    /// `Nearest(line, point)`: the projection of a point on a line.
    /// Example: `PrsDim::nearest_on_line(&line, &point)?`.
    pub fn nearest_on_line(line: &Line3, point: &Point3) -> Result<Point3, Failure> {
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(*line)).cast();
        ProjectPointOnCurve::with_curve(*point, &curve)?.point(1)
    }

    /// `Nearest(curve, point, first, last, nearest)`: the projection of a point on a curve,
    /// with whether it lies between the two points; `PrsDim::nearest_on_curve(&c, &p, &a, &b)?`.
    pub fn nearest_on_curve(
        curve: &Handle<dyn GeomCurve>,
        point: &Point3,
        first: &Point3,
        last: &Point3,
    ) -> Result<(bool, Point3), Failure> {
        let projector = ProjectPointOnCurve::with_curve(*point, curve)?;
        let value = curve.get().ok_or_else(null_curve)?;
        let nearest = value.value(projector.lower_distance_parameter()?)?;
        let length = first.distance(last);
        Ok((
            nearest.distance(first) <= length && nearest.distance(last) <= length,
            nearest,
        ))
    }

    /// `Farest`: the vertex of a shape farthest from a point (the origin for none).
    /// Example: `PrsDim::farest(&shape, &point)?`.
    pub fn farest(shape: &Shape, point: &Point3) -> Result<Point3, Failure> {
        let mut best = (0.0, Point3::new(0.0, 0.0, 0.0));
        for vertex in vertices(shape)? {
            let distance = point.square_distance(&vertex);
            if distance > best.0 {
                best = (distance, vertex);
            }
        }
        Ok(best.1)
    }

    /// `ComputeGeometry(edge, curve, first, last)`: the 3D curve of an edge (located,
    /// untrimmed) with its ends, for a line, circle or ellipse only.
    /// Example: `PrsDim::compute_edge_geometry(&edge)?`.
    pub fn compute_edge_geometry(edge: &Edge) -> Result<Option<PrsDimCurveEnds>, Failure> {
        let found = BRepTool::curve_with_location(edge)?;
        let Some(curve) = found.curve.get() else {
            return Ok(None);
        };
        let mut curve: Handle<dyn GeomCurve> = if found.location.is_identity() {
            found.curve.clone()
        } else {
            curve.transformed(found.location.transformation())?.cast()
        };
        curve = untrimmed(curve);
        Ok(conic_ends(&curve, found.first, found.last)?.map(|ends| (curve, ends)))
    }

    /// `ComputeGeometry(edge, curve, first, last, isInfinite)`: the located untrimmed curve
    /// of an edge adaptor, its ends (the origin when infinite) and whether it is infinite.
    /// Example: `PrsDim::compute_edge_geometry_infinite(&edge)?`.
    pub fn compute_edge_geometry_infinite(
        edge: &Edge,
    ) -> Result<Option<PrsDimCurveInfinite>, Failure> {
        let adaptor = BRepAdaptorCurve::with_edge(edge)?;
        let curve = adaptor_curve(&adaptor)?;
        if curve.is_null() {
            return Ok(None);
        }
        let (first, last) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
        let infinite = precision::is_infinite(first) || precision::is_infinite(last);
        let curve = untrimmed(curve);
        let value = curve.get().ok_or_else(null_curve)?;
        let ends = if infinite {
            (gp::origin(), gp::origin())
        } else {
            (value.value(first)?, value.value(last)?)
        };
        Ok(Some((curve, ends, infinite)))
    }

    /// `ComputeGeometry(edge, ..., extCurve, isInfinite, isOnPlane, plane)`: the curve of an
    /// edge projected on a plane, with its original when off the plane.
    /// Example: `PrsDim::compute_edge_geometry_on_plane(&edge, &plane)?`.
    pub fn compute_edge_geometry_on_plane(
        edge: &Edge,
        plane: &Handle<GeomPlane>,
    ) -> Result<Option<PrsDimProjectedEdge>, Failure> {
        let Some(geom_plane) = plane.get() else {
            return Ok(None);
        };
        let adaptor = BRepAdaptorCurve::with_edge(edge)?;
        let curve = adaptor_curve(&adaptor)?;
        if curve.is_null() {
            return Ok(None);
        }
        let (first, last) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
        let infinite = precision::is_infinite(first) || precision::is_infinite(last);
        let pln = geom_plane.pln();
        let ext = untrimmed(curve.clone());
        let mut on_plane = true;
        if let Some(line) = ext.cast::<GeomLine>().get() {
            on_plane = pln.contains_line(line.lin(), precision::confusion(), precision::angular());
        } else if let Some(circle) = ext.cast::<GeomCircle>().get() {
            let position = Axis3::from_axis2(circle.circ()?.position());
            on_plane =
                position.is_coplanar(pln.position(), precision::confusion(), precision::angular());
        }
        let ext_curve = if on_plane { Handle::default() } else { ext };
        let projected = project_on_plane(&curve, plane, pln.axis().direction(), false)?;
        let mut ends = (Point3::default(), Point3::default());
        if let Some(line) = projected.cast::<GeomLine>().get() {
            if !infinite {
                ends = (
                    elclib::value(first, &line.lin()),
                    elclib::value(last, &line.lin()),
                );
            }
        } else if let Some(found) = conic_ends(&projected, first, last)? {
            ends = found;
        } else {
            return Ok(None);
        }
        Ok(Some(PrsDimProjectedEdge {
            curve: projected,
            ends,
            ext_curve,
            infinite,
            on_plane,
        }))
    }

    /// `ComputeGeomCurve`: whether a curve lies in a plane (20 samples), its ends, and when
    /// off the plane the curve and ends projected on it.
    /// Example: `PrsDim::compute_geom_curve(&mut curve, f, l, &mut ends, &plane)?`.
    pub fn compute_geom_curve(
        curve: &mut Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        ends: &mut PrsDimEnds,
        plane: &Handle<GeomPlane>,
    ) -> Result<bool, Failure> {
        const NODES: i32 = 20;
        let pln = plane.get().ok_or_else(null_curve)?.pln();
        let mut delta = (last - first) / f64::from(NODES - 1);
        if delta <= precision::p_confusion() {
            delta = last - first;
        }
        let value = curve.get().ok_or_else(null_curve)?;
        let mut on_plane = true;
        let mut parameter = first;
        for _ in 0..NODES {
            if pln.square_distance(value.value(parameter)?) > SQUARE_TOLERANCE {
                on_plane = false;
                break;
            }
            parameter += delta;
        }
        let finite = !precision::is_infinite(first) && !precision::is_infinite(last);
        if finite {
            *ends = (value.value(first)?, value.value(last)?);
        }
        if !on_plane {
            *curve = untrimmed(project_on_plane(
                curve,
                plane,
                pln.axis().direction(),
                false,
            )?);
            if finite {
                *ends = (
                    Self::project_point_on_plane(&ends.0, &pln),
                    Self::project_point_on_plane(&ends.1, &pln),
                );
            }
        }
        Ok(on_plane)
    }

    /// `ComputeGeometry(vertex, point, plane, isOnPlane)`: the vertex point, projected on the
    /// plane when off it; `PrsDim::compute_vertex_geometry(&vertex, &plane)?`.
    pub fn compute_vertex_geometry(
        vertex: &Vertex,
        plane: &Handle<GeomPlane>,
    ) -> Result<(Point3, bool), Failure> {
        let point = BRepTool::pnt(vertex)?;
        let pln = plane.get().ok_or_else(null_curve)?.pln();
        let on_plane = pln.contains(point, precision::confusion());
        if on_plane {
            return Ok((point, true));
        }
        Ok((Self::project_point_on_plane(&point, &pln), false))
    }

    /// `ProjectPointOnPlane`; `PrsDim::project_point_on_plane(&point, &plane)`.
    pub fn project_point_on_plane(point: &Point3, plane: &Plane3) -> Point3 {
        let vector = Vector3::between(plane.location(), *point);
        let normal = Vector3::from(plane.axis().direction());
        let normal = normal * vector.dot(normal);
        point.translated(normal.reversed())
    }

    /// `ProjectPointOnLine`; `PrsDim::project_point_on_line(&point, &line)`.
    pub fn project_point_on_line(point: &Point3, line: &Line3) -> Point3 {
        let location = line.location().xyz();
        let direction = line.direction().xyz();
        let parameter = (point.xyz() - location).dot(direction);
        Point3::from(location + direction * parameter)
    }
}

/// The result of `ComputeGeometry` on a plane: the projected curve, its ends, the original
/// curve when off the plane, and the infinite and in-plane flags.
#[derive(Clone)]
pub struct PrsDimProjectedEdge {
    /// The curve projected on the plane.
    pub curve: Handle<dyn GeomCurve>,
    /// Its ends (unset for an infinite line).
    pub ends: PrsDimEnds,
    /// The untrimmed original curve when it is off the plane, else null.
    pub ext_curve: Handle<dyn GeomCurve>,
    /// Whether a parameter is infinite.
    pub infinite: bool,
    /// Whether the original lies in the plane.
    pub on_plane: bool,
}

pub(super) fn null_curve() -> Failure {
    Failure::with_message("Standard_NullObject", "PrsDim: null geometry")
}
