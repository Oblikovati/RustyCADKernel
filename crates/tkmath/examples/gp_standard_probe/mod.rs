//! gp_Dir::D / gp_Dir2d::D constructors and gp statics; prints the same token stream as
//! tools/oracle/tkmath_gp_standard_probe.cxx.
use crate::elementary_probe::{Fields, ax1, ax2, ax3, dir, end, flag, real, xyz};
use crate::gp_planar_probe::{ax, pair};
use rustycad_tkmath::gp::{
    self, Axis1, Axis2, Axis2d, Axis3, Direction2, Direction3, Point2, StandardDirection2,
    StandardDirection3,
};

const NAMES: [StandardDirection3; 6] = [
    StandardDirection3::X,
    StandardDirection3::Y,
    StandardDirection3::Z,
    StandardDirection3::NegX,
    StandardDirection3::NegY,
    StandardDirection3::NegZ,
];
const NAMES2: [StandardDirection2; 4] = [
    StandardDirection2::X,
    StandardDirection2::Y,
    StandardDirection2::NegX,
    StandardDirection2::NegY,
];
fn frame(value: Axis3) {
    ax3(value);
    flag(value.direct().unwrap());
}
pub fn gp_standard_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (n, x) = (fields.int() as usize, fields.int() as usize);
    let p = fields.point();
    let (main, second) = (NAMES[n], NAMES[x]);
    dir(Direction3::from_standard(main));
    ax1(Axis1::with_standard(p, main));
    ax1(Axis1::from_standard(main));
    ax2(Axis2::with_standard_frame(p, main, second));
    ax2(Axis2::with_standard(p, main));
    ax2(Axis2::from_standard(main));
    frame(Axis3::with_standard_frame(p, main, second));
    frame(Axis3::with_standard(p, main));
    frame(Axis3::from_standard(main));
    let name2 = NAMES2[x % 4];
    pair(Direction2::from_standard(name2).xy());
    ax(Axis2d::with_standard(Point2::new(p.x(), p.y()), name2));
    ax(Axis2d::from_standard(name2));
    statics();
    end();
}
fn statics() {
    real(gp::resolution());
    xyz(gp::origin().xyz());
    [gp::dx(), gp::dy(), gp::dz()].into_iter().for_each(dir);
    [gp::ox(), gp::oy(), gp::oz()].into_iter().for_each(ax1);
    [gp::xoy(), gp::zox(), gp::yoz()].into_iter().for_each(ax2);
    pair(gp::origin2d().xy());
    pair(gp::dx2d().xy());
    pair(gp::dy2d().xy());
    ax(gp::ox2d());
    ax(gp::oy2d());
}
