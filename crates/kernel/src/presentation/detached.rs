// SPDX-License-Identifier: GPL-2.0-only
//! The detached, freshly meshed copy a presentation is read from.
//!
//! BRepMesh stores triangulations and edge polygons inside the topology it meshes.
//! Meshing a copy keeps the caller's body and its history unchanged, and dropping
//! the source meshes (`copy_mesh = false`) makes the result independent of any
//! earlier meshing of the body.
use super::options::{DisplayOptions, EffectiveMeshSettings};
use crate::query_error::{BodyQuery, BodyQueryError, BodyQueryFailure, QueryEntity};
use crate::topology::{BodyTopology, EdgeIndex, FaceIndex, enumerate_for};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkmesh::brep_mesh::{BRepMeshDiscretRoot, BRepMeshIncrementalMesh};
use rustycad_tkmesh::imesh_tools::MeshParameters;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};

/// The meshed copy and the images of the original faces and edges, by stable index.
pub(crate) struct DetachedBody {
    /// The images of the original faces; position = stable face index.
    pub faces: Vec<Face>,
    /// The images of the original edges; position = stable edge index.
    pub edges: Vec<Edge>,
    /// The settings the mesher used.
    pub effective: EffectiveMeshSettings,
}

/// Copies `body`, maps its faces and edges through the copy history and meshes the
/// copy with `options`.
pub(crate) fn meshed_copy(
    body: &Shape,
    topology: &BodyTopology,
    options: &DisplayOptions,
) -> Result<DetachedBody, BodyQueryError> {
    let (mut copier, copy) = detached_copy(body)?;
    let copied = enumerate_for(&copy, BodyQuery::Tessellate)?;
    let faces = face_images(&mut copier, topology, &copied)?;
    let edges = edge_images(&mut copier, topology, &copied)?;
    let effective = mesh(&copy, options)?;
    Ok(DetachedBody {
        faces,
        edges,
        effective,
    })
}

fn detached_copy(body: &Shape) -> Result<(BRepBuilderAPICopy, Shape), BodyQueryError> {
    // copy_geom=true: the mesher must not reach geometry handles shared with `body`.
    let mut copier = BRepBuilderAPICopy::with_shape(body, true, false)
        .map_err(|failure| native(QueryEntity::Body, "copy body", &failure))?;
    let copy = copier
        .shape()
        .map_err(|failure| native(QueryEntity::Body, "copied body", &failure))?
        .clone();
    Ok((copier, copy))
}

// The history image of a sub-shape carries no context orientation, so each image
// is replaced by its occurrence in the copy, which keeps the body's orientation.
fn face_images(
    copier: &mut BRepBuilderAPICopy,
    topology: &BodyTopology,
    copied: &BodyTopology,
) -> Result<Vec<Face>, BodyQueryError> {
    (0..topology.face_count() as u32)
        .map(|index| {
            let entity = QueryEntity::Face(FaceIndex(index));
            let source = topology
                .face(FaceIndex(index))
                .expect("index below the count");
            let image = single_image(copier, &source, entity)?;
            let occurrence = copied.face_index(&image).and_then(|at| copied.face(at));
            occurrence.ok_or_else(|| missing_occurrence(entity))
        })
        .collect()
}

fn edge_images(
    copier: &mut BRepBuilderAPICopy,
    topology: &BodyTopology,
    copied: &BodyTopology,
) -> Result<Vec<Edge>, BodyQueryError> {
    (0..topology.edge_count() as u32)
        .map(|index| {
            let entity = QueryEntity::Edge(EdgeIndex(index));
            let source = topology
                .edge(EdgeIndex(index))
                .expect("index below the count");
            let image = single_image(copier, &source, entity)?;
            let occurrence = copied.edge_index(&image).and_then(|at| copied.edge(at));
            occurrence.ok_or_else(|| missing_occurrence(entity))
        })
        .collect()
}

fn missing_occurrence(entity: QueryEntity) -> BodyQueryError {
    let cause = "copy history image is not a sub-shape of the copy: expected one occurrence";
    refused(entity, BodyQueryFailure::IncompleteCopyHistory, cause)
}

/// The one image of `source` in the copy history; identity comes from the history,
/// never from the order of the copy.
fn single_image(
    copier: &mut BRepBuilderAPICopy,
    source: &Shape,
    entity: QueryEntity,
) -> Result<Shape, BodyQueryError> {
    let images = copier
        .modified(source)
        .map_err(|failure| native(entity, "copy history", &failure))?;
    if images.len() != 1 {
        let cause = format!("copy history has {} images: expected one", images.len());
        return Err(refused(
            entity,
            BodyQueryFailure::IncompleteCopyHistory,
            cause,
        ));
    }
    Ok(images.iter().next().expect("one image").clone())
}

fn mesh(copy: &Shape, options: &DisplayOptions) -> Result<EffectiveMeshSettings, BodyQueryError> {
    let parameters = MeshParameters {
        deflection: options.linear_deflection(),
        angle: options.angular_deflection(),
        relative: false,
        // Sequential meshing keeps the operation free of shared worker state.
        in_parallel: false,
        ..MeshParameters::default()
    };
    let range = ProgressRange::default();
    let mesher = BRepMeshIncrementalMesh::with_parameters(copy, &parameters, &range)
        .map_err(|failure| native(QueryEntity::Body, "mesh copy", &failure))?;
    require_complete(&mesher, options)?;
    Ok(effective_settings(&mesher.parameters()))
}

fn require_complete(
    mesher: &BRepMeshIncrementalMesh,
    options: &DisplayOptions,
) -> Result<(), BodyQueryError> {
    let (done, flags) = (mesher.is_done(), mesher.get_status_flags());
    if done && flags == 0 {
        return Ok(());
    }
    let cause = format!(
        "mesh {options:?} done={done}, status flags={flags}: expected a completed mesh \
         without defects"
    );
    Err(refused(
        QueryEntity::Body,
        BodyQueryFailure::IncompleteMesh,
        cause,
    ))
}

fn effective_settings(parameters: &MeshParameters) -> EffectiveMeshSettings {
    EffectiveMeshSettings {
        linear_deflection: parameters.deflection,
        angular_deflection: parameters.angle,
        interior_linear_deflection: parameters.deflection_interior,
        interior_angular_deflection: parameters.angle_interior,
        min_size: parameters.min_size,
    }
}

/// A native failure of the tessellation query.
pub(crate) fn native(
    entity: QueryEntity,
    context: &str,
    failure: &impl std::fmt::Display,
) -> BodyQueryError {
    BodyQueryError::native(BodyQuery::Tessellate, entity, context, failure)
}

/// A tessellation refusal of `class` about `entity`.
pub(crate) fn refused(
    entity: QueryEntity,
    class: BodyQueryFailure,
    cause: impl Into<String>,
) -> BodyQueryError {
    BodyQueryError::new(BodyQuery::Tessellate, entity, class, cause)
}
