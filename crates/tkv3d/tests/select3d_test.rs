//! Select3D behaviour: construction, boxes, centers and the BVH traversal of the sensitive
//! entities against `BoxVolumeManager`, a named fake box selection, and the translation of
//! GTests/Select3D_SensitivePrimitiveArray_Test.cxx.
use rustycad_tkernel::ncollection::{Array1, DynamicArray, Vec3, Vec4};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Transform3};
use rustycad_tkmath::poly::{Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;
use rustycad_tkv3d::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use rustycad_tkv3d::select_mgr::{
    SelectMgrEntityOwner, SelectMgrEntityOwnerSource, SelectMgrSelectionType,
};
use rustycad_tkv3d::select3d::{
    Select3DPrimArrayInit, Select3DSensitiveEntity, Select3DSensitiveGroup, Select3DSensitivePoint,
    Select3DSensitivePoly, Select3DSensitivePrimitiveArray, Select3DSensitiveSegment,
    Select3DSensitiveSet, Select3DSensitiveTriangulation, Select3DSensitiveWire,
};

/// A box selection along -Z: a point inside `[min, max]` overlaps at depth `-z`; a segment
/// or triangle overlaps when one of its points does; a box overlaps when it intersects.
struct BoxVolumeManager {
    fail_distance: bool,
    min: Point3,
    max: Point3,
}

impl BoxVolumeManager {
    fn inside(&self, p: &Point3) -> bool {
        (self.min.x()..=self.max.x()).contains(&p.x())
            && (self.min.y()..=self.max.y()).contains(&p.y())
            && (self.min.z()..=self.max.z()).contains(&p.z())
    }
    fn pick(&self, points: &[&Point3], pick: &mut SelectBasicsPickResult) -> bool {
        let Some(hit) = points.iter().find(|p| self.inside(p)) else {
            return false;
        };
        pick.set_depth(-hit.z());
        pick.set_picked_point(**hit);
        true
    }
}

impl SelectBasicsSelectingVolumeManager for BoxVolumeManager {
    fn get_active_selection_type(&self) -> SelectMgrSelectionType {
        SelectMgrSelectionType::Box
    }
    fn overlaps_box(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok({
            let hit = self.overlaps_box_inside(min, max, None)?;
            if hit {
                pick.set_depth(-max.z);
            }
            hit
        })
    }
    fn overlaps_box_inside(
        &self,
        min: &Vec3<f64>,
        max: &Vec3<f64>,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        Ok({
            let overlaps = min.x <= self.max.x()
                && max.x >= self.min.x()
                && min.y <= self.max.y()
                && max.y >= self.min.y()
                && min.z <= self.max.z()
                && max.z >= self.min.z();
            if let Some(flag) = inside {
                *flag = self.inside(&Point3::new(min.x, min.y, min.z))
                    && self.inside(&Point3::new(max.x, max.y, max.z));
            }
            overlaps
        })
    }
    fn overlaps_point(
        &self,
        point: &Point3,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(self.pick(&[point], pick))
    }
    fn overlaps_point_only(&self, point: &Point3) -> Result<bool, Failure> {
        Ok(self.inside(point))
    }
    fn overlaps_polygon(
        &self,
        points: &Array1<'_, Point3>,
        _sens_type: i32,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok({
            let refs: Vec<&Point3> = points.iter().collect();
            self.pick(&refs, pick)
        })
    }
    fn overlaps_segment(
        &self,
        a: &Point3,
        b: &Point3,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(self.pick(&[a, b], pick))
    }
    fn overlaps_triangle(
        &self,
        a: &Point3,
        b: &Point3,
        c: &Point3,
        _sens_type: i32,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(self.pick(&[a, b, c], pick))
    }
    fn overlaps_sphere(
        &self,
        center: &Point3,
        _radius: f64,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(self.pick(&[center], pick))
    }
    fn overlaps_sphere_inside(
        &self,
        center: &Point3,
        _radius: f64,
        _inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        Ok(self.inside(center))
    }
    fn overlaps_cylinder(
        &self,
        _b: f64,
        _t: f64,
        _h: f64,
        _trsf: &Transform3,
        _hollow: bool,
        _pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    fn overlaps_cylinder_inside(
        &self,
        _b: f64,
        _t: f64,
        _h: f64,
        _trsf: &Transform3,
        _hollow: bool,
        _inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    fn overlaps_circle(
        &self,
        _radius: f64,
        _trsf: &Transform3,
        _filled: bool,
        _pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    fn overlaps_circle_inside(
        &self,
        _radius: f64,
        _trsf: &Transform3,
        _filled: bool,
        _inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    fn dist_to_geometry_center(&self, cog: &Point3) -> Result<f64, Failure> {
        if self.fail_distance {
            return Err(Failure::with_message(
                "Standard_ProgramError",
                "distance requested from failing selector; expected available geometry center",
            ));
        }
        Ok(cog.distance(&Point3::new(0.0, 0.0, 0.0)))
    }
    fn detected_point(&self, depth: f64) -> Result<Point3, Failure> {
        Ok(Point3::new(0.0, 0.0, -depth))
    }
    fn is_overlap_allowed(&self) -> bool {
        true
    }
    fn get_near_picked_pnt(&self) -> Result<Point3, Failure> {
        Ok(Point3::default())
    }
    fn get_far_picked_pnt(&self) -> Result<Point3, Failure> {
        Ok(Point3::default())
    }
    fn get_view_ray_direction(&self) -> Result<Direction3, Failure> {
        Ok(Direction3::default())
    }
    fn is_scalable_active_volume(&self) -> bool {
        false
    }
    fn get_mouse_position(&self) -> Point2 {
        Point2::default()
    }
    fn get_planes(&self, _planes: &mut DynamicArray<Vec4<f64>>) {}
}

fn owner() -> Handle<dyn SelectMgrEntityOwnerSource> {
    SelectMgrEntityOwner::new(0).cast()
}

fn volume(x0: f64, x1: f64) -> BoxVolumeManager {
    BoxVolumeManager {
        fail_distance: false,
        min: Point3::new(x0, -1.0, -1.0),
        max: Point3::new(x1, 1.0, 1.0),
    }
}

#[test]
fn primitive_array_detects_the_nearest_triangle() {
    let tris = Graphic3dArrayOfTriangles::new(6, 0, 0).unwrap();
    for (x, z) in [(0.0, -0.5), (10.0, -0.2)] {
        tris.add_vertex_f64(x, 0.0, z).unwrap();
        tris.add_vertex_f64(x + 1.0, 0.0, z).unwrap();
        tris.add_vertex_f64(x, 0.5, z).unwrap();
    }
    let sens = Select3DSensitivePrimitiveArray::new(&owner()).unwrap();
    let init = Select3DPrimArrayInit::whole(
        &tris.attributes(),
        &tris.indices(),
        &Location::default(),
        true,
        1,
    );
    assert!(sens.init_triangulation(&init).unwrap());
    assert_eq!(sens.size(), 2);
    let mut pick = SelectBasicsPickResult::default();
    assert!(sens.matches(&volume(-1.0, 0.5), &mut pick).unwrap());
    assert_eq!(sens.last_detected_element(), 0);
    let mut miss = SelectBasicsPickResult::default();
    assert!(!sens.matches(&volume(3.0, 4.0), &mut miss).unwrap());
}

#[test]
fn poly_rounds_points_to_single_precision_and_builds_its_segments() {
    let points = [
        Point3::new(0.1, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
    ];
    let poly = Select3DSensitivePoly::new(&owner(), &points, true).unwrap();
    assert_eq!(poly.nb_sub_elements(), 3);
    assert_eq!(poly.size(), 2);
    assert_eq!(poly.get_point_3d(0).x(), f64::from(0.1f32));
    // The box of the constructor keeps the double coordinates.
    assert_eq!(poly.bounding_box().corner_min().x, 0.1);
    let mut pick = SelectBasicsPickResult::default();
    assert!(poly.matches(&volume(1.5, 2.5), &mut pick).unwrap());
    assert!(poly.set_base().detected_idx() >= 0);
}

#[test]
fn wire_center_is_the_running_half_sum() {
    let wire = Select3DSensitiveWire::new(&owner());
    let a: Handle<dyn Select3DSensitiveEntity> = Handle::new(Select3DSensitivePoint::new(
        &owner(),
        Point3::new(2.0, 0.0, 0.0),
    ))
    .cast();
    let b: Handle<dyn Select3DSensitiveEntity> = Handle::new(Select3DSensitivePoint::new(
        &owner(),
        Point3::new(4.0, 0.0, 0.0),
    ))
    .cast();
    let c: Handle<dyn Select3DSensitiveEntity> = Handle::new(Select3DSensitivePoint::new(
        &owner(),
        Point3::new(8.0, 0.0, 0.0),
    ))
    .cast();
    for edge in [&a, &b, &c] {
        wire.add(edge).unwrap();
    }
    // ((2 + 4) / 2 + 8) / 2 = 5.5, not the mean.
    assert_eq!(wire.center_of_geometry().x(), 5.5);
    assert!(wire.add(&Handle::default()).is_err());
    let mut pick = SelectBasicsPickResult::default();
    assert!(wire.matches(&volume(7.0, 9.0), &mut pick).unwrap());
    assert_eq!(wire.get_last_detected(), c);
}

#[test]
fn group_keeps_distinct_entities_and_matches_all_for_a_box() {
    let a: Handle<dyn Select3DSensitiveEntity> = Handle::new(Select3DSensitiveSegment::new(
        &owner(),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ))
    .cast();
    let b: Handle<dyn Select3DSensitiveEntity> = Handle::new(Select3DSensitiveSegment::new(
        &owner(),
        Point3::new(5.0, 0.0, 0.0),
        Point3::new(6.0, 0.0, 0.0),
    ))
    .cast();
    let group =
        Select3DSensitiveGroup::with_entities(&owner(), &[a.clone(), b.clone(), a.clone()], true);
    assert_eq!(group.nb_sub_elements(), 2);
    assert_eq!(group.center_of_geometry().x(), 3.0);
    let mut pick = SelectBasicsPickResult::default();
    assert!(!group.matches(&volume(-1.0, 2.0), &mut pick).unwrap());
    group.set_match_type(false);
    assert!(group.matches(&volume(-1.0, 2.0), &mut pick).unwrap());
    group.remove(&a);
    assert!(!group.is_in(&a) && group.is_in(&b));
}

#[test]
fn triangulation_counts_free_edges_and_centers_on_the_triangles() {
    let trg = Triangulation::with_sizes(4, 2, false, false).unwrap();
    for (index, (x, y)) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        trg.set_node(index as i32 + 1, Point3::new(x, y, 0.0))
            .unwrap();
    }
    trg.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    trg.set_triangle(2, Triangle::new(1, 3, 4)).unwrap();
    let trg = Handle::new(trg);
    let boundary =
        Select3DSensitiveTriangulation::new(&owner(), &trg, &Location::default(), false).unwrap();
    assert_eq!(boundary.size(), 4);
    let interior =
        Select3DSensitiveTriangulation::new(&owner(), &trg, &Location::default(), true).unwrap();
    assert_eq!(interior.size(), 2);
    assert_eq!(
        interior.center_of_geometry().x(),
        (2.0 / 3.0 + 1.0 / 3.0) / 2.0
    );
    let mut pick = SelectBasicsPickResult::default();
    assert!(interior.matches(&volume(0.9, 1.1), &mut pick).unwrap());
    assert!(interior.last_detected_triangle_index() >= 1);
}

#[test]
fn sensitivity_factor_refuses_negative_values() {
    use rustycad_tkv3d::select3d::Select3DSensitiveEntityMembers as _;
    let point = Select3DSensitivePoint::new(&owner(), Point3::new(0.0, 0.0, 0.0));
    assert_eq!(point.sensitivity_factor(), 12);
    assert!(point.set_sensitivity_factor(-1).is_err());
    let connected = point.get_connected().unwrap();
    assert!(!connected.is_null());
}

#[test]
fn sensitive_matches_propagates_distance_failure_after_successful_overlap() {
    let mut manager = volume(-2., 2.);
    manager.fail_distance = true;
    let mut pick = SelectBasicsPickResult::default();
    let segment =
        Select3DSensitiveSegment::new(&owner(), Point3::new(-1., 0., 0.), Point3::new(1., 0., 0.));
    assert_eq!(
        segment
            .matches(&manager, &mut pick)
            .unwrap_err()
            .exception_type(),
        "Standard_ProgramError"
    );
    assert_eq!(pick.depth(), 0.);
    let poly = Select3DSensitivePoly::new(
        &owner(),
        &[Point3::new(-1., 0., 0.), Point3::new(1., 0., 0.)],
        false,
    )
    .unwrap();
    assert_eq!(
        poly.distance_to_cog(&manager).unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
}

#[test]
fn protected_set_members_bind_and_traverse_the_elements() {
    use rustycad_tkmath::bvh::BvhSet;
    use rustycad_tkv3d::select3d::{
        Select3DSensitiveEntityMembers, Select3DTypeOfSensitivity, select3d_set_clear,
        select3d_set_process_elements,
    };
    let points: Vec<Point3> = (0..5).map(|i| Point3::new(f64::from(i), 0.0, 0.0)).collect();
    let poly = Select3DSensitivePoly::new(&owner(), &points, true).unwrap();
    let (size, depth) = poly.set_base().with_primitive_set(&poly, |content| {
        let tree = content.get_bvh().unwrap();
        (content.size(), tree.read().depth())
    });
    assert_eq!((size, depth), (4, 0));
    assert!(!poly.set_base().is_dirty());
    // Elements 1..=2 are the segments [1, 2] and [2, 3]: only the first one touches x <= 1.5.
    let mut pick = SelectBasicsPickResult::default();
    let mut matches = 0;
    let found = select3d_set_process_elements(
        &poly, &volume(0.5, 1.5), 1, 2, false, false, &mut pick, &mut matches,
    )
    .unwrap();
    assert!(found);
    assert_eq!(matches, 1);
    select3d_set_clear(&poly);
    assert!(!poly.owner_id().is_null());
    let trg = Triangulation::with_sizes(3, 1, false, false).unwrap();
    for (index, (x, y)) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)].into_iter().enumerate() {
        trg.set_node(index as i32 + 1, Point3::new(x, y, 0.0)).unwrap();
    }
    trg.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    let trg = Handle::new(trg);
    let sens =
        Select3DSensitiveTriangulation::new(&owner(), &trg, &Location::default(), false).unwrap();
    assert_eq!(sens.primitives_nb(), 3);
    assert_eq!(sens.sens_type(), Select3DTypeOfSensitivity::Boundary);
    assert_eq!(sens.free_edges().get().map(|edges| edges.array1().length()), Some(6));
    assert_eq!(sens.cdg_3d().x(), 1.0 / 3.0);
    sens.compute_bounding_box();
    assert_eq!(sens.bnd_box().corner_max().x(), 1.0);
    assert_eq!(sens.apply_transformation().corner_max().y(), 1.0);
}
