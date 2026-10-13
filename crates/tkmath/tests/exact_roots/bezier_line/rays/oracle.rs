// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn ray_signs_match_independent_exact_algebraic_comparisons() {
    let corpus = include_str!("oracle.tsv");
    assert_eq!(corpus.lines().count(), 16);
    let mut count = 0;
    for row in corpus.lines() {
        count += check_ray_fixture(row).signs().len();
    }
    assert_eq!(count, 18);
}
fn check_ray_fixture(row: &str) -> RootSignEvaluation {
    let fields: Vec<&str> = row.split('\t').collect();
    let controls: Vec<[f64; 2]> = fields[1]
        .split(';')
        .map(|point| decode_bits(point).try_into().unwrap())
        .collect();
    let weights = (fields[2] != "-").then(|| decode_bits(fields[2]));
    let endpoints = decode_bits(fields[3]);
    let request = BezierLineQuery {
        controls: &controls,
        weights: weights.as_deref(),
        line: [[endpoints[0], endpoints[1]], [endpoints[2], endpoints[3]]],
        interval: decode_bits(fields[4]).try_into().unwrap(),
        options: options(0.0001),
    };
    let expected: Vec<Ordering> = fields[5]
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<i32>().unwrap().cmp(&0))
        .collect();
    let result = directed(request);
    assert_eq!(result.signs(), expected, "{}", fields[0]);
    result
}
#[test]
fn ray_ordinary_sides_match_retained_occt_receipts() {
    let receipts = include_str!("occt-8.0.0.tsv");
    assert_eq!(receipts.lines().count(), 6);
    for receipt in receipts.lines() {
        let fields: Vec<&str> = receipt.split('\t').collect();
        let row = include_str!("oracle.tsv")
            .lines()
            .find(|row| row.split('\t').next() == Some(fields[0]))
            .unwrap();
        let result = check_ray_fixture(row);
        assert_eq!(fields[1], "0");
        let native: Vec<Ordering> = fields[2]
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<i32>().unwrap().cmp(&0))
            .collect();
        assert_eq!(result.signs(), native);
    }
}
