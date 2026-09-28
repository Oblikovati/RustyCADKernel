//! Rust counterpart of tkmath_bvh_probe.cxx: BVH boxes, tools, sorters, builds and transforms.
use rustycad_tkernel::ncollection::{Mat4, MatrixScalar, Vec4};
use rustycad_tkmath::bvh::{BvhBox, BvhScalar, BvhTransform, BvhVector, int_floor};
mod box_queries;
mod builders;
mod distances;
mod field;
mod selectors;
mod sort_build;
mod tool_classes;
mod traversal;
pub use box_queries::bvh_box_case;
pub use builders::bvh_builders_case;
pub use field::{bvh_field_case, bvh_nearest_case};
pub use sort_build::{bvh_build_case, bvh_sort_case};
pub use tool_classes::bvh_tool_classes_case;
pub use traversal::bvh_traverse_case;

/// Prints a scalar as the bits of its exact double promotion, like `bits(double(v))`.
fn scalar_out<T: BvhScalar>(value: T) {
    print!("{} ", value.to_double().to_bits());
}
fn vector_out<V: BvhVector>(value: &V) {
    for axis in 0..V::DIMENSION {
        scalar_out(value[axis]);
    }
}
fn box_out<V: BvhVector>(bounding: &BvhBox<V>) {
    vector_out(bounding.corner_min());
    vector_out(bounding.corner_max());
}
/// `bvh_vec<T,N>`: the first N doubles narrowed to T; other components stay zero.
fn vector_in<V: BvhVector>(values: &[f64]) -> V {
    let mut result = V::default();
    for axis in 0..V::DIMENSION {
        result[axis] = V::Scalar::from_double(values[axis]);
    }
    result
}
fn flag(value: bool) -> i32 {
    i32::from(value)
}

/// Selects the vector type as the native `dimension`/`precision` branches do.
macro_rules! dispatch_vector {
    ($dimension:expr, $precision:expr, $run:ident, $($argument:expr),*) => {
        match ($dimension as i32, $precision as i32) {
            (2, 0) => $run::<rustycad_tkernel::ncollection::Vec2<f64>>($($argument),*),
            (3, 0) => $run::<rustycad_tkernel::ncollection::Vec3<f64>>($($argument),*),
            (_, 0) => $run::<rustycad_tkernel::ncollection::Vec4<f64>>($($argument),*),
            (2, _) => $run::<rustycad_tkernel::ncollection::Vec2<f32>>($($argument),*),
            (3, _) => $run::<rustycad_tkernel::ncollection::Vec3<f32>>($($argument),*),
            _ => $run::<rustycad_tkernel::ncollection::Vec4<f32>>($($argument),*),
        }
    };
}
pub(crate) use dispatch_vector;

pub fn bvh_int_floor_case(v: &[f64]) {
    let double = v[0] / v[1];
    let single = (v[0] as f32) / (v[1] as f32);
    println!("{} {}", int_floor(double), int_floor(single));
}

fn matrix_out<T: BvhScalar>(matrix: &Mat4<T>) {
    for value in matrix.data() {
        scalar_out(value);
    }
}
fn transform_run<T>(v: &[f64])
where
    T: BvhScalar + MatrixScalar,
    Vec4<T>: BvhVector<Scalar = T>,
{
    let matrix = Mat4::from_data(std::array::from_fn(|index| T::from_double(v[index])));
    let mut inverse = Mat4::default();
    let mut determinant = T::from_double(5.0);
    print!(
        "{} ",
        flag(matrix.inverted_into(&mut inverse, &mut determinant))
    );
    scalar_out(determinant);
    matrix_out(&inverse);
    matrix_out(BvhTransform::new(matrix).inversed());
    let mut assigned = BvhTransform::default();
    assigned.set_transform(matrix);
    matrix_out(assigned.inversed());
    let bounding = BvhBox::from_corners(vector_in(&v[16..]), vector_in(&v[20..]));
    box_out(&assigned.apply(&bounding));
    println!();
}
pub fn bvh_transform_case(v: &[f64]) {
    if v[0] == 0.0 {
        transform_run::<f64>(&v[1..]);
    } else {
        transform_run::<f32>(&v[1..]);
    }
}
