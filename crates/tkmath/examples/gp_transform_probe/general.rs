//! gp_GTrsf requests.
use super::{mat, mat4, read_mat, read_transform, trsf_state};
use crate::elementary_probe::{Fields, attempt, end, flag, real, xyz};
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, GeneralTransform3, Matrix3, Xyz};

fn read_general(f: &mut Fields) -> GeneralTransform3 {
    let kind = f.int();
    let t = read_transform(f);
    let (m, v) = (read_mat(f), f.xyz());
    let (axis_point, axis_direction, ratio) = (f.point(), f.xyz(), f.real());
    let (row, col, value) = (f.int(), f.int(), f.real());
    let mut g = GeneralTransform3::from(t);
    match kind {
        1 => g = GeneralTransform3::new(m, v),
        2 => attempt(|| {
            g.set_affinity_axis(
                Axis1::new(axis_point, Direction3::from_xyz(axis_direction)?),
                ratio,
            );
            Ok(())
        }),
        3 => attempt(|| {
            let frame = Axis2::new(axis_point, Direction3::from_xyz(axis_direction)?)?;
            g.set_affinity_plane(frame, ratio);
            Ok(())
        }),
        4 => attempt(|| g.set_value(row, col, value)),
        5 => g.set_translation_part(v),
        6 => g.set_vectorial_part(m),
        7 => g.set_mat4(&homogeneous(m, v, ratio)),
        8 => {
            g = GeneralTransform3::new(m, v);
            attempt(|| g.set_form());
        }
        9 => attempt(|| g.set_form()),
        _ => {}
    }
    g
}
/// The native probe's `NCollection_Mat4<double>` with the bottom row's first entry set to
/// the ratio, which SetMat4 must ignore.
fn homogeneous(m: Matrix3, v: Xyz, ratio: f64) -> Mat4<f64> {
    let mut m4 = Mat4::<f64>::default();
    for i in 0..3 {
        for j in 0..3 {
            m4.set_value(i, j, m.coefficients()[i][j]);
        }
        m4.set_value(i, 3, v.coordinates()[i]);
    }
    m4.set_value(3, 0, ratio);
    m4
}
fn gtrsf_state(g: GeneralTransform3) {
    print!("{} ", g.form() as i32);
    flag(g.is_negative());
    flag(g.is_singular());
    for row in 1..=3 {
        for col in 1..=4 {
            real(g.value(row, col).unwrap());
        }
    }
    mat(g.vectorial_part());
    xyz(g.translation_part());
    attempt(|| g.trsf().map(trsf_state));
    let mut m4 = Mat4::<f64>::default();
    g.get_mat4(&mut m4);
    mat4(m4);
    let mut dump = DumpStream::default();
    g.dump_json(&mut dump, -1).unwrap();
    print!("{} | ", dump.text());
}
pub fn gp3_gtrsf_case(values: &[f64]) {
    let mut f = Fields::new(values);
    let (a, b) = (read_general(&mut f), read_general(&mut f));
    let (p, n) = (f.xyz(), f.int());
    [a, b, a.multiplied(b), a * b]
        .into_iter()
        .for_each(gtrsf_state);
    let mut c = a;
    c.pre_multiply(b);
    gtrsf_state(c);
    c = a;
    c *= b;
    gtrsf_state(c);
    attempt(|| a.inverted().map(gtrsf_state));
    attempt(|| a.powered(n).map(gtrsf_state));
    xyz(a.apply(p));
    let (x, y, z) = a.transforms_coordinates(p.x(), p.y(), p.z());
    [x, y, z].into_iter().for_each(real);
    end();
}
