// SPDX-License-Identifier: GPL-2.0-only
//! Authored source IDs select each boundary; coefficients never supply identity.
use super::*;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_prism::surface_charts::*;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use std::collections::BTreeSet;

pub(super) fn assert_charts(receipt: &PrismReceipt, charts: &PrismSurfaceInspection) {
    for role in [PrismImageRole::First, PrismImageRole::Last] {
        let cap = by_role(charts, role);
        assert_eq!(cap.len(), 1);
        assert_cap(receipt, cap[0], role);
    }
    let sides = by_role(charts, PrismImageRole::Generated);
    assert_eq!(sides.len(), 2);
    for (source, poles) in authored_poles() {
        assert_source_side(receipt, &sides, source, &poles);
    }
}
fn assert_source_side(
    receipt: &PrismReceipt,
    sides: &[&PrismSurfaceEntry],
    source: u64,
    poles: &[[f64; 3]; 4],
) {
    let matches: Vec<_> = sides
        .iter()
        .filter(|s| parent_sources(s) == BTreeSet::from([source]))
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "source {source}: expected one actual generated side"
    );
    assert_side(receipt, matches[0], poles);
}

fn by_role(charts: &PrismSurfaceInspection, role: PrismImageRole) -> Vec<&PrismSurfaceEntry> {
    charts.entries.iter().filter(|c| c.role == role).collect()
}
fn parent_sources(chart: &PrismSurfaceEntry) -> BTreeSet<u64> {
    chart.parents.iter().map(|p| p.source.0).collect()
}
fn authored_poles() -> [(u64, [[f64; 3]; 4]); 2] {
    [
        (
            101,
            [[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 0.]],
        ),
        (
            202,
            [
                [0.5, 0.5, 0.],
                [2.5, 0.5, 0.],
                [0.5, 2.5, 0.],
                [0.5, 0.5, 0.],
            ],
        ),
    ]
}
fn capture(entry: &PrismSurfaceEntry) -> &CapturedSurfaceChart {
    assert!(
        entry.decision.is_ok(),
        "actual annular chart refusal: {:?}",
        entry.decision
    );
    let chart = entry.captured.as_ref().expect("captured actual chart");
    assert_eq!(entry.proof.trim_roles.len(), chart.trims.len());
    let pcurves = entry
        .proof
        .identities
        .iter()
        .filter(|s| **s == PrismChartStage::TrimPCurve)
        .count();
    assert_eq!(pcurves, chart.trims.len());
    chart
}
fn assert_cap(receipt: &PrismReceipt, entry: &PrismSurfaceEntry, role: PrismImageRole) {
    let chart = capture(entry);
    assert_eq!(parent_sources(entry), BTreeSet::from([101, 202]));
    assert_wire_count(receipt, entry, 2);
    assert_eq!(chart.trims.len(), 2);
    assert_cap_plane(chart, role);
    assert_cap_trims(&chart.trims);
}
fn assert_cap_plane(chart: &CapturedSurfaceChart, role: PrismImageRole) {
    let CapturedPrismSurface::Plane(plane) = &chart.actual else {
        panic!("expected cap plane")
    };
    assert_eq!(plane.frame.origin, [0.; 3]);
    let height = if role == PrismImageRole::First {
        0.
    } else {
        2.
    };
    assert_eq!(plane.placement.origin, [0., 0., height]);
}
fn assert_cap_trims(trims: &[CapturedPrismTrim]) {
    for trim in trims {
        assert_eq!(trim.interval, [0., 1.]);
        assert_eq!(
            trim.parents
                .iter()
                .map(|p| p.source.0)
                .collect::<BTreeSet<_>>()
                .len(),
            1
        );
    }
}
fn assert_side(receipt: &PrismReceipt, entry: &PrismSurfaceEntry, poles: &[[f64; 3]; 4]) {
    let chart = capture(entry);
    assert_wire_count(receipt, entry, 1);
    assert_eq!(chart.trims.len(), 4);
    assert_eq!(entry.proof.side_height, Some(2.));
    assert_side_surface(&chart.actual, poles);
    assert_side_trims(&chart.trims);
    assert_seam(&chart.trims);
}
fn assert_side_surface(actual: &CapturedPrismSurface, poles: &[[f64; 3]; 4]) {
    let CapturedPrismSurface::Extrusion {
        basis, direction, ..
    } = actual
    else {
        panic!("expected side extrusion")
    };
    let CapturedMetricCurve::Bezier {
        poles: actual,
        weights,
    } = basis
    else {
        panic!("expected polynomial side")
    };
    assert_eq!(actual, poles);
    assert!(weights.is_none());
    assert_eq!(*direction, [0., 0., -1.]);
}
fn assert_side_trims(trims: &[CapturedPrismTrim]) {
    use PrismTrimRole::*;
    for (role, origin, direction, end) in [
        (First, [0., 0.], [1., 0.], 1.),
        (Last, [0., -2.], [1., 0.], 1.),
        (Start, [0., 0.], [0., -1.], 2.),
        (End, [1., 0.], [0., -1.], 2.),
    ] {
        let matches: Vec<_> = trims.iter().filter(|t| t.role == role).collect();
        assert_eq!(matches.len(), 1);
        let trim = matches[0];
        assert_eq!(trim.interval, [0., end]);
        assert!(
            matches!(trim.pcurve, CapturedMetricCurve::Line {origin:o,direction:d} if o==origin && d==direction)
        );
    }
}
fn assert_seam(trims: &[CapturedPrismTrim]) {
    let start = trims
        .iter()
        .find(|t| t.role == PrismTrimRole::Start)
        .unwrap();
    let end = trims.iter().find(|t| t.role == PrismTrimRole::End).unwrap();
    assert!(start.output.same_physical(&end.output));
    assert_ne!(start.output, end.output);
    let orientations = [start.output.orientation, end.output.orientation];
    assert!(orientations.contains(&Orientation::Forward));
    assert!(orientations.contains(&Orientation::Reversed));
}
fn assert_wire_count(receipt: &PrismReceipt, entry: &PrismSurfaceEntry, expected: usize) {
    let diagnostic = receipt.diagnostic.as_ref().unwrap();
    let faces = super::super::surface_charts::actual_faces(diagnostic)
        .unwrap_or_else(|_| panic!("expected current faces"));
    let face = faces
        .iter()
        .find(|f| {
            crate::sketch_region::PartitionOccurrenceIdentity::capture(&f.occurrence)
                == entry.output
        })
        .unwrap();
    let contexts = super::super::contexts::body_contexts(&face.occurrence)
        .unwrap_or_else(|_| panic!("expected actual face contexts"));
    let wires = contexts
        .iter()
        .filter(|c| c.occurrence.shape_type().unwrap() == ShapeEnum::Wire)
        .count();
    assert_eq!(wires, expected);
}
