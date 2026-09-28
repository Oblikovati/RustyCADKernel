//! Rust counterpart for the line, plane and bounding-box OCCT oracle.
use super::xyz;
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Axis1, Axis2, Direction3, Line3, Plane3, Point3, Transform3},
};
fn box_values(bounds: &BoundingBox3) {
    for value in [
        bounds.is_void(),
        bounds.is_whole(),
        bounds.is_open(),
        bounds.has_finite_part(),
    ] {
        print!("{} ", i32::from(value));
    }
    print!(
        "{} {} ",
        bounds.gap().to_bits(),
        bounds.square_extent().to_bits()
    );
    if let Ok(values) = bounds.get() {
        for value in values {
            print!("{} ", value.to_bits());
        }
    }
}
fn plane_values(plane: Plane3) {
    xyz(plane.location().xyz());
    xyz(plane.position().direction().into());
    xyz(plane.position().x_direction().into());
    xyz(plane.position().y_direction().into());
    for value in plane.coefficients().unwrap() {
        print!("{} ", value.to_bits());
    }
}
fn open_flags(bounds: &mut BoundingBox3, flags: i32) {
    if flags & 1 != 0 {
        bounds.open_xmin();
    }
    if flags & 2 != 0 {
        bounds.open_xmax();
    }
    if flags & 4 != 0 {
        bounds.open_ymin();
    }
    if flags & 8 != 0 {
        bounds.open_ymax();
    }
    if flags & 16 != 0 {
        bounds.open_zmin();
    }
    if flags & 32 != 0 {
        bounds.open_zmax();
    }
    if flags == 64 {
        bounds.set_void();
    }
    if flags == 65 {
        bounds.set_whole();
    }
}
pub(super) fn bounds_case(v: &[f64]) {
    let first = Point3::new(v[0], v[1], v[2]);
    let second = Point3::new(v[3], v[4], v[5]);
    let direction = Direction3::new(v[6], v[7], v[8]).unwrap();
    let mut bounds = BoundingBox3::default();
    bounds.add_point(first);
    bounds.add_point(second);
    bounds.set_gap(v[9]);
    open_flags(&mut bounds, v[15] as i32);
    let point = Point3::new(v[10], v[11], v[12]);
    let line = Line3::new(point, direction);
    let plane = Plane3::new(point, direction).unwrap();
    box_values(&bounds);
    for value in [
        bounds.is_out_point(point),
        bounds.is_out_line(line),
        bounds.is_out_plane(plane).unwrap(),
        bounds.is_out_segment(first, point, direction),
        bounds.is_x_thin(v[13]),
        bounds.is_y_thin(v[13]),
        bounds.is_z_thin(v[13]),
        bounds.is_thin(v[13]),
    ] {
        print!("{} ", i32::from(value));
    }
    let mut other = BoundingBox3::default();
    other.add_point(point);
    other.set_gap(v[14]);
    print!(
        "{} {} ",
        i32::from(bounds.is_out_box(&other)),
        bounds.distance(&other).to_bits()
    );
    bounds.add_box(&other);
    box_values(&bounds);
    let rotation = Transform3::rotation(Axis1::new(point, direction), v[13]).unwrap();
    box_values(&bounds.transformed(rotation));
    box_values(&bounds.finite_part());
    println!();
}
pub(super) fn line_plane_case(v: &[f64]) {
    let origin = Point3::new(v[0], v[1], v[2]);
    let point = Point3::new(v[3], v[4], v[5]);
    let direction = Direction3::new(v[6], v[7], v[8]).unwrap();
    let line = Line3::new(origin, direction);
    let other = Line3::new(point, Direction3::new(v[3], v[4], v[5]).unwrap());
    print!(
        "{} {} {} ",
        line.distance(point).to_bits(),
        line.square_distance(point).to_bits(),
        line.distance_line(other).unwrap().to_bits()
    );
    let plane = Plane3::new(origin, direction).unwrap();
    plane_values(plane);
    print!(
        "{} {} ",
        plane.distance(point).to_bits(),
        plane.signed_distance(point).to_bits()
    );
    plane_values(Plane3::from_equation(v[6], v[7], v[8], v[9]).unwrap());
    plane_values(plane.scaled(point, v[10]));
    plane_values(plane.mirrored_plane(Axis2::new(point, direction).unwrap()));
    plane_values(plane.rotated(Axis1::new(point, direction), v[11]).unwrap());
    println!();
}
fn box2_values(bounds: &rustycad_tkmath::bnd::BoundingBox2) {
    print!(
        "{} {} {} {} ",
        i32::from(bounds.is_void()),
        i32::from(bounds.is_whole()),
        bounds.gap().to_bits(),
        bounds.square_extent().to_bits()
    );
    let limits = bounds.limits();
    for value in [limits.x_min, limits.y_min, limits.x_max, limits.y_max] {
        print!("{} ", value.to_bits());
    }
}
pub(super) fn bounds2_case(v: &[f64]) {
    use rustycad_tkmath::{
        bnd::BoundingBox2,
        gp::{Direction2, Line2, Point2, Transform2},
    };
    let first = Point2::new(v[0], v[1]);
    let second = Point2::new(v[2], v[3]);
    let point = Point2::new(v[4], v[5]);
    let direction = Direction2::new(v[6], v[7]).unwrap();
    let mut bounds = BoundingBox2::default();
    bounds.add_point(first);
    bounds.add_point(second);
    bounds.set_gap(v[8]);
    let flags = v[12] as i32;
    if flags & 1 != 0 {
        bounds.open_xmin();
    }
    if flags & 2 != 0 {
        bounds.open_xmax();
    }
    if flags & 4 != 0 {
        bounds.open_ymin();
    }
    if flags & 8 != 0 {
        bounds.open_ymax();
    }
    if flags == 16 {
        bounds.set_void();
    }
    if flags == 17 {
        bounds.set_whole();
    }
    box_values(&BoundingBox3::default());
    box2_values(&bounds);
    for value in [
        bounds.is_out_point(point),
        bounds.is_out_line(Line2::new(point, direction)),
        bounds.is_out_segment(first, point),
    ] {
        print!("{} ", i32::from(value));
    }
    let mut other = BoundingBox2::default();
    other.add_point(point);
    other.set_gap(v[9]);
    print!(
        "{} {} ",
        i32::from(bounds.is_out_box(&other)),
        bounds.distance(&other).to_bits()
    );
    bounds.add_box(&other);
    box2_values(&bounds);
    bounds.add_point(point);
    box2_values(&bounds);
    box2_values(&bounds.transformed(Transform2::rotation(point, v[10])));
    box2_values(&bounds.transformed(Transform2::translation([v[10], v[11]])));
    println!();
}
pub(super) fn sphere_case(v: &[f64]) {
    use rustycad_tkmath::{bnd::BoundingSphere, gp::Xyz};
    let mut sphere = BoundingSphere::new(Xyz::new(v[0], v[1], v[2]), v[6], 3, 4);
    let mut other = BoundingSphere::new(Xyz::new(v[3], v[4], v[5]), v[7], 8, 9);
    sphere.set_valid(v[8] > 0.0);
    other.set_valid(v[9] > 0.0);
    let point = Xyz::new(v[3], v[10], v[11]);
    let (low, high) = sphere.distances(point);
    print!("{} {} ", low.to_bits(), high.to_bits());
    let (low, high) = sphere.square_distances(point);
    print!("{} {} ", low.to_bits(), high.to_bits());
    let mut limit = v[10];
    let outside = sphere.is_out_point(point, &mut limit);
    print!(
        "{} {} {} ",
        i32::from(outside),
        limit.to_bits(),
        i32::from(sphere.is_out_sphere(&other))
    );
    sphere.add(&other);
    xyz(*sphere.center());
    println!(
        "{} {} {} {} {}",
        sphere.radius().to_bits(),
        i32::from(sphere.is_valid()),
        sphere.u(),
        sphere.v(),
        sphere.square_extent().to_bits()
    );
}

mod compact;
pub(super) use compact::{compact2_case, compact3_case};
