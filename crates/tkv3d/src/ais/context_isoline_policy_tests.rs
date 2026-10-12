use super::*;
use crate::{prs3d::Prs3dDrawer, v3d::V3dViewer};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::null_driver::NullGraphicDriver;

type PolicySnapshot = [f64; 13];

fn context_policy_snapshot(context: &AisInteractiveContext, step: i32) -> PolicySnapshot {
    let drawer = context.default_drawer();
    [
        f64::from(step),
        f64::from(context.iso_number(AisTypeOfIso::U)),
        f64::from(context.iso_number(AisTypeOfIso::V)),
        f64::from(context.iso_number(AisTypeOfIso::Both)),
        f64::from(context.iso_number(AisTypeOfIso::Unnamed)),
        f64::from(context.iso_on_plane()),
        f64::from(context.iso_on_triangulation()),
        context.deviation_coefficient(),
        context.deviation_angle(),
        f64::from(drawer.has_own_deviation_coefficient()),
        f64::from(drawer.has_own_deviation_angle()),
        drawer.previous_deviation_coefficient(),
        drawer.previous_deviation_angle(),
    ]
}

fn context_count_trace(context: &AisInteractiveContext) -> Vec<PolicySnapshot> {
    let mut trace = vec![context_policy_snapshot(context, 0)];
    for count in [-7, 0, 4] {
        for direction in [
            AisTypeOfIso::U,
            AisTypeOfIso::V,
            AisTypeOfIso::Both,
            AisTypeOfIso::Unnamed,
        ] {
            context.set_iso_number(count, direction);
            trace.push(context_policy_snapshot(context, trace.len() as i32));
        }
    }
    context_plane_trace(context, &mut trace);
    let u = context.default_drawer().u_iso_aspect();
    context.default_drawer().set_v_iso_aspect(u);
    context.set_iso_number(9, AisTypeOfIso::V);
    trace.push(context_policy_snapshot(context, 15));
    trace
}

fn context_plane_trace(context: &AisInteractiveContext, trace: &mut Vec<PolicySnapshot>) {
    for enabled in [true, false] {
        context.set_iso_on_plane(enabled);
        context.set_iso_on_triangulation(!enabled);
        trace.push(context_policy_snapshot(context, trace.len() as i32));
    }
}

fn context_deviation_trace(context: &AisInteractiveContext) -> Vec<PolicySnapshot> {
    let mut trace = Vec::new();
    for (index, value) in [0.02, 0.04, 0.0, -1.0].into_iter().enumerate() {
        context.set_deviation_coefficient(value);
        context.set_deviation_angle(value);
        trace.push(context_policy_snapshot(context, 16 + index as i32));
    }
    trace.extend(context_linked_deviation_trace(context));
    trace
}

fn context_linked_deviation_trace(context: &AisInteractiveContext) -> Vec<PolicySnapshot> {
    let link = Handle::new(Prs3dDrawer::new());
    link.setup_own_defaults().unwrap();
    link.set_deviation_coefficient(0.3);
    link.set_deviation_angle(0.4);
    let replacement = Handle::new(Prs3dDrawer::new());
    replacement.link_to(&link);
    context.set_default_drawer(&replacement);
    let first = context_policy_snapshot(context, 20);
    link.set_deviation_coefficient(0.5);
    link.set_deviation_angle(0.6);
    vec![first, context_policy_snapshot(context, 21)]
}

#[test]
fn native_context_isoline_and_deviation_rows_replay_exactly() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let mut trace = context_count_trace(&context);
    trace.extend(context_deviation_trace(&context));
    let expected = expected_context_policy_rows();
    assert_eq!(trace.len(), expected.len());
    for (actual, expected) in trace.iter().zip(expected) {
        assert_eq!(actual.map(f64::to_bits), expected.map(f64::to_bits));
    }
}

fn expected_context_policy_rows() -> Vec<PolicySnapshot> {
    include_str!("../../tests/fixtures/context-isoline-policies.txt")
        .lines()
        .map(|line| {
            let values: Vec<f64> = line
                .split_whitespace()
                .map(|v| v.parse().unwrap())
                .collect();
            assert_eq!(values.len(), 13, "fixture row={line}; expected 13 columns");
            values.try_into().unwrap()
        })
        .collect()
}

#[test]
fn unnamed_isoline_direction_does_not_access_null_drawer() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    context.set_default_drawer(&Handle::default());
    context.set_iso_number(47, AisTypeOfIso::Unnamed);
    let actual = format!(
        "{} {}\n",
        i32::from(context.default_drawer().is_null()),
        context.iso_number(AisTypeOfIso::Unnamed)
    );
    assert_eq!(
        actual,
        include_str!("../../tests/fixtures/context-isoline-null.txt")
    );
}
