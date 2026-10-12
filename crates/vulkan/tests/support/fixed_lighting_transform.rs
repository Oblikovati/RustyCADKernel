//! Genuine model, group and zoom-persistence transforms for paired lighting fixtures.
use super::LightingScene;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Axis1, Direction3, Point3, Transform3},
    toploc::Datum3,
};
use rustycad_tkservice::graphic3d::*;
pub(super) fn apply_fixed_fixture_transform(fixture: &LightingScene, mode: &str) {
    if mode.starts_with("lit-persist-") {
        apply_fixed_zoom_persistence(fixture);
        return;
    }
    let Some(transform) = fixed_fixture_transform(mode) else {
        return;
    };
    if mode == "lit-group-scale" {
        fixture.base.group.set_transformation(&transform);
        return;
    }
    fixture
        .base
        .structure
        .set_transformation(&Handle::new(Datum3::new(transform)));
}
fn apply_fixed_zoom_persistence(fixture: &LightingScene) {
    let persistence =
        Graphic3dTransformPers::with_point(Graphic3dTransModeFlags::ZOOM_PERS, &Point3::default())
            .unwrap();
    fixture
        .base
        .structure
        .set_transform_persistence(&Handle::new(persistence).cast());
}
fn fixed_fixture_transform(mode: &str) -> Option<Transform3> {
    let mut transform = Transform3::default();
    match mode {
        "lit-group-scale" | "lit-scale" => transform.set_scale(Point3::default(), 0.75).unwrap(),
        "lit-rotation" => transform
            .set_rotation(
                Axis1::new(Point3::default(), Direction3::new(0., 1., 0.).unwrap()),
                0.35,
            )
            .unwrap(),
        _ => return None,
    }
    Some(transform)
}
