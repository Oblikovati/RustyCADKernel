// SPDX-License-Identifier: GPL-2.0-only
//! Generated near-degenerate offset and periodic-spline cases against the exact
//! Fraction oracle (`oracle_offsets.py`, ADR 0080).
use super::generated_curves::check_row;

#[test]
fn generated_offset_cases_match_the_exact_oracle() {
    let rows: Vec<&str> = include_str!("oracle-offsets.csv").lines().collect();
    assert_eq!(rows.len(), 211);
    for row in rows {
        check_row(row);
    }
}
