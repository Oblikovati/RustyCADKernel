// SPDX-License-Identifier: GPL-2.0-only
use super::{faces, replay, test_fixture::*};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_prism::{frozen::*, surface_charts::*};
use crate::sketch_region_source_conformance::CapturedPlaneSupport;
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;
fn plane() -> CapturedPlaneSupport {
    CapturedPlaneSupport {
        frame: MetricEmbedding {
            linear: [[1., 0.], [0., 1.], [0., 0.]],
            origin: [0., -0., 0.],
            scale: 1.,
        },
        placement: placement(),
    }
}
fn trim() -> FrozenPrismTrim {
    let mut occurrence = cap_endpoint(&names());
    occurrence.0.pop();
    FrozenPrismTrim {
        occurrence,
        source: AuthoredPrismId(2051),
        parents: vec![],
        role: PrismTrimRole::First,
        pcurve: CapturedMetricCurve::Line {
            origin: [-0., 1.],
            direction: [1., 0.],
        },
        interval: [-0., 1.],
        space: space(),
        expected: CapturedTrimExpectation::SideGenerating,
    }
}
pub(super) fn captured_face() -> FrozenPrismFace {
    FrozenPrismFace {
        key: key(2056, FrozenSweepRole::First, 4),
        source: CapturedChartSource::Cap(plane()),
        surface: CapturedPrismSurface::Plane(plane()),
        trims: vec![trim()],
        proof: Default::default(),
    }
}
#[test]
fn face_trim_read_projection_preserves_coefficients_roles_and_ranges() {
    let source = captured_face();
    let names = names();
    let row = faces::project(&source, &names).unwrap();
    assert_eq!(row.key(), names.member(&source.key).unwrap());
    let projected = &row.trims()[0];
    assert_eq!(
        projected.occurrence(),
        names.occurrence(&source.trims[0].occurrence).unwrap()
    );
    assert_eq!(projected.role(), PrismTrimRole::First);
    assert_eq!(
        projected.native_interval().map(f64::to_bits),
        [(-0_f64).to_bits(), 1_f64.to_bits()]
    );
    assert!(matches!(
        row.surface(),
        super::PrismBodySurfaceView::Plane { .. }
    ));
}
#[test]
fn face_coefficients_and_trim_ranges_are_owned_after_input_drop() {
    let mut source = captured_face();
    let row = faces::project(&source, &names()).unwrap();
    assert!(row.trims()[0].space_curve().same_parameter);
    let mut before = Vec::new();
    replay::face(&row, &mut before);
    source.trims[0].interval = [3., 5.];
    source.trims.clear();
    drop(source);
    let mut after = Vec::new();
    replay::face(&row, &mut after);
    assert_eq!(before, after);
}
