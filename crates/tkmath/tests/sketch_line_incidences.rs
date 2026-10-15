// SPDX-License-Identifier: GPL-2.0-only
//! Closed-form analytic fixtures for certified sketch line incidences (ADR 0060).
//! Expected values are derived by hand below, independently of the implementation.
use rustycad_tkmath::predicates::roots::sketch_incidence::*;

mod fixture;
use fixture::*;

mod angle_arcs;
mod conic_pairs;
mod conics;
mod curve_isolation;
mod curve_selection;
mod curve_sources;
mod failures;
mod generated;
mod generated_curves;
mod isolation;
mod lines;
mod ordering;
mod selection;
mod spline_pairs;
mod splines;

#[test]
fn every_public_result_type_is_send_and_sync() {
    fn shared<T: Send + Sync>() {}
    shared::<SketchLineIncidenceReport>();
    shared::<SketchIncidenceFailure>();
    shared::<SketchSelectionError>();
    shared::<SketchIncidenceTarget>();
}
#[test]
fn report_retains_request_options_work_and_contact_owners() {
    let line = source([[-2., 0.], [2., 0.]], SketchLineDomain::Segment);
    let options = SketchIncidenceOptions::v1(ERROR);
    let report = certify_with(line.clone(), &[target(7, circle([0., 0.], 1.))], options);
    assert_eq!(report.source(), &line);
    assert_eq!(report.options(), options);
    assert!(report.refinements() <= options.max_refinements);
    let contact = &report.targets()[0].contacts()[0];
    assert_eq!((contact.target_index(), contact.target_id()), (0, 7));
    let collinear = certify(line, &[target(9, segment([-1., 0.], [1., 0.]))]);
    let overlap = &collinear.targets()[0].overlaps()[0];
    assert_eq!((overlap.target_index(), overlap.target_id()), (0, 9));
}
