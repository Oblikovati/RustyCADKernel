// SPDX-License-Identifier: GPL-2.0-only
//! Same topology keys cannot freeze mutable representations reachable through native aliases.
use super::source_binding_tests::source_policy;
use super::tests::{capture, curved_crossing, policy};
use super::*;
use crate::sketch_region_source_conformance::*;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge},
};
use rustycad_tkmath::gp::{Point2, Point3};
fn live_census() -> BoundaryCensus {
    enumerate_source_bound_line_diagnostic(
        capture(&curved_crossing()),
        policy(),
        source_policy(),
        || false,
    )
    .unwrap_or_else(|_| panic!("census"))
}
fn assert_live_geometry_incomplete(census: &BoundaryCensus) {
    let historical = census.source_binding.as_ref().unwrap();
    assert!(historical.representation_certified());
    assert!(!census.source_representation_certified());
    assert!(
        census
            .current_obligations()
            .contains(&CertificationObligation::SourceConversionAndPlane)
    );
    assert!(
        !source_binding_faults(
            &census.inputs,
            source_policy(),
            census.source_binding.as_ref().unwrap().inspection()
        )
        .is_empty()
    );
}
#[test]
fn live_copied_curve_mutation_preserves_scalar_history_but_invalidates_current_geometry() {
    let census = live_census();
    assert!(census.source_representation_certified());
    mutate_actual_copied_curve(&census.inputs.segments[0].edge);
    assert_live_geometry_incomplete(&census);
}
fn mutate_actual_copied_curve(shape: &Shape) {
    use rustycad_tkg3d::geom::GeomBezierCurve;
    let (curve, _, _) = BRepTool::curve(&edge(shape).unwrap()).unwrap();
    curve
        .cast::<GeomBezierCurve>()
        .set_pole(2, Point3::new(8., 0., 1.))
        .unwrap();
}
#[test]
fn live_stored_pcurve_mutation_rejects_current_map_without_changing_scalar_capture() {
    use rustycad_tkg2d::geom2d::Geom2dLine;
    let census = live_census();
    let actual = edge(&census.inputs.segments[1].edge).unwrap();
    let stored = BRepTool::curve_on_face(&actual, &census.inputs.construction_support).unwrap();
    stored
        .pcurve
        .cast::<Geom2dLine>()
        .set_location(Point2::new(9., 8.));
    assert_live_geometry_incomplete(&census);
    assert_current_parameter_map_rejected(&census, 1);
}
#[test]
fn live_plane_mutation_is_not_hidden_by_unchanged_face_identity() {
    use rustycad_tkg3d::geom::GeomPlane;
    let census = live_census();
    let (surface, _) =
        BRepTool::surface_with_location(&census.inputs.construction_support).unwrap();
    let plane = surface.cast::<GeomPlane>();
    let mut changed = plane.pln();
    changed.set_location(Point3::new(0., 0., 1.));
    plane.set_pln(changed);
    assert_live_geometry_incomplete(&census);
}
fn assert_current_parameter_map_rejected(census: &BoundaryCensus, index: usize) {
    let contact = &census.reports[0].contacts()[0];
    assert!(matches!(
        super::parameter_binding::contact_native_membership(census, index, contact, 0.),
        Err(SourceConformanceCause::CurrentDescriptorMismatch { .. })
    ));
}
#[test]
fn missing_live_support_retains_actual_typed_capture_failure() {
    use rustycad_tkbrep::topods::Face;
    let mut census = live_census();
    census.inputs.construction_support = Face::default();
    assert_live_geometry_incomplete(&census);
    let faults = source_binding_faults(
        &census.inputs,
        source_policy(),
        census.source_binding.as_ref().unwrap().inspection(),
    );
    assert!(faults.iter().any(|fault| matches!(fault,
        SourceBindingFault::CurrentGeometry { cause, .. } if matches!(cause.as_ref(), SourceConformanceCause::Capture(_)))));
}

#[test]
fn unsupported_actual_support_is_typed_and_never_matches_historical_plane() {
    use rustycad_tkbrep::brep::BRepTFace;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg3d::geom::GeomCylindricalSurface;
    use rustycad_tkmath::gp::Axis3;
    let census = live_census();
    let cylinder = Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap());
    census
        .inputs
        .construction_support
        .tshape()
        .cast::<BRepTFace>()
        .set_surface(&cylinder.cast());
    assert_live_geometry_incomplete(&census);
    assert_unsupported_support_preserved(&census);
}
fn assert_unsupported_support_preserved(census: &BoundaryCensus) {
    let current = capture_construction(
        &census.inputs.segments[0],
        &census.inputs.construction_support,
    );
    assert!(
        matches!(current.support, Err(cause) if matches!(cause.as_ref(),
        crate::sketch_region_geometry_capture::CapturedGeometryCause::Unsupported { representation: "support plane", .. }))
    );
}

#[test]
fn current_correspondence_inspection_preserves_reachable_geometry_and_topology_flags() {
    let census = live_census();
    let actual = edge(&census.inputs.segments[0].edge).unwrap();
    let before = capture_space_representation(&actual).unwrap();
    let flags = copied_flags(&census);
    for _ in 0..3 {
        assert!(census.source_representation_certified());
    }
    let after = capture_space_representation(&actual).unwrap();
    assert!(same_space_representation(&before, &after));
    assert_eq!(flags, copied_flags(&census));
}
fn copied_flags(census: &BoundaryCensus) -> Vec<[bool; 3]> {
    census
        .inputs
        .segments
        .iter()
        .map(|segment| {
            let actual = segment.edge.tshape();
            let actual = actual.get().unwrap();
            [actual.modified(), actual.checked(), actual.free()]
        })
        .collect()
}
