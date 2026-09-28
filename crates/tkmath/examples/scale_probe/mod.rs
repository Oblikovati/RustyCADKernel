//! Native scale/composition and circle counterpart.
use super::{matrix, xyz};
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Point3, Transform3};
fn transform_values(transform: Transform3, point: Point3) {
    print!(
        "{} {} ",
        transform.form() as i32,
        transform.scale_factor().to_bits()
    );
    xyz(transform.translation_part());
    matrix(transform.matrix());
    xyz(point.transformed(transform).xyz());
}
fn circle_values(circle: Circle3, point: Point3) {
    xyz(circle.location().xyz());
    xyz(circle.position().direction().into());
    xyz(circle.position().x_direction().into());
    xyz(circle.position().y_direction().into());
    print!(
        "{} {} {} {} ",
        circle.radius().to_bits(),
        circle.area().to_bits(),
        circle.length().to_bits(),
        circle.square_distance(point).to_bits()
    );
}
fn scale_result(v: &[f64]) -> Result<(), rustycad_tkernel::standard::Failure> {
    let point = Point3::new(v[0], v[1], v[2]);
    let origin = Point3::new(v[3], v[4], v[5]);
    let direction = Direction3::new(v[6], v[7], v[8]).unwrap();
    let axis = Axis1::new(origin, direction);
    let first = Transform3::scaling(origin, v[10])?;
    let second = Transform3::scaling(point, v[11])?;
    let rotation = Transform3::rotation(axis, v[9]).unwrap();
    let translation = Transform3::translation(*point.coordinates());
    transform_values(first, point);
    transform_values(first.inverted()?, point);
    transform_values(first.powered(v[12] as i32)?, point);
    for result in [
        first.multiplied(second),
        first.multiplied(rotation),
        rotation.multiplied(first),
        translation.multiplied(first),
        first.multiplied(translation),
        rotation.multiplied(first).multiplied(second),
        first.multiplied(rotation).multiplied(second),
    ] {
        transform_values(result, point);
        transform_values(result.inverted()?, point);
        transform_values(result.powered(v[12] as i32)?, point);
    }
    xyz(direction.transformed(first).into());
    xyz(direction.transformed(rotation.multiplied(first)).into());
    let circle = Circle3::new(Axis2::new(origin, direction).unwrap(), v[13].abs()).unwrap();
    circle_values(circle, point);
    circle_values(circle.scaled(point, v[10]), point);
    circle_values(circle.transformed(first).unwrap(), point);
    circle_values(circle.rotated(axis, v[9]).unwrap(), point);
    circle_values(circle.mirrored_point(point), point);
    println!();
    Ok(())
}

pub(super) fn scale_case(v: &[f64]) {
    if let Err(failure) = scale_result(v) {
        println!("{} {}", failure.exception_type(), failure.message());
    }
}
