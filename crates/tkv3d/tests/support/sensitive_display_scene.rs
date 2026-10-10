//! Deterministic native entities for sensitive geometry and display probes.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::{
    gp::{Axis1, Axis2, Circle3, Direction3, Point3, Transform3, Vector3},
    poly::{Triangle, Triangulation},
    toploc::Location,
};
use rustycad_tkv3d::{select_mgr::*, select3d::*};
pub fn display_location(kind: i32) -> Transform3 {
    let mut location = Transform3::default();
    if kind == 2 {
        location
            .set_rotation(
                Axis1::new(Point3::default(), Direction3::new(1., 2., 3.).unwrap()),
                0.7,
            )
            .unwrap();
    }
    if kind == 3 {
        location.set_scale(Point3::new(1., 2., 3.), -1.25).unwrap();
    }
    if kind != 0 {
        location.set_translation_part(Vector3::new(4., -3., 2.));
    }
    location
}
pub fn display_points() -> [Point3; 4] {
    [
        Point3::new(0., 0., 0.),
        Point3::new(2., 0., 0.),
        Point3::new(2., 3., 1.),
        Point3::new(0., 3., 0.),
    ]
}
pub fn display_selection(kind: i32, flags: i32) -> Result<Handle<SelectMgrSelection>, Failure> {
    let owner: Handle<dyn SelectMgrEntityOwnerSource> = SelectMgrEntityOwner::new(0).cast();
    let selection = Handle::new(SelectMgrSelection::new(0));
    let kinds = display_fixture_kinds(kind);
    for kind in kinds {
        let entity = display_entity(&owner, kind)?;
        if flags & 1 != 0 {
            entity.set_flipping_options(
                true,
                &Axis2::new(Point3::new(1., 2., 3.), Direction3::new(0., 0., 1.)?)?,
            );
        }
        selection.add(&entity)?;
    }
    Ok(selection)
}
fn display_fixture_kinds(kind: i32) -> Vec<i32> {
    match kind {
        12 => (0..12).collect(),
        13 => vec![],
        _ => vec![kind],
    }
}
fn display_entity(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    kind: i32,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let p = display_points();
    Ok(match kind {
        0 => Handle::new(Select3DSensitivePoint::new(owner, p[2])).cast(),
        1 => Handle::new(Select3DSensitiveSegment::new(owner, p[0], p[2])).cast(),
        2 => Handle::new(Select3DSensitiveBox::from_bounds(
            owner, -1., -2., -3., 4., 5., 6.,
        ))
        .cast(),
        3 => display_circle(owner)?,
        4 | 14 => display_cylinder(owner, kind),
        5 => Handle::new(Select3DSensitiveFace::new(
            owner,
            &p,
            Select3DTypeOfSensitivity::Boundary,
        )?)
        .cast(),
        6 => Handle::new(Select3DSensitivePoly::new(owner, &p, true)?).cast(),
        7 => Handle::new(Select3DSensitiveCurve::from_points(owner, &p)?).cast(),
        8 => display_wire(owner)?,
        9 => display_triangle(owner),
        10 => display_triangulation(owner)?,
        _ => Handle::new(Select3DSensitiveSphere::new(owner, p[2], 2.)).cast(),
    })
}
fn display_wire(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let wire = Handle::new(Select3DSensitiveWire::new(owner));
    for kind in [1, 7, 0, 6] {
        wire.add(&display_entity(owner, kind)?)?;
    }
    Ok(wire.cast())
}
fn display_triangulation(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let mesh = Triangulation::with_sizes(4, 2, false, false)?;
    for (index, point) in display_points().into_iter().enumerate() {
        mesh.set_node(index as i32 + 1, point)?;
    }
    mesh.set_triangle(1, Triangle::new(1, 2, 3))?;
    mesh.set_triangle(2, Triangle::new(1, 3, 4))?;
    Ok(Handle::new(Select3DSensitiveTriangulation::new(
        owner,
        &Handle::new(mesh),
        &Location::new(display_location(1)),
        true,
    )?)
    .cast())
}

fn display_circle(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let plane = Axis2::new(Point3::new(1., 2., 3.), Direction3::new(0., 1., 1.)?)?;
    let circle = Circle3::new(plane, 2.)?;
    Ok(Handle::new(Select3DSensitiveCircle::new(owner, &circle, false)).cast())
}
fn display_cylinder(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    kind: i32,
) -> Handle<dyn Select3DSensitiveEntity> {
    let (bottom, top) = if kind == 14 { (0., 0.) } else { (2., 0.5) };
    Handle::new(Select3DSensitiveCylinder::new(
        owner,
        bottom,
        top,
        4.,
        display_location(2),
        false,
    ))
    .cast()
}
fn display_triangle(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Handle<dyn Select3DSensitiveEntity> {
    let p = display_points();
    Handle::new(Select3DSensitiveTriangle::new(
        owner,
        p[0],
        p[1],
        p[2],
        Select3DTypeOfSensitivity::Interior,
    ))
    .cast()
}
