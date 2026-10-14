// SPDX-License-Identifier: GPL-2.0-only
use super::super::annular_join::operation_tests::joined;
use super::*;
use Orientation::{Forward as F, Reversed as R};

fn certified(join: &AnnularFrozenDomain) -> AnnularOrientationCertificate {
    inspect_annular_orientation(join, AnnularOrientationOptions::v2(), || false).unwrap_or_else(
        |failure| panic!("annular direction {:?}: {:?}", failure.phase, failure.cause),
    )
}
#[test]
fn actual_annular_domain_has_independent_outer_and_inner_directions() {
    let join = joined();
    let proof = certified(&join);
    assert!(proof.binds(&join));
    assert_eq!(proof.source_binding(), join.replay_bytes());
    assert_nominal_counts(&proof);
    for face in proof.faces() {
        assert_nominal_face(face, proof.boundaries());
    }
    assert!(!join.projection().is_valid_body());
    static_assertions::assert_impl_all!(AnnularOrientationCertificate: Send, Sync);
    static_assertions::assert_impl_all!(AnnularDirectionFailure: Send, Sync);
}
fn assert_nominal_face(row: &AnnularFaceDirection, boundaries: &[AnnularBoundaryDirection; 2]) {
    let expected = match &row.named().domain {
        NamedAnnularDomain::Cap { .. } => {
            if row.named().chart.role == PrismImageRole::First {
                R
            } else {
                F
            }
        }
        NamedAnnularDomain::Side { boundary } => {
            if boundary.source == boundaries[0].source {
                R
            } else {
                F
            }
        }
    };
    assert_eq!(row.orientations(), (expected, expected));
    for trim in row.trims() {
        assert_nominal_trim(row, trim, boundaries);
    }
}
fn assert_nominal_trim(
    face: &AnnularFaceDirection,
    trim: &AnnularTrimDirection,
    boundaries: &[AnnularBoundaryDirection; 2],
) {
    let outer = trim.boundary() == boundaries[0].source;
    let expected = match face.named().chart.role {
        PrismImageRole::First => {
            if outer {
                R
            } else {
                F
            }
        }
        PrismImageRole::Last => {
            if outer {
                F
            } else {
                R
            }
        }
        PrismImageRole::Generated => nominal_side_trim(outer, trim.named().role),
    };
    assert_eq!(trim.orientations(), (expected, expected));
}

fn assert_nominal_counts(proof: &AnnularOrientationCertificate) {
    assert_eq!(
        proof
            .boundaries()
            .map(|row| (row.native_winding, row.material_side)),
        [(1, 1), (1, -1)]
    );
    assert_eq!(proof.jacobian_sign(), 1);
    assert_eq!(proof.faces().len(), 4);
    assert_eq!(
        proof
            .faces()
            .iter()
            .map(|row| row.trims().len())
            .sum::<usize>(),
        12
    );
}

fn nominal_side_trim(outer: bool, role: PrismTrimRole) -> Orientation {
    match (outer, role) {
        (true, PrismTrimRole::First | PrismTrimRole::End)
        | (false, PrismTrimRole::Last | PrismTrimRole::Start) => F,
        _ => R,
    }
}
