// SPDX-License-Identifier: GPL-2.0-only
use super::*;
/// Check owned annular directions; `inspect_annular_orientation(join,options,cancel)` captures no live geometry.
/// Publication still requires every ordered stage and the existing final live-source rebind.
pub(in crate::sketch_region_prism) fn inspect_annular_orientation(
    join: &AnnularFrozenDomain,
    requested: AnnularOrientationOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<AnnularOrientationCertificate, Box<AnnularDirectionFailure>> {
    let mut phase = AnnularDirectionPhase::Admission;
    construct(join, requested, &mut phase, &mut cancelled).map_err(|cause| {
        Box::new(AnnularDirectionFailure {
            operation: join.projection().operation,
            requested,
            phase,
            cause,
            source_binding: join.replay_bytes().into(),
        })
    })
}
fn construct(
    join: &AnnularFrozenDomain,
    requested: AnnularOrientationOptions,
    phase: &mut AnnularDirectionPhase,
    cancelled: &mut impl FnMut() -> bool,
) -> AnnularDirectionCheck<AnnularOrientationCertificate> {
    admit(requested)?;
    poll(cancelled)?;
    *phase = AnnularDirectionPhase::Winding;
    let boundaries = annular_boundary_directions(join)?;
    let jacobian_sign = prism_determinant_sign(join.embedding().binding().signed_jacobian())
        .map_err(|_| AnnularDirectionCause::ZeroJacobian)?;
    *phase = AnnularDirectionPhase::Directions;
    let faces = faces::inspect(join, &boundaries, jacobian_sign, cancelled)?;
    *phase = AnnularDirectionPhase::Commit;
    poll(cancelled)?;
    Ok(AnnularOrientationCertificate {
        requested,
        binding: join.replay_bytes().into(),
        jacobian_sign,
        boundaries,
        faces,
    })
}
fn admit(requested: AnnularOrientationOptions) -> AnnularDirectionCheck<()> {
    if requested.version == 2 {
        return Ok(());
    }
    Err(AnnularDirectionCause::Version {
        actual: requested.version,
        expected: 2,
    })
}
pub(super) fn poll(cancelled: &mut impl FnMut() -> bool) -> AnnularDirectionCheck<()> {
    if cancelled() {
        return Err(AnnularDirectionCause::Cancelled);
    }
    Ok(())
}
