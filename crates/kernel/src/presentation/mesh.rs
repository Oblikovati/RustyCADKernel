// SPDX-License-Identifier: GPL-2.0-only
//! The owned presentation mesh of a body: plain data, `Send + Sync`, detached from
//! the body it was derived from.
use super::options::{DisplayOptions, EffectiveMeshSettings};
use crate::topology::{EdgeIndex, FaceIndex};

/// The triangles of one face, with face-local vertex arrays.
#[derive(Clone, Debug, PartialEq)]
pub struct FaceTriangles {
    /// The stable index of the face in the body.
    pub face: FaceIndex,
    /// Vertex positions in model coordinates.
    pub positions: Vec<[f64; 3]>,
    /// Unit normals per vertex, pointing out of the material side of the face.
    pub normals: Vec<[f64; 3]>,
    /// Zero-based vertex indices, counter-clockwise seen from the normal side.
    pub triangles: Vec<[u32; 3]>,
    /// The mesher's own deviation estimate for this face (a sampled estimate,
    /// not a certified bound).
    pub sampled_deflection_estimate: f64,
}

/// The display polyline of one non-degenerated edge.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgePolyline {
    /// The stable index of the edge in the body.
    pub edge: EdgeIndex,
    /// Points along the edge in its parameter order; they coincide with vertices of
    /// the adjacent face meshes.
    pub points: Vec<[f64; 3]>,
}

/// A presentation mesh: per-face triangle groups, edge polylines and vertices, all
/// keyed by stable topology indices, plus the requested and effective settings.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationMesh {
    /// The requested display options.
    pub requested: DisplayOptions,
    /// The settings the mesher used.
    pub effective: EffectiveMeshSettings,
    /// One entry per face, in stable face order.
    pub faces: Vec<FaceTriangles>,
    /// One entry per non-degenerated edge, in stable edge order.
    pub edges: Vec<EdgePolyline>,
    /// Degenerated edges (poles, apexes): they have no length and no polyline.
    pub degenerated_edges: Vec<EdgeIndex>,
    /// Vertex positions, in stable vertex order.
    pub vertices: Vec<[f64; 3]>,
}

// Published presentation results are read from any thread (AGENTS.md ownership rules).
const _: fn() = || {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<PresentationMesh>();
    require_send_sync::<DisplayOptions>();
};

impl PresentationMesh {
    /// A canonical little-endian encoding of the whole mesh; equal meshes encode to
    /// equal bytes, so runs and threads can be compared byte for byte;
    /// `let bytes = mesh.canonical_bytes();`.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = b"MartensitePresentationMesh/v1".to_vec();
        push_settings(&mut out, &self.requested, &self.effective);
        push_len(&mut out, self.faces.len());
        self.faces.iter().for_each(|face| push_face(&mut out, face));
        push_len(&mut out, self.edges.len());
        for edge in &self.edges {
            out.extend_from_slice(&edge.edge.0.to_le_bytes());
            push_points(&mut out, &edge.points);
        }
        push_len(&mut out, self.degenerated_edges.len());
        for edge in &self.degenerated_edges {
            out.extend_from_slice(&edge.0.to_le_bytes());
        }
        push_points(&mut out, &self.vertices);
        out
    }

    /// The total triangle count of all faces; `mesh.triangle_count()`.
    pub fn triangle_count(&self) -> usize {
        self.faces.iter().map(|face| face.triangles.len()).sum()
    }
}

fn push_settings(out: &mut Vec<u8>, requested: &DisplayOptions, effective: &EffectiveMeshSettings) {
    let values = [
        requested.linear_deflection(),
        requested.angular_deflection(),
        effective.linear_deflection,
        effective.angular_deflection,
        effective.interior_linear_deflection,
        effective.interior_angular_deflection,
        effective.min_size,
    ];
    values.iter().for_each(|value| push_f64(out, *value));
}

fn push_face(out: &mut Vec<u8>, face: &FaceTriangles) {
    out.extend_from_slice(&face.face.0.to_le_bytes());
    push_points(out, &face.positions);
    push_points(out, &face.normals);
    push_len(out, face.triangles.len());
    for index in face.triangles.iter().flatten() {
        out.extend_from_slice(&index.to_le_bytes());
    }
    push_f64(out, face.sampled_deflection_estimate);
}

fn push_points(out: &mut Vec<u8>, points: &[[f64; 3]]) {
    push_len(out, points.len());
    points
        .iter()
        .flatten()
        .for_each(|value| push_f64(out, *value));
}

fn push_len(out: &mut Vec<u8>, len: usize) {
    out.extend_from_slice(&(len as u64).to_le_bytes());
}

fn push_f64(out: &mut Vec<u8>, value: f64) {
    out.extend_from_slice(&value.to_bits().to_le_bytes());
}
