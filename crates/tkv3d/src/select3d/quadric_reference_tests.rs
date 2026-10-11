//! Retained detection aliases update across mutation without holding locks or sharing copies.
use super::quadric_fault_volume_fake::QuadricFaultVolumeFake;
use super::quadric_oracle_snapshot::quadric_seed;
use super::quadric_oracle_volume::quadric_volume;
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use super::{Select3DSensitiveEntity, Select3DSensitiveSphere};
use rustycad_tkernel::standard::{Failure, Handle, real::real_last};
use rustycad_tkmath::gp::Point3;

fn reference_step(
    original: &Select3DSensitiveSphere,
    connected: &Select3DSensitiveSphere,
    step: i32,
) -> Result<(), Failure> {
    let mut pick = quadric_seed();
    if step == 10 {
        original.reset_last_detected_point();
        return Ok(());
    }
    match step {
        0 => Ok(()),
        4 | 5 | 7 => reference_fault_step(original, connected, step, &mut pick),
        _ => reference_real_volume_step(original, step, &mut pick),
    }
}
fn reference_real_volume_step(
    original: &Select3DSensitiveSphere,
    step: i32,
    pick: &mut crate::select_basics::SelectBasicsPickResult,
) -> Result<(), Failure> {
    // Real misses and nonpoint selection reset the same retained native detection field.
    let volume = quadric_volume(if step == 8 { 5 } else { 0 }, false, false, step == 2);
    original.matches(&volume, pick).map(|_| ())
}
fn reference_fault_step(
    original: &Select3DSensitiveSphere,
    connected: &Select3DSensitiveSphere,
    step: i32,
    pick: &mut crate::select_basics::SelectBasicsPickResult,
) -> Result<(), Failure> {
    let volume = QuadricFaultVolumeFake::new(match step {
        4 => 2,
        5 => 1,
        _ => 0,
    });
    let probe = if step == 7 { connected } else { original };
    probe.matches(&volume, pick).map(|_| ())
}
fn append_reference_point(columns: &mut Vec<f64>, point: Point3) {
    let detected = point.x() != real_last();
    let point = if detected { point } else { Point3::default() };
    columns.extend([f64::from(detected), point.x(), point.y(), point.z()]);
}
fn replay_detection_references(
    original: &Select3DSensitiveSphere,
    connected: &Select3DSensitiveSphere,
) {
    let native = include_str!("../../tests/fixtures/sensitive-quadrics-reference.txt");
    assert_eq!(native.lines().count(), 11);
    let first = original.last_detected_point_ref();
    let second = original.last_detected_point_ref();
    let separate = connected.last_detected_point_ref();
    for (step, native) in native.lines().enumerate() {
        let outcome = reference_step(original, connected, step as i32);
        let mut columns = vec![step as f64, f64::from(outcome.is_err())];
        append_reference_point(&mut columns, first.point());
        append_reference_point(&mut columns, second.point());
        append_reference_point(&mut columns, separate.point());
        compare_oracle_snapshot(&columns, native);
    }
}
#[test]
fn native_retained_detection_references_observe_hits_misses_failures_and_copy_isolation() {
    let original = Select3DSensitiveSphere::new(&Handle::default(), Point3::default(), 1.);
    let connected = original
        .get_connected()
        .unwrap()
        .cast::<Select3DSensitiveSphere>();
    replay_detection_references(&original, connected.get().unwrap());
    assert_eq!(original.last_detected_point_ref().point().x(), real_last());
    assert_eq!(
        connected.get().unwrap().last_detected_point_ref().point(),
        Point3::new(8., 9., 10.)
    );
}
