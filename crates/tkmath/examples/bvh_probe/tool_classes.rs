//! Rust counterpart of tkmath_bvh_tool_classes_probe.cxx: the `BVH` namespace tool classes.
use super::{scalar_out, vector_in, vector_out};
use rustycad_tkernel::ncollection::{Mat4, MatrixScalar, Vec2, Vec3, Vec4};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::bvh::{
    BvhBox, BvhScalar, BvhVector, cwise_std_max, cwise_std_min, main_axis, matrix_op_inverse,
    matrix_op_multiply, surface_area, to_bnd_box_scalar,
};

fn bnd_out(bounding: &BoundingBox3) {
    for value in bounding.get().unwrap() {
        print!("{} ", value.to_bits());
    }
}

/// `bvh_tool_dimension<T,N>`: every per-dimension tool class on the pair `a`, `b`.
fn dimension_run<V: BvhVector>(a: &[f64], b: &[f64]) {
    let (p, q): (V, V) = (vector_in(a), vector_in(b));
    bnd_out(&V::to_bnd_box(&p, &q));
    for axis in -1..=4 {
        scalar_out(p.component(axis));
    }
    vector_out(&V::unit_x());
    vector_out(&V::unit_y());
    vector_out(&V::unit_z());
    scalar_out(surface_area(&p));
    print!("{} ", main_axis(&p));
    let bounding = BvhBox::from_corners(p, q);
    for axis in 0..V::DIMENSION as i32 {
        scalar_out(bounding.center_axis(axis));
    }
    let (mut low, mut high) = (p, p);
    cwise_std_min(&mut low, &q);
    cwise_std_max(&mut high, &q);
    vector_out(&low);
    vector_out(&high);
}

fn precision_run<T>(v: &[f64])
where
    T: BvhScalar + MatrixScalar,
    Vec2<T>: BvhVector<Scalar = T>,
    Vec3<T>: BvhVector<Scalar = T>,
    Vec4<T>: BvhVector<Scalar = T>,
{
    bnd_out(&to_bnd_box_scalar(
        T::from_double(v[0]),
        T::from_double(v[4]),
    ));
    dimension_run::<Vec2<T>>(v, &v[4..]);
    dimension_run::<Vec3<T>>(v, &v[4..]);
    dimension_run::<Vec4<T>>(v, &v[4..]);
    let matrix = Mat4::from_data(std::array::from_fn(|index| T::from_double(v[8 + index])));
    let mut inverse = Mat4::default();
    matrix_op_inverse(&matrix, &mut inverse);
    for value in inverse.data() {
        scalar_out(value);
    }
    vector_out(&matrix_op_multiply(&matrix, &vector_in::<Vec4<T>>(v)));
    println!();
}

/// `bvh_tool_classes precision a(4) b(4) matrix(16)`.
pub fn bvh_tool_classes_case(v: &[f64]) {
    if v[0] == 0.0 {
        precision_run::<f64>(&v[1..]);
    } else {
        precision_run::<f32>(&v[1..]);
    }
}
