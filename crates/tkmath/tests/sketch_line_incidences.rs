// SPDX-License-Identifier: GPL-2.0-only
//! Closed-form analytic fixtures for certified sketch line incidences (ADR 0060).
//! Expected values are derived by hand below, independently of the implementation.
use rustycad_tkmath::predicates::roots::sketch_incidence::*;

mod fixture;
use fixture::*;

mod conics;
mod failures;
mod generated;
mod isolation;
mod lines;
mod ordering;
mod selection;
mod splines;

#[test]
fn every_public_result_type_is_send_and_sync() {
    fn shared<T: Send + Sync>() {}
    shared::<SketchLineIncidenceReport>();
    shared::<SketchIncidenceFailure>();
    shared::<SketchSelectionError>();
    shared::<SketchIncidenceTarget>();
}
