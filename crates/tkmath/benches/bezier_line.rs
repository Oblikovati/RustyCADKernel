// SPDX-License-Identifier: GPL-2.0-only
//! Replay accepted exact curve/line inputs; pass --rays to certify directed sides too.
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};
use std::{cmp::Ordering, time::Instant};
struct ContactCase {
    controls: Vec<[f64; 2]>,
    weights: Option<Vec<f64>>,
    line: [[f64; 2]; 2],
    interval: [f64; 2],
    expected: ExpectedContacts,
}
enum ExpectedContacts {
    Line(usize),
    Ray(Vec<Ordering>),
    Winding(BezierWinding),
}
fn values(text: &str) -> Vec<f64> {
    text.split(',')
        .map(|bits| f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
        .collect()
}
fn corpus(rays: bool, winding: bool) -> Vec<ContactCase> {
    let text = if rays {
        include_str!("../tests/exact_roots/bezier_line/rays/oracle.tsv")
    } else {
        include_str!("../tests/exact_roots/bezier_line/oracle.tsv")
    };
    text.lines().map(|row| case(row, rays, winding)).collect()
}
fn case(row: &str, rays: bool, winding: bool) -> ContactCase {
    let fields: Vec<&str> = row.split('\t').collect();
    let endpoints = values(fields[3]);
    ContactCase {
        controls: fields[1]
            .split(';')
            .map(|p| values(p).try_into().unwrap())
            .collect(),
        weights: (fields[2] != "-").then(|| values(fields[2])),
        line: [[endpoints[0], endpoints[1]], [endpoints[2], endpoints[3]]],
        interval: if rays {
            values(fields[4]).try_into().unwrap()
        } else {
            [0., 1.]
        },
        expected: if winding {
            winding_expected(fields[0])
        } else {
            expected_contacts(&fields, rays)
        },
    }
}
fn expected_contacts(fields: &[&str], rays: bool) -> ExpectedContacts {
    if rays {
        return ExpectedContacts::Ray(
            fields[5]
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<i32>().unwrap().cmp(&0))
                .collect(),
        );
    }
    ExpectedContacts::Line(fields[4].split('|').filter(|root| !root.is_empty()).count())
}
fn query(case: &ContactCase) -> usize {
    let request = BezierLineQuery {
        controls: &case.controls,
        weights: case.weights.as_deref(),
        line: case.line,
        interval: case.interval,
        options: RootIsolationOptions::new(0.0001),
    };
    match &case.expected {
        ExpectedContacts::Line(expected) => line_query(request, *expected),
        ExpectedContacts::Ray(expected) => ray_query(request, expected),
        ExpectedContacts::Winding(expected) => winding_query(request, *expected),
    }
}
fn line_query(request: BezierLineQuery<'_>, expected: usize) -> usize {
    let result = intersect_bezier_line(request, || false).unwrap();
    let BezierLineContact::Isolated(roots) = result else {
        panic!("expected isolated contacts");
    };
    assert_eq!(roots.roots().len(), expected);
    std::hint::black_box(roots).roots().len()
}
fn ray_query(request: BezierLineQuery<'_>, expected: &[Ordering]) -> usize {
    let result = classify_bezier_ray_contacts(request, || false).unwrap();
    assert_eq!(result.signs(), expected);
    std::hint::black_box(result).signs().len()
}
fn run(cases: &[ContactCase], repetitions: usize) -> usize {
    let mut count = 0;
    for _ in 0..repetitions {
        for case in cases {
            count += query(case);
        }
    }
    count
}
fn main() {
    let winding = std::env::args().any(|arg| arg == "--winding");
    let rays = winding || std::env::args().any(|arg| arg == "--rays");
    let cases = corpus(rays, winding);
    run(&cases, 1);
    let start = Instant::now();
    let contacts = run(&cases, 1000);
    println!(
        "{{\"rays\":{rays},\"winding\":{winding},\"requests\":{},\"observations\":{contacts},\"elapsed_ns\":{}}}",
        cases.len() * 1000,
        start.elapsed().as_nanos()
    );
}

fn winding_expected(name: &str) -> ExpectedContacts {
    let row = include_str!("../tests/exact_roots/bezier_line/winding/oracle.tsv")
        .lines()
        .find(|row| row.split('\t').next() == Some(name))
        .unwrap();
    let value = row.split_once('\t').unwrap().1;
    ExpectedContacts::Winding(if value == "boundary" {
        BezierWinding::Boundary
    } else {
        BezierWinding::Clear(value.parse().unwrap())
    })
}
fn winding_query(request: BezierLineQuery<'_>, expected: BezierWinding) -> usize {
    let result = bezier_winding_contribution(request, || false).unwrap();
    assert_eq!(result, expected);
    std::hint::black_box(result);
    1
}
