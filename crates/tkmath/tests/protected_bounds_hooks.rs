//! Direct protected-hook transitions, checked against pinned native OCCT.
use rustycad_tkmath::bnd::{BoundingBox3, OrientedBoundingBox};
use rustycad_tkmath::gp::{Direction3, Point3, Transform3};
use rustycad_tkmath::poly::Triangulation;
use std::fmt::Write;

fn emit_obb(box_: &OrientedBoundingBox) -> String {
    let mut line: String = format!(
        "obb {} {}",
        u8::from(box_.is_void()),
        u8::from(box_.is_aa_box())
    );
    append_axes(&mut line, box_);
    write!(
        line,
        " {} {} {}",
        box_.x_h_size(),
        box_.y_h_size(),
        box_.z_h_size()
    )
    .unwrap();
    line
}

fn append_axes(line: &mut String, box_: &OrientedBoundingBox) {
    for xyz in [
        box_.center(),
        box_.x_direction(),
        box_.y_direction(),
        box_.z_direction(),
    ] {
        write!(line, " {} {} {}", xyz.x(), xyz.y(), xyz.z()).unwrap();
    }
}

fn emit_mesh(mesh: &Triangulation) -> String {
    let mut bounds: BoundingBox3 = BoundingBox3::default();
    let found: bool = mesh.min_max(&mut bounds, Transform3::default(), false);
    let mut line: String = format!(
        "mesh {} {} {} {} {}",
        u8::from(mesh.has_cached_min_max()),
        u8::from(mesh.cached_min_max().is_void()),
        mesh.nb_nodes(),
        mesh.nb_triangles(),
        u8::from(found)
    );
    if found {
        for value in bounds.get().unwrap() {
            write!(line, " {value}").unwrap();
        }
    }
    line
}

fn expected_rows(prefix: &str) -> Vec<&'static str> {
    include_str!("../../../tools/oracle/tkmath_protected_bounds.expected")
        .lines()
        .filter(|line| line.starts_with(prefix))
        .collect()
}

#[test]
fn protected_one_point_resets_void_and_oriented_boxes() {
    let mut box_: OrientedBoundingBox = OrientedBoundingBox::default();
    box_.process_one_point(Point3::new(3.0, -4.0, 5.0));
    let mut rows: Vec<String> = vec![emit_obb(&box_)];
    box_.set_center(Point3::new(17.0, 18.0, 19.0));
    box_.set_x_component(Direction3::Y, 6.0);
    box_.set_y_component(Direction3::new(-1.0, 0.0, 0.0).unwrap(), 7.0);
    box_.set_z_component(Direction3::Z, 8.0);
    box_.set_aa_box(false);
    box_.process_one_point(Point3::new(-8.0, 9.0, -10.0));
    rows.push(emit_obb(&box_));
    box_.process_one_point(Point3::new(-8.0, 9.0, -10.0));
    rows.push(emit_obb(&box_));
    assert_eq!(rows, expected_rows("obb"));
}

#[test]
fn protected_cache_clear_is_idempotent_and_preserves_nodes() {
    let mesh: Triangulation = Triangulation::new();
    mesh.unset_cached_min_max();
    let mut rows: Vec<String> = vec![emit_mesh(&mesh)];
    let mut cached: BoundingBox3 = BoundingBox3::default();
    cached.update_bounds([-10.0, -20.0, -30.0], [10.0, 20.0, 30.0]);
    mesh.set_cached_min_max(&cached);
    rows.push(emit_mesh(&mesh));
    mesh.unset_cached_min_max();
    rows.push(emit_mesh(&mesh));
    append_geometry_transitions(&mesh, &cached, &mut rows);
    assert_eq!(rows, expected_rows("mesh"));
}

fn append_geometry_transitions(
    mesh: &Triangulation,
    cached: &BoundingBox3,
    rows: &mut Vec<String>,
) {
    mesh.resize_nodes(2, false).unwrap();
    mesh.set_node(1, Point3::new(1.0, 2.0, 3.0)).unwrap();
    mesh.set_node(2, Point3::new(4.0, 5.0, 6.0)).unwrap();
    mesh.set_cached_min_max(cached);
    mesh.unset_cached_min_max();
    rows.push(emit_mesh(mesh));
    mesh.unset_cached_min_max();
    rows.push(emit_mesh(mesh));
}
