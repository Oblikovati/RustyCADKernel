// SPDX-License-Identifier: GPL-2.0-only
//! Borrowed metadata is not isolation: observe actual geometry, mesh caches and retry state.
use super::super::{
    isolation_tests::{assert_original_mesh, attached_mesh, first_edge, pole, set_pole},
    tests::{progress, request},
};
use super::{tests::*, *};
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkbrep::topods::edge;
use rustycad_tkmath::gp::Point3;

#[test]
fn borrowed_source_and_candidate_storage_survive_each_stop_failure_and_success() {
    let (bridge, shapes) = fixture();
    let candidate = &borrowed_batch(&bridge).unwrap().candidates[0];
    let mesh = attached_mesh(&candidate.face);
    let before = boundary::capture_boundary(&candidate.face).unwrap();
    for phase in [
        PrismPhase::Selection,
        PrismPhase::Copy,
        PrismPhase::Sweep,
        PrismPhase::History,
        PrismPhase::Validation,
    ] {
        assert_stopped_isolated(&bridge, &shapes, phase);
        assert_original_mesh(&mesh);
        let after = boundary::capture_boundary(&candidate.face).unwrap();
        assert!(boundary::same_boundary_geometry(&before, &after));
    }
    assert_eq!(proof(&run(&bridge, &shapes)).selected_winding, 1);
    assert_original_mesh(&mesh);
}
fn assert_stopped_isolated(
    bridge: &PairedContactTransaction,
    shapes: &[rustycad_tkbrep::topods::Shape],
    phase: PrismPhase,
) {
    for fail in [false, true] {
        let mut observer = progress((!fail).then_some(phase), fail.then_some(phase));
        let receipt = retain_source_bound_prism(
            bridge,
            &binding_sources(shapes),
            request([1., 1.]),
            &mut observer,
        )
        .ok()
        .unwrap();
        assert_four_present(&receipt);
        assert_eq!(pole(&edge(&shapes[0]).unwrap()), Point3::new(4., 0., 0.));
    }
}
#[test]
fn copied_geometry_and_immutable_coefficients_survive_original_edit_and_retry() {
    let (bridge, shapes) = fixture();
    let first = run(&bridge, &shapes);
    let copied_face = rustycad_tkbrep::topods::face(&first.copied.as_ref().unwrap().base).unwrap();
    let copied_edge = first_edge(&copied_face);
    set_pole(&edge(&shapes[0]).unwrap(), 5.);
    assert_four_present(&run(&bridge, &shapes));
    assert_eq!(pole(&copied_edge), Point3::new(4., 0., 0.));
    set_pole(&copied_edge, 6.);
    assert_eq!(pole(&edge(&shapes[0]).unwrap()), Point3::new(5., 0., 0.));
    assert_eq!(proof(&first).selected_winding, 1);
    set_pole(&edge(&shapes[0]).unwrap(), 4.);
    assert_eq!(proof(&run(&bridge, &shapes)).selected_winding, 1);
}
#[test]
fn copied_current_geometry_cannot_inherit_pre_sweep_certificate_after_edit() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let copied_face =
        rustycad_tkbrep::topods::face(&receipt.copied.as_ref().unwrap().base).unwrap();
    set_pole(&first_edge(&copied_face), 6.);
    let failed = inspect::inspect(&bridge, &binding_sources(&shapes), &receipt);
    assert!(matches!(
        failed,
        Err(RegionCertificateCause::Hypothesis {
            name: "pre-sweep/current copied geometry",
            ..
        })
    ));
    assert_eq!(proof(&receipt).selected_winding, 1);
}
