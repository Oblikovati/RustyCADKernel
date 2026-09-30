// Rust translation of BRepBndLib::Add and BRepBndLib::AddClose (BRepBndLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBndLib::Add` and `AddClose`, and the polygon node loops that `AddOptimal`
//! shares with `Add`.
use super::BRepBndLib;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::bnd_lib::{Add3dCurve, AddSurface};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;

impl BRepBndLib {
    /// `Add(S, B, useTriangulation)`: adds each face of `shape` (its triangulation
    /// when it has one and `use_triangulation`, or when it has no surface; else its
    /// surface, or the 3d curves of its edges for a plane), then each edge outside a
    /// face (its 3d polygon, its polygon on a triangulation, or its curve), then each
    /// vertex outside an edge, each enlarged by its tolerance (and the deflection of
    /// its mesh); `BRepBndLib::add(&shape, &mut bounds, true)?`.
    pub fn add(
        shape: &Shape,
        bounds: &mut BoundingBox3,
        use_triangulation: bool,
    ) -> Result<(), Failure> {
        for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            add_face(&topods::face(&face)?, bounds, use_triangulation)?;
        }
        for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Face)? {
            add_free_edge(&topods::edge(&edge)?, bounds, use_triangulation)?;
        }
        add_free_vertices(shape, bounds, true)
    }

    /// `AddClose(S, B)`: adds the exact 3d curve of each edge of `shape` and each
    /// vertex outside an edge, without any tolerance; the faces themselves are not
    /// added (only right for polygonal planar faces, as natively);
    /// `BRepBndLib::add_close(&shape, &mut bounds)?`.
    pub fn add_close(shape: &Shape, bounds: &mut BoundingBox3) -> Result<(), Failure> {
        for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = topods::edge(&edge)?;
            if BRepTool::is_geometric_edge(&edge)? {
                Add3dCurve::add(&BRepAdaptorCurve::with_edge(&edge)?, 0.0, bounds)?;
            }
        }
        add_free_vertices(shape, bounds, false)
    }
}

/// One face of `Add`: its triangulation when allowed and not void, else its surface.
fn add_face(
    face: &Face,
    bounds: &mut BoundingBox3,
    use_triangulation: bool,
) -> Result<(), Failure> {
    let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
    let (surface, _) = BRepTool::surface_with_location(face)?;
    // The mesh is only asked for its bounds when the first two tests pass (`&&`).
    if (use_triangulation || surface.is_null()) && mesh_min_max(&mesh, &location, bounds) {
        let deflection: f64 = mesh.get().map_or(0.0, Triangulation::deflection);
        bounds.enlarge(deflection + BRepTool::face_tolerance(face)?);
        return Ok(());
    }
    if surface.is_null() {
        return Ok(());
    }
    add_face_geometry(face, bounds)
}

/// `!T.IsNull() && T->MinMax(B, l)`: extends `bounds` by the (located) mesh nodes.
fn mesh_min_max(
    mesh: &rustycad_tkernel::standard::Handle<Triangulation>,
    location: &Location,
    bounds: &mut BoundingBox3,
) -> bool {
    mesh.get()
        .is_some_and(|mesh| mesh.min_max(bounds, location.transformation(), false))
}

/// The exact face of `Add`: the surface box, except on a plane with edges, where the
/// 3d curves of its geometric edges are added and the box enlarged by the face
/// tolerance ("on travaille directement sur les courbes 3d").
fn add_face_geometry(face: &Face, bounds: &mut BoundingBox3) -> Result<(), Failure> {
    let unrestricted = BRepAdaptorSurface::with_face(face, false)?;
    let mut edges = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    if unrestricted.get_type()? != SurfaceType::Plane || !edges.more() {
        let surface = BRepAdaptorSurface::with_face(face, true)?;
        return AddSurface::add(&surface, BRepTool::face_tolerance(face)?, bounds);
    }
    for edge in &mut edges {
        let edge = topods::edge(&edge)?;
        if BRepTool::is_geometric_edge(&edge)? {
            let curve = BRepAdaptorCurve::with_edge(&edge)?;
            Add3dCurve::add(&curve, BRepTool::edge_tolerance(&edge)?, bounds)?;
        }
    }
    bounds.enlarge(BRepTool::face_tolerance(face)?);
    Ok(())
}

/// One edge outside a face in `Add`: the curve when the meshes are not wanted, else
/// its 3d polygon, its first polygon on a triangulation, or its curve.
fn add_free_edge(
    edge: &Edge,
    bounds: &mut BoundingBox3,
    use_triangulation: bool,
) -> Result<(), Failure> {
    if !use_triangulation && BRepTool::is_geometric_edge(edge)? {
        return add_edge_curve(edge, BRepTool::edge_tolerance(edge)?, bounds);
    }
    let (polygon, location) = BRepTool::polygon3d(edge)?;
    if let Some(polygon) = polygon.get().filter(|p| p.nb_nodes() > 0) {
        add_polygon3d_nodes(polygon, &location, bounds);
        bounds.enlarge(polygon.deflection() + BRepTool::edge_tolerance(edge)?);
        return Ok(());
    }
    let (polygon, mesh, location) = BRepTool::first_polygon_on_triangulation(edge)?;
    if let Some((polygon, mesh)) = mesh_polygon(&polygon, &mesh) {
        add_mesh_polygon_nodes(polygon, mesh, &location, bounds)?;
        bounds.enlarge(polygon.deflection() + BRepTool::edge_tolerance(edge)?);
        return Ok(());
    }
    if BRepTool::is_geometric_edge(edge)? {
        add_edge_curve(edge, BRepTool::edge_tolerance(edge)?, bounds)?;
    }
    Ok(())
}

/// `BndLib_Add3dCurve::Add(BRepAdaptor_Curve(E), Tol, B)`.
pub(super) fn add_edge_curve(
    edge: &Edge,
    tol: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    Add3dCurve::add(&BRepAdaptorCurve::with_edge(edge)?, tol, bounds)
}

/// `!Poly.IsNull() && !T.IsNull() && T->NbNodes() > 0`: the polygon and its mesh.
pub(super) fn mesh_polygon<'m>(
    polygon: &'m rustycad_tkernel::standard::Handle<PolygonOnTriangulation>,
    mesh: &'m rustycad_tkernel::standard::Handle<Triangulation>,
) -> Option<(&'m PolygonOnTriangulation, &'m Triangulation)> {
    let polygon = polygon.get()?;
    let mesh = mesh.get().filter(|mesh| mesh.nb_nodes() > 0)?;
    Some((polygon, mesh))
}

/// The nodes of a 3d polygon, moved by `location` unless it is the identity.
pub(super) fn add_polygon3d_nodes(
    polygon: &Polygon3D,
    location: &Location,
    bounds: &mut BoundingBox3,
) {
    let nodes = polygon.nodes();
    for i in 1..=polygon.nb_nodes() {
        bounds.add_point(located(nodes[i], location));
    }
}

/// The mesh nodes of a polygon on a triangulation, moved by `location` unless it is the
/// identity; Standard_OutOfRange for a node index outside the mesh.
pub(super) fn add_mesh_polygon_nodes(
    polygon: &PolygonOnTriangulation,
    mesh: &Triangulation,
    location: &Location,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    let indices = polygon.nodes();
    for i in 1..=indices.length() {
        bounds.add_point(located(mesh.node(indices[i])?, location));
    }
    Ok(())
}

/// `l.IsIdentity() ? P : P.Transformed(l)`.
pub(super) fn located(
    point: rustycad_tkmath::gp::Point3,
    location: &Location,
) -> rustycad_tkmath::gp::Point3 {
    if location.is_identity() {
        point
    } else {
        point.transformed(location.transformation())
    }
}

/// The vertices outside an edge, each enlarged by its tolerance when `enlarge`.
fn add_free_vertices(
    shape: &Shape,
    bounds: &mut BoundingBox3,
    enlarge: bool,
) -> Result<(), Failure> {
    for vertex in Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Edge)? {
        let vertex = topods::vertex(&vertex)?;
        bounds.add_point(BRepTool::pnt(&vertex)?);
        if enlarge {
            bounds.enlarge(BRepTool::vertex_tolerance(&vertex)?);
        }
    }
    Ok(())
}
