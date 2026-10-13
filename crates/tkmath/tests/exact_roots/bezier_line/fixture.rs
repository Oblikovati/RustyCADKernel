// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn with_ray_fixture<T>(row: &str, check: impl FnOnce(BezierLineQuery<'_>) -> T) -> T {
    let fields: Vec<&str> = row.split('\t').collect();
    let controls: Vec<[f64; 2]> = fields[1]
        .split(';')
        .map(|point| decode_bits(point).try_into().unwrap())
        .collect();
    let weights = (fields[2] != "-").then(|| decode_bits(fields[2]));
    let endpoints = decode_bits(fields[3]);
    check(BezierLineQuery {
        controls: &controls,
        weights: weights.as_deref(),
        line: [[endpoints[0], endpoints[1]], [endpoints[2], endpoints[3]]],
        interval: decode_bits(fields[4]).try_into().unwrap(),
        options: options(0.0001),
    })
}
