//! `SelectBasics_PickResult`: the depth, picked point, surface normal and distance to the
//! geometry center of one detection.
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The all-in-one result of `Select3D_SensitiveEntity::Matches`.
///
/// Example: `let mut pick = SelectBasicsPickResult::default(); pick.set_depth(2.0); assert!(pick.is_valid());`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectBasicsPickResult {
    obj_picked_pnt: Point3,
    normal: Vec3<f32>,
    depth: f64,
    dist_to_center: f64,
}

impl Default for SelectBasicsPickResult {
    /// The invalid result: depth, distance and picked X at `RealLast`.
    fn default() -> Self {
        Self {
            obj_picked_pnt: Point3::new(real_last(), 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 0.0),
            depth: real_last(),
            dist_to_center: real_last(),
        }
    }
}

impl SelectBasicsPickResult {
    /// A defined result; `SelectBasicsPickResult::new(1.0, 0.5, Point3::new(0.0, 0.0, 1.0))`.
    pub fn new(depth: f64, dist_to_center: f64, obj_picked_pnt: Point3) -> Self {
        Self {
            obj_picked_pnt,
            normal: Vec3::new(0.0, 0.0, 0.0),
            depth,
            dist_to_center,
        }
    }
    /// `Min`: the closest of two results by depth, the first one on a tie.
    /// Example: `SelectBasicsPickResult::min(&a, &b).depth()`.
    pub fn min<'a>(first: &'a Self, second: &'a Self) -> &'a Self {
        if first.depth() <= second.depth() {
            first
        } else {
            second
        }
    }
    /// Whether the depth is defined; `pick.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.depth != real_last()
    }
    /// Resets the depth, the picked point and the normal; `pick.invalidate()`.
    pub fn invalidate(&mut self) {
        self.depth = real_last();
        self.obj_picked_pnt = Point3::new(real_last(), 0.0, 0.0);
        self.normal = Vec3::new(0.0, 0.0, 0.0);
    }
    /// The depth along the picking ray; `pick.depth()`.
    pub fn depth(&self) -> f64 {
        self.depth
    }
    /// Sets the depth; `pick.set_depth(1.5)`.
    pub fn set_depth(&mut self, depth: f64) {
        self.depth = depth;
    }
    /// Whether a picked point was set; `pick.has_picked_point()`.
    pub fn has_picked_point(&self) -> bool {
        self.obj_picked_pnt.x() != real_last()
    }
    /// The picked point, in the local coordinate system of the entity; `pick.picked_point()`.
    pub fn picked_point(&self) -> &Point3 {
        &self.obj_picked_pnt
    }
    /// Sets the picked point; `pick.set_picked_point(Point3::new(1.0, 2.0, 3.0))`.
    pub fn set_picked_point(&mut self, point: Point3) {
        self.obj_picked_pnt = point;
    }
    /// The distance to the geometry center; `pick.dist_to_geom_center()`.
    pub fn dist_to_geom_center(&self) -> f64 {
        self.dist_to_center
    }
    /// Sets the distance to the geometry center; `pick.set_dist_to_geom_center(0.25)`.
    pub fn set_dist_to_geom_center(&mut self, distance: f64) {
        self.dist_to_center = distance;
    }
    /// The unnormalized surface normal (zero when undefined), local coordinates; `pick.surface_normal()`.
    pub fn surface_normal(&self) -> &Vec3<f32> {
        &self.normal
    }
    /// Sets the surface normal; `pick.set_surface_normal(Vec3::new(0.0, 0.0, 1.0))`.
    pub fn set_surface_normal(&mut self, normal: Vec3<f32>) {
        self.normal = normal;
    }
    /// Sets the surface normal from a vector, narrowed to float; `pick.set_surface_normal_vec(&v)`.
    pub fn set_surface_normal_vec(&mut self, normal: &Vector3) {
        self.normal = Vec3::new(normal.x() as f32, normal.y() as f32, normal.z() as f32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_result_is_invalid_and_has_no_point() {
        let pick = SelectBasicsPickResult::default();
        assert!(!pick.is_valid());
        assert!(!pick.has_picked_point());
        assert_eq!(pick.dist_to_geom_center(), real_last());
    }

    #[test]
    fn min_prefers_the_first_on_ties() {
        let a = SelectBasicsPickResult::new(1.0, 2.0, Point3::default());
        let b = SelectBasicsPickResult::new(1.0, 3.0, Point3::default());
        assert_eq!(
            SelectBasicsPickResult::min(&a, &b).dist_to_geom_center(),
            2.0
        );
        let c = SelectBasicsPickResult::new(0.5, 4.0, Point3::default());
        assert_eq!(
            SelectBasicsPickResult::min(&a, &c).dist_to_geom_center(),
            4.0
        );
    }

    #[test]
    fn invalidate_resets_depth_point_and_normal() {
        let mut pick = SelectBasicsPickResult::new(1.0, 2.0, Point3::new(1.0, 2.0, 3.0));
        pick.set_surface_normal_vec(&Vector3::new(0.1, 0.2, 0.3));
        assert_eq!(pick.surface_normal().z, 0.3f32);
        pick.invalidate();
        assert!(!pick.is_valid() && !pick.has_picked_point());
        assert_eq!(pick.surface_normal().x, 0.0);
        pick.set_depth(3.0);
        pick.set_picked_point(Point3::new(4.0, 0.0, 0.0));
        pick.set_dist_to_geom_center(5.0);
        assert!(pick.is_valid() && pick.has_picked_point());
        assert_eq!((pick.depth(), pick.picked_point().x()), (3.0, 4.0));
    }
}
