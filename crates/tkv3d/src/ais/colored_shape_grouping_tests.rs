//! Actual finite planar topology replays native grouping, including inherited hidden drawers and vertex removal.
use super::*;
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::fmt::Write;
struct ColoredDispatchFixture {
    input: colored_shape_expansion_inputs::ColoredExpansionInputs,
    _parent: Handle<Prs3dDrawer>,
    base: Handle<AisColoredDrawer>,
    custom: Handle<AisColoredDrawer>,
    styles: AisShapeDrawerMap,
    opened: AisColoredOpenedCompounds,
    closed: AisColoredDrawerCompounds,
}
impl ColoredDispatchFixture {
    fn new(scenario: usize) -> Self {
        let input = colored_shape_expansion_inputs::ColoredExpansionInputs::new();
        let parent = Handle::new(Prs3dDrawer::new());
        let base = Handle::new(AisColoredDrawer::new(&parent));
        let custom = Handle::new(AisColoredDrawer::new(&parent));
        let styles = dispatch_fixture_styles(&input, scenario, &custom);
        base.set_hidden(scenario == 7);
        Self {
            input,
            _parent: parent,
            base,
            custom,
            styles,
            opened: std::array::from_fn(|_| Default::default()),
            closed: Default::default(),
        }
    }
    fn dispatch(&self, alias: bool) -> bool {
        let output = if alias { &self.opened[4] } else { &self.closed };
        ais_colored_shape_dispatch_colors(
            &self.base,
            &self.input.base,
            &self.styles,
            ShapeEnum::Compound,
            false,
            &self.opened,
            output,
        )
        .unwrap()
    }
}
#[test]
fn native_dispatch_replays_ordered_groups_without_replacing_face_topology() {
    let mut rows = String::new();
    for scenario in 0..8 {
        for alias in [false, true] {
            dispatch_fixture_row(&mut rows, scenario, alias);
        }
    }
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-dispatch.txt")
    );
}
fn dispatch_fixture_row(rows: &mut String, scenario: usize, alias: bool) {
    let scene = ColoredDispatchFixture::new(scenario);
    let overridden = scene.dispatch(alias);
    write!(
        rows,
        "dispatch {scenario} {} {}",
        i32::from(alias),
        i32::from(overridden)
    )
    .unwrap();
    for group in scene.opened.iter().chain(std::iter::once(&scene.closed)) {
        dispatch_group_row(rows, group, &scene.base, &scene.custom);
    }
    rows.push('\n');
}
fn dispatch_fixture_styles(
    input: &colored_shape_expansion_inputs::ColoredExpansionInputs,
    scenario: usize,
    custom: &Handle<AisColoredDrawer>,
) -> AisShapeDrawerMap {
    let styles = AisShapeDrawerMap::default();
    if (1..=6).contains(&scenario) {
        styles
            .bind(dispatch_fixture_override(input, scenario), custom)
            .unwrap();
    }
    custom.set_hidden(scenario == 6);
    styles
}
fn dispatch_fixture_override(
    input: &colored_shape_expansion_inputs::ColoredExpansionInputs,
    scenario: usize,
) -> &Shape {
    match scenario {
        1 => &input.vertex,
        2 => &input.edges[0],
        3 | 6 => &input.faces[0],
        4 => &input.wire,
        5 => &input.own,
        _ => panic!("scenario={scenario}; expected override scenario 1..=6"),
    }
}
pub(super) fn dispatch_group_row(
    rows: &mut String,
    groups: &AisColoredDrawerCompounds,
    base: &Handle<AisColoredDrawer>,
    custom: &Handle<AisColoredDrawer>,
) {
    write!(rows, " {}", groups.len()).unwrap();
    for index in 1..=groups.len() {
        let alias = groups.node_ref(index).unwrap();
        let address = alias.live_node().unwrap().drawer.object_address();
        let style = dispatch_fixture_style(address, base, custom);
        dispatch_compound_row(rows, style, &alias.clone_compound().unwrap());
    }
}
fn dispatch_fixture_style(
    address: *const (),
    base: &Handle<AisColoredDrawer>,
    custom: &Handle<AisColoredDrawer>,
) -> i32 {
    if address == base.object_address() {
        return 0;
    }
    if address == custom.object_address() {
        return 1;
    }
    2
}
fn dispatch_compound_row(rows: &mut String, style: i32, compound: &Shape) {
    let faces = dispatch_topology_count(compound, ShapeEnum::Face);
    let edges = dispatch_topology_count(compound, ShapeEnum::Edge);
    let vertices = dispatch_topology_count(compound, ShapeEnum::Vertex);
    write!(
        rows,
        " {style}:{}:{faces}:{edges}:{vertices}",
        compound.nb_children()
    )
    .unwrap();
}
fn dispatch_topology_count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
}
const PRIORITY_KINDS: [ShapeEnum; 9] = [
    ShapeEnum::Compound,
    ShapeEnum::CompSolid,
    ShapeEnum::Solid,
    ShapeEnum::Shell,
    ShapeEnum::Face,
    ShapeEnum::Wire,
    ShapeEnum::Edge,
    ShapeEnum::Vertex,
    ShapeEnum::Shape,
];
#[test]
fn native_standard_priority_matrix_uses_explicit_leaf_before_shape_type() {
    let input = colored_shape_expansion_inputs::ColoredExpansionInputs::new();
    let shapes = [
        &input.base,
        &input.compsolid,
        &input.solid,
        &input.shell,
        &input.faces[0],
        &input.wire,
        &input.edges[0],
        &input.vertex,
    ];
    let mut rows = String::new();
    for (index, shape) in shapes.into_iter().enumerate() {
        priority_shape_row(&mut rows, index, shape);
    }
    priority_null_rows(&mut rows);
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-priority.txt")
    );
}
fn priority_shape_row(rows: &mut String, index: usize, shape: &Shape) {
    write!(rows, "priority {index}").unwrap();
    for kind in PRIORITY_KINDS {
        let priority =
            crate::std_select::StdSelectBrepSelectionTool::get_standard_priority(shape, kind)
                .unwrap();
        write!(rows, " {priority}").unwrap();
    }
    rows.push('\n');
}
fn priority_null_rows(rows: &mut String) {
    for kind in [
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        ShapeEnum::Wire,
        ShapeEnum::Face,
    ] {
        let priority = crate::std_select::StdSelectBrepSelectionTool::get_standard_priority(
            &Shape::default(),
            kind,
        )
        .unwrap();
        writeln!(rows, "null-priority {} {priority}", kind as usize).unwrap();
    }
}
#[test]
fn native_dispatch_failure_preserves_completed_groups_and_repair_appends_again() {
    let scene = ColoredDispatchFixture::new(0);
    let mut frozen = rustycad_tkbrep::topods::Compound::default();
    rustycad_tkbrep::topods::Builder.make_compound(&mut frozen);
    frozen.set_free(false).unwrap();
    scene.opened[4].add(&scene.base, &frozen).unwrap();
    let mut rows = String::new();
    record_dispatch_failure_repair(&scene, &mut frozen, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-dispatch-failure.txt")
    );
}
fn record_dispatch_failure_repair(
    scene: &ColoredDispatchFixture,
    frozen: &mut rustycad_tkbrep::topods::Compound,
    rows: &mut String,
) {
    let failed = dispatch_failure_result(scene).unwrap_err();
    write!(rows, "failed {}", failed.exception_type()).unwrap();
    record_failure_group_rows(scene, rows);
    frozen.set_free(true).unwrap();
    let overridden = dispatch_failure_result(scene).unwrap();
    write!(rows, "repaired {}", i32::from(overridden)).unwrap();
    record_failure_group_rows(scene, rows);
}
fn dispatch_failure_result(
    scene: &ColoredDispatchFixture,
) -> Result<bool, rustycad_tkernel::standard::Failure> {
    ais_colored_shape_dispatch_colors(
        &scene.base,
        &scene.input.base,
        &scene.styles,
        ShapeEnum::Compound,
        false,
        &scene.opened,
        &scene.closed,
    )
}
fn record_failure_group_rows(scene: &ColoredDispatchFixture, rows: &mut String) {
    dispatch_group_row(rows, &scene.opened[4], &scene.base, &scene.base);
    dispatch_group_row(rows, &scene.opened[5], &scene.base, &scene.base);
    rows.push('\n');
}
