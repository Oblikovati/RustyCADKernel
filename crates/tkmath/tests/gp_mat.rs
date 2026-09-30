//! OCC22595: the default matrix initializes every coefficient.
#![allow(non_snake_case)]
use rustycad_tkmath::gp::Matrix3;
#[test]
fn gp_MatTest__OCC22595_DefaultConstructor() {
    let matrix = Matrix3::default();
    for row in 1..=3 {
        for column in 1..=3 {
            assert_eq!(0.0, matrix.value(row, column).unwrap());
        }
    }
}
