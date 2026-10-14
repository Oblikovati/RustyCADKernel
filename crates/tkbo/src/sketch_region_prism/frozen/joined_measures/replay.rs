// SPDX-License-Identifier: GPL-2.0-only
//! Whole-source replay plus exact numeric report bits, without live occurrence addresses.
use super::super::encoding::numbers;
use super::*;

impl JoinedPrismMeasures {
    /// Replay full source and every scalar report; `measures.replay_bytes()` is independent of process addresses.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = b"martensite.joined-prism-measures/v1".to_vec();
        bytes.extend((self.binding.len() as u64).to_le_bytes());
        bytes.extend(&*self.binding);
        bytes.extend(self.requested.version.to_le_bytes());
        reports(&self.measures, &mut bytes);
        bytes
    }
}
fn reports(measures: &CompletePrismMeasures, bytes: &mut Vec<u8>) {
    plane(&measures.subset.first_cap, bytes);
    plane(&measures.subset.last_cap, bytes);
    let sweep = &measures.subset.signed_volume;
    scalar(sweep.enclosure(), sweep.absolute_error_bound(), bytes);
    integral_options(sweep.requested_options(), bytes);
    integral_options(sweep.effective_options(), bytes);
    numbers(measures.subset.extrusion, bytes);
    side(&measures.side, bytes);
    total(&measures.total, bytes);
}
fn scalar(enclosure: [f64; 2], error: f64, bytes: &mut Vec<u8>) {
    numbers(enclosure, bytes);
    numbers([error], bytes);
}
fn plane(report: &CertifiedPolynomialPlaneMeasure, bytes: &mut Vec<u8>) {
    scalar(report.enclosure(), report.absolute_error_bound(), bytes);
    integral_options(report.requested_options(), bytes);
    integral_options(report.effective_options(), bytes);
}
fn integral_options(options: PolynomialIntegralOptions, bytes: &mut Vec<u8>) {
    bytes.extend(options.behavior_version.to_le_bytes());
    bytes.extend(options.max_degree.to_le_bytes());
    bytes.extend(options.max_curves.to_le_bytes());
    numbers([options.absolute_error_limit], bytes);
}
fn side(report: &CertifiedPolynomialSideIntegral, bytes: &mut Vec<u8>) {
    scalar(report.enclosure(), report.absolute_error_bound(), bytes);
    side_options(report.requested_options(), bytes);
    side_options(report.effective_options(), bytes);
    bytes.extend(report.work().subdivisions.to_le_bytes());
    bytes.extend(report.work().interval_bounds.to_le_bytes());
}
fn side_options(options: PolynomialSideOptions, bytes: &mut Vec<u8>) {
    bytes.extend(options.behavior_version.to_le_bytes());
    bytes.extend(options.max_degree.to_le_bytes());
    bytes.extend(options.max_subdivisions.to_le_bytes());
    numbers([options.absolute_error_limit], bytes);
}
fn total(report: &CertifiedEnclosureSum, bytes: &mut Vec<u8>) {
    scalar(report.enclosure(), report.absolute_error_bound(), bytes);
    total_options(report.requested_options(), bytes);
    total_options(report.effective_options(), bytes);
    bytes.extend((report.components().len() as u64).to_le_bytes());
    for component in report.components() {
        numbers(*component, bytes);
    }
}
fn total_options(options: EnclosureSumOptions, bytes: &mut Vec<u8>) {
    bytes.extend(options.behavior_version.to_le_bytes());
    bytes.extend(options.max_components.to_le_bytes());
    numbers([options.absolute_error_limit], bytes);
}
