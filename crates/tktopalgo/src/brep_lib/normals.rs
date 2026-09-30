// Rust translation of BRepLib::EnsureNormalConsistency (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::EnsureNormalConsistency`: the normals of the face triangulations, equal on
//! both sides of smooth edges.
use super::errors::null_object;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tkmath::gp::{Direction3, Xyz};
use rustycad_tkmath::poly::{PolygonOnTriangulation, Triangulation, mesh_purpose};

/// The default arguments of `BRepLib::EnsureNormalConsistency` (angular tolerance 0.001,
/// no forced computation).
pub const ENSURE_NORMAL_CONSISTENCY_DEFAULTS: (f64, bool) = (0.001, false);

/// `BRepLib::EnsureNormalConsistency(S, theAngTol, ForceComputeNormals)`: computes the
/// surface normals of the triangulations without normals (all of them when forced;
/// a null normal where the surface normal is undefined), then, on every edge of two
/// triangulated faces with normals, replaces the normals of the edge nodes by their
/// mean where they differ by less than `ang_tol`; true when a normal was set.
/// `brep_lib::ensure_normal_consistency(&shape, 0.001, false)?`.
pub fn ensure_normal_consistency(
    shape: &Shape,
    ang_tol: f64,
    force_compute_normals: bool,
) -> Result<bool, Failure> {
    let thres_dot: f64 = ang_tol.cos();
    let (mut ret_val, found) = compute_face_normals(shape, force_compute_normals)?;
    if !found {
        return Ok(ret_val);
    }
    // loop by edges
    let mut map_ef = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut map_ef)?;
    for index in 1..=map_ef.extent() {
        let faces = map_ef.find_from_index(index)?;
        if faces.extent() != 2 {
            continue;
        }
        let edge = Edge::unchecked(map_ef.find_key(index)?.clone());
        let mut it = faces.iter();
        let face1 = Face::unchecked(it.next().expect("two faces").clone());
        let face2 = Face::unchecked(it.next().expect("two faces").clone());
        ret_val = smooth_edge_normals(&edge, (&face1, &face2), thres_dot)? || ret_val;
    }
    Ok(ret_val)
}

/// "compute normals if they are absent": the surface normals of every triangulation
/// without normals (or of every one when forced); returns (a normal was set, some
/// triangulation has normals).
fn compute_face_normals(shape: &Shape, force: bool) -> Result<(bool, bool), Failure> {
    let (mut ret_val, mut found) = (false, false);
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while explorer.more() {
        let face = Face::unchecked(explorer.current().clone());
        explorer.next()?;
        let surface = BRepTool::surface(&face)?;
        if surface.is_null() {
            continue;
        }
        let (triangulation, _) = BRepTool::triangulation(&face, mesh_purpose::NONE)?;
        let Some(mesh) = triangulation.get() else {
            continue;
        };
        if !force && mesh.has_normals() {
            found = true;
            continue;
        }
        mesh.add_normals()?;
        let mut props = SLProps::new(surface, 2, precision::confusion())?;
        for node in 1..=mesh.nb_nodes() {
            let uv = mesh.uv_node(node)?;
            props.set_parameters(uv.x(), uv.y())?;
            if !props.is_normal_defined()? {
                mesh.set_normal_vec3(node, Vec3::new(0.0f32, 0.0f32, 0.0f32))?;
                continue;
            }
            let mut normal: Direction3 = props.normal()?;
            if face.orientation() == Orientation::Reversed {
                normal.reverse();
            }
            mesh.set_normal(node, normal)?;
        }
        ret_val = true;
        found = true;
    }
    Ok((ret_val, found))
}

/// A face triangulation and the polygon of an edge on it.
type EdgeMesh = (Handle<Triangulation>, Handle<PolygonOnTriangulation>);

/// The nodes of the edge on both triangulations, when both exist with normals and have
/// the same bounds.
fn edge_polygons(
    edge: &Edge,
    (face1, face2): (&Face, &Face),
) -> Result<Option<[EdgeMesh; 2]>, Failure> {
    let (pt1, loc1) = BRepTool::triangulation(face1, mesh_purpose::NONE)?;
    let (pt2, loc2) = BRepTool::triangulation(face2, mesh_purpose::NONE)?;
    let (Some(mesh1), Some(mesh2)) = (pt1.get(), pt2.get()) else {
        return Ok(None);
    };
    if !mesh1.has_normals() || !mesh2.has_normals() {
        return Ok(None);
    }
    let poly1 = BRepTool::polygon_on_triangulation(edge, &pt1, &loc1)?;
    let poly2 = BRepTool::polygon_on_triangulation(edge, &pt2, &loc2)?;
    // Natively a missing polygon is dereferenced (a crash).
    let missing = || {
        null_object("BRepLib::EnsureNormalConsistency: no polygon of the edge on a triangulation")
    };
    let (nodes1, nodes2) = (
        poly1.get().ok_or_else(missing)?.nodes(),
        poly2.get().ok_or_else(missing)?.nodes(),
    );
    if nodes1.lower() != nodes2.lower() || nodes1.upper() != nodes2.upper() {
        return Ok(None);
    }
    Ok(Some([
        (pt1.clone(), poly1.clone()),
        (pt2.clone(), poly2.clone()),
    ]))
}

/// The mean normal at the edge nodes whose normals on both faces are closer than the
/// threshold; true when one was set.
fn smooth_edge_normals(
    edge: &Edge,
    faces: (&Face, &Face),
    thres_dot: f64,
) -> Result<bool, Failure> {
    let Some([(pt1, poly1), (pt2, poly2)]) = edge_polygons(edge, faces)? else {
        return Ok(false);
    };
    let (mesh1, mesh2) = (pt1.get().expect("checked"), pt2.get().expect("checked"));
    let (nodes1, nodes2) = (
        poly1.get().expect("checked").nodes(),
        poly2.get().expect("checked").nodes(),
    );
    let mut ret_val = false;
    for edge_node in nodes1.lower()..=nodes1.upper() {
        // Number of node
        let node1: i32 = *nodes1
            .value(edge_node)
            .map_err(|e| e.into_failure("BRepLib::EnsureNormalConsistency"))?;
        let node2: i32 = *nodes2
            .value(edge_node)
            .map_err(|e| e.into_failure("BRepLib::EnsureNormalConsistency"))?;
        let (n1, n2) = (mesh1.normal_vec3(node1)?, mesh2.normal_vec3(node2)?);
        let norm1 = Xyz::new(f64::from(n1.x()), f64::from(n1.y()), f64::from(n1.z()));
        let norm2 = Xyz::new(f64::from(n2.x()), f64::from(n2.y()), f64::from(n2.z()));
        if norm1.dot(norm2) > thres_dot {
            let mut sum = norm1;
            sum += norm2;
            let new_norm = Direction3::from_xyz(sum.normalized()?)?;
            mesh1.set_normal(node1, new_norm)?;
            mesh2.set_normal(node2, new_norm)?;
            ret_val = true;
        }
    }
    Ok(ret_val)
}
