// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.hxx
// and IntPolyh_MaillageAffinage.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPolyh_MaillageAffinage`: the triangulations of two surfaces, refined where their
//! boxes interfere, compared triangle by triangle, and chained into section lines.
//!
//! The body is split by responsibility: the fields and accessors (this file), the points
//! (`fill.rs`), the common box (`common_box.rs`), the triangles and edges of the grid
//! (`grid.rs`), the refinement (`refine.rs`, `interfering.rs`), the contact of two
//! triangles (`contact.rs`), the intersection points of a triangle and an edge
//! (`edge_contact.rs`, `coplanar.rs`), the start points of two triangles (`research.rs`)
//! and their chaining (`chain.rs`, `chain_next.rs`).
mod chain;
mod chain_next;
mod common_box;
mod contact;
mod coplanar;
mod edge_contact;
mod fill;
mod grid;
mod interfering;
mod refine;
mod research;

use super::array::{IntPolyhArrayOfEdges, IntPolyhArrayOfPoints, IntPolyhArrayOfTriangles};
use super::edge::IntPolyhCouple;
use super::triangle_refine::Mesh;
pub use edge_contact::IntPolyhTriangleEdge;
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::bnd::BoundingBox3;

/// The coincidence precision of the algorithm (`MyConfusionPrecision`, 10.0e-12).
pub(crate) const MY_CONFUSION_PRECISION: f64 = 10.0e-12;
/// Its square (`SquareMyConfusionPrecision`, 10.0e-24).
pub(crate) const SQUARE_MY_CONFUSION_PRECISION: f64 = 10.0e-24;
/// The enlargement of the boxes and the degeneracy tolerance (`MyTolerance`, 10.0e-7).
pub(crate) const MY_TOLERANCE: f64 = 10.0e-7;

/// The data of one surface: `MaSurface`, `MyBox`, `NbSamplesU/V`, `FlecheMax/Min`,
/// `TPoints`, `TEdges` and `TTriangles`.
#[derive(Clone)]
pub(crate) struct Side {
    pub(crate) surface: Adaptor3dSurfaceHandle,
    pub(crate) bbox: BoundingBox3,
    pub(crate) nb_samples_u: i32,
    pub(crate) nb_samples_v: i32,
    pub(crate) fleche_max: f64,
    pub(crate) fleche_min: f64,
    pub(crate) points: IntPolyhArrayOfPoints,
    pub(crate) edges: IntPolyhArrayOfEdges,
    pub(crate) triangles: IntPolyhArrayOfTriangles,
}

impl Side {
    fn new(surface: &Adaptor3dSurfaceHandle, nb_su: i32, nb_sv: i32) -> Self {
        Self {
            surface: surface.clone(),
            bbox: BoundingBox3::default(),
            nb_samples_u: nb_su,
            nb_samples_v: nb_sv,
            fleche_max: 0.0,
            fleche_min: 0.0,
            points: IntPolyhArrayOfPoints::default(),
            edges: IntPolyhArrayOfEdges::default(),
            triangles: IntPolyhArrayOfTriangles::default(),
        }
    }

    /// The three arrays, for a refinement.
    pub(crate) fn mesh(&mut self) -> Mesh<'_> {
        Mesh {
            points: &mut self.points,
            triangles: &mut self.triangles,
            edges: &mut self.edges,
        }
    }
}

/// `IntPolyh_MaillageAffinage`.
///
/// ```ignore
/// let mut m = IntPolyhMaillageAffinage::with_samples(&s1, 10, 10, &s2, 10, 10, 0);
/// m.fill_array_of_pnt(1)?;
/// ```
#[derive(Clone)]
pub struct IntPolyhMaillageAffinage {
    pub(crate) sides: [Side; 2],
    pub(crate) t_triangles_contacts: List<IntPolyhCouple>,
    pub(crate) my_enlarge_zone: bool,
}

impl IntPolyhMaillageAffinage {
    /// `IntPolyh_MaillageAffinage(S1, NbSU1, NbSV1, S2, NbSU2, NbSV2, PRINT)`: `PRINT` is
    /// not read. `IntPolyhMaillageAffinage::with_samples(&s1, 10, 10, &s2, 10, 10, 0)`.
    pub fn with_samples(
        s1: &Adaptor3dSurfaceHandle,
        (nb_su1, nb_sv1): (i32, i32),
        s2: &Adaptor3dSurfaceHandle,
        (nb_su2, nb_sv2): (i32, i32),
        _print: i32,
    ) -> Self {
        Self {
            sides: [Side::new(s1, nb_su1, nb_sv1), Side::new(s2, nb_su2, nb_sv2)],
            t_triangles_contacts: List::default(),
            my_enlarge_zone: false,
        }
    }

    /// `IntPolyh_MaillageAffinage(S1, S2, PRINT)`: 10 x 10 samples on each surface.
    /// `IntPolyhMaillageAffinage::new(&s1, &s2, 0)`.
    pub fn new(s1: &Adaptor3dSurfaceHandle, s2: &Adaptor3dSurfaceHandle, print: i32) -> Self {
        Self::with_samples(s1, (10, 10), s2, (10, 10), print)
    }

    /// The side of `SurfID`: the first for 1, the second otherwise (the native
    /// `(SurfID == 1) ? ... : ...`).
    pub(crate) const fn rank(surf_id: i32) -> usize {
        if surf_id == 1 { 0 } else { 1 }
    }

    /// `GetArrayOfPoints(SurfID)`. `m.get_array_of_points(1)`.
    pub fn get_array_of_points(&self, surf_id: i32) -> &IntPolyhArrayOfPoints {
        &self.sides[Self::rank(surf_id)].points
    }

    /// `GetArrayOfEdges(SurfID)`. `m.get_array_of_edges(1)`.
    pub fn get_array_of_edges(&self, surf_id: i32) -> &IntPolyhArrayOfEdges {
        &self.sides[Self::rank(surf_id)].edges
    }

    /// `GetArrayOfTriangles(SurfID)`. `m.get_array_of_triangles(1)`.
    pub fn get_array_of_triangles(&self, surf_id: i32) -> &IntPolyhArrayOfTriangles {
        &self.sides[Self::rank(surf_id)].triangles
    }

    /// `GetBox(SurfID)`: a copy of the box of the points. `m.get_box(1)`.
    pub fn get_box(&self, surf_id: i32) -> BoundingBox3 {
        self.sides[Self::rank(surf_id)].bbox.clone()
    }

    /// `GetCouples()`: the couples of triangles in contact. `m.get_couples().extent()`.
    pub fn get_couples(&mut self) -> &mut List<IntPolyhCouple> {
        &mut self.t_triangles_contacts
    }

    /// `SetEnlargeZone(EnlargeZone)`. `m.set_enlarge_zone(true)`.
    pub fn set_enlarge_zone(&mut self, enlarge_zone: bool) {
        self.my_enlarge_zone = enlarge_zone;
    }

    /// `GetEnlargeZone()`. `m.get_enlarge_zone()`.
    pub fn get_enlarge_zone(&self) -> bool {
        self.my_enlarge_zone
    }

    /// `GetMinDeflection(SurfID)`: `FlecheMin`. `m.get_min_deflection(1)`.
    pub fn get_min_deflection(&self, surf_id: i32) -> f64 {
        self.sides[Self::rank(surf_id)].fleche_min
    }

    /// `GetMaxDeflection(SurfID)`: `FlecheMax`. `m.get_max_deflection(1)`.
    pub fn get_max_deflection(&self, surf_id: i32) -> f64 {
        self.sides[Self::rank(surf_id)].fleche_max
    }
}
