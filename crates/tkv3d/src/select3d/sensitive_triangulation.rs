//! `Select3D_SensitiveTriangulation`: the triangles (interior) or the free edges (boundary) of
//! a `Poly_Triangulation` in a BVH tree, at an initial location.
use super::sensitive_triangle::points_box;
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, Select3DSensitiveSet, Select3DSensitiveSetBase,
    Select3DTypeOfSensitivity, select3d_set_build_bvh, select3d_set_dump_json,
    select3d_set_matches,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::{Array1, Vec3};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::bvh::{BvhBox, std_max, std_min};
use rustycad_tkmath::gp::{GeneralTransform3, Point3, Xyz};
use rustycad_tkmath::poly::{Connect, Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::io;

/// The node of a triangulation (the origin for an invalid index, which natively reads out of
/// the node array).
fn node(triangulation: &Triangulation, index: i32) -> Point3 {
    triangulation.node(index).unwrap_or_default()
}

/// The three nodes of a one-based triangle.
fn triangle_nodes(triangulation: &Triangulation, index: i32) -> [Point3; 3] {
    let (n1, n2, n3) = triangulation
        .triangle(index)
        .map(|t| t.get())
        .unwrap_or((0, 0, 0));
    [
        node(triangulation, n1),
        node(triangulation, n2),
        node(triangulation, n3),
    ]
}

/// `NbOfFreeEdges` and the free edges: the node pairs of the triangle sides without
/// neighbour (one-based array natively), and the sum of the triangle centers.
fn free_edges(triangulation: &Handle<Triangulation>) -> Result<(Vec<i32>, Xyz), Failure> {
    let Some(trg) = triangulation.get() else {
        return Ok((Vec::new(), Xyz::new(0.0, 0.0, 0.0)));
    };
    let connect = Connect::new(triangulation)?;
    let (mut edges, mut center) = (Vec::new(), Xyz::new(0.0, 0.0, 0.0));
    for index in 1..=trg.nb_triangles() {
        let (a0, a1, a2) = connect.triangles(index)?;
        let (n0, n1, n2) = trg.triangle(index)?.get();
        let [p0, p1, p2] = triangle_nodes(trg, index);
        center += (p0.xyz() + p1.xyz() + p2.xyz()) / 3.0;
        let (adjacent, nodes) = ([a0, a1, a2], [n0, n1, n2]);
        for vert in 0..3 {
            if adjacent[vert] == 0 {
                edges.extend([nodes[vert], nodes[(vert + 1) % 3]]);
            }
        }
    }
    Ok((edges, center))
}

/// The sum of the triangle centers.
fn triangle_centers(trg: &Triangulation) -> Xyz {
    let mut center = Xyz::new(0.0, 0.0, 0.0);
    for index in 1..=trg.nb_triangles() {
        let [p0, p1, p2] = triangle_nodes(trg, index);
        center += (p0.xyz() + p1.xyz() + p2.xyz()) / 3.0;
    }
    center
}

/// The fields of a sensitive triangulation.
struct TriangulationState {
    free_edges: Vec<i32>,
    primitives_nb: i32,
    bvh_prim_indexes: Option<Vec<i32>>,
    bnd_box: Select3DBndBox3d,
}

/// A sensitive triangulation.
///
/// Example: `let sens = Select3DSensitiveTriangulation::new(&owner, &triangulation, &Location::default(), true)?;`.
pub struct Select3DSensitiveTriangulation {
    transient: StandardTransient,
    entity: Select3DSensitiveEntityBase,
    set: Select3DSensitiveSetBase,
    triangul: Handle<Triangulation>,
    init_location: Location,
    cdg3d: Point3,
    sens_type: Select3DTypeOfSensitivity,
    inv_init_location: GeneralTransform3,
    state: SharedState<TriangulationState>,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveTriangulation, "Select3D_SensitiveTriangulation", dyn Select3DSensitiveSet, transient,
    interfaces: [dyn Select3DSensitiveEntity, dyn Select3DSensitiveSet]);

/// The BVH indexes `0..n-1` of `n` primitives.
fn prim_indexes(count: i32) -> Vec<i32> {
    (0..count.max(0)).collect()
}

impl Select3DSensitiveTriangulation {
    /// The triangulation of an owner at a location: the triangles (interior) or the free edges,
    /// the mean of the triangle centers (or the cached box center without geometry); a
    /// singular location raises `Standard_ConstructionError`.
    /// Example: `Select3DSensitiveTriangulation::new(&owner, &trg, &location, true)?`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        triangulation: &Handle<Triangulation>,
        init_loc: &Location,
        is_interior: bool,
    ) -> Result<Self, Failure> {
        let trg = triangulation.get();
        let has_geometry = trg.is_some_and(Triangulation::has_geometry);
        let (mut edges, mut center, mut nb_triangles) = (Vec::new(), Xyz::new(0.0, 0.0, 0.0), 0);
        if !has_geometry {
            if let Some(cached) = trg
                .filter(|t| t.has_cached_min_max())
                .map(Triangulation::cached_min_max)
            {
                center = (cached.corner_min()?.xyz() + cached.corner_max()?.xyz()) * 0.5;
            }
        } else if let Some(value) = trg {
            nb_triangles = value.nb_triangles();
            (edges, center) = if is_interior {
                (Vec::new(), triangle_centers(value))
            } else {
                free_edges(triangulation)?
            };
        }
        if nb_triangles != 0 {
            center /= f64::from(nb_triangles);
        }
        let primitives_nb = if !has_geometry {
            0
        } else if is_interior {
            nb_triangles
        } else {
            edges.len() as i32 / 2
        };
        let bvh = has_geometry.then(|| prim_indexes(primitives_nb));
        let entity = Self::assemble(
            owner,
            triangulation,
            init_loc,
            is_interior,
            Point3::from(center),
            TriangulationState {
                free_edges: edges,
                primitives_nb,
                bvh_prim_indexes: bvh,
                bnd_box: Select3DBndBox3d::default(),
            },
        )?;
        entity.compute_bounding_box();
        Ok(entity)
    }
    /// The triangulation with given free edges and center (`GetConnected`); the bounding box is
    /// left to compute. Example: `Select3DSensitiveTriangulation::with_free_edges(&owner, &trg, &loc, &edges, cog, false)?`.
    pub fn with_free_edges(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        triangulation: &Handle<Triangulation>,
        init_loc: &Location,
        free_edges: &[i32],
        cog: Point3,
        is_interior: bool,
    ) -> Result<Self, Failure> {
        let has_geometry = triangulation.get().is_some_and(Triangulation::has_geometry);
        let primitives_nb = match (has_geometry, is_interior) {
            (false, _) => 0,
            (true, true) => triangulation.get().map_or(0, Triangulation::nb_triangles),
            (true, false) => free_edges.len() as i32 / 2,
        };
        let bvh = has_geometry.then(|| prim_indexes(primitives_nb));
        Self::assemble(
            owner,
            triangulation,
            init_loc,
            is_interior,
            cog,
            TriangulationState {
                free_edges: free_edges.to_vec(),
                primitives_nb,
                bvh_prim_indexes: bvh,
                bnd_box: Select3DBndBox3d::default(),
            },
        )
    }
    /// The entity of its parts.
    fn assemble(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        triangulation: &Handle<Triangulation>,
        init_loc: &Location,
        is_interior: bool,
        cog: Point3,
        state: TriangulationState,
    ) -> Result<Self, Failure> {
        let inv_init_location = GeneralTransform3::from(init_loc.transformation().inverted()?);
        Ok(Self {
            transient: StandardTransient::default(),
            entity: Select3DSensitiveEntityBase::new(owner),
            set: Select3DSensitiveSetBase::default(),
            triangul: triangulation.clone(),
            init_location: init_loc.clone(),
            cdg3d: cog,
            sens_type: if is_interior {
                Select3DTypeOfSensitivity::Interior
            } else {
                Select3DTypeOfSensitivity::Boundary
            },
            inv_init_location,
            state: SharedState::new(state),
        })
    }
    /// `LastDetectedTriangleIndex`: the one-based detected triangle of an interior, -1 without.
    /// Example: `sens.last_detected_triangle_index()`.
    pub fn last_detected_triangle_index(&self) -> i32 {
        let detected = self.set.detected_idx();
        let state = self.state.read();
        match (
            &state.bvh_prim_indexes,
            detected != -1 && self.sens_type == Select3DTypeOfSensitivity::Interior,
        ) {
            (Some(indexes), true) => indexes.get(detected as usize).map_or(-1, |i| i + 1),
            _ => -1,
        }
    }
    /// `LastDetectedTriangle`: the detected triangle, None without; `sens.last_detected_triangle()`.
    pub fn last_detected_triangle(&self) -> Option<Triangle> {
        let index = self.last_detected_triangle_index();
        if index == -1 {
            return None;
        }
        self.triangul.get().and_then(|t| t.triangle(index).ok())
    }
    /// `LastDetectedTriangle(triangle, nodes)`: the detected triangle and its nodes at the
    /// initial location. Example: `sens.last_detected_triangle_nodes()`.
    pub fn last_detected_triangle_nodes(&self) -> Option<(Triangle, [Point3; 3])> {
        let triangle = self.last_detected_triangle()?;
        let trg = self.triangul.get()?;
        let trsf = self.init_location.transformation();
        let (n1, n2, n3) = triangle.get();
        Some((
            triangle,
            [n1, n2, n3].map(|n| node(trg, n).transformed(trsf)),
        ))
    }
    /// `Triangulation`; `sens.triangulation()`.
    pub fn triangulation(&self) -> Handle<Triangulation> {
        self.triangul.clone()
    }
    /// `GetInitLocation`; `sens.get_init_location()`.
    pub fn get_init_location(&self) -> Location {
        self.init_location.clone()
    }
    /// `computeBoundingBox` (protected): the cached box of the triangulation, or the box of its nodes.
    fn compute_bounding_box(&self) {
        let mut bounds = Select3DBndBox3d::default();
        if let Some(trg) = self.triangul.get() {
            if trg.has_cached_min_max() {
                // A cached box is never void (HasCachedMinMax), so its corners exist.
                let cached = trg.cached_min_max();
                if let (Ok(min), Ok(max)) = (cached.corner_min(), cached.corner_max()) {
                    bounds.add(&Vec3::new(min.x(), min.y(), min.z()));
                    bounds.add(&Vec3::new(max.x(), max.y(), max.z()));
                }
            } else if trg.has_geometry() {
                for index in 1..=trg.nb_nodes() {
                    let p = node(trg, index);
                    bounds.add(&Vec3::new(p.x(), p.y(), p.z()));
                }
            }
        }
        self.state.write().bnd_box = bounds;
    }
    /// `applyTransformation` (protected): the box of the eight corners at the initial location.
    fn apply_transformation(&self) -> Select3DBndBox3d {
        let local = self.state.read().bnd_box;
        if !self.has_init_location() {
            return local;
        }
        let (min, max, trsf) = (
            *local.corner_min(),
            *local.corner_max(),
            self.init_location.transformation(),
        );
        let mut bounds = Select3DBndBox3d::default();
        for (x, y, z) in (0..8).map(|i| (i / 4, (i / 2) % 2, i % 2)) {
            let corner = Point3::new(
                if x == 0 { min.x } else { max.x },
                if y == 0 { min.y } else { max.y },
                if z == 0 { min.z } else { max.z },
            );
            let p = corner.transformed(trsf);
            bounds.add(&Vec3::new(p.x(), p.y(), p.z()));
        }
        bounds
    }
    /// The primitive index of a BVH index.
    fn primitive(&self, idx: i32) -> i32 {
        self.state
            .read()
            .bvh_prim_indexes
            .as_ref()
            .and_then(|v| v.get(idx as usize).copied())
            .unwrap_or(0)
    }
    /// The free edge value at a one-based index.
    fn free_edge(&self, index: i32) -> i32 {
        usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.state.read().free_edges.get(i).copied())
            .unwrap_or(0)
    }
    /// The two nodes of a free edge primitive.
    fn edge_nodes(&self, prim: i32) -> (Point3, Point3) {
        let trg = self.triangul.get();
        let at = |i: i32| trg.map(|t| node(t, self.free_edge(i))).unwrap_or_default();
        (at(prim * 2 + 1), at(prim * 2 + 2))
    }
    /// The nodes of a triangle primitive.
    fn prim_triangle(&self, prim: i32) -> [Point3; 3] {
        self.triangul
            .get()
            .map_or([Point3::default(); 3], |t| triangle_nodes(t, prim + 1))
    }
    /// `Matches` without geometry: the (cached) box tested as a box.
    fn matches_box(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let bounds = self.bounding_box();
        if !bounds.is_valid() {
            return Ok(false);
        }
        if !manager.is_overlap_allowed() {
            let mut is_inside = true;
            return Ok(manager.overlaps_box_inside(
                bounds.corner_min(),
                bounds.corner_max(),
                Some(&mut is_inside),
            )? && is_inside);
        }
        if !manager.overlaps_box(bounds.corner_min(), bounds.corner_max(), pick_result)? {
            return Ok(false);
        }
        pick_result.set_dist_to_geom_center(manager.dist_to_geometry_center(&self.cdg3d));
        Ok(true)
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveTriangulation {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.entity
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if self.triangul.get().is_some_and(Triangulation::has_geometry) {
            return select3d_set_matches(self, manager, pick_result, false);
        }
        self.matches_box(manager, pick_result)
    }
    fn nb_sub_elements(&self) -> i32 {
        self.triangul.get().map_or(0, Triangulation::nb_nodes)
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        if !self.state.read().bnd_box.is_valid() {
            self.compute_bounding_box();
        }
        self.apply_transformation()
    }
    fn center_of_geometry(&self) -> Point3 {
        self.cdg3d
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let edges = self.state.read().free_edges.clone();
        let is_interior = self.sens_type == Select3DTypeOfSensitivity::Interior;
        Ok(Handle::new(Self::with_free_edges(
            &self.owner_id(),
            &self.triangul,
            &self.init_location,
            &edges,
            self.cdg3d,
            is_interior,
        )?)
        .cast())
    }
    fn bvh(&self) -> Result<(), Failure> {
        select3d_set_build_bvh(self)
    }
    fn to_build_bvh(&self) -> bool {
        self.set.is_dirty()
    }
    fn clear(&self) {}
    fn has_init_location(&self) -> bool {
        !self.init_location.is_identity()
    }
    fn inv_init_location(&self) -> GeneralTransform3 {
        self.inv_init_location
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitiveTriangulation")?;
        base_class(stream, depth, &|s, d| select3d_set_dump_json(self, s, d))?;
        if let Some(trg) = self.triangul.get() {
            field_dumped(stream, depth, "myTriangul.get()", &|s, _| trg.dump_json(s))?;
        }
        let location = self.init_location.clone();
        field_dumped(stream, depth, "&myInitLocation", &|s, d| {
            location.dump_json(s, d)
        })?;
        field_integer(stream, "mySensType", self.sens_type as i32)?;
        let (count, bounds) = {
            let state = self.state.read();
            (state.primitives_nb, state.bnd_box)
        };
        field_integer(stream, "myPrimitivesNb", count)?;
        field_dumped(stream, depth, "&myBndBox", &|s, _| bounds.dump_json(s))
    }
}

impl Select3DSensitiveSet for Select3DSensitiveTriangulation {
    fn set_base(&self) -> &Select3DSensitiveSetBase {
        &self.set
    }
    fn size(&self) -> i32 {
        self.state.read().primitives_nb
    }
    /// `Box`: the triangle box, or for a free edge the box of the nodes at `Lower + prim` and
    /// `Lower + prim + 1` (natively not `2 * prim`: the boundary boxes are those of other edges).
    fn element_box(&self, idx: i32) -> Select3DBndBox3d {
        let prim = self.primitive(idx);
        if self.sens_type == Select3DTypeOfSensitivity::Interior {
            return points_box(&self.prim_triangle(prim));
        }
        let trg = self.triangul.get();
        let at = |i: i32| trg.map(|t| node(t, self.free_edge(i))).unwrap_or_default();
        let (a, b) = (at(1 + prim), at(1 + prim + 1));
        let min = Vec3::new(
            std_min(a.x(), b.x()),
            std_min(a.y(), b.y()),
            std_min(a.z(), b.z()),
        );
        let max = Vec3::new(
            std_max(a.x(), b.x()),
            std_max(a.y(), b.y()),
            std_max(a.z(), b.z()),
        );
        BvhBox::from_corners(min, max)
    }
    fn center(&self, idx: i32, axis: i32) -> Result<f64, Failure> {
        let bounds = self.element_box(idx);
        let center = (*bounds.corner_min() + *bounds.corner_max()) * 0.5;
        Ok(match axis {
            0 => center.x,
            1 => center.y,
            _ => center.z,
        })
    }
    fn swap(&self, idx1: i32, idx2: i32) {
        if let Some(indexes) = self.state.write().bvh_prim_indexes.as_mut() {
            indexes.swap(idx1 as usize, idx2 as usize);
        }
    }
    /// `overlapsElement`: the free edge as a boundary polygon, or the interior triangle.
    fn overlaps_element(
        &self,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if is_full_inside {
            return Ok(true);
        }
        let prim = self.primitive(elem_idx);
        if self.sens_type == Select3DTypeOfSensitivity::Boundary {
            let (a, b) = self.edge_nodes(prim);
            let points = [a, b];
            return manager.overlaps_polygon(
                &Array1::from_shared(&points, 1),
                Select3DTypeOfSensitivity::Boundary as i32,
                pick_result,
            );
        }
        let [a, b, c] = self.prim_triangle(prim);
        manager.overlaps_triangle(
            &a,
            &b,
            &c,
            Select3DTypeOfSensitivity::Interior as i32,
            pick_result,
        )
    }
    /// `elementIsInside`: the segment or triangle (polyline) or all its points.
    fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if is_full_inside {
            return Ok(true);
        }
        let prim = self.primitive(elem_idx);
        let polyline = manager.get_active_selection_type() == SelectMgrSelectionType::Polyline;
        let mut dummy = SelectBasicsPickResult::default();
        if self.sens_type == Select3DTypeOfSensitivity::Boundary {
            let (a, b) = self.edge_nodes(prim);
            return Ok(if polyline {
                manager.overlaps_segment(&a, &b, &mut dummy)?
            } else {
                manager.overlaps_point_only(&a)? && manager.overlaps_point_only(&b)?
            });
        }
        let [a, b, c] = self.prim_triangle(prim);
        if polyline {
            return manager.overlaps_triangle(&a, &b, &c, self.sens_type as i32, &mut dummy);
        }
        Ok(manager.overlaps_point_only(&a)?
            && manager.overlaps_point_only(&b)?
            && manager.overlaps_point_only(&c)?)
    }
    fn distance_to_cog(&self, manager: &dyn SelectBasicsSelectingVolumeManager) -> f64 {
        manager.dist_to_geometry_center(&self.cdg3d)
    }
}
