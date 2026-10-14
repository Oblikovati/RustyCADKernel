// SPDX-License-Identifier: GPL-2.0-only
//! Independent finite pair words exercise equality and multiplicity, never synthetic geometry authority.
use super::*;
use static_assertions::assert_impl_all;

#[derive(Clone, Debug, PartialEq, Eq)]
struct AuthoredContextKey {
    role: u8,
    source: u64,
    ancestry: [u64; 2],
}
const PAIRS: [[usize; 2]; 10] = [
    [0, 0],
    [0, 1],
    [0, 2],
    [0, 3],
    [1, 1],
    [1, 2],
    [1, 3],
    [2, 2],
    [2, 3],
    [3, 3],
];
fn annular_fixture_contexts() -> [AuthoredContextKey; 4] {
    [
        (0, 7, [8, 9]),
        (1, 7, [8, 10]),
        (2, 101, [8, 11]),
        (2, 202, [8, 12]),
    ]
    .map(|(role, source, ancestry)| AuthoredContextKey {
        role,
        source,
        ancestry,
    })
}
fn annular_fixture_pairs(
    keys: &[AuthoredContextKey; 4],
) -> Vec<(&AuthoredContextKey, &AuthoredContextKey)> {
    PAIRS.iter().map(|[a, b]| (&keys[*a], &keys[*b])).collect()
}

#[test]
fn all_ten_unordered_pairs_keep_two_distinct_generated_roles() {
    let keys = annular_fixture_contexts();
    assert_eq!(keys[2].role, keys[3].role);
    assert_ne!(keys[2], keys[3]);
    assert_eq!(
        check_annular_pair_keys(&keys, &annular_fixture_pairs(&keys)),
        Ok(())
    );
}

#[test]
fn pair_and_domain_order_and_pair_direction_do_not_create_identity() {
    let keys = annular_fixture_contexts();
    let mut rows = annular_fixture_pairs(&keys);
    rows.reverse();
    for row in &mut rows {
        std::mem::swap(&mut row.0, &mut row.1);
    }
    let mut reordered = keys.clone();
    reordered.reverse();
    assert_eq!(check_annular_pair_keys(&reordered, &rows), Ok(()));
}

#[test]
fn missing_and_extra_rows_fail_the_bounded_count_before_traversal() {
    let keys = annular_fixture_contexts();
    let mut rows = annular_fixture_pairs(&keys);
    rows.pop();
    assert_row_count(&keys, &rows, 9);
    rows.extend([rows[0], rows[0]]);
    assert_row_count(&keys, &rows, 11);
}

fn assert_row_count(
    keys: &[AuthoredContextKey; 4],
    rows: &[(&AuthoredContextKey, &AuthoredContextKey)],
    actual: usize,
) {
    assert_eq!(
        check_annular_pair_keys(keys, rows),
        Err(AnnularPairCoverageFailure::RowCount {
            actual,
            expected: 10
        })
    );
}

#[test]
fn every_duplicate_pair_refuses_even_when_row_count_is_ten() {
    let keys = annular_fixture_contexts();
    let expected = annular_fixture_pairs(&keys);
    for replaced in 0..10 {
        for replacement in (0..10).filter(|slot| *slot != replaced) {
            assert_duplicate_pair(&keys, &expected, replaced, replacement);
        }
    }
}

fn assert_duplicate_pair(
    keys: &[AuthoredContextKey; 4],
    expected: &[(&AuthoredContextKey, &AuthoredContextKey)],
    replaced: usize,
    replacement: usize,
) {
    let mut rows = expected.to_vec();
    rows[replaced] = rows[replacement];
    assert!(matches!(
        check_annular_pair_keys(keys, &rows),
        Err(AnnularPairCoverageFailure::PairMultiplicity {
            actual: 0 | 2,
            expected: 1,
            ..
        })
    ));
}

#[test]
fn repeated_domains_cannot_hide_behind_distinct_row_positions() {
    let mut keys = annular_fixture_contexts();
    keys[3] = keys[2].clone();
    assert_eq!(
        check_annular_pair_keys(&keys, &annular_fixture_pairs(&keys)),
        Err(AnnularPairCoverageFailure::RepeatedDomain {
            key: Box::new(keys[2].clone()),
            actual: 2,
            expected: 1,
        })
    );
}

#[test]
fn source_and_full_ancestry_mutations_are_foreign_pair_keys() {
    let keys = annular_fixture_contexts();
    let mut foreign_source = keys[2].clone();
    foreign_source.source = 303;
    let mut foreign_path = keys[2].clone();
    foreign_path.ancestry[1] = 99;
    for foreign in [&foreign_source, &foreign_path] {
        assert_foreign_pair(&keys, foreign);
    }
}

fn assert_foreign_pair(keys: &[AuthoredContextKey; 4], foreign: &AuthoredContextKey) {
    let mut rows = annular_fixture_pairs(keys);
    rows[0] = (&keys[0], foreign);
    assert_eq!(
        check_annular_pair_keys(keys, &rows),
        Err(AnnularPairCoverageFailure::ForeignPair {
            first: Box::new(keys[0].clone()),
            second: Box::new(foreign.clone()),
        })
    );
}

#[test]
fn immutable_pair_checks_match_across_workers_and_repeated_calls() {
    assert_impl_all!(AnnularPairCoverageFailure<AuthoredContextKey>: Send, Sync);
    let shared = std::sync::Arc::new(annular_fixture_contexts());
    let workers: Vec<_> = (0..4)
        .map(|_| spawn_pair_reader(std::sync::Arc::clone(&shared)))
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
}

fn spawn_pair_reader(
    shared: std::sync::Arc<[AuthoredContextKey; 4]>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for _ in 0..32 {
            assert_eq!(
                check_annular_pair_keys(&shared, &annular_fixture_pairs(&shared)),
                Ok(())
            );
        }
    })
}
