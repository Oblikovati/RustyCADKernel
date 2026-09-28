//! Bnd_B2/Bnd_B3 array constructors, SetCenter/SetHSize overloads, Center/HSize, Add and
//! Clear; twin of `compact_arrays_case` in tools/oracle/tkmath_bnd_probe.cxx.
use super::Reader;
use rustycad_tkmath::bnd::{BoxReal, CompactBox2, CompactBox3};
use rustycad_tkmath::gp::{Point2, Point3, Xy, Xyz};

fn state<T: BoxReal, const N: usize>(void: bool, center: &[T; N], half: &[T; N], extent: f64) {
    print!("{} ", i32::from(void));
    for (c, h) in center.iter().zip(half) {
        print!("{} {} ", c.to_double().to_bits(), h.to_double().to_bits());
    }
    print!("{} ", extent.to_bits());
}
fn state3<T: BoxReal>(b: &CompactBox3<T>) {
    state(b.is_void(), b.center(), b.half_size(), b.square_extent());
}
fn state2<T: BoxReal>(b: &CompactBox2<T>) {
    state(b.is_void(), b.center(), b.half_size(), b.square_extent());
}
fn arrays<T: BoxReal, const N: usize>(v: &[f64]) -> ([T; N], [T; N]) {
    (
        std::array::from_fn(|i| T::from_double(v[i])),
        std::array::from_fn(|i| T::from_double(v[3 + i])),
    )
}
fn typed3<T: BoxReal>(v: &[f64]) {
    let (center, half) = arrays::<T, 3>(v);
    let (mut a, mut b) = (
        CompactBox3::from_arrays(center, half),
        CompactBox3::<T>::default(),
    );
    let other = CompactBox3::<T>::new(Xyz::new(v[9], v[10], v[11]), Xyz::new(v[12], v[13], v[14]));
    state3(&b);
    state3(&a);
    b.set_center_array(center);
    b.set_half_size_array(half);
    state3(&b);
    b.set_center(Xyz::new(v[6], v[7], v[8]));
    b.set_half_size(Xyz::new(v[12], v[13], v[14]));
    state3(&b);
    a.add_point(Point3::new(v[6], v[7], v[8]));
    state3(&a);
    a.add_box(&other);
    state3(&a);
    a.clear();
    state3(&a);
}
fn typed2<T: BoxReal>(v: &[f64]) {
    let (center, half) = arrays::<T, 2>(v);
    let (mut p, mut q) = (
        CompactBox2::from_arrays(center, half),
        CompactBox2::<T>::default(),
    );
    let other = CompactBox2::<T>::new(Xy::new(v[9], v[10]), Xy::new(v[12], v[13]));
    state2(&q);
    state2(&p);
    q.set_center_array(center);
    q.set_half_size_array(half);
    state2(&q);
    q.set_center(Xy::new(v[6], v[7]));
    q.set_half_size(Xy::new(v[12], v[13]));
    state2(&q);
    p.add_point(Point2::new(v[6], v[7]));
    state2(&p);
    p.add_box(&other);
    state2(&p);
    p.clear();
    state2(&p);
}

/// `compact_arrays center(3) half(3) point(3) other_center(3) other_half(3)`
pub(crate) fn compact_arrays_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let v: Vec<f64> = (0..15).map(|_| reader.real()).collect();
    typed3::<f64>(&v);
    typed2::<f64>(&v);
    typed3::<f32>(&v);
    typed2::<f32>(&v);
    println!();
}
