//! Native UV early no-write and closure-failure partial outputs preserve caller references.
use super::{
    StdPrsIsolineUvOutputs, StdPrsIsolines, isoline_alias_surface_fake::IsolineAliasSurfaceFake,
    isoline_face_fixture::isoline_fixture_face, isoline_live_input_tests::live_isoline_input_cells,
};
use crate::select3d::compare_oracle_snapshot;
use rustycad_tkbrep::{brep::BRepBuilder, topods::Face};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Handle, SharedState},
};
use rustycad_tkmath::gp::Plane3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
const NATIVE_EDGES: &str = include_str!("../../tests/fixtures/isolines/native-parameter-edges.txt");
#[test]
fn native_null_infinite_negative_and_closure_failure_outputs() {
    for (scenario, row) in NATIVE_EDGES.lines().enumerate() {
        compare_oracle_snapshot(&parameter_edge_snapshot(scenario as i32), row);
    }
}
fn parameter_edge_topology(scenario: i32) -> Face {
    let mut face = Face::default();
    let builder = BRepBuilder::default();
    match scenario {
        0 => builder.make_face(&mut face),
        1 => {
            return BRepBuilderAPIMakeFace::from_plane(Plane3::default())
                .unwrap()
                .face()
                .unwrap();
        }
        2 => return isoline_fixture_face(0),
        _ => builder
            .make_face_on_surface(&mut face, &failing_closure_surface(), 1e-7)
            .unwrap(),
    }
    face
}
fn failing_closure_surface() -> Handle<dyn rustycad_tkg3d::geom::GeomSurface> {
    let source = Handle::new(IsolineAliasSurfaceFake::new(&live_isoline_input_cells()));
    *source.fail_closed.write() = true;
    source.cast()
}
fn parameter_edge_snapshot(scenario: i32) -> Vec<f64> {
    let parameters = [
        SharedState::new(Sequence::default()),
        SharedState::new(Sequence::default()),
    ];
    parameters[0].write().append(91.);
    parameters[1].write().append(92.);
    let bounds = [71., 72., 73., 74.].map(SharedState::new);
    let failed = generate_parameter_edge_outputs(scenario, &parameters, &bounds);
    let mut row = vec![f64::from(scenario), f64::from(failed)];
    row.extend(bounds.iter().map(|cell| *cell.read()));
    append_parameter_edge_values(&mut row, &parameters);
    row
}
fn append_parameter_edge_values(row: &mut Vec<f64>, parameters: &[SharedState<Sequence<f64>>; 2]) {
    for sequence in parameters {
        row.push(sequence.read().length() as f64);
        row.extend(sequence.read().iter().copied());
    }
}

fn generate_parameter_edge_outputs(
    scenario: i32,
    parameters: &[SharedState<Sequence<f64>>; 2],
    bounds: &[SharedState<f64>; 4],
) -> bool {
    let outputs = StdPrsIsolineUvOutputs {
        u: &parameters[0],
        v: &parameters[1],
        bounds: bounds.each_ref(),
    };
    let count = if scenario == 2 { -1 } else { 2 };
    StdPrsIsolines::uv_iso_parameters(
        &parameter_edge_topology(scenario),
        [count, 3],
        100.,
        outputs,
    )
    .is_err()
}
