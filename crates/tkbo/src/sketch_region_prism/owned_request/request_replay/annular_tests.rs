// SPDX-License-Identifier: GPL-2.0-only
//! Independent literal prefix and per-field perturbations: no annular operand is left unbound.
use super::super::super::annular::tests::owned_request;
use super::*;

fn changed(edit: impl FnOnce(&mut OwnedAnnularPrismRequest)) {
    let original = owned_request(false);
    let mut altered = original.clone();
    edit(&mut altered);
    assert_ne!(
        encode_owned_annular_request(&altered),
        encode_owned_annular_request(&original)
    );
}
/// Hand-built little-endian prefix: header, version, operation, then the outer boundary.
#[test]
fn prefix_matches_independent_little_endian_literal() {
    let mut expected = b"Martensite.OwnedAnnularPrismRequest/v1\0".to_vec();
    expected.extend(1u32.to_le_bytes());
    for word in [77u64, 101, 1, 601, 701, 801] {
        expected.extend(word.to_le_bytes());
    }
    expected.push(0);
    expected.extend(4u64.to_le_bytes());
    for value in [0., 0., 0., 4., 0., 0., 0., 4., 0., 0., 0., 0.] {
        expected.extend(f64::to_bits(value).to_le_bytes());
    }
    let actual = encode_owned_annular_request(&owned_request(false));
    assert_eq!(&actual[..expected.len()], &expected[..]);
}
#[test]
fn every_role_name_point_and_identity_is_bound() {
    let edits: [fn(&mut OwnedAnnularPrismRequest); 14] = [
        |r| r.version = 2,
        |r| r.operation.0 += 1,
        |r| r.inner.source.0 += 1,
        |r| r.inner.revision.0 += 1,
        |r| r.inner.reversed_traversal = true,
        |r| r.inner.world_poles[1][0] = -0.,
        |r| r.regions.annulus += 1,
        |r| r.regions.outer_complement_wire += 1,
        |r| r.inner_interior[1] = f64::from_bits(1f64.to_bits() + 1),
        |r| r.material_seed[0] = -r.material_seed[0],
        |r| r.extrusion[2] = f64::NAN,
        |r| std::mem::swap(&mut r.outer, &mut r.inner),
        |r| r.policy.spatial_limit = 2e-8,
        |r| r.policy.region.maximum_poles += 1,
    ];
    for edit in edits {
        changed(edit);
    }
}
#[test]
fn distinct_nan_payloads_remain_distinct() {
    let encode = |bits: u64| {
        let mut request = owned_request(false);
        request.extrusion[0] = f64::from_bits(bits);
        encode_owned_annular_request(&request)
    };
    assert_ne!(encode(0x7ff8_0000_0000_0001), encode(0x7ff8_0000_0000_0002));
    assert_eq!(encode(0x7ff8_0000_0000_0001), encode(0x7ff8_0000_0000_0001));
}
