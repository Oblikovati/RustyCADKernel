//! Retained group transformations exercise native model-only light-binding updates.
use super::LightingScene;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3};
use rustycad_tkservice::graphic3d::*;

pub fn fixed_adjacent_groups(fixture: &LightingScene, mode: &str) {
    if !mode.starts_with("lit-adjacent-") {
        return;
    }
    let group = fixture.base.structure.new_group().unwrap();
    group.set_group_primitives_aspect(&fixture.aspect.cast());
    group.add_primitive_array(&fixture.vertices, true);
    let mut rotation = Transform3::default();
    rotation
        .set_rotation(
            Axis1::new(Point3::default(), Direction3::new(0., 1., 0.).unwrap()),
            0.7,
        )
        .unwrap();
    group.set_transformation(&rotation);
}
