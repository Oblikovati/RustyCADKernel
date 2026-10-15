// SPDX-License-Identifier: GPL-2.0-only
//! Reads one meshed face of the detached copy into owned triangles and normals.
use super::detached::{native, refused};
use super::mesh::FaceTriangles;
use crate::query_error::{BodyQueryError, BodyQueryFailure, QueryEntity};
use crate::topology::FaceIndex;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::poly::{Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::BRepLibToolTriangulatedShape;

/// A meshed face of the copy: the face, its triangulation and that triangulation's
/// location. Edge polylines index into the same triangulation.
pub(crate) struct MeshedFace {
    pub face: Face,
    pub triangulation: Handle<Triangulation>,
    pub location: Location,
}

/// The triangulation of the face at `index`; a face without one is refused.
pub(crate) fn meshed_face(face: &Face, index: FaceIndex) -> Result<MeshedFace, BodyQueryError> {
    let entity = QueryEntity::Face(index);
    let (triangulation, location) = BRepTool::triangulation(face, mesh_purpose::NONE)
        .map_err(|failure| native(entity, "face triangulation", &failure))?;
    let triangles = triangulation.get().map_or(0, |mesh| mesh.nb_triangles());
    if triangles <= 0 {
        let cause = format!("face triangles={triangles}: expected a nonempty triangulation");
        return Err(refused(entity, BodyQueryFailure::MissingFaceMesh, cause));
    }
    let face = face.clone();
    Ok(MeshedFace {
        face,
        triangulation,
        location,
    })
}

/// The owned triangles of a meshed face, wound and normal-oriented by the face
/// orientation in the body.
pub(crate) fn face_triangles(
    meshed: &MeshedFace,
    index: FaceIndex,
) -> Result<FaceTriangles, BodyQueryError> {
    let entity = QueryEntity::Face(index);
    BRepLibToolTriangulatedShape::compute_normals(&meshed.face, &meshed.triangulation)
        .map_err(|failure| native(entity, "face normals", &failure))?;
    let mesh = meshed.triangulation.get().expect("checked triangulation");
    let reversed = reversed_face(&meshed.face, entity)?;
    let transform = meshed.location.transformation();
    let (positions, normals) = oriented_nodes(mesh, transform, reversed, entity)?;
    Ok(FaceTriangles {
        face: index,
        positions,
        normals,
        triangles: wound_triangles(mesh, reversed, entity)?,
        sampled_deflection_estimate: mesh.deflection(),
    })
}

fn reversed_face(face: &Face, entity: QueryEntity) -> Result<bool, BodyQueryError> {
    match face.orientation() {
        Orientation::Forward => Ok(false),
        Orientation::Reversed => Ok(true),
        other => {
            let cause = format!("face orientation {other:?}: expected FORWARD or REVERSED");
            Err(refused(
                entity,
                BodyQueryFailure::UnsupportedOrientation,
                cause,
            ))
        }
    }
}

type NodeArrays = (Vec<[f64; 3]>, Vec<[f64; 3]>);

fn oriented_nodes(
    mesh: &Triangulation,
    transform: Transform3,
    reversed: bool,
    entity: QueryEntity,
) -> Result<NodeArrays, BodyQueryError> {
    let mut arrays: NodeArrays = (Vec::new(), Vec::new());
    for node in 1..=mesh.nb_nodes() {
        arrays.0.push(mesh_point(mesh, node, transform, entity)?);
        arrays
            .1
            .push(oriented_normal(mesh, node, transform, reversed, entity)?);
    }
    Ok(arrays)
}

/// The model position of `node` of `mesh` under `transform`, refused when not finite.
pub(crate) fn mesh_point(
    mesh: &Triangulation,
    node: i32,
    transform: Transform3,
    entity: QueryEntity,
) -> Result<[f64; 3], BodyQueryError> {
    let point = mesh
        .node(node)
        .map_err(|failure| native(entity, "mesh node", &failure))?
        .transformed(transform);
    finite([point.x(), point.y(), point.z()], node, entity)
}

fn oriented_normal(
    mesh: &Triangulation,
    node: i32,
    transform: Transform3,
    reversed: bool,
    entity: QueryEntity,
) -> Result<[f64; 3], BodyQueryError> {
    let normal = mesh
        .normal(node)
        .map_err(|failure| native(entity, "mesh normal", &failure))?
        .transformed(transform);
    let normal = if reversed { normal.reversed() } else { normal };
    finite([normal.x(), normal.y(), normal.z()], node, entity)
}

fn finite(value: [f64; 3], node: i32, entity: QueryEntity) -> Result<[f64; 3], BodyQueryError> {
    if value.iter().all(|coordinate| coordinate.is_finite()) {
        return Ok(value);
    }
    let cause = format!("mesh node {node} value {value:?}: expected finite coordinates");
    Err(refused(entity, BodyQueryFailure::NonFiniteGeometry, cause))
}

fn wound_triangles(
    mesh: &Triangulation,
    reversed: bool,
    entity: QueryEntity,
) -> Result<Vec<[u32; 3]>, BodyQueryError> {
    (1..=mesh.nb_triangles())
        .map(|index| {
            let triangle = mesh
                .triangle(index)
                .map_err(|failure| native(entity, "triangle", &failure))?;
            let (a, b, c) = triangle.get();
            let [a, b, c] = [a, b, c].map(|node| (node - 1) as u32);
            Ok(if reversed { [a, c, b] } else { [a, b, c] })
        })
        .collect()
}
