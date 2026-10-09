// Rust translation of BRepFill::Axe (ModelingAlgorithms/TKBool/BRepFill/BRepFill.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill::Axe`: the frame of a profile at its nearest point on a planar spine.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_extrema::BRepExtremaExtPC;
use rustycad_tktopalgo::brep_lib::{BRepLibFindSurface, BRepLibMakeFace};

/// The squared distance under which a profile vertex is on a spine vertex.
const VERTEX_CONTACT_SQUARED: f64 = 1.0e-10;

/// A point and tangent of the spine.
type SpinePoint = (Point3, Vector3);

/// `BRepFill::Axe(Spine, Profile, AxeProf, ProfOnSpine, Tol)`: the frame at the point of
/// the planar spine (a face or a wire) nearest to the vertices of the profile, its X
/// direction along the spine tangent and its main direction the normal of the plane;
/// the flag tells whether that distance is below `tol`. Standard_NoSuchObject for a
/// non-planar face, Standard_DomainError when no plane is found.
///
/// ```ignore
/// let (axe, on_spine) = rustycad_tkbool::brep_fill::axe(&spine, &profile, 1e-7)?;
/// ```
pub fn axe(spine: &Shape, profile: &Wire, tol: f64) -> Result<(Axis3, bool), Failure> {
    let (face, normal) = spine_plane(spine)?;
    let face_forward: Shape = face.oriented(Orientation::Forward);
    let (spine_point, dist_min) = match vertex_contact(&face_forward, profile)? {
        Some(vertex) => (vertex_frame(&face, &face_forward, &vertex, profile)?, 0.),
        None => nearest_on_edges(&face_forward, profile)?,
    };
    let (location, tangent) = spine_point;
    let frame = Axis3::new(location, normal, Direction3::from_vector(tangent)?)?;
    Ok((frame, dist_min < tol))
}

/// The face of the spine and the normal of its plane (the planar face, or the plane
/// found through the face or the wire).
fn spine_plane(spine: &Shape) -> Result<(Face, Direction3), Failure> {
    let (face, mut surface, mut location): (Face, Handle<dyn GeomSurface>, Location) =
        match spine.shape_type()? {
            ShapeEnum::Face => {
                let face: Face = topods::face(spine)?;
                let (s, l) = BRepTool::surface_with_location(&face)?;
                (face, s, l)
            }
            ShapeEnum::Wire => {
                let face: Face = BRepLibMakeFace::from_wire(&topods::wire(spine)?, true)?.face()?;
                let (s, l) = BRepTool::surface_with_location(&face)?;
                (face, s, l)
            }
            _ => (Face::default(), Handle::default(), Location::default()),
        };
    if spine.shape_type()? == ShapeEnum::Face && surface.cast::<GeomPlane>().is_null() {
        let finder = BRepLibFindSurface::new(&face, -1., true, false)?;
        if !finder.found() {
            return Err(Failure::with_type(
                "Standard_NoSuchObject",
                "BRepFill_Evolved : The Face is not planar",
                "BRepFill",
            ));
        }
        surface = finder.surface();
        location = finder.location();
    }
    let Some(geometry) = surface.get() else {
        return Err(Failure::with_type(
            "Standard_DomainError",
            "BRepFill_Evolved::Axe",
            "BRepFill",
        ));
    };
    if !location.is_identity() {
        surface = geometry
            .transformed(location.transformation())?
            .cast::<dyn GeomSurface>();
    }
    let plane: Handle<GeomPlane> = surface.cast::<GeomPlane>();
    let Some(plane) = plane.get() else {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "BRepFill::Axe: the spine surface is not a Geom_Plane; expected a plane",
            "BRepFill",
        ));
    };
    Ok((face, plane.pln().axis().direction()))
}

/// The first vertex of the spine face (FORWARD) within `sqrt(1e-10)` of a vertex of
/// the profile, if any.
fn vertex_contact(face_forward: &Shape, profile: &Wire) -> Result<Option<Shape>, Failure> {
    let mut se = Explorer::new(face_forward, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    while se.more() {
        let p1: Point3 = BRepTool::pnt(&topods::vertex(se.current())?)?;
        let mut pe = Explorer::new(profile, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        while pe.more() {
            let p2: Point3 = BRepTool::pnt(&topods::vertex(pe.current())?)?;
            if p1.square_distance(&p2) <= VERTEX_CONTACT_SQUARED {
                return Ok(Some(se.current().clone()));
            }
            pe.next()?;
        }
        se.next()?;
    }
    Ok(None)
}

/// The point and unit tangent at a spine vertex of one of its two edges: the edge along
/// which the profile edges extend least.
fn vertex_frame(
    face: &Face,
    face_forward: &Shape,
    vertex: &Shape,
    profile: &Wire,
) -> Result<SpinePoint, Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(face_forward, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map)?;
    let edges = map.find_from_key(vertex)?;
    let (Some(first), Some(last)) = (edges.first(), edges.last()) else {
        return Err(Failure::with_type(
            "Standard_NoSuchObject",
            "BRepFill::Axe: a spine vertex without edge; expected two edges",
            "BRepFill",
        ));
    };
    let (loc1, mut tang1) = edge_tangent(&topods::edge(first)?, vertex, face)?;
    let (loc2, mut tang2) = edge_tangent(&topods::edge(last)?, vertex, face)?;
    tang1.normalize()?;
    tang2.normalize()?;
    let (sca1, sca2) = profile_extents(profile, tang1, tang2)?;
    if sca1.abs() < sca2.abs() {
        return Ok((loc1, tang1));
    }
    Ok((loc2, tang2))
}

/// The point and derivative of the located 3d curve of an edge at a vertex, the
/// derivative reversed for a REVERSED edge.
fn edge_tangent(edge: &Edge, vertex: &Shape, face: &Face) -> Result<SpinePoint, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let parameter: f64 = BRepTool::parameter_on_face(&topods::vertex(vertex)?, edge, face)?;
    let Some(curve) = found.curve.get() else {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "BRepFill::Axe: a spine edge without 3d curve; expected a curve",
            "BRepFill",
        ));
    };
    let (mut point, mut tangent) = curve.d1(parameter)?;
    if !found.location.is_identity() {
        tangent.transform(found.location.transformation());
        point.transform(found.location.transformation());
    }
    if edge.orientation() == Orientation::Reversed {
        tangent.reverse();
    }
    Ok((point, tangent))
}

/// The sums over the profile edges of the absolute projections of their chords on the
/// two tangents.
fn profile_extents(profile: &Wire, tang1: Vector3, tang2: Vector3) -> Result<(f64, f64), Failure> {
    let (mut sca1, mut sca2) = (0., 0.);
    let mut pe = Explorer::new(profile, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while pe.more() {
        let (v1, v2) = topexp::vertices(&topods::edge(pe.current())?, false);
        let chord = Vector3::between(BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?);
        sca1 += tang1.dot(chord).abs();
        sca2 += tang2.dot(chord).abs();
        pe.next()?;
    }
    Ok((sca1, sca2))
}

/// The spine point nearest to the profile vertices over the edges of the face (the
/// first minimum of BRepExtrema_ExtPC of each pair), its tangent and the distance.
fn nearest_on_edges(face_forward: &Shape, profile: &Wire) -> Result<(SpinePoint, f64), Failure> {
    let mut dist_min: f64 = precision::infinite();
    let mut nearest: SpinePoint = (Point3::default(), Vector3::default());
    let mut par: f64 = 0.;
    let mut se = Explorer::new(face_forward, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while se.more() {
        let edge: Edge = topods::edge(se.current())?;
        let mut extrema = BRepExtremaExtPC::new();
        extrema.initialize(&edge)?;
        let mut pe = Explorer::new(profile, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        while pe.more() {
            extrema.perform(&topods::vertex(pe.current())?)?;
            let dist: f64 = first_minimum(&extrema, &mut par)?;
            if dist < dist_min {
                dist_min = dist;
                let (point, mut tangent) = BRepAdaptorCurve::with_edge(&edge)?.d1(par)?;
                if edge.orientation() == Orientation::Reversed {
                    tangent.reverse();
                }
                nearest = (point, tangent);
            }
            pe.next()?;
        }
        se.next()?;
    }
    Ok((nearest, dist_min))
}

/// The distance of the first minimum of the extrema (infinite when there is none) and
/// its parameter, written into `par` (left unchanged otherwise, as natively).
fn first_minimum(extrema: &BRepExtremaExtPC, par: &mut f64) -> Result<f64, Failure> {
    if !extrema.is_done() {
        return Ok(precision::infinite());
    }
    for i in 1..=extrema.nb_ext()? {
        if extrema.is_min(i)? {
            *par = extrema.parameter(i)?;
            return Ok(extrema.square_distance(i)?.sqrt());
        }
    }
    Ok(precision::infinite())
}
