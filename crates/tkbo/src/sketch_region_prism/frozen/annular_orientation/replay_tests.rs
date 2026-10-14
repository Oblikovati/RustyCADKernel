// SPDX-License-Identifier: GPL-2.0-only
use super::super::annular_join::operation_tests::joined;
use super::*;
fn proof() -> AnnularOrientationCertificate {
    inspect_annular_orientation(&joined(), AnnularOrientationOptions::v2(), || false)
        .unwrap_or_else(|_| panic!("actual orientation"))
}
#[test]
fn every_retained_conclusion_operand_changes_full_replay() {
    let proof = proof();
    let before = proof.replay_bytes();
    for change in edits() {
        let mut changed = clone_proof(&proof);
        change(&mut changed);
        assert_ne!(changed.replay_bytes(), before);
    }
}
fn clone_proof(value: &AnnularOrientationCertificate) -> AnnularOrientationCertificate {
    AnnularOrientationCertificate {
        requested: value.requested,
        binding: value.binding.clone(),
        jacobian_sign: value.jacobian_sign,
        boundaries: value.boundaries,
        faces: value.faces.clone(),
    }
}
fn edit_version(v: &mut AnnularOrientationCertificate) {
    v.requested.version += 1;
}
fn edit_binding(v: &mut AnnularOrientationCertificate) {
    v.binding[0] ^= 1;
}
fn edit_jacobian(v: &mut AnnularOrientationCertificate) {
    v.jacobian_sign = -v.jacobian_sign;
}
fn edit_winding(v: &mut AnnularOrientationCertificate) {
    v.boundaries[0].traversed_winding = -1;
}
fn edit_traversal(v: &mut AnnularOrientationCertificate) {
    v.boundaries[0].reversed = true;
}
fn edit_material(v: &mut AnnularOrientationCertificate) {
    v.boundaries[1].material_side = 1;
}
fn edit_face_direction(v: &mut AnnularOrientationCertificate) {
    v.faces[0].expected = Orientation::Internal;
}
fn edit_face_source(v: &mut AnnularOrientationCertificate) {
    v.faces[0].named.chart.source.0 += 1;
}
fn edit_trim_interval(v: &mut AnnularOrientationCertificate) {
    v.faces[0].trims[0].named.interval[0] = -0.;
}
fn edit_trim_boundary(v: &mut AnnularOrientationCertificate) {
    v.faces[0].trims[0].boundary.0 += 1;
}
fn edit_parent_multiplicity(v: &mut AnnularOrientationCertificate) {
    let p = v.faces[0].named.chart.parents[0].clone();
    v.faces[0].named.chart.parents.push(p);
}
fn edit_wire_source(v: &mut AnnularOrientationCertificate) {
    if let NamedAnnularDomain::Cap { outer, .. } = &mut v.faces[0].named.domain {
        outer.source.0 += 1;
    } else {
        panic!("first authored face cap");
    }
}
fn edit_wire_ancestry(v: &mut AnnularOrientationCertificate) {
    if let NamedAnnularDomain::Cap { inner, .. } = &mut v.faces[0].named.domain {
        inner.occurrence.0[0].1.0 += 1;
    } else {
        panic!("first authored face cap");
    }
}
fn edit_trim_role(v: &mut AnnularOrientationCertificate) {
    v.faces[0].trims[0].named.role = PrismTrimRole::End;
}
#[test]
fn failure_tags_and_original_payloads_are_distinct() {
    let source = joined();
    let key = Box::new(source.faces()[0].chart.occurrence.clone());
    let causes = refusal_examples(key);
    let mut bytes: Vec<_> = causes
        .into_iter()
        .map(|cause| {
            AnnularDirectionFailure {
                operation: AuthoredPrismId(77),
                requested: AnnularOrientationOptions::v2(),
                phase: AnnularDirectionPhase::Directions,
                cause,
                source_binding: source.replay_bytes().into(),
            }
            .replay_bytes()
        })
        .collect();
    bytes.sort();
    bytes.dedup();
    assert_eq!(bytes.len(), 9);
}

fn edits() -> [fn(&mut AnnularOrientationCertificate); 14] {
    [
        edit_version,
        edit_binding,
        edit_jacobian,
        edit_winding,
        edit_traversal,
        edit_material,
        edit_face_direction,
        edit_face_source,
        edit_trim_interval,
        edit_trim_boundary,
        edit_parent_multiplicity,
        edit_wire_source,
        edit_wire_ancestry,
        edit_trim_role,
    ]
}

fn refusal_examples(key: Box<FrozenUseKey>) -> Vec<AnnularDirectionCause> {
    let mut causes = scalar_refusals();
    causes.extend([
        AnnularDirectionCause::Unsupported {
            occurrence: key.clone(),
            actual: Orientation::Internal,
        },
        AnnularDirectionCause::Direction {
            occurrence: key.clone(),
            actual: Orientation::Reversed,
            expected: Orientation::Forward,
        },
        AnnularDirectionCause::Context {
            occurrence: key,
            required: "full wire",
        },
        AnnularDirectionCause::ZeroJacobian,
        AnnularDirectionCause::Cancelled,
    ]);
    causes
}
fn scalar_refusals() -> Vec<AnnularDirectionCause> {
    vec![
        AnnularDirectionCause::Version {
            actual: 9,
            expected: 2,
        },
        AnnularDirectionCause::Count {
            subject: "cap",
            actual: 1,
            expected: 2,
        },
        AnnularDirectionCause::Winding {
            source: RegionSourceId(101),
            actual: Some(-2),
        },
        AnnularDirectionCause::Winding {
            source: RegionSourceId(101),
            actual: None,
        },
    ]
}
