// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::super::isolation_tests::{first_edge, pole, set_pole};
use super::super::super::evaluator_tests::{redirect_pcurve, redirect_space};
use super::*;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Face, Shape, edge, face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomBezierCurve, GeomSurfaceOfLinearExtrusion, GeomSweptSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::Point3;

pub(in crate::sketch_region_prism::region_certificate) fn live_face(
    receipt: &PrismReceipt,
    role: PrismImageRole,
) -> Face {
    let diagnostic = receipt.diagnostic.as_ref().unwrap();
    let binding = diagnostic
        .bindings
        .iter()
        .find(|binding| {
            binding.native.role == role
                && binding.context.occurrence.shape_type().unwrap() == ShapeEnum::Face
        })
        .unwrap();
    face(&binding.context.occurrence).unwrap()
}
pub(super) fn side_basis(receipt: &PrismReceipt) -> Handle<GeomBezierCurve> {
    let surface = BRepTool::surface(&live_face(receipt, PrismImageRole::Generated)).unwrap();
    surface
        .cast::<GeomSurfaceOfLinearExtrusion>()
        .basis_curve()
        .cast()
}
#[test]
fn edited_original_copy_and_side_never_reuse_measured_coefficients() {
    for target in 0..3 {
        let (bridge, shapes) = fixture();
        let receipt = run(&bridge, &shapes);
        let original = measured(&bridge, &shapes, &receipt);
        edit_target(target, &shapes, &receipt);
        let error = measurement_attempt(&bridge, &shapes, &receipt, options(), || false)
            .err()
            .unwrap();
        assert_target_failure(target, error);
        assert_encloses(original.measures.first_cap.enclosure(), 2.4);
    }
}
pub(in crate::sketch_region_prism::region_certificate) fn edit_target(
    target: usize,
    shapes: &[Shape],
    receipt: &PrismReceipt,
) {
    match target {
        0 => set_pole(&edge(&shapes[0]).unwrap(), 5.),
        1 => set_pole(
            &first_edge(&face(&receipt.copied.as_ref().unwrap().base).unwrap()),
            5.,
        ),
        2 => side_basis(receipt)
            .set_pole(2, Point3::new(5., 0., 0.))
            .unwrap(),
        _ => panic!("target {target}; expected original/copy/side"),
    }
}
#[test]
fn original_and_copied_evaluator_overrides_remain_explicit_incomplete() {
    for target in 0..3 {
        let (bridge, shapes) = fixture();
        let receipt = run(&bridge, &shapes);
        let copied = face(&receipt.copied.as_ref().unwrap().base).unwrap();
        match target {
            0 => redirect_space(&shapes[0]),
            1 => redirect_space(&first_edge(&copied)),
            _ => redirect_pcurve(&first_edge(&copied), &copied),
        }
        let result = measurement_attempt(&bridge, &shapes, &receipt, options(), || false);
        assert!(matches!(
            result,
            Err(PrismMeasureFailure::Region(
                RegionCertificateCause::Capture(_)
            ))
        ));
    }
}
#[test]
fn edited_cap_trim_range_is_not_hidden_by_unchanged_surface_measure() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let last = live_face(&receipt, PrismImageRole::Last);
    BRepBuilder::default()
        .range_on_face(&first_edge(&last), &last, 0., 0.5)
        .unwrap();
    let result = measurement_attempt(&bridge, &shapes, &receipt, options(), || false);
    let failure = result.err().expect("trim range must be rejected");
    assert_shared_trim_range(&receipt);
    assert_target_failure(1, failure);
}
#[test]
fn source_and_output_mutation_at_last_callback_are_caught_by_final_rebind() {
    for target in [0, 1, 2] {
        let (bridge, shapes) = fixture();
        let receipt = run(&bridge, &shapes);
        let polls = count_polls(&bridge, &shapes, &receipt);
        assert_late_mutation(&bridge, &shapes, &receipt, target, polls);
    }
}
fn count_polls(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
) -> usize {
    let mut count = 0;
    let report = measurement_attempt(bridge, shapes, receipt, options(), || {
        count += 1;
        false
    });
    assert!(report.is_ok());
    count
}
fn assert_late_mutation(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    target: usize,
    cutoff: usize,
) {
    let mut count = 0;
    let result = measurement_attempt(bridge, shapes, receipt, options(), || {
        mutate_at_poll(&mut count, cutoff, target, shapes, receipt)
    });
    assert_eq!(count, cutoff);
    assert_target_failure(
        target,
        result.err().expect("late mutation must be rejected"),
    );
}
#[test]
fn success_failure_and_cancellation_leave_supplied_original_and_copy_unchanged() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let original = pole(&edge(&shapes[0]).unwrap());
    let copied = first_edge(&face(&receipt.copied.as_ref().unwrap().base).unwrap());
    let before = pole(&copied);
    measured(&bridge, &shapes, &receipt);
    let mut limits = options();
    limits.signed_volume.absolute_error_limit = 0.;
    assert!(measurement_attempt(&bridge, &shapes, &receipt, limits, || { false }).is_err());
    assert!(measurement_attempt(&bridge, &shapes, &receipt, options(), || true).is_err());
    assert_eq!(pole(&edge(&shapes[0]).unwrap()), original);
    assert_eq!(pole(&copied), before);
    measured(&bridge, &shapes, &receipt);
}

pub(super) fn assert_target_failure(target: usize, failure: PrismMeasureFailure) {
    if target == 2 {
        assert!(matches!(
            failure,
            PrismMeasureFailure::Chart(PrismSurfaceCause::Identity {
                stage: PrismChartStage::SideBasis,
                ..
            })
        ));
        return;
    }
    let expected = match target {
        0 => "current original coefficients/range/placement",
        1 => "pre-sweep/current copied geometry",
        _ => panic!("target {target}; expected original/copy/side"),
    };
    assert_hypothesis(failure, expected);
}
fn assert_shared_trim_range(receipt: &PrismReceipt) {
    let copied = receipt.copied.as_ref().unwrap();
    let actual = face(&copied.base).unwrap();
    let raw = BRepTool::curve_on_face(&first_edge(&actual), &actual).unwrap();
    assert_eq!([raw.first, raw.last], [0., 0.5]);
    assert_eq!(
        copied.chart_sources.as_ref().unwrap().boundaries[0].interval,
        [0., 1.]
    );
    assert_eq!(pole(&first_edge(&actual)), Point3::new(4., 0., 0.));
}

fn mutate_at_poll(
    count: &mut usize,
    cutoff: usize,
    target: usize,
    shapes: &[Shape],
    receipt: &PrismReceipt,
) -> bool {
    *count += 1;
    if *count == cutoff {
        edit_target(target, shapes, receipt);
    }
    false
}

#[test]
fn measurement_reads_preserve_candidate_geometry_and_mesh_caches() {
    use super::super::super::super::isolation_tests::{assert_original_mesh, attached_mesh};
    let (bridge, shapes) = fixture();
    let candidate = &borrowed_batch(&bridge).unwrap().candidates[0];
    let mesh = attached_mesh(&candidate.face);
    let before = boundary::capture_boundary(&candidate.face).unwrap();
    let receipt = run(&bridge, &shapes);
    for mode in 0..3 {
        exercise_measurement_mode(&bridge, &shapes, &receipt, mode);
        assert_original_mesh(&mesh);
        let after = boundary::capture_boundary(&candidate.face).unwrap();
        assert!(boundary::same_boundary_geometry(&before, &after));
    }
}
fn exercise_measurement_mode(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    mode: usize,
) {
    let mut limits = options();
    if mode == 1 {
        limits.signed_volume.absolute_error_limit = 0.;
    }
    let result = measurement_attempt(bridge, shapes, receipt, limits, || mode == 2);
    assert_eq!(result.is_ok(), mode == 0);
}
