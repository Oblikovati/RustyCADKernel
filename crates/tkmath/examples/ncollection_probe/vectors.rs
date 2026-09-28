//! Vector requests for float, double and int elements.
use super::{JsonDump, Printed, flag, vec2, vec3, vec4};
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};

/// `ncv_common`: operators, extrema, dot and equality, in native print order.
macro_rules! common {
    ($a:expr, $b:expr, $s:expr, $out:ident) => {{
        let (a, b, s) = ($a, $b, $s);
        [
            a + b,
            a - b,
            -a,
            a * b,
            a * s,
            a.multiplied(s),
            a / b,
            a / s,
        ]
        .into_iter()
        .for_each($out);
        let mut c = a;
        c += b;
        $out(c);
        c = a;
        c -= b;
        $out(c);
        c = a;
        c *= b;
        $out(c);
        c = a;
        c *= s;
        $out(c);
        c = a;
        c.multiply(s);
        $out(c);
        c = a;
        c /= s;
        $out(c);
        c = a;
        c /= b;
        $out(c);
        [a.cwise_min(&b), a.cwise_max(&b), a.cwise_abs()]
            .into_iter()
            .for_each($out);
        [a.max_comp(), a.min_comp(), a.dot(&b)]
            .into_iter()
            .for_each(Printed::print);
        [a.is_equal(&b), a == b, a != b, a.is_equal(&a)]
            .into_iter()
            .for_each(flag);
    }};
}
macro_rules! typed_vectors {
    ($name:ident, $t:ty) => {
        fn $name(raw: &[f64]) {
            let p: [$t; 4] = std::array::from_fn(|i| raw[i] as $t);
            let q: [$t; 4] = std::array::from_fn(|i| raw[4 + i] as $t);
            let (s, t) = (raw[8] as $t, raw[9] as $t);
            planar(p, q, s);
            spatial(p, q, s, t);
            homogeneous(p, q, s);
            println!();
        }
        paste_vectors!($t);
    };
}
/// The per-dimension request bodies, generated once per element type in a local module.
macro_rules! paste_vectors {
    ($t:ty) => {
        fn planar(p: [$t; 4], q: [$t; 4], s: $t) {
            let (a, b) = (Vec2::new(p[0], p[1]), Vec2::new(q[0], q[1]));
            common!(a, b, s, vec2);
            [a.modulus(), a.square_modulus()]
                .into_iter()
                .for_each(Printed::print);
            [
                a.xy(),
                a.yx(),
                Vec2::from_value(s),
                Vec2::<$t>::dx(),
                Vec2::<$t>::dy(),
            ]
            .into_iter()
            .for_each(vec2);
            let mut c = Vec2::default();
            c.set_values(q[1], q[0]);
            vec2(c);
            vec2(Vec2::<f32>::convert_from(a));
            vec2(Vec2::<f64>::convert_from(a));
            vec2(Vec2::<i32>::convert_from(a));
            a.dump();
        }
        fn spatial(p: [$t; 4], q: [$t; 4], s: $t, t: $t) {
            let (a, b) = (Vec3::new(p[0], p[1], p[2]), Vec3::new(q[0], q[1], q[2]));
            common!(a, b, s, vec3);
            [a.modulus(), a.square_modulus()]
                .into_iter()
                .for_each(Printed::print);
            vec3(a.normalized());
            let mut c = a;
            c.normalize();
            vec3(c);
            vec3(Vec3::cross(&a, &b));
            vec3(Vec3::lerp(&a, &b, t));
            [a.xy(), a.yx(), a.xz(), a.zx(), a.yz(), a.zy()]
                .into_iter()
                .for_each(vec2);
            [a.xyz(), a.xzy(), a.yxz(), a.yzx(), a.zyx(), a.zxy()]
                .into_iter()
                .for_each(vec3);
            [a.r(), a.g(), a.b()].into_iter().for_each(Printed::print);
            let planar = Vec2::new(q[0], q[1]);
            [
                Vec3::from_value(s),
                Vec3::from_vec2(planar, Default::default()),
                Vec3::from_vec2(planar, s),
            ]
            .into_iter()
            .for_each(vec3);
            [Vec3::<$t>::dx(), Vec3::<$t>::dy(), Vec3::<$t>::dz()]
                .into_iter()
                .for_each(vec3);
            c.set_values(q[2], q[1], q[0]);
            vec3(c);
            c.set_values_vec2(Vec2::new(p[1], p[0]), s);
            vec3(c);
            vec3(Vec3::<f32>::convert_from(a));
            vec3(Vec3::<f64>::convert_from(a));
            vec3(Vec3::<i32>::convert_from(a));
            a.dump();
        }
        fn homogeneous(p: [$t; 4], q: [$t; 4], s: $t) {
            let (a, b) = (
                Vec4::new(p[0], p[1], p[2], p[3]),
                Vec4::new(q[0], q[1], q[2], q[3]),
            );
            common!(a, b, s, vec4);
            [
                a.xy(),
                a.yx(),
                a.xz(),
                a.zx(),
                a.xw(),
                a.wx(),
                a.yz(),
                a.zy(),
                a.yw(),
                a.wy(),
                a.zw(),
                a.wz(),
            ]
            .into_iter()
            .for_each(vec2);
            [
                a.xyz(),
                a.xzy(),
                a.yxz(),
                a.yzx(),
                a.zyx(),
                a.zxy(),
                a.xyw(),
                a.xwy(),
                a.yxw(),
                a.ywx(),
                a.wyx(),
                a.wxy(),
                a.xzw(),
                a.xwz(),
                a.zxw(),
                a.zwx(),
                a.wzx(),
                a.wxz(),
                a.yzw(),
                a.ywz(),
                a.zyw(),
                a.zwy(),
                a.wzy(),
                a.wyz(),
                a.rgb(),
                a.rbg(),
                a.grb(),
                a.gbr(),
                a.bgr(),
                a.brg(),
            ]
            .into_iter()
            .for_each(vec3);
            [a.r(), a.g(), a.b(), a.a()]
                .into_iter()
                .for_each(Printed::print);
            let spatial = Vec3::new(q[0], q[1], q[2]);
            [
                Vec4::from_value(s),
                Vec4::from_vec2(Vec2::new(q[0], q[1])),
                Vec4::from_vec3(spatial, Default::default()),
                Vec4::from_vec3(spatial, s),
            ]
            .into_iter()
            .for_each(vec4);
            let mut c = Vec4::default();
            c.set_values(q[3], q[2], q[1], q[0]);
            vec4(c);
            c.set_values_vec3(Vec3::new(p[2], p[1], p[0]), s);
            vec4(c);
            vec4(Vec4::<f32>::convert_from(a));
            vec4(Vec4::<f64>::convert_from(a));
            vec4(Vec4::<i32>::convert_from(a));
            a.dump();
        }
    };
}
mod float_kind {
    use super::*;
    typed_vectors!(run, f32);
    pub fn request(raw: &[f64]) {
        run(raw);
    }
}
mod double_kind {
    use super::*;
    typed_vectors!(run, f64);
    pub fn request(raw: &[f64]) {
        run(raw);
    }
}
mod int_kind {
    use super::*;
    typed_vectors!(run, i32);
    pub fn request(raw: &[f64]) {
        run(raw);
    }
}
pub fn ncollection_vec_case(values: &[f64]) {
    let raw: &[f64] = &values[1..];
    match values[0] as i32 {
        0 => float_kind::request(raw),
        1 => double_kind::request(raw),
        _ => int_kind::request(raw),
    }
}
