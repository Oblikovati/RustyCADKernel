//! Behavior tests of `BRepLib_PointCloudShape` (no upstream suite uses it) on the box
//! [0, 10]^3 and the sphere of radius 10 built by the probe's shape reader: the counts
//! by density, the default density quirk, the random points on the faces, the offset
//! along the normal, and the triangulation path on a face without mesh.
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;
#[allow(dead_code)]
#[path = "../examples/tp_probe/shape.rs"]
mod shape;
#[allow(dead_code)]
#[path = "../examples/tp_probe/shape_geometry.rs"]
mod shape_geometry;
mod solid_records;

use print::Reader;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};
use rustycad_tktopalgo::brep_lib::{BRepLibPointCloudShape, BRepLibPointCloudShapeBase};
use shape::tp_shape;
use solid_records::{BOX10, SPHERE10};

/// A cloud that keeps every point and normal it receives.
struct CollectedCloud {
    base: BRepLibPointCloudShapeBase,
    points: Vec<(Point3, Vector3, Point2)>,
}

impl CollectedCloud {
    fn on(record: &[f64]) -> Self {
        let shape = tp_shape(&mut Reader::new(record))
            .expect("shape record")
            .shape;
        Self {
            base: BRepLibPointCloudShapeBase::new(shape, 1.0e-7),
            points: Vec::new(),
        }
    }
}

impl BRepLibPointCloudShape for CollectedCloud {
    fn point_cloud_base(&self) -> &BRepLibPointCloudShapeBase {
        &self.base
    }
    fn point_cloud_base_mut(&mut self) -> &mut BRepLibPointCloudShapeBase {
        &mut self.base
    }
    fn add_point(&mut self, point: Point3, normal: Vector3, uv: Point2, _face: &Shape) {
        self.points.push((point, normal, uv));
    }
}

fn on_box_surface(point: Point3) -> bool {
    let coordinates = [point.x(), point.y(), point.z()];
    let inside = coordinates
        .iter()
        .all(|c| (-1.0e-9..=10.0 + 1.0e-9).contains(c));
    let on_side = coordinates
        .iter()
        .any(|c| c.abs() <= 1.0e-9 || (c - 10.0).abs() <= 1.0e-9);
    inside && on_side
}

#[test]
fn density_counts_ceil_of_area_over_density_per_face() {
    let mut cloud = CollectedCloud::on(&BOX10);
    // Six faces of area 100: 34 points each at density 3.
    assert_eq!(cloud.nb_points_by_density(3.0).unwrap(), 6 * 34);
}

#[test]
fn zero_density_gives_one_point_per_face() {
    // The default density is computed (a tenth of the smallest face) but the count still
    // divides by the given 0, so every face gets max(INT_MIN, 1) = 1 point.
    let mut cloud = CollectedCloud::on(&BOX10);
    assert_eq!(cloud.nb_points_by_density(0.0).unwrap(), 6);
}

#[test]
fn density_points_lie_on_the_faces_with_unit_normals() {
    let mut cloud = CollectedCloud::on(&BOX10);
    assert!(cloud.generate_points_by_density(10.0).unwrap());
    assert_eq!(cloud.points.len(), 60);
    for (point, normal, _) in &cloud.points {
        assert!(on_box_surface(*point), "{point:?}");
        assert!((normal.magnitude() - 1.0).abs() <= 1.0e-12);
    }
}

#[test]
fn distance_moves_the_points_off_the_sphere_within_the_distance() {
    let mut cloud = CollectedCloud::on(&SPHERE10);
    cloud.set_distance(0.5);
    assert!(cloud.generate_points_by_density(40.0).unwrap());
    assert!(!cloud.points.is_empty());
    for (point, _, _) in &cloud.points {
        let radius = point.distance(&Point3::new(0.0, 0.0, 0.0));
        assert!(
            (10.0 - 1.0e-9..=10.5 + 1.0e-9).contains(&radius),
            "{radius}"
        );
    }
}

#[test]
fn counts_are_kept_between_generations() {
    let mut cloud = CollectedCloud::on(&BOX10);
    assert_eq!(cloud.nb_points_by_density(50.0).unwrap(), 12);
    // The counts of density 50 are reused although density 10 is asked.
    assert!(cloud.generate_points_by_density(10.0).unwrap());
    assert_eq!(cloud.points.len(), 12);
}

#[test]
fn faces_without_mesh_give_no_triangulation_points() {
    let mut cloud = CollectedCloud::on(&BOX10);
    assert_eq!(cloud.nb_points_by_triangulation().unwrap(), 0);
    assert!(!cloud.generate_points_by_triangulation().unwrap());
    assert!(cloud.points.is_empty());
}
