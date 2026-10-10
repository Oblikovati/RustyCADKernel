//! `SelectMgr_Frustum<N>`: the separating axis tests of a frustum of `N` side planes (4 for the
//! rectangular frustum, 3 for the triangular one) against boxes, points, segments, polygons
//! and triangles. The quadric tests are in the sphere, cylinder and circle modules.
use rustycad_tkmath::gp::{Point3, Xyz};

/// The planes, vertices, cached projections and edge directions of a frustum.
#[derive(Clone, Debug)]
pub struct SelectMgrFrustumCore<const N: usize> {
    /// `myPlanes` (`N + 2` unnormalized normals).
    pub(crate) planes: Vec<Xyz>,
    /// `myVertices` (`2 N` points).
    pub(crate) vertices: Vec<Point3>,
    /// `myMaxVertsProjections`.
    pub(crate) max_verts_projections: Vec<f64>,
    /// `myMinVertsProjections`.
    pub(crate) min_verts_projections: Vec<f64>,
    /// `myMaxOrthoVertsProjections`.
    pub(crate) max_ortho_verts_projections: [f64; 3],
    /// `myMinOrthoVertsProjections`.
    pub(crate) min_ortho_verts_projections: [f64; 3],
    /// `myEdgeDirs`.
    pub(crate) edge_dirs: [Xyz; 6],
}

impl<const N: usize> SelectMgrFrustumCore<N> {
    /// A zeroed frustum of `N` side planes (the native constructor clears the projections).
    /// Example: `SelectMgrFrustumCore::<4>::new()`.
    pub fn new() -> Self {
        Self {
            planes: vec![Xyz::new(0.0, 0.0, 0.0); N + 2],
            vertices: vec![Point3::default(); 2 * N],
            max_verts_projections: vec![0.0; N + 2],
            min_verts_projections: vec![0.0; N + 2],
            max_ortho_verts_projections: [0.0; 3],
            min_ortho_verts_projections: [0.0; 3],
            edge_dirs: [Xyz::new(0.0, 0.0, 0.0); 6],
        }
    }
    /// The plane step: the opposite planes of an orthographic rectangular frustum are skipped.
    pub(crate) fn inc_factor(&self, is_ortho: bool) -> usize {
        if is_ortho && N == 4 { 2 } else { 1 }
    }
    /// The number of edge directions tested (4 for an orthographic camera).
    pub(crate) fn directions_nb(is_ortho: bool) -> usize {
        if is_ortho { 4 } else { 6 }
    }
}
