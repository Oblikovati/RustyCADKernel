// Rust translation of DBRep_DrawableShape::addMeshNormals and ::addSurfaceNormals
// (Draw/TKDraw/DBRep/DBRep_DrawableShape.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The normal segments the `normals` command draws: at the triangulation nodes, or on
//! a grid of surface parameters.
use super::DBRepDrawableShape;
use crate::draw::DrawConsole;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape, face as to_face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::format_general;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::poly::mesh_purpose;

/// A normal as a segment: `[from, to]`.
pub type NormalSegment = (Point3, Point3);

/// The normals of each face (`NCollection_DataMap<TopoDS_Face, ...>`, keyed with
/// `IsEqual`), in first-seen order.
pub type MeshNormals = Vec<(Face, Vec<NormalSegment>)>;

/// The length below which a normal counts as null.
const NULL_NORMAL: f64 = 1.0e-10;

impl DBRepDrawableShape {
    /// `addMeshNormals(theNormals, theFace, theLength)`: a segment of `length` along the
    /// normal at each triangulation node (stored normals, else the surface normal at
    /// the UV node); false without a triangulation or without normals and UV nodes.
    /// Native quirk kept: stored normals are not moved by the face location.
    pub fn add_mesh_normals(
        normals: &mut Vec<NormalSegment>,
        face: &Face,
        length: f64,
        out: &dyn DrawConsole,
    ) -> Result<bool, Failure> {
        let (triangulation, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
        let Some(mesh) = triangulation.get() else {
            return Ok(false);
        };
        let has_normals = mesh.has_normals();
        if !has_normals && !mesh.has_uv_nodes() {
            return Ok(false);
        }
        let surface = BRepAdaptorSurface::with_face(face, true)?;
        for node in 1..=mesh.nb_nodes() {
            let mut start = mesh.node(node)?;
            if !location.is_identity() {
                start = start.transformed(location.transformation());
            }
            let normal = if has_normals {
                Vector3::from(mesh.normal(node)?)
            } else {
                let uv = mesh.uv_node(node)?;
                let d1 = surface.d1(uv.x(), uv.y())?;
                d1.d1u.crossed(d1.d1v)
            };
            let normal = scaled_normal(normal, length, || {
                let (x, y, z) = (
                    format_general(start.x(), 6),
                    format_general(start.y(), 6),
                    format_general(start.z(), 6),
                );
                out.write_out(&format!("Null normal at node X = {x}, Y = {y}, Z = {z}\n"));
            });
            normals.push((start, start.translated(normal)));
        }
        Ok(true)
    }

    /// `addMeshNormals(theNormals, theShape, theLength)`: the mesh normals of each face.
    pub fn add_shape_mesh_normals(
        normals: &mut MeshNormals,
        shape: &Shape,
        length: f64,
        out: &dyn DrawConsole,
    ) -> Result<(), Failure> {
        for face in faces_of(shape)? {
            let segments = entry(normals, &face);
            Self::add_mesh_normals(segments, &face, length, out)?;
        }
        Ok(())
    }

    /// `addSurfaceNormals(theNormals, theFace, theLength, theNbAlongU, theNbAlongV)`: a
    /// segment of `length` along the surface normal at `nb_u` x `nb_v` parameters
    /// spanning the UV bounds (the middle for a count of 1); false without surface.
    pub fn add_surface_normals(
        normals: &mut Vec<NormalSegment>,
        face: &Face,
        length: f64,
        (nb_u, nb_v): (i32, i32),
        out: &dyn DrawConsole,
    ) -> Result<bool, Failure> {
        if BRepTool::surface_with_location(face)?.0.is_null() {
            return Ok(false);
        }
        let (u_min, u_max, v_min, v_max) = uv_bounds(face)?;
        let (mid_u, mid_v) = (nb_u == 1, nb_v == 1);
        let du = (u_max - u_min) / f64::from(if mid_u { 2 } else { nb_u - 1 });
        let dv = (v_max - v_min) / f64::from(if mid_v { 2 } else { nb_v - 1 });
        let surface = BRepAdaptorSurface::with_face(face, true)?;
        for i in 0..nb_u {
            let u = u_min + f64::from(if mid_u { 1 } else { i }) * du;
            for j in 0..nb_v {
                let v = v_min + f64::from(if mid_v { 1 } else { j }) * dv;
                let d1 = surface.d1(u, v)?;
                let normal = scaled_normal(d1.d1u.crossed(d1.d1v), length, || {
                    let (u, v) = (format_general(u, 6), format_general(v, 6));
                    out.write_out(&format!("Null normal at U = {u}, V = {v}\n"));
                });
                normals.push((d1.point, d1.point.translated(normal)));
            }
        }
        Ok(true)
    }

    /// `addSurfaceNormals(theNormals, theShape, ...)`: the surface normals of each face.
    pub fn add_shape_surface_normals(
        normals: &mut MeshNormals,
        shape: &Shape,
        length: f64,
        counts: (i32, i32),
        out: &dyn DrawConsole,
    ) -> Result<(), Failure> {
        for face in faces_of(shape)? {
            let segments = entry(normals, &face);
            Self::add_surface_normals(segments, &face, length, counts, out)?;
        }
        Ok(())
    }
}

/// The normal scaled to `length`; a null one becomes `(|n| / 2, 0, 0)` and is reported.
fn scaled_normal(normal: Vector3, length: f64, report: impl FnOnce()) -> Vector3 {
    let magnitude = normal.magnitude();
    if magnitude > NULL_NORMAL {
        return normal * (length / magnitude);
    }
    report();
    Vector3::new(magnitude / 2.0, 0.0, 0.0)
}

/// The faces of `shape` in exploration order.
fn faces_of(shape: &Shape) -> Result<Vec<Face>, Failure> {
    let mut faces = Vec::new();
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while explorer.more() {
        faces.push(to_face(explorer.current())?);
        explorer.next()?;
    }
    Ok(faces)
}

/// The segments of `face` in the map, bound to an empty list on first use.
fn entry<'a>(normals: &'a mut MeshNormals, face: &Face) -> &'a mut Vec<NormalSegment> {
    let index = match normals.iter().position(|(key, _)| key.is_equal(face)) {
        Some(index) => index,
        None => {
            normals.push((face.clone(), Vec::new()));
            normals.len() - 1
        }
    };
    &mut normals[index].1
}
