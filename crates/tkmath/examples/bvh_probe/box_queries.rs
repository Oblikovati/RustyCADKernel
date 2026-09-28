//! Box, ray, tool, transform and JSON queries of the `bvh_box` request.
use super::{box_out, dispatch_vector, flag, scalar_out, vector_in, vector_out};
use rustycad_tkernel::ncollection::{Mat4, Vec2, Vec3, Vec4};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::bvh::tools::{
    PrjStateInTriangle, TriangleProjectionState, box_box_square_distance, point_box_projection,
    point_box_square_distance, point_triangle_projection, point_triangle_square_distance,
    ray_box_intersection, ray_box_intersection_by_direction,
};
use rustycad_tkmath::bvh::{BvhBox, BvhRay, BvhScalar, BvhVector};

/// The `if constexpr(N==3)` transform block; other dimensions print nothing.
trait ProbeTransform: BvhVector {
    fn transform_out(_bounding: &BvhBox<Self>, _matrix: &[f64]) {}
}
impl ProbeTransform for Vec2<f64> {}
impl ProbeTransform for Vec2<f32> {}
impl ProbeTransform for Vec4<f64> {}
impl ProbeTransform for Vec4<f32> {}
macro_rules! spatial_transform {
    ($scalar:ty) => {
        impl ProbeTransform for Vec3<$scalar> {
            fn transform_out(bounding: &BvhBox<Self>, matrix: &[f64]) {
                let matrix = Mat4::from_data(std::array::from_fn(|index| {
                    <$scalar>::from_double(matrix[index])
                }));
                box_out(&bounding.transformed(&matrix));
                let mut moved = *bounding;
                moved.transform(&matrix);
                box_out(&moved);
            }
        }
    };
}
spatial_transform!(f64);
spatial_transform!(f32);

fn box_queries<V: BvhVector>(a: &BvhBox<V>, b: &BvhBox<V>, point: &V) {
    print!("{} ", flag(a.is_valid()));
    scalar_out(a.area());
    vector_out(&a.size());
    vector_out(&a.center());
    for axis in 0..V::DIMENSION as i32 {
        scalar_out(a.center_axis(axis));
    }
    print!("{} ", flag(a.is_out(b)));
    print!(
        "{} ",
        flag(a.is_out_corners(b.corner_min(), b.corner_max()))
    );
    let (inside, overlap) = a.contains(b);
    print!("{} {} ", flag(inside), flag(overlap));
    let (inside, overlap) = a.contains_corners(b.corner_min(), b.corner_max());
    print!(
        "{} {} {} ",
        flag(inside),
        flag(overlap),
        flag(a.is_out_point(point))
    );
    let mut combined = *a;
    combined.combine(b);
    box_out(&combined);
    let mut added = *a;
    added.add(point);
    box_out(&added);
}

fn hit_out<T: BvhScalar>(hit: Option<(T, T)>) {
    let (enter, leave) = hit.unwrap_or((T::from_double(91.0), T::from_double(92.0)));
    print!("{} ", flag(hit.is_some()));
    scalar_out(enter);
    scalar_out(leave);
}

fn tool_queries<V: BvhVector>(a: &BvhBox<V>, b: &BvhBox<V>, v: &[f64]) {
    let (point, direction): (V, V) = (vector_in(&v[17..]), vector_in(&v[21..]));
    scalar_out(point_box_square_distance(&point, a));
    scalar_out(box_box_square_distance(a, b));
    vector_out(&point_box_projection(&point, a));
    let ray = BvhRay::new(point, direction);
    vector_out(&ray.inv_direct);
    hit_out(ray_box_intersection(&ray, a));
    hit_out(ray_box_intersection_by_direction(&point, &direction, a));
    let nodes: [V; 3] = [
        vector_in(&v[25..]),
        vector_in(&v[29..]),
        vector_in(&v[33..]),
    ];
    let mut state = TriangleProjectionState {
        state: PrjStateInTriangle::Edge,
        first_node: -7,
        last_node: -7,
    };
    let projection =
        point_triangle_projection(&point, &nodes[0], &nodes[1], &nodes[2], Some(&mut state));
    vector_out(&projection);
    print!(
        "{} {} {} ",
        state.state as i32, state.first_node, state.last_node
    );
    scalar_out(point_triangle_square_distance(
        &point, &nodes[0], &nodes[1], &nodes[2],
    ));
}

/// Dumps `a`, re-reads a prefix of the dump into a box preset to 7s, and prints
/// the outcome (or the exception type), the parsed corners and the dump text.
fn json_queries<V: BvhVector>(a: &BvhBox<V>, cut: i32) {
    let mut dump = DumpStream::default();
    a.dump_json(&mut dump).unwrap();
    let text = dump.text();
    let mut parsed = BvhBox::from_point(vector_in::<V>(&[7.0; 4]));
    let prefix = if cut < 0 || cut as usize >= text.len() {
        text
    } else {
        &text[..cut as usize]
    };
    let mut position = 1;
    match parsed.init_from_json(prefix, &mut position) {
        Ok(ok) => print!("{} {} ", flag(ok), position),
        Err(failure) => print!("{} ", failure.exception_type()),
    }
    box_out(&parsed);
    print!("{text}");
}

fn box_run<V: ProbeTransform>(v: &[f64]) {
    let a = if v[0] != 0.0 {
        BvhBox::default()
    } else {
        BvhBox::from_corners(vector_in::<V>(&v[1..]), vector_in(&v[5..]))
    };
    let b = BvhBox::from_corners(vector_in::<V>(&v[9..]), vector_in(&v[13..]));
    box_queries(&a, &b, &vector_in(&v[17..]));
    tool_queries(&a, &b, v);
    V::transform_out(&a, &v[37..]);
    json_queries(&a, v[53] as i32);
    println!();
}

pub fn bvh_box_case(v: &[f64]) {
    dispatch_vector!(v[0], v[1], box_run, &v[2..]);
}
