//! Typed analytic geometry mirrors the pinned native selection scenarios.
use super::{
    Select3DSensitiveCircle, Select3DSensitiveCylinder, Select3DSensitiveEntity,
    Select3DSensitiveSphere,
};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Point3, Transform3, Vector3, xoy};

pub(super) fn quadric_entity(
    kind: i32,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Handle<dyn Select3DSensitiveEntity> {
    if kind < 10 {
        return circle_entity(kind, owner);
    }
    if kind < 19 {
        return cylinder_entity(kind, owner);
    }
    sphere_entity(kind, owner)
}
fn circle_entity(
    kind: i32,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Handle<dyn Select3DSensitiveEntity> {
    let circle = quadric_circle(kind);
    let filled = kind % 2 != 0;
    let probe = if kind >= 8 {
        Select3DSensitiveCircle::with_ignored_point_count(
            owner,
            &circle,
            filled,
            if kind == 8 { -7 } else { 64 },
        )
    } else {
        Select3DSensitiveCircle::new(owner, &circle, filled)
    };
    Handle::new(probe).cast()
}
fn cylinder_entity(
    kind: i32,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Handle<dyn Select3DSensitiveEntity> {
    let (bottom, top, height) = quadric_cylinder_extents(kind);
    Handle::new(Select3DSensitiveCylinder::new(
        owner,
        bottom,
        top,
        height,
        quadric_transform(kind),
        matches!(kind, 11 | 13 | 15 | 17),
    ))
    .cast()
}
fn quadric_cylinder_extents(kind: i32) -> (f64, f64, f64) {
    match kind {
        12 | 13 => (2., 0.5, 3.),
        14 => (1., 0., 2.),
        15 => (0., 1., 2.),
        _ => (1., 1., 2.),
    }
}
fn sphere_entity(
    kind: i32,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Handle<dyn Select3DSensitiveEntity> {
    let center = match kind {
        20 => Point3::new(1., 0., 0.5),
        22 => Point3::new(0., 0., -1.),
        _ => Point3::default(),
    };
    let radius = match kind {
        20 => 2.,
        21 => 0.,
        22 => 0.25,
        _ => 1.,
    };
    Handle::new(Select3DSensitiveSphere::new(owner, center, radius)).cast()
}
fn quadric_circle(kind: i32) -> Circle3 {
    let axis = match kind {
        2 | 3 => Axis2::with_x_direction(Point3::default(), Direction3::X, Direction3::Y).unwrap(),
        4 | 5 => Axis2::with_x_direction(
            Point3::new(0.5, -0.25, 0.75),
            Direction3::new(1., 2., 3.).unwrap(),
            Direction3::new(2., -1., 0.).unwrap(),
        )
        .unwrap(),
        _ => xoy(),
    };
    let radius = match kind {
        6 | 7 => 0.,
        4 | 5 => 2.,
        _ => 1.,
    };
    Circle3::new(axis, radius).unwrap()
}
fn quadric_transform(kind: i32) -> Transform3 {
    let mut transform = Transform3::default();
    if kind == 16 {
        transform
            .set_rotation(Axis1::new(Point3::default(), Direction3::Y), 0.6)
            .unwrap();
        transform.set_translation_part(Vector3::new(0., 0., -1.));
    }
    if kind == 17 {
        transform.set_scale_factor(2.).unwrap();
        transform.set_translation_part(Vector3::new(0.5, -0.25, -1.));
    }
    if kind == 18 {
        transform.set_scale_factor(-1.).unwrap();
    }
    transform
}
