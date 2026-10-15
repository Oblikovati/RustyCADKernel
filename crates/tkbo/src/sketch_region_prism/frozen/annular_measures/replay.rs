// SPDX-License-Identifier: GPL-2.0-only
//! Shared unchanged scalar codecs with explicit V2 quantities, semantic roles and complete input bytes.
use super::super::proof_join::{encode_joined_block, encode_joined_text};
use super::*;
use crate::sketch_region_prism::owned_request::names::encode_frozen_use_key;

pub(super) fn encode(report: &AnnularPrismMeasures) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_joined_text("martensite.annular-prism-measures/v2", &mut bytes);
    encode_joined_block(&report.binding, &mut bytes);
    options(report.requested, &mut bytes);
    for boundary in report.boundaries {
        direction(boundary, &mut bytes);
    }
    bytes.extend((report.faces.len() as u64).to_le_bytes());
    for face in &report.faces {
        measured_face(face, &mut bytes);
    }
    sweep(&report.signed_sweep, &mut bytes);
    encode_joined_total(&report.total, &mut bytes);
    bytes.extend(report.side_subdivisions.to_le_bytes());
    bytes
}
pub(in crate::sketch_region_prism) fn options(value: AnnularMeasureOptions, bytes: &mut Vec<u8>) {
    bytes.extend(value.version.to_le_bytes());
    encode_joined_integral_options(value.cap_area, bytes);
    encode_joined_integral_options(value.signed_sweep, bytes);
    encode_joined_side_options(value.side_area, bytes);
    bytes.extend(value.maximum_side_subdivisions.to_le_bytes());
    encode_joined_total_options(value.total_area, bytes);
}
fn direction(value: AnnularBoundaryDirection, bytes: &mut Vec<u8>) {
    bytes.extend(value.source.0.to_le_bytes());
    bytes.extend(value.traversed_winding.to_le_bytes());
    bytes.push(u8::from(value.reversed));
    bytes.extend(value.native_winding.to_le_bytes());
    bytes.extend(value.material_side.to_le_bytes());
}
pub(super) fn measured_face(face: &AnnularMeasuredFace, bytes: &mut Vec<u8>) {
    encode_frozen_use_key(bytes, &face.named.chart.occurrence);
    match &face.measure {
        AnnularFaceMeasure::Cap(report) => {
            bytes.push(0);
            encode_joined_plane(report, bytes);
        }
        AnnularFaceMeasure::Side {
            requested,
            effective,
            report,
        } => {
            bytes.push(1);
            encode_joined_side_options(*requested, bytes);
            encode_joined_side_options(*effective, bytes);
            encode_joined_side(report, bytes);
        }
    }
}
pub(super) fn sweep(report: &CertifiedPolynomialMeasure, bytes: &mut Vec<u8>) {
    // The V2 quantity is normalized material boundary × signed exact placed Jacobian.
    bytes.push(2);
    encode_joined_scalar(report.enclosure(), report.absolute_error_bound(), bytes);
    encode_joined_integral_options(report.requested_options(), bytes);
    encode_joined_integral_options(report.effective_options(), bytes);
}
