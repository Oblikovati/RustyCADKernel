//! Shared concrete selecting-volume queries with observable picks and inside flags.
use super::common::{Reader, attempt, flag, pnt, real};
use super::frustum_core::{points, transformation};
use rustycad_tkernel::{ncollection::Vec3, standard::Failure};
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkv3d::{
    select_basics::SelectBasicsPickResult,
    select_mgr::{SelectMgrBaseIntersector, SelectMgrViewClipRange},
};
pub(super) fn intersector_pick_state(pick: &SelectBasicsPickResult) {
    real(pick.depth());
    real(pick.dist_to_geom_center());
    pnt(*pick.picked_point());
    for value in [
        pick.surface_normal().x,
        pick.surface_normal().y,
        pick.surface_normal().z,
    ] {
        real(f64::from(value));
    }
}
fn box_query(
    volume: &dyn SelectMgrBaseIntersector,
    r: &mut Reader<'_>,
    output: Option<&mut bool>,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) {
    let (a, b) = (r.xyz(), r.xyz());
    let (lo, hi) = (
        Vec3::new(a.x(), a.y(), a.z()),
        Vec3::new(b.x(), b.y(), b.z()),
    );
    attempt(|| {
        flag(volume.overlaps_box(&lo, &hi, output)?);
        Ok(())
    });
    attempt(|| {
        flag(volume.overlaps_box_pick(&lo, &hi, clip, pick)?);
        Ok(())
    });
}
fn triangle_point_query(
    volume: &dyn SelectMgrBaseIntersector,
    r: &mut Reader<'_>,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> Result<(), Failure> {
    let point = r.pnt();
    attempt(|| {
        flag(volume.overlaps_point(&point)?);
        Ok(())
    });
    attempt(|| {
        flag(volume.overlaps_point_pick(&point, clip, pick)?);
        Ok(())
    });
    Ok(())
}
fn triangle_segment_query(
    volume: &dyn SelectMgrBaseIntersector,
    r: &mut Reader<'_>,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> Result<(), Failure> {
    let (a, b) = (r.pnt(), r.pnt());
    attempt(|| {
        flag(volume.overlaps_segment(&a, &b, clip, pick)?);
        Ok(())
    });
    Ok(())
}
fn triangle_polygon_query(
    volume: &dyn SelectMgrBaseIntersector,
    r: &mut Reader<'_>,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> Result<(), Failure> {
    let sensitivity = r.integer();
    let vertices = points(r);
    attempt(|| {
        flag(volume.overlaps_polygon(&vertices, sensitivity, clip, pick)?);
        Ok(())
    });
    Ok(())
}
fn triangle_triangle_query(
    volume: &dyn SelectMgrBaseIntersector,
    r: &mut Reader<'_>,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> Result<(), Failure> {
    let sensitivity = r.integer();
    let (a, b, c) = (r.pnt(), r.pnt(), r.pnt());
    attempt(|| {
        flag(volume.overlaps_triangle(&a, &b, &c, sensitivity, clip, pick)?);
        Ok(())
    });
    Ok(())
}
fn triangle_sphere_query(
    volume: &dyn SelectMgrBaseIntersector,
    r: &mut Reader<'_>,
    output: Option<&mut bool>,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> Result<(), Failure> {
    let (center, radius) = (r.pnt(), r.real());
    attempt(|| {
        flag(volume.overlaps_sphere(&center, radius, output)?);
        Ok(())
    });
    attempt(|| {
        flag(volume.overlaps_sphere_pick(&center, radius, clip, pick)?);
        Ok(())
    });
    Ok(())
}
fn triangle_cylinder_query(
    volume: &dyn SelectMgrBaseIntersector,
    r: &mut Reader<'_>,
    output: Option<&mut bool>,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> Result<(), Failure> {
    let (bottom, top, height, hollow) = (r.real(), r.real(), r.real(), r.flag());
    let transform = transformation(r)?;
    attempt(|| {
        flag(volume.overlaps_cylinder(bottom, top, height, &transform, hollow, output)?);
        Ok(())
    });
    attempt(|| {
        flag(volume.overlaps_cylinder_pick(bottom, top, height, &transform, hollow, clip, pick)?);
        Ok(())
    });
    Ok(())
}
fn triangle_circle_query(
    volume: &dyn SelectMgrBaseIntersector,
    r: &mut Reader<'_>,
    output: Option<&mut bool>,
    clip: &SelectMgrViewClipRange,
    pick: &mut SelectBasicsPickResult,
) -> Result<(), Failure> {
    let (radius, filled) = (r.real(), r.flag());
    let transform = transformation(r)?;
    attempt(|| {
        flag(volume.overlaps_circle(radius, &transform, filled, output)?);
        Ok(())
    });
    attempt(|| {
        flag(volume.overlaps_circle_pick(radius, &transform, filled, clip, pick)?);
        Ok(())
    });
    Ok(())
}
pub(super) fn intersector_query(
    volume: &dyn SelectMgrBaseIntersector,
    r: &mut Reader<'_>,
    clip: &SelectMgrViewClipRange,
) -> Result<(), Failure> {
    let (kind, mode) = (r.integer(), r.integer());
    let mut inside = mode == 1;
    let output = if mode < 0 { None } else { Some(&mut inside) };
    let mut pick = SelectBasicsPickResult::new(123., 456., Point3::new(7., 8., 9.));
    pick.set_surface_normal_vec(&Vector3::new(1., 2., 3.));
    match kind {
        0 => box_query(volume, r, output, clip, &mut pick),
        1 => triangle_point_query(volume, r, clip, &mut pick)?,
        2 => triangle_segment_query(volume, r, clip, &mut pick)?,
        3 => triangle_polygon_query(volume, r, clip, &mut pick)?,
        4 => triangle_triangle_query(volume, r, clip, &mut pick)?,
        5 => triangle_sphere_query(volume, r, output, clip, &mut pick)?,
        6 => triangle_cylinder_query(volume, r, output, clip, &mut pick)?,
        7 => triangle_circle_query(volume, r, output, clip, &mut pick)?,
        _ => panic!("triangle query kind={kind}; expected 0..=7"),
    }
    flag(inside);
    intersector_pick_state(&pick);
    Ok(())
}
