// SPDX-License-Identifier: GPL-2.0-only
//! ADR0063: physical volume of the named annular prism from its error-bounded signed sweep and the
//! exact orientation certificate of the same join. The sweep integrates both boundaries normalized
//! to material-left (outer native winding x +1, inner x -1), so its exact sign is the placed
//! Jacobian sign; transport x -> +/-x is an isometry and keeps the certified error unchanged.
mod types;
use super::annular_join::AnnularFrozenDomain;
use super::annular_measures::AnnularPrismMeasures;
use super::annular_orientation::{AnnularBoundaryDirection, AnnularOrientationCertificate};
use super::joined_volume::{JoinedPrismVolumeCause, VolumeCheckpoint, transport_volume_scalar};
use super::proof_join::encode_joined_block;
pub(in crate::sketch_region_prism) use types::*;

#[cfg(test)]
mod tests;

/// Certify the body volume; `certify_annular_prism_volume(join, orientation, measures, 1, cancel)`.
/// Foreign bindings or inconsistent sign evidence are incomplete; cancellation yields no volume.
pub(in crate::sketch_region_prism) fn certify_annular_prism_volume(
    join: &AnnularFrozenDomain,
    orientation: &AnnularOrientationCertificate,
    measures: &AnnularPrismMeasures,
    version: u32,
    mut cancelled: impl FnMut() -> bool + Send,
) -> Result<AnnularPrismVolume, Box<AnnularVolumeFailure>> {
    let failure = |cause| failure(join, version, cause);
    admit(join, orientation, measures, version).map_err(failure)?;
    poll(VolumeCheckpoint::Begin, &mut cancelled).map_err(failure)?;
    let volume = transport(join, orientation, measures).map_err(failure)?;
    poll(VolumeCheckpoint::Finish, &mut cancelled).map_err(failure)?;
    Ok(volume)
}
fn admit(
    join: &AnnularFrozenDomain,
    orientation: &AnnularOrientationCertificate,
    measures: &AnnularPrismMeasures,
    version: u32,
) -> Result<(), AnnularVolumeCause> {
    if version != 1 {
        return Err(AnnularVolumeCause::Version {
            actual: version,
            expected: 1,
        });
    }
    if !orientation.binds(join) {
        return Err(AnnularVolumeCause::Binding(
            AnnularVolumeOperand::Orientation,
        ));
    }
    if !measures.matches(join) {
        return Err(AnnularVolumeCause::Binding(AnnularVolumeOperand::Measures));
    }
    // The sweep's material normalization must be exactly the certified boundary directions.
    if measures.boundaries() != orientation.boundaries() {
        return Err(AnnularVolumeCause::Binding(
            AnnularVolumeOperand::Boundaries,
        ));
    }
    Ok(())
}
fn poll(
    checkpoint: VolumeCheckpoint,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), AnnularVolumeCause> {
    match cancelled() {
        true => Err(AnnularVolumeCause::Cancelled(checkpoint)),
        false => Ok(()),
    }
}
fn transport(
    join: &AnnularFrozenDomain,
    orientation: &AnnularOrientationCertificate,
    measures: &AnnularPrismMeasures,
) -> Result<AnnularPrismVolume, AnnularVolumeCause> {
    let boundaries = *orientation.boundaries();
    let sign = exact_sign(orientation.jacobian_sign(), &boundaries)?;
    let source = measures.signed_sweep().clone();
    let (enclosure, representative) =
        transport_volume_scalar(&source, sign).map_err(|cause| match cause {
            JoinedPrismVolumeCause::ScalarContradiction {
                signed_enclosure,
                exact_sign,
            } => AnnularVolumeCause::ScalarContradiction {
                signed_enclosure,
                exact_sign,
            },
            _ => unreachable!("scalar transport reports contradictions only"),
        })?;
    let mut volume = AnnularPrismVolume {
        jacobian_sign: sign,
        boundaries,
        source,
        enclosure,
        representative,
        binding: join.replay_bytes().into(),
        replay: Vec::new(),
    };
    volume.replay = encode(&volume, orientation, measures);
    Ok(volume)
}
/// Outer must be material-left with native winding +/-1, inner material-right likewise.
fn exact_sign(
    jacobian: i8,
    boundaries: &[AnnularBoundaryDirection; 2],
) -> Result<i8, AnnularVolumeCause> {
    let windings = boundaries
        .iter()
        .all(|b| matches!(b.native_winding, -1 | 1));
    let sides = [boundaries[0].material_side, boundaries[1].material_side] == [1, -1];
    if !matches!(jacobian, -1 | 1) || !windings || !sides {
        return Err(AnnularVolumeCause::SignFactors {
            jacobian,
            boundaries: *boundaries,
        });
    }
    Ok(jacobian)
}
fn encode(
    volume: &AnnularPrismVolume,
    orientation: &AnnularOrientationCertificate,
    measures: &AnnularPrismMeasures,
) -> Vec<u8> {
    let mut bytes = b"Martensite.AnnularPrismVolume/v1\0".to_vec();
    bytes.push(volume.jacobian_sign as u8);
    for boundary in &volume.boundaries {
        bytes.extend(boundary.source.0.to_le_bytes());
        bytes.extend([boundary.native_winding as u8, boundary.material_side as u8]);
    }
    let scalars = [
        volume.enclosure[0],
        volume.enclosure[1],
        volume.representative,
        volume.absolute_error_bound(),
    ];
    bytes.extend(scalars.map(|v| v.to_bits().to_le_bytes()).concat());
    encode_joined_block(&volume.binding, &mut bytes);
    encode_joined_block(&orientation.replay_bytes(), &mut bytes);
    encode_joined_block(measures.replay_bytes(), &mut bytes);
    bytes
}
fn failure(
    join: &AnnularFrozenDomain,
    version: u32,
    cause: AnnularVolumeCause,
) -> Box<AnnularVolumeFailure> {
    Box::new(AnnularVolumeFailure {
        operation: join.projection().operation,
        requested_version: version,
        cause,
    })
}
