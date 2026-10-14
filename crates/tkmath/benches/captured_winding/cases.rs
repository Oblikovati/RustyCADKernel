// SPDX-License-Identifier: GPL-2.0-only
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::BezierWireLocation};
pub struct OwnedWindingSegment {
    pub physical: u64,
    pub controls: Vec<[f64; 2]>,
    pub weights: Option<Vec<f64>>,
}
pub struct WindingReuseWorkload {
    pub name: &'static str,
    pub segments: Vec<OwnedWindingSegment>,
    pub line: [[f64; 2]; 2],
    pub expected: [BezierWireLocation; 2],
    pub options: RootIsolationOptions,
}
pub fn accepted_workloads() -> Vec<WindingReuseWorkload> {
    vec![closed_cubic(), shared_polygon(), rational_tangency()]
}
fn closed_cubic() -> WindingReuseWorkload {
    let segment = OwnedWindingSegment {
        physical: 7,
        weights: None,
        controls: vec![[0., 0.], [4., 0.], [0., 4.], [0., 0.]],
    };
    WindingReuseWorkload {
        name: "opposite-cubic-faces",
        segments: vec![segment],
        line: [[0.5, 0.5], [1.5, 0.5]],
        expected: oriented_interior(),
        options: RootIsolationOptions::new(0.001),
    }
}
fn shared_polygon() -> WindingReuseWorkload {
    let points = [[-2., -2.], [2., -2.], [2., 2.], [-2., 2.], [-2., -2.]];
    let segments = points
        .windows(2)
        .enumerate()
        .map(|(index, controls)| OwnedWindingSegment {
            physical: index as u64,
            controls: controls.to_vec(),
            weights: None,
        })
        .collect();
    WindingReuseWorkload {
        name: "shared-polygon-edges",
        segments,
        line: [[0., 0.], [1., 0.]],
        expected: oriented_interior(),
        options: RootIsolationOptions::new(0.001),
    }
}
fn rational_tangency() -> WindingReuseWorkload {
    let segment = OwnedWindingSegment {
        physical: 9,
        controls: vec![[1., 1.], [1., -1.], [1., 1.]],
        weights: Some(vec![-1.; 3]),
    };
    WindingReuseWorkload {
        name: "regular-rational-tangency",
        segments: vec![segment],
        line: [[0., 0.], [1., 0.]],
        expected: [BezierWireLocation::Outside, BezierWireLocation::Outside],
        options: RootIsolationOptions::new(0.001),
    }
}
fn oriented_interior() -> [BezierWireLocation; 2] {
    [
        BezierWireLocation::Inside { winding: 1 },
        BezierWireLocation::Inside { winding: -1 },
    ]
}
