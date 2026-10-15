//! Cached and transformed triangulation bounds, including no-geometry selection fallback.
use super::sensitive_triangulation::Select3DSensitiveTriangulation;
use super::triangulation_failure::native_checked;
use super::triangulation_geometry::node;
use super::{Select3DBndBox3d, Select3DSensitiveEntity};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
impl Select3DSensitiveTriangulation {
    /// `computeBoundingBox` (protected): the cached box of the triangulation, or the box of its nodes.
    /// Example: `sens.compute_bounding_box()`.
    pub fn compute_bounding_box(&self) {
        let bounds = self
            .triangul
            .get()
            .map_or_else(|| Ok(Select3DBndBox3d::default()), mesh_bounds);
        self.state.write().bnd_box = native_checked(bounds);
    }

    /// `applyTransformation` (protected): the box of the eight corners at the initial location.
    /// Example: `sens.apply_transformation()`.
    pub fn apply_transformation(&self) -> Select3DBndBox3d {
        let local = self.state.read().bnd_box;
        if !self.has_init_location() {
            return local;
        }
        let mut bounds = Select3DBndBox3d::default();
        for index in 0..8 {
            let corner = transformed_box_corner(&local, index, self.init_location.transformation());
            bounds.add(&Vec3::new(corner.x(), corner.y(), corner.z()));
        }
        bounds
    }

    /// `Matches` without geometry: the (cached) box tested as a box.
    pub(super) fn matches_box(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let bounds = self.bounding_box();
        if !bounds.is_valid() {
            return Ok(false);
        }
        if !manager.is_overlap_allowed() {
            return contained_mesh_box(manager, &bounds);
        }

        if !manager.overlaps_box(bounds.corner_min(), bounds.corner_max(), pick_result)? {
            return Ok(false);
        }
        pick_result.set_dist_to_geom_center(manager.dist_to_geometry_center(&self.cdg3d)?);
        Ok(true)
    }
}

fn mesh_bounds(mesh: &rustycad_tkmath::poly::Triangulation) -> Result<Select3DBndBox3d, Failure> {
    let mut bounds = Select3DBndBox3d::default();
    if mesh.has_cached_min_max() {
        return Ok(cached_mesh_bounds(mesh));
    }
    if !mesh.has_geometry() {
        return Ok(bounds);
    }
    for index in 1..=mesh.nb_nodes() {
        let point = node(mesh, index)?;
        bounds.add(&Vec3::new(point.x(), point.y(), point.z()));
    }
    Ok(bounds)
}
fn cached_mesh_bounds(mesh: &rustycad_tkmath::poly::Triangulation) -> Select3DBndBox3d {
    // A cached box is never void (HasCachedMinMax), so its corners exist.
    let cached = mesh.cached_min_max();
    let mut bounds = Select3DBndBox3d::default();
    if let (Ok(min), Ok(max)) = (cached.corner_min(), cached.corner_max()) {
        bounds.add(&Vec3::new(min.x(), min.y(), min.z()));
        bounds.add(&Vec3::new(max.x(), max.y(), max.z()));
    }
    bounds
}
fn transformed_box_corner(
    bounds: &Select3DBndBox3d,
    index: i32,
    transform: rustycad_tkmath::gp::Transform3,
) -> Point3 {
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    Point3::new(
        if index / 4 == 0 { min.x } else { max.x },
        if index / 2 % 2 == 0 { min.y } else { max.y },
        if index % 2 == 0 { min.z } else { max.z },
    )
    .transformed(transform)
}
fn contained_mesh_box(
    manager: &dyn SelectBasicsSelectingVolumeManager,
    bounds: &Select3DBndBox3d,
) -> Result<bool, Failure> {
    let mut inside = true;
    let hit =
        manager.overlaps_box_inside(bounds.corner_min(), bounds.corner_max(), Some(&mut inside))?;
    Ok(hit && inside)
}
