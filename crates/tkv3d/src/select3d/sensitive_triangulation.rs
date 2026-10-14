//! `Select3D_SensitiveTriangulation`: the triangles (interior) or the free edges (boundary) of
//! a `Poly_Triangulation` in a BVH tree, at an initial location.
use super::triangulation_failure::native_checked;
use super::triangulation_geometry::triangle_nodes;
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase, Select3DSensitiveSet,
    Select3DSensitiveSetBase, Select3DTypeOfSensitivity,
};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{GeneralTransform3, Point3};
use rustycad_tkmath::poly::{Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;
/// The fields of a sensitive triangulation.
pub(super) struct TriangulationState {
    pub(super) free_edges: Handle<HArray1<i32>>,
    pub(super) primitives_nb: i32,
    pub(super) bvh_prim_indexes: Option<Vec<i32>>,
    pub(super) bnd_box: Select3DBndBox3d,
}

/// A sensitive triangulation.
///
/// Example: `let sens = Select3DSensitiveTriangulation::new(&owner, &triangulation, &Location::default(), true)?;`.
pub struct Select3DSensitiveTriangulation {
    pub(super) transient: StandardTransient,
    pub(super) entity: Select3DSensitiveEntityBase,
    pub(super) set: Select3DSensitiveSetBase,
    pub(super) triangul: Handle<Triangulation>,
    pub(super) init_location: Location,
    pub(super) cdg3d: Point3,
    pub(super) sens_type: Select3DTypeOfSensitivity,
    pub(super) inv_init_location: GeneralTransform3,
    pub(super) state: SharedState<TriangulationState>,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveTriangulation, "Select3D_SensitiveTriangulation", dyn Select3DSensitiveSet, transient,
    interfaces: [dyn Select3DSensitiveEntity, dyn Select3DSensitiveSet]);

impl Select3DSensitiveTriangulation {
    /// `LastDetectedTriangleIndex`: the one-based detected triangle of an interior, -1 without.
    /// Example: `sens.last_detected_triangle_index()`.
    pub fn last_detected_triangle_index(&self) -> i32 {
        // A nonfallible convenience still exposes the checked native exception payload.
        native_checked(self.checked_detection_index())
    }
    pub(super) fn checked_detection_index(&self) -> Result<i32, Failure> {
        let detected = self.set.detected_idx();
        if detected == -1 || self.sens_type != Select3DTypeOfSensitivity::Interior {
            return Ok(-1);
        }
        if self.state.read().bvh_prim_indexes.is_none() {
            return Ok(-1);
        }
        Ok(self.primitive(detected)? + 1)
    }
    /// `LastDetectedTriangle`: the detected triangle, None without; `sens.last_detected_triangle()`.
    pub fn last_detected_triangle(&self) -> Option<Triangle> {
        let mut triangle = Triangle::new(0, 0, 0);
        native_checked(self.last_detected_triangle_into(&mut triangle)).then_some(triangle)
    }
    /// `LastDetectedTriangle(triangle, nodes)`: the detected triangle and its nodes at the
    /// initial location. Example: `sens.last_detected_triangle_nodes()`.
    pub fn last_detected_triangle_nodes(&self) -> Option<(Triangle, [Point3; 3])> {
        let mut triangle = Triangle::new(0, 0, 0);
        let mut nodes = [Point3::default(); 3];
        native_checked(self.last_detected_triangle_nodes_into(&mut triangle, &mut nodes))
            .then_some((triangle, nodes))
    }
    /// `Triangulation`; `sens.triangulation()`.
    pub fn triangulation(&self) -> Handle<Triangulation> {
        // Cloning the handle preserves shared mesh edits without duplicating geometry.
        self.triangul.clone()
    }
    /// `GetInitLocation`; `sens.get_init_location()`.
    pub fn get_init_location(&self) -> Location {
        // Value copying preserves immutable native datum-chain ownership.
        self.init_location.clone()
    }
    /// The primitive index of a BVH index.
    pub(super) fn primitive(&self, idx: i32) -> Result<i32, Failure> {
        let state = self.state.read();
        let indexes = state.bvh_prim_indexes.as_deref().unwrap_or(&[]);
        indexes.get(idx as usize).copied().ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "BVH primitive index {idx}: expected 0..{}",
                    indexes.len() as i32 - 1
                ),
            )
        })
    }
    /// The free edge value at a one-based index.
    fn free_edge(&self, index: i32) -> Result<i32, Failure> {
        let edges = self.state.read().free_edges.clone();
        let Some(edges) = edges.get() else {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "null edge handle: expected free-edge storage",
            ));
        };
        edges
            .array1()
            .value(index)
            .copied()
            .map_err(|error| error.into_failure("NCollection_Array1::Value"))
    }
    /// The two nodes of a free edge primitive.
    pub(super) fn edge_nodes(&self, prim: i32) -> Result<(Point3, Point3), Failure> {
        // OCCT 3d097a0 deliberately uses absolute one-based indices for matching.
        self.nodes_for_edge_indices(prim * 2 + 1, prim * 2 + 2)
    }
    pub(super) fn boundary_box_nodes(&self, prim: i32) -> Result<(Point3, Point3), Failure> {
        let edges = self.state.read().free_edges.clone();
        let lower = edges.get().map_or(1, |edges| edges.array1().lower());
        // Box uses Lower + prim, unlike matching; preserve this native indexing quirk.
        self.nodes_for_edge_indices(lower + prim, lower + prim + 1)
    }
    fn nodes_for_edge_indices(&self, first: i32, second: i32) -> Result<(Point3, Point3), Failure> {
        let Some(mesh) = self.triangul.get() else {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "null mesh: expected triangulation geometry",
            ));
        };
        Ok((
            mesh.node(self.free_edge(first)?)?,
            mesh.node(self.free_edge(second)?)?,
        ))
    }
    /// The nodes of a triangle primitive.
    pub(super) fn prim_triangle(&self, prim: i32) -> Result<[Point3; 3], Failure> {
        self.triangul.get().map_or_else(
            || {
                Err(Failure::with_message(
                    "Standard_NullObject",
                    "null mesh: expected triangulation geometry",
                ))
            },
            |t| triangle_nodes(t, prim + 1),
        )
    }
}

/// The protected fields of `Select3D_SensitiveTriangulation`, read only.
impl Select3DSensitiveTriangulation {
    /// `myFreeEdges`: the free edge node pairs of a boundary, null for an interior.
    /// Example: `sens.free_edges()`.
    pub fn free_edges(&self) -> Handle<HArray1<i32>> {
        self.state.read().free_edges.clone()
    }
    /// `myPrimitivesNb`: the triangles (interior) or free edges (boundary); `sens.primitives_nb()`.
    pub fn primitives_nb(&self) -> i32 {
        self.state.read().primitives_nb
    }
    /// `myBVHPrimIndexes`: the primitive of every BVH index; `sens.bvh_prim_indexes()`.
    pub fn bvh_prim_indexes(&self) -> Option<Vec<i32>> {
        self.state.read().bvh_prim_indexes.clone()
    }
    /// `myBndBox`: the cached box at the initial location; `sens.bnd_box()`.
    pub fn bnd_box(&self) -> Select3DBndBox3d {
        self.state.read().bnd_box
    }
    /// `myCDG3D`: the center of the triangles; `sens.cdg_3d()`.
    pub fn cdg_3d(&self) -> Point3 {
        self.cdg3d
    }
    /// `mySensType`: interior or boundary; `sens.sens_type()`.
    pub fn sens_type(&self) -> Select3DTypeOfSensitivity {
        self.sens_type
    }
}
