// SPDX-License-Identifier: GPL-2.0-only
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};
pub struct WireFixture {
    pub name: String,
    controls: Vec<Vec<[f64; 2]>>,
    weights: Vec<Option<Vec<f64>>>,
    intervals: Vec<[f64; 2]>,
    reversed: Vec<bool>,
    line: [[f64; 2]; 2],
    pub expected: BezierWireLocation,
}
impl WireFixture {
    pub fn with_query<T>(&self, check: impl FnOnce(BezierWireQuery<'_>) -> T) -> T {
        let segments: Vec<BezierWireSegment<'_>> = (0..self.controls.len())
            .map(|index| BezierWireSegment {
                id: index as u64,
                controls: &self.controls[index],
                weights: self.weights[index].as_deref(),
                interval: self.intervals[index],
                reversed: self.reversed[index],
            })
            .collect();
        check(BezierWireQuery {
            segments: &segments,
            line: self.line,
            options: RootIsolationOptions::new(0.0001),
        })
    }
}
pub fn corpus() -> Vec<WireFixture> {
    include_str!("../exact_roots/bezier_line/wire/oracle.tsv")
        .lines()
        .map(parse)
        .collect()
}
fn parse(row: &str) -> WireFixture {
    let fields: Vec<&str> = row.split('\t').collect();
    let entries: Vec<Vec<&str>> = fields[1]
        .split('|')
        .map(|entry| entry.split('/').collect())
        .collect();
    let line = decode(fields[2]);
    WireFixture {
        name: fields[0].into(),
        controls: entries
            .iter()
            .map(|e| {
                e[0].split(';')
                    .map(|p| decode(p).try_into().unwrap())
                    .collect()
            })
            .collect(),
        weights: entries
            .iter()
            .map(|e| (e[1] != "-").then(|| decode(e[1])))
            .collect(),
        intervals: entries
            .iter()
            .map(|e| decode(e[2]).try_into().unwrap())
            .collect(),
        reversed: entries.iter().map(|e| e[3] == "1").collect(),
        line: [[line[0], line[1]], [line[2], line[3]]],
        expected: expected(fields[3]),
    }
}
fn decode(text: &str) -> Vec<f64> {
    text.split(',')
        .map(|bits| f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
        .collect()
}
fn expected(text: &str) -> BezierWireLocation {
    if text == "outside" {
        return BezierWireLocation::Outside;
    }
    let (kind, values) = text.split_once(':').unwrap();
    if kind == "inside" {
        return BezierWireLocation::Inside {
            winding: values.parse().unwrap(),
        };
    }
    assert_eq!(kind, "boundary");
    BezierWireLocation::Boundary {
        segment_ids: values
            .split(',')
            .map(|value| value.parse().unwrap())
            .collect(),
    }
}
