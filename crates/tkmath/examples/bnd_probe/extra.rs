//! Remaining Bnd_Box/Box2d members, Bnd_Tools and the transformed Bnd_B2/Bnd_B3 relations.
use super::{Reader, box_read, failure};
use rustycad_tkmath::bnd::{
    BoundingBox2, BoundingBox3, BoxLimits2, BoxLimits3, BoxReal, CompactBox2, CompactBox3,
    box2_to_bvh, box3_to_bvh,
};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Direction2, Direction3, Point2, Point3, Transform2, Transform3, Vector2, Vector3,
    Xy, Xyz,
};

fn bits(values: &[f64]) {
    for value in values {
        print!("{} ", value.to_bits());
    }
}
fn dump_line(text: Vec<u8>) {
    print!("{} ", String::from_utf8(text).unwrap().replace('\n', "|"));
}
fn box3_extra(bounds: &BoundingBox3) {
    let BoxLimits3 {
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    } = bounds.limits();
    bits(&[x_min, x_max, y_min, y_max, z_min, z_max]);
    bits(&[
        bounds.x_min(),
        bounds.x_max(),
        bounds.y_min(),
        bounds.y_max(),
        bounds.z_min(),
        bounds.z_max(),
    ]);
    let mut text = Vec::new();
    bounds.dump(&mut text).unwrap();
    dump_line(text);
    match box3_to_bvh(bounds) {
        Ok(converted) => {
            let (low, high) = (converted.corner_min(), converted.corner_max());
            bits(&[low.x, low.y, low.z, high.x, high.y, high.z]);
        }
        Err(error) => failure(&error),
    }
}

/// `box_extra box(6+gap+flags) point(3)`
pub(crate) fn box_extra_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let mut bounds = box_read(&mut reader);
    let point = reader.triple();
    box3_extra(&bounds);
    bounds.update_point(point.x(), point.y(), point.z());
    box3_extra(&bounds);
    println!();
}

fn box2_read(reader: &mut Reader<'_>) -> BoundingBox2 {
    let (x, y, u, v) = (reader.real(), reader.real(), reader.real(), reader.real());
    let (gap, flags) = (reader.real(), reader.integer());
    let mut bounds = BoundingBox2::default();
    if flags & 16 == 0 {
        bounds.update_bounds([x, y], [u, v]);
    }
    bounds.set_gap(gap);
    let openers: [fn(&mut BoundingBox2); 4] = [
        BoundingBox2::open_xmin,
        BoundingBox2::open_xmax,
        BoundingBox2::open_ymin,
        BoundingBox2::open_ymax,
    ];
    for (bit, open) in openers.iter().enumerate() {
        if flags & (1 << bit) != 0 {
            open(&mut bounds);
        }
    }
    bounds
}
fn box2_extra(bounds: &BoundingBox2) {
    let BoxLimits2 {
        x_min,
        x_max,
        y_min,
        y_max,
    } = bounds.limits();
    print!("{} ", i32::from(bounds.is_void()));
    bits(&[x_min, x_max, y_min, y_max]);
    bits(&[
        bounds.x_min(),
        bounds.x_max(),
        bounds.y_min(),
        bounds.y_max(),
    ]);
    let mut text = Vec::new();
    bounds.dump(&mut text).unwrap();
    dump_line(text);
    match box2_to_bvh(bounds) {
        Ok(converted) => {
            let (low, high) = (converted.corner_min(), converted.corner_max());
            bits(&[low.x, low.y, high.x, high.y]);
        }
        Err(error) => failure(&error),
    }
}
fn trsf2_read(reader: &mut Reader<'_>) -> Transform2 {
    let kind = reader.integer();
    let (a, b, c) = (reader.real(), reader.real(), reader.real());
    let mut transform = Transform2::default();
    match kind {
        1 => transform.set_translation(Vector2::new(a, b)),
        2 => transform.set_rotation(Point2::new(a, b), c),
        3 => transform.set_scale(Point2::new(a, b), c),
        4 => transform.set_mirror_point(Point2::new(a, b)),
        5 => {
            let mut scale = Transform2::default();
            scale.set_scale(Point2::new(b, a), c);
            transform.set_rotation(Point2::new(a, b), c);
            transform *= scale;
        }
        _ => {}
    }
    transform
}

/// `box2_extra box(4+gap+flags) other(4+gap+flags) point(2) dir(2) trsf(4) trsf(4)`
pub(crate) fn box2_extra_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let (bounds, other) = (box2_read(&mut reader), box2_read(&mut reader));
    let point = Point2::new(reader.real(), reader.real());
    let direction = Direction2::new(reader.real(), reader.real()).unwrap();
    let (first, second) = (trsf2_read(&mut reader), trsf2_read(&mut reader));
    box2_extra(&bounds);
    print!(
        "{}{} ",
        i32::from(bounds.is_out_transformed(&other, first)),
        i32::from(bounds.is_out_both(first, &other, second))
    );
    let mut ray = bounds.clone();
    ray.update_point(point.x(), point.y());
    box2_extra(&ray);
    ray.set_ray(point, direction);
    box2_extra(&ray);
    ray = bounds.clone();
    ray.add_ray(point, direction);
    box2_extra(&ray);
    println!();
}

fn trsf3_read(reader: &mut Reader<'_>) -> Transform3 {
    let kind = reader.integer();
    let (a, d, c) = (reader.triple(), reader.triple(), reader.real());
    let mut transform = Transform3::default();
    let axis = || Axis1::new(Point3::from(a), Direction3::from_xyz(d).unwrap());
    match kind {
        1 => transform.set_translation(Vector3::from(a)),
        2 => transform.set_rotation(axis(), c).unwrap(),
        3 => transform.set_scale(Point3::from(a), c).unwrap(),
        4 => transform.set_mirror_point(Point3::from(a)),
        5 => {
            let mut scale = Transform3::default();
            scale.set_scale(Point3::from(d), c).unwrap();
            transform.set_rotation(axis(), c).unwrap();
            transform *= scale;
        }
        6 => {
            let frame = Axis2::new(Point3::from(a), Direction3::from_xyz(d).unwrap()).unwrap();
            transform.set_mirror_plane(frame);
        }
        _ => {}
    }
    transform
}
fn compact_relations<T: BoxReal>(v: &[f64], t3: Transform3, t2: Transform2) {
    let a = CompactBox3::<T>::new(Xyz::new(v[0], v[1], v[2]), Xyz::new(v[3], v[4], v[5]));
    let b = CompactBox3::<T>::new(Xyz::new(v[6], v[7], v[8]), Xyz::new(v[9], v[10], v[11]));
    for value in [
        a.is_out_transformed_box(&b, t3),
        b.is_out_transformed_box(&a, t3),
        a.is_in_transformed_box(&b, t3),
        b.is_in_transformed_box(&a, t3),
    ] {
        print!("{}", i32::from(value));
    }
    print!(" ");
    let c = CompactBox2::<T>::new(Xy::new(v[0], v[1]), Xy::new(v[3], v[4]));
    let d = CompactBox2::<T>::new(Xy::new(v[6], v[7]), Xy::new(v[9], v[10]));
    for value in [
        c.is_out_transformed_box(&d, t2),
        d.is_out_transformed_box(&c, t2),
        c.is_in_transformed_box(&d, t2),
        d.is_in_transformed_box(&c, t2),
    ] {
        print!("{}", i32::from(value));
    }
    print!(" ");
}

/// `compact_trsf a(6) b(6) trsf3(8) trsf2(4)`
pub(crate) fn compact_trsf_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let boxes: Vec<f64> = (0..12).map(|_| reader.real()).collect();
    let (t3, t2) = (trsf3_read(&mut reader), trsf2_read(&mut reader));
    compact_relations::<f64>(&boxes, t3, t2);
    compact_relations::<f32>(&boxes, t3, t2);
    println!();
}
