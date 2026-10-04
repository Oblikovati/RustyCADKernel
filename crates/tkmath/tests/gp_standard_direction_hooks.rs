//! Native `gp_Ax2` protected helper behavior from tools/oracle.

use rustycad_tkmath::gp::StandardDirection3::{self as D, *};

const AXES: [D; 6] = [X, Y, Z, NegX, NegY, NegZ];
const PERPENDICULAR_X: [D; 6] = [Y, Z, X, Y, Z, X];
const PERPENDICULAR_Y: [D; 6] = [Z, X, Y, NegZ, NegX, NegY];
const CROSS: [[D; 6]; 6] = [
    [Z, Z, NegY, Z, NegZ, Y],
    [NegZ, X, X, Z, X, NegX],
    [Y, NegX, Y, NegY, X, Y],
    [NegZ, NegZ, Y, NegZ, Z, NegY],
    [Z, NegX, NegX, NegZ, NegX, X],
    [NegY, X, NegY, Y, NegX, NegY],
];

#[test]
fn gp_ax2_perpendicular_hooks_match_all_native_axes() {
    for (index, axis) in AXES.into_iter().enumerate() {
        assert_eq!(axis.perpendicular_x(), PERPENDICULAR_X[index]);
        assert_eq!(axis.perpendicular_y(), PERPENDICULAR_Y[index]);
    }
}

#[test]
fn gp_ax2_cross_hook_matches_all_native_pairs_and_parallel_fallbacks() {
    for (first, axis) in AXES.into_iter().enumerate() {
        for (second, other) in AXES.into_iter().enumerate() {
            assert_eq!(axis.cross(other), CROSS[first][second]);
        }
    }
}
