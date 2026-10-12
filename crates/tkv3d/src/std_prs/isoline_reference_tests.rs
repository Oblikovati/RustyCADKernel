//! Native shared outputs, initialized point storage and pinned GNU stable segment order.
use super::{StdPrsIsolineUvOutputs, StdPrsIsolines, StdPrsPntOnIso, StdPrsSegOnIso};
use super::{
    isoline_face_fixture::isoline_fixture_face, isoline_segment_sort::sort_isoline_segments,
};
use crate::select3d::compare_oracle_snapshot;
use rustycad_tkernel::{ncollection::Sequence, standard::SharedState};
use rustycad_tkmath::gp::Point3;
const NATIVE_REFERENCES: &str = include_str!("../../tests/fixtures/isolines/native-references.txt");
#[test]
fn native_sequence_and_scalar_reference_aliases_preserve_write_order() {
    for (scenario, row) in NATIVE_REFERENCES.lines().take(30).enumerate() {
        let actual = shared_isoline_parameter_snapshot(scenario);
        compare_oracle_snapshot(&actual, row);
    }
}
fn shared_isoline_parameter_snapshot(scenario: usize) -> Vec<f64> {
    let [u, v] = [
        SharedState::new(Sequence::default()),
        SharedState::new(Sequence::default()),
    ];
    u.write().append(91.);
    v.write().append(92.);
    let bounds = [71., 72., 73., 74.].map(SharedState::new);
    populate_shared_isoline_parameters(scenario, &u, &v, &bounds);
    let mut values = vec![scenario as f64];
    values.extend(bounds.iter().map(|value| *value.read()));
    append_parameter_snapshot(&mut values, &u);
    append_parameter_snapshot(&mut values, &v);
    values
}
fn append_parameter_snapshot(values: &mut Vec<f64>, parameters: &SharedState<Sequence<f64>>) {
    let parameters = parameters.read();
    values.push(parameters.length() as f64);
    values.extend(parameters.iter().copied());
}
fn initialized_reference_segment(index: i32) -> StdPrsSegOnIso {
    StdPrsSegOnIso {
        pnts: [
            StdPrsPntOnIso {
                pnt: Point3::new(f64::from(index), 2., 3.),
                param: f64::from(index % 9),
            },
            StdPrsPntOnIso {
                pnt: Point3::new(f64::from(index), 5., 6.),
                param: f64::from(index % 9) + 1.,
            },
        ],
    }
}
#[test]
fn native_initialized_inline_point_aliases_and_nan_interval_comparison() {
    let mut segment = initialized_reference_segment(2);
    let retained = segment.as_ptr();
    let writable = segment.as_mut_ptr();
    // Both conversions refer to live initialized array storage; no move or lifetime violation is performed.
    assert_eq!(retained, writable.cast_const());
    assert_eq!(retained, segment.pnts.as_ptr());
    segment.pnts[1].pnt = Point3::new(7., 8., 9.);
    segment.pnts[1].param = 4.;
    compare_mutated_isoline_segment(&segment);
    compare_nan_isoline_segments(&mut segment);
}
fn compare_mutated_isoline_segment(segment: &StdPrsSegOnIso) {
    let point = segment.pnts[1];
    let other = initialized_reference_segment(4);
    let actual = [
        30.,
        point.pnt.x(),
        point.pnt.y(),
        point.pnt.z(),
        point.param,
        f64::from(segment.less_than(&other)),
    ];
    compare_oracle_snapshot(&actual, NATIVE_REFERENCES.lines().nth(30).unwrap());
}
fn compare_nan_isoline_segments(segment: &mut StdPrsSegOnIso) {
    let other = initialized_reference_segment(4);
    segment.pnts[1].param = f64::NAN;
    compare_oracle_snapshot(
        &[
            31.,
            f64::from(segment.less_than(&other)),
            f64::from(other.less_than(segment)),
        ],
        NATIVE_REFERENCES.lines().nth(31).unwrap(),
    );
}
#[test]
fn native_touching_duplicate_intervals_preserve_large_gnu_stable_sort_order() {
    let mut segments: Vec<StdPrsSegOnIso> =
        (0..=36).rev().map(initialized_reference_segment).collect();
    sort_isoline_segments(&mut segments);
    let mut actual = vec![32.];
    actual.extend(segments.iter().map(|segment| segment.pnts[0].pnt.x()));
    compare_oracle_snapshot(&actual, NATIVE_REFERENCES.lines().nth(32).unwrap());
}

fn populate_shared_isoline_parameters(
    scenario: usize,
    u: &SharedState<Sequence<f64>>,
    v: &SharedState<Sequence<f64>>,
    bounds: &[SharedState<f64>; 4],
) {
    let outputs = shared_isoline_parameter_outputs(scenario, u, v, bounds);
    StdPrsIsolines::uv_iso_parameters(
        &isoline_fixture_face((scenario / 3) as i32),
        [2, 3],
        100.,
        outputs,
    )
    .unwrap();
}

fn shared_isoline_parameter_outputs<'a>(
    scenario: usize,
    u: &'a SharedState<Sequence<f64>>,
    v: &'a SharedState<Sequence<f64>>,
    bounds: &'a [SharedState<f64>; 4],
) -> StdPrsIsolineUvOutputs<'a> {
    let aliased = if scenario % 3 == 2 {
        [&bounds[0]; 4]
    } else {
        bounds.each_ref()
    };
    StdPrsIsolineUvOutputs {
        u,
        v: if scenario % 3 == 1 { u } else { v },
        bounds: aliased,
    }
}
