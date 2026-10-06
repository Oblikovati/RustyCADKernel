use super::super::{CustomZero, Vec2, Vec3, Vec4};
use std::fmt::Write;
fn mask<const N: usize>(components: [bool; N]) -> usize {
    components
        .into_iter()
        .enumerate()
        .fold(0, |bits, (index, value)| {
            bits | (usize::from(value) << index)
        })
}
macro_rules! truth_rows {
    ($name:ident, $vector:ident, $count:literal) => {
        fn $name(output: &mut String) {
            for a in 0..(1 << $count) {
                for b in 0..(1 << $count) {
                    let left = $vector::from_array(std::array::from_fn(|i| a & (1 << i) != 0));
                    let right = $vector::from_array(std::array::from_fn(|i| b & (1 << i) != 0));
                    let (mut add, mut sub, mut mul, mut div) = (left, left, left, left);
                    let pointers = (&raw mut add, &raw mut sub, &raw mut mul, &raw mut div);
                    let identity = std::ptr::eq(add.add_assign_returning(&right), pointers.0)
                        && std::ptr::eq(sub.sub_assign_returning(&right), pointers.1)
                        && std::ptr::eq(mul.mul_assign_returning(&right), pointers.2)
                        && std::ptr::eq(
                            div.div_assign_returning(&$vector::from_value(true)),
                            pointers.3,
                        );
                    writeln!(
                        output,
                        "truth{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                        $count,
                        a,
                        b,
                        mask((left + right).data()),
                        mask((left - right).data()),
                        mask((left * right).data()),
                        u8::from(left.dot(&right)),
                        mask(add.data()),
                        mask(sub.data()),
                        mask(mul.data()),
                        mask(div.data()),
                        u8::from(identity),
                        mask((-left).data()),
                        mask(left.cwise_abs().data()),
                        mask((left / true).data()),
                        mask((left * false).data()),
                        mask((left * true).data())
                    )
                    .unwrap();
                }
            }
        }
    };
}
truth_rows!(truth2, Vec2, 2);
truth_rows!(truth3, Vec3, 3);
truth_rows!(truth4, Vec4, 4);
fn spatial_rows(output: &mut String) {
    for a in 0..8 {
        let left = Vec3::from_array(std::array::from_fn(|i| a & (1 << i) != 0));
        for b in 0..8 {
            let right = Vec3::from_array(std::array::from_fn(|i| b & (1 << i) != 0));
            writeln!(
                output,
                "space {a} {b} {} {} {} {} {} {}",
                mask(Vec3::cross(&left, &right).data()),
                mask(Vec3::lerp(&left, &right, false).data()),
                mask(Vec3::lerp(&left, &right, true).data()),
                mask(left.normalized().data()),
                u8::from(left.modulus()),
                u8::from(left.square_modulus())
            )
            .unwrap();
        }
    }
}
fn unsigned_rows(output: &mut String) {
    let a = -Vec2::new(0_u8, 1);
    writeln!(output, "neg_u8 {} {}", a.x, a.y).unwrap();
    let b = -Vec2::new(1_u16, 32768);
    writeln!(output, "neg_u16 {} {}", b.x, b.y).unwrap();
    let c = -Vec2::new(1_u32, 0x80000000);
    writeln!(output, "neg_u32 {} {}", c.x, c.y).unwrap();
    let d = -Vec2::new(1_u64, 0x8000000000000000);
    writeln!(output, "neg_u64 {} {}", d.x, d.y).unwrap();
}
fn pair_row(output: &mut String, label: &str, vector: Vec2<bool>) {
    writeln!(
        output,
        "{label} {} {}",
        u8::from(vector.x),
        u8::from(vector.y)
    )
    .unwrap();
}
fn boolean_rows(output: &mut String) {
    let (left, right) = (Vec2::new(true, false), Vec2::new(true, true));
    pair_row(output, "bool_add", left + right);
    pair_row(output, "bool_sub", left - right);
    pair_row(output, "bool_mul", left * right);
    pair_row(output, "bool_div", left / true);
    pair_row(output, "bool_neg", -left);
    pair_row(output, "bool_abs", left.cwise_abs());
    writeln!(
        output,
        "bool_dot {} {} {}",
        u8::from(left.dot(&right)),
        u8::from(left.modulus()),
        u8::from(left.square_modulus())
    )
    .unwrap();
    let mut spatial = Vec3::new(true, false, true);
    spatial.normalize();
    writeln!(
        output,
        "bool_normalize {} {} {}",
        u8::from(spatial.x),
        u8::from(spatial.y),
        u8::from(spatial.z)
    )
    .unwrap();
}
fn constructor_rows(output: &mut String) {
    let pair = Vec2::<CustomZero>::default();
    let triple = Vec3::<CustomZero>::default();
    let quadruple = Vec4::from_vec2(Vec2::new(CustomZero(1), CustomZero(2)));
    writeln!(
        output,
        "custom_zero {} {} {} {} {}",
        pair.x.0, pair.y.0, triple.z.0, quadruple.z.0, quadruple.w.0
    )
    .unwrap();
    let fourth = Vec4::<CustomZero>::default();
    let axis = Vec3::<CustomZero>::dz();
    writeln!(
        output,
        "extra_zero {} {} {} {} {}",
        fourth.x.0,
        fourth.w.0,
        axis.x.0,
        axis.z.0,
        Vec4::from_vec3_zero(axis).w.0
    )
    .unwrap();
}
pub fn rows() -> String {
    let mut output = String::new();
    unsigned_rows(&mut output);
    boolean_rows(&mut output);
    constructor_rows(&mut output);
    truth2(&mut output);
    truth3(&mut output);
    truth4(&mut output);
    spatial_rows(&mut output);
    output
}
