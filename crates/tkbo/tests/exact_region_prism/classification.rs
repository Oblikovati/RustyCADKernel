// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkg2d::geom2d::Geom2dBezierCurve;
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions,
    bezier_line::{BezierWireLocation, BezierWireQuery, BezierWireSegment, classify_bezier_wire},
};
struct ExtractedCubic {
    controls: Vec<[f64; 2]>,
    interval: [f64; 2],
    reversed: bool,
}
fn extract_cubic(shape: &Shape, scale: f64) -> ExtractedCubic {
    let edges = shapes(shape, ShapeEnum::Edge);
    assert_eq!(edges.extent(), 1);
    let segment = edge(edges.find_key(1).unwrap()).unwrap();
    let pcurve = BRepTool::curve_on_face(&segment, &face(shape).unwrap()).unwrap();
    let controls = cubic_controls(&pcurve.pcurve, scale);
    assert_eq!([pcurve.first, pcurve.last], [0., 1.]);
    ExtractedCubic {
        controls,
        interval: [pcurve.first, pcurve.last],
        reversed: segment.orientation() == Orientation::Reversed,
    }
}
fn cubic_controls(
    pcurve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    scale: f64,
) -> Vec<[f64; 2]> {
    let curve = pcurve.cast::<Geom2dBezierCurve>();
    let curve = curve.get().expect("pcurve: expected Bézier representation");
    assert!(!curve.is_rational());
    let controls: Vec<[f64; 2]> = (1..=curve.nb_poles())
        .map(|index| {
            let pole = curve.pole(index).unwrap();
            [pole.x(), pole.y()]
        })
        .collect();
    assert_eq!(
        controls,
        [[0., 0.], [4. * scale, 0.], [0., 4. * scale], [0., 0.]]
    );
    controls
}
fn seed_location(cubic: &ExtractedCubic, seed: [f64; 2], scale: f64) -> BezierWireLocation {
    let segments = [BezierWireSegment {
        id: 1,
        controls: &cubic.controls,
        weights: None,
        interval: cubic.interval,
        reversed: cubic.reversed,
    }];
    let query = BezierWireQuery {
        segments: &segments,
        line: [seed, [seed[0] + scale, seed[1]]],
        options: RootIsolationOptions::new(0.001),
    };
    classify_bezier_wire(query, &mut || false)
        .unwrap()
        .location()
        .clone()
}
/// Select this fixture's positively oriented bounded component; example: `bounded_cubic(&builder, 1.)`.
pub(super) fn bounded_cubic(builder: &BuilderFace, scale: f64) -> Shape {
    let located: Vec<(i64, Shape)> = builder
        .areas()
        .iter()
        .map(|region| (cubic_winding(region, scale), region.clone()))
        .collect();
    let mut windings: Vec<i64> = located.iter().map(|(winding, _)| *winding).collect();
    // Both paths contain the seed. Nonzero membership alone cannot select a bounded face.
    windings.sort();
    assert_eq!(windings, [-1, 1]);
    let mut bounded: Vec<Shape> = located
        .into_iter()
        .filter_map(|(winding, region)| (winding == 1).then_some(region))
        .collect();
    assert_eq!(bounded.len(), 1);
    bounded.remove(0)
}
fn cubic_winding(region: &Shape, scale: f64) -> i64 {
    let cubic = extract_cubic(region, scale);
    let location = seed_location(&cubic, [scale, scale], scale);
    let BezierWireLocation::Inside { winding } = location else {
        panic!("cubic at {scale}: got {location:?}, expected interior seed");
    };
    winding
}
/// Check the cubic's independently established implicit-curve corpus; example: `check_seed_corpus(&region, 1.)`.
pub(super) fn check_seed_corpus(region: &Shape, scale: f64) {
    let cubic = extract_cubic(region, scale);
    for (seeds, expected) in cubic_seed_cases() {
        for seed in seeds {
            assert_eq!(
                seed_location(&cubic, seed.map(|x| x * scale), scale),
                expected
            );
        }
    }
}
fn cubic_seed_cases() -> [(Vec<[f64; 2]>, BezierWireLocation); 3] {
    [
        (
            vec![[1., 1.], [0.5, 0.5]],
            BezierWireLocation::Inside { winding: 1 },
        ),
        (
            vec![[-1., 1.], [3., 3.], [0., -1.]],
            BezierWireLocation::Outside,
        ),
        (
            vec![[0., 0.], [1.5, 1.5]],
            BezierWireLocation::Boundary {
                segment_ids: vec![1],
            },
        ),
    ]
}
