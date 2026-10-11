//! Snapshots retain preseeded pick outputs, owner sharing and sphere detection state.
use super::{
    Select3DSensitiveCylinder, Select3DSensitiveEntity, Select3DSensitiveEntityMembers,
    Select3DSensitiveSphere,
};
use crate::select_basics::SelectBasicsPickResult;
use crate::select_mgr::{SelectMgrEntityOwnerMembers, SelectMgrEntityOwnerSource};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Handle, real::real_last};
use rustycad_tkmath::gp::Point3;

pub(super) fn quadric_seed() -> SelectBasicsPickResult {
    let mut pick = SelectBasicsPickResult::new(42., 43., Point3::new(44., 45., 46.));
    pick.set_surface_normal(Vec3::new(47., 48., 49.));
    pick
}
pub(super) fn quadric_snapshot(
    probe: &Handle<dyn Select3DSensitiveEntity>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ordinal: i32,
    step: i32,
    failure: bool,
    hit: i32,
    pick: &SelectBasicsPickResult,
) -> Vec<f64> {
    let mut columns = vec![
        f64::from(ordinal),
        f64::from(step),
        f64::from(failure),
        f64::from(hit),
    ];
    append_quadric_pick(&mut columns, pick);
    append_quadric_entity(&mut columns, probe.get().unwrap(), owner);
    append_quadric_detection(&mut columns, probe);
    columns
}
pub(super) fn append_quadric_pick(columns: &mut Vec<f64>, pick: &SelectBasicsPickResult) {
    columns.extend([
        f64::from(pick.is_valid()),
        pick.depth(),
        pick.dist_to_geom_center(),
        f64::from(pick.has_picked_point()),
    ]);
    append_quadric_point(columns, pick.picked_point());
    let normal = pick.surface_normal();
    columns.extend([
        f64::from(normal.x),
        f64::from(normal.y),
        f64::from(normal.z),
    ]);
}
fn append_quadric_entity(
    columns: &mut Vec<f64>,
    probe: &dyn Select3DSensitiveEntity,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) {
    columns.extend([
        f64::from(probe.sensitivity_factor()),
        f64::from(probe.nb_sub_elements()),
        f64::from(probe.to_build_bvh()),
        f64::from(probe.owner_id() == *owner),
        f64::from(probe.owner_id().get().unwrap().priority()),
    ]);
    let bounds = probe.bounding_box();
    columns.push(f64::from(bounds.is_valid()));
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    columns.extend([min.x, min.y, min.z, max.x, max.y, max.z]);
    append_quadric_point(columns, &probe.center_of_geometry());
}
fn append_quadric_detection(columns: &mut Vec<f64>, probe: &Handle<dyn Select3DSensitiveEntity>) {
    let sphere = probe.clone().cast::<Select3DSensitiveSphere>();
    let point = sphere
        .get()
        .map(Select3DSensitiveSphere::last_detected_point);
    let detected = point.is_some_and(|point| point.x() != real_last());
    columns.push(f64::from(detected));
    append_quadric_point(columns, &point.filter(|_| detected).unwrap_or_default());
    let cylinder = probe.clone().cast::<Select3DSensitiveCylinder>();
    columns.push(
        cylinder
            .get()
            .map_or(-1., |cylinder| f64::from(cylinder.is_hollow())),
    );
}
pub(super) fn append_quadric_point(columns: &mut Vec<f64>, point: &Point3) {
    // Coordinate columns preserve exact earlier outputs when later callbacks fail.
    columns.extend([point.x(), point.y(), point.z()]);
}
pub(super) fn reset_quadric_detection(probe: &Handle<dyn Select3DSensitiveEntity>) {
    let sphere = probe.clone().cast::<Select3DSensitiveSphere>();
    if let Some(sphere) = sphere.get() {
        sphere.reset_last_detected_point();
    }
}
