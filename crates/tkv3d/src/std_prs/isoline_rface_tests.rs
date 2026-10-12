//! Borrowed p-curve adaptor identity, forward-face orientation and sticky missing-curve repair replay.
use super::{StdPrsToolRFace, isoline_face_fixture::isoline_fixture_face};
use crate::select3d::compare_oracle_snapshot;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_adaptor::BRepAdaptorSurface,
    topexp::Explorer,
    topods::edge,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::top_abs::ShapeEnum;
const NATIVE_RFACE: &str = include_str!("../../tests/fixtures/isolines/native-rface.txt");
#[test]
fn native_borrowed_curve_reloads_and_sticky_geometry_repairs() {
    let mut rows = NATIVE_RFACE.lines();
    let tool = StdPrsToolRFace::new();
    compare_oracle_snapshot(
        &rface_snapshot(&tool, tool.value(), -1, 0),
        rows.next().unwrap(),
    );
    tool.init().unwrap();
    compare_oracle_snapshot(
        &rface_snapshot(&tool, tool.value(), -1, 1),
        rows.next().unwrap(),
    );
    for scenario in 0..5 {
        replay_rface_scenario(scenario, &mut rows);
    }
    assert!(rows.next().is_none());
}
fn replay_rface_scenario(scenario: i32, rows: &mut std::str::Lines<'_>) {
    let mut face = isoline_fixture_face(1);
    if scenario == 1 {
        face.reverse();
    }
    let (edge, original) = prepare_missing_isoline_pcurve(&face, scenario);
    if scenario >= 3 {
        set_unoriented_rface_edge(&face, &edge, scenario);
    }
    let surface = Handle::new(BRepAdaptorSurface::with_face(&face, true).unwrap());
    let tool = StdPrsToolRFace::with_surface(&surface).unwrap();
    let alias = tool.value();
    let mut step = 0;
    compare_rface_step(&tool, alias, scenario, &mut step, rows);
    replay_rface_pass(&tool, alias, scenario, &mut step, rows);
    repair_missing_rface_curve(scenario, &face, &edge, &original.pcurve);
    replay_rface_pass(&tool, alias, scenario, &mut step, rows);
}
fn replay_rface_pass(
    tool: &StdPrsToolRFace,
    alias: &dyn Adaptor2dCurve2d,
    scenario: i32,
    step: &mut i32,
    rows: &mut std::str::Lines<'_>,
) {
    tool.init().unwrap();
    compare_rface_step(tool, alias, scenario, step, rows);
    while tool.more() {
        tool.next().unwrap();
        compare_rface_step(tool, alias, scenario, step, rows);
    }
}
fn compare_rface_step(
    tool: &StdPrsToolRFace,
    alias: &dyn Adaptor2dCurve2d,
    scenario: i32,
    step: &mut i32,
    rows: &mut std::str::Lines<'_>,
) {
    compare_oracle_snapshot(
        &rface_snapshot(tool, alias, scenario, *step),
        rows.next().unwrap(),
    );
    *step += 1;
}
fn rface_snapshot(
    tool: &StdPrsToolRFace,
    alias: &dyn Adaptor2dCurve2d,
    scenario: i32,
    step: i32,
) -> Vec<f64> {
    let mut row = vec![
        f64::from(scenario),
        f64::from(step),
        f64::from(tool.more()),
        f64::from(tool.is_oriented()),
        f64::from(tool.is_invalid_geometry()),
        f64::from(std::ptr::eq(alias, tool.value())),
        alias.get_type().unwrap() as i32 as f64,
        alias.first_parameter().unwrap(),
        alias.last_parameter().unwrap(),
    ];
    row.extend(rface_current_curve_snapshot(tool, alias));
    row
}

fn prepare_missing_isoline_pcurve(
    face: &rustycad_tkbrep::topods::Face,
    scenario: i32,
) -> (
    rustycad_tkbrep::topods::Edge,
    rustycad_tkbrep::brep::EdgePCurve,
) {
    let explorer = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let edge = edge(explorer.current()).unwrap();
    let original = BRepTool::curve_on_face(&edge, face).unwrap();
    if scenario == 2 {
        BRepBuilder::default()
            .update_edge_pcurve_on_face(&edge, &Handle::default(), face, 1e-7)
            .unwrap();
    }
    (edge, original)
}

fn rface_current_curve_snapshot(tool: &StdPrsToolRFace, alias: &dyn Adaptor2dCurve2d) -> [f64; 3] {
    if !tool.more() {
        return [-1., 0., 0.];
    }
    let parameter = (alias.first_parameter().unwrap() + alias.last_parameter().unwrap()) * 0.5;
    let point = alias.value(parameter).unwrap();
    [tool.orientation() as i32 as f64, point.x(), point.y()]
}

fn set_unoriented_rface_edge(
    face: &rustycad_tkbrep::topods::Face,
    edge: &rustycad_tkbrep::topods::Edge,
    scenario: i32,
) {
    let explorer = Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape).unwrap();
    let mut wire = explorer.current().clone();
    let builder = BRepBuilder::default();
    // MakeFace freezes attached wires; explicitly reopen this genuine topology mutation first.
    wire.set_free(true).unwrap();
    builder.remove(&mut wire, edge).unwrap();
    let mut changed = edge.clone();
    let orientation = if scenario == 3 {
        rustycad_tkg3d::top_abs::Orientation::Internal
    } else {
        rustycad_tkg3d::top_abs::Orientation::External
    };
    changed.set_orientation(orientation);
    builder.add(&mut wire, &changed).unwrap();
}

fn repair_missing_rface_curve(
    scenario: i32,
    face: &rustycad_tkbrep::topods::Face,
    edge: &rustycad_tkbrep::topods::Edge,
    original: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) {
    if scenario == 2 {
        BRepBuilder::default()
            .update_edge_pcurve_on_face(edge, original, face, 1e-7)
            .unwrap();
    }
}
