//! Compact-box probes exercise native float and double instantiations.
fn compact2_values<T: rustycad_tkmath::bnd::BoxReal>(
    bounds: &rustycad_tkmath::bnd::CompactBox2<T>,
) {
    print!("{} ", i32::from(bounds.is_void()));
    super::super::xy(bounds.corner_min());
    super::super::xy(bounds.corner_max());
    print!("{} ", bounds.square_extent().to_bits());
}
fn compact2_typed<T: rustycad_tkmath::bnd::BoxReal>(v: &[f64]) {
    use rustycad_tkmath::{
        bnd::CompactBox2,
        gp::{Axis2d, Direction2, Point2, Transform2, Xy},
    };
    let mut bounds = CompactBox2::<T>::new(Xy::new(v[0], v[1]), Xy::new(v[2], v[3]));
    let other = CompactBox2::<T>::new(Xy::new(v[4], v[5]), Xy::new(v[6], v[7]));
    compact2_values(&bounds);
    let point = Xy::new(v[8], v[9]);
    for value in [
        bounds.is_out_point(point),
        bounds.is_out_box(&other),
        bounds.is_in_box(&other),
        bounds.is_out_circle(point, v[10], false),
        bounds.is_out_circle(point, v[10], true),
        bounds.is_out_line(Axis2d::new(Point2::from(point), Direction2::X)),
        bounds.is_out_segment(point, Xy::new(v[4], v[5])),
    ] {
        print!("{} ", i32::from(value));
    }
    let mut limited = bounds;
    print!("{} ", i32::from(limited.limit(&other)));
    compact2_values(&limited);
    bounds.add_xy(point);
    bounds.add_box(&other);
    bounds.enlarge(v[10]);
    compact2_values(&bounds);
    compact2_values(&bounds.transformed(Transform2::rotation(Point2::from(point), v[11])));
}
pub(crate) fn compact2_case(v: &[f64]) {
    compact2_typed::<f64>(v);
    compact2_typed::<f32>(v);
    println!();
}
fn compact3_values<T: rustycad_tkmath::bnd::BoxReal>(
    bounds: &rustycad_tkmath::bnd::CompactBox3<T>,
) {
    print!("{} ", i32::from(bounds.is_void()));
    super::super::xyz(bounds.corner_min());
    super::super::xyz(bounds.corner_max());
    print!("{} ", bounds.square_extent().to_bits());
}
fn compact3_typed<T: rustycad_tkmath::bnd::BoxReal>(v: &[f64]) {
    use rustycad_tkmath::{
        bnd::CompactBox3,
        gp::{Axis1, Axis3, Direction3, Point3, Transform3, Xyz},
    };
    let mut bounds = CompactBox3::<T>::new(Xyz::new(v[0], v[1], v[2]), Xyz::new(v[3], v[4], v[5]));
    let other = CompactBox3::<T>::new(Xyz::new(v[6], v[7], v[8]), Xyz::new(v[9], v[10], v[11]));
    compact3_values(&bounds);
    let point = Xyz::new(v[12], v[13], v[14]);
    let direction = Direction3::new(v[6], v[7], v[8]).unwrap();
    let axis = Axis1::new(Point3::from(point), direction);
    for value in [
        bounds.is_out_point(point),
        bounds.is_out_box(&other),
        bounds.is_in_box(&other),
        bounds.is_out_sphere(point, v[15], false),
        bounds.is_out_sphere(point, v[15], true),
        bounds.is_out_line(axis, false, v[17]),
        bounds.is_out_line(axis, true, v[17]),
        bounds.is_out_plane(Axis3::from_normal(Point3::from(point), direction).unwrap()),
    ] {
        print!("{} ", i32::from(value));
    }
    let mut limited = bounds;
    print!("{} ", i32::from(limited.limit(&other)));
    compact3_values(&limited);
    bounds.add_xyz(point);
    bounds.add_box(&other);
    bounds.enlarge(v[15]);
    compact3_values(&bounds);
    compact3_values(&bounds.transformed(Transform3::rotation(axis, v[16]).unwrap()));
}
pub(crate) fn compact3_case(v: &[f64]) {
    compact3_typed::<f64>(v);
    compact3_typed::<f32>(v);
    println!();
}
