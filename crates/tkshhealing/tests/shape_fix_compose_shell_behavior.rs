//! Native constructor and public mutable-option parity for ShapeFix_ComposeShell.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::ReShape,
    topods::{self, Builder},
};
use rustycad_tkernel::{
    ncollection::{HArray1, HArray2, Sequence},
    standard::Handle,
};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Direction2, Line2, Point2, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisTransferParameters,
    shape_build::ShapeBuildReShape,
    shape_extend::{
        ShapeExtendCompositeSurface, ShapeExtendParametrisation, ShapeExtendStatus,
        ShapeExtendWireData,
    },
    shape_fix::{FixRoot, ShapeFixComposeShell, ShapeFixRoot, ShapeFixWireSegment},
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
    BRepBuilderAPIMakeWire,
};

#[test]
fn constructor_modes_and_transfer_tool_match_native() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_compose_shell.expected");
    let composer = ShapeFixComposeShell::new();
    let tool = composer.get_transfer_param_tool();
    assert_eq!(
        format!(
            "default {} {} {} {}",
            composer.closed_mode() as u8,
            composer.status(ShapeExtendStatus::Done1) as u8,
            composer.result().is_null() as u8,
            tool.dynamic_type().unwrap().name()
        ),
        expected.lines().next().unwrap()
    );
    composer.set_closed_mode(true);
    assert_eq!(
        format!("closed_mode {}", composer.closed_mode() as u8),
        expected
            .lines()
            .find(|row| row.starts_with("closed_mode "))
            .unwrap()
    );
    let replacement = Handle::new(ShapeAnalysisTransferParameters::new()).cast();
    composer.set_transfer_param_tool(&replacement);
    assert_eq!(
        format!(
            "transfer_type {}",
            composer
                .get_transfer_param_tool()
                .dynamic_type()
                .unwrap()
                .name()
        ),
        expected
            .lines()
            .find(|row| row.starts_with("transfer_type "))
            .unwrap()
    );
    assert_init_matches_native(expected);
}

fn assert_init_matches_native(expected: &str) {
    let face = face_fixture();
    let grid = plane_grid();
    let composer = ShapeFixComposeShell::new();
    composer
        .init(&grid, &Location::default(), &face, 1.0e-7)
        .unwrap();
    assert_eq!(
        format!(
            "init {} {} {}",
            composer.closed_mode() as u8,
            composer.status(ShapeExtendStatus::Done1) as u8,
            composer.result().is_null() as u8,
        ),
        expected
            .lines()
            .find(|row| row.starts_with("init "))
            .unwrap()
    );
}

fn plane_grid() -> Handle<ShapeExtendCompositeSurface> {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let cells = HArray2::with_value(1, 1, 1, 1, Handle::<dyn GeomSurface>::default()).unwrap();
    cells.change_array2().set_value(1, 1, plane).unwrap();
    let patches = Handle::new(cells);
    Handle::new(
        ShapeExtendCompositeSurface::with_grid(&patches, ShapeExtendParametrisation::Natural)
            .unwrap(),
    )
}

fn face_fixture() -> rustycad_tkbrep::topods::Face {
    let points = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 2.0, 0.0),
        Point3::new(0.0, 2.0, 0.0),
    ];
    let mut builder = BRepBuilderAPIMakeWire::new();
    for index in 0..points.len() {
        let edge = BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % 4])
            .unwrap()
            .edge()
            .unwrap();
        builder.add_edge(&edge).unwrap();
    }
    BRepBuilderAPIMakeFace::from_plane_wire(
        rustycad_tkmath::gp::Plane3::default(),
        &builder.wire().unwrap(),
        true,
    )
    .unwrap()
    .face()
    .unwrap()
}

#[test]
fn root_base_cast_shares_compose_shell_fix_state() {
    let composer = Handle::new(ShapeFixComposeShell::new());
    let root = composer.cast::<ShapeFixRoot>();
    assert!(!root.is_null());
    root.get().unwrap().set_precision(0.125);
    assert_eq!(composer.precision(), 0.125);
    assert_eq!(root.get().unwrap().precision(), 0.125);
}

#[test]
fn protected_wire_helpers_are_reachable_and_match_native_rows() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_compose_shell.expected");
    let composer = ShapeFixComposeShell::new();
    let context = Handle::new(ShapeBuildReShape::new());
    composer.set_context(&context);
    composer
        .init(&plane_grid(), &Location::default(), &face_fixture(), 1.0e-7)
        .unwrap();
    let segments = composer.load_wires().unwrap();
    let wire = segments.value(1).unwrap().wire_data().get().unwrap();
    assert_eq!(segments.length(), 1);
    assert_eq!(segments.value(1).unwrap().nb_edges(), 4);
    assert_eq!(
        native_compute_code(&composer, wire),
        native_row(expected, "compute_code ")
    );
}

#[test]
fn load_wires_safe_null_replacement_failure_retains_prior_segment() {
    let (face, context) = face_with_removed_later_vertex();
    let composer = initialized_composer(&face, &context);
    let mut segments = Sequence::default();
    segments.append(ShapeFixWireSegment::new());
    let error = composer.load_wires_into(&mut segments).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    assert_eq!(segments.length(), 1);
    assert_eq!(segments.value(1).unwrap().nb_edges(), 4);
}

fn face_with_removed_later_vertex() -> (rustycad_tkbrep::topods::Face, Handle<ShapeBuildReShape>) {
    let mut face = face_fixture();
    let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 1.0, 0.0))
        .unwrap()
        .vertex()
        .unwrap();
    Builder
        .add(&mut face, &vertex.clone().into_shape())
        .unwrap();
    let context = Handle::new(ShapeBuildReShape::new());
    context.remove(&vertex.clone().into_shape()).unwrap();
    (face, context)
}

fn initialized_composer(
    face: &rustycad_tkbrep::topods::Face,
    context: &Handle<ShapeBuildReShape>,
) -> ShapeFixComposeShell {
    let composer = ShapeFixComposeShell::new();
    composer.set_context(context);
    composer
        .init(&plane_grid(), &Location::default(), face, 1.0e-7)
        .unwrap();
    composer
}

fn native_compute_code(composer: &ShapeFixComposeShell, wire: &ShapeExtendWireData) -> String {
    let line = rustycad_tkmath::gp::Line2::new(
        rustycad_tkmath::gp::Point2::new(1.0, 0.0),
        rustycad_tkmath::gp::Direction2::new(0.0, 1.0).unwrap(),
    );
    format!(
        "compute_code {}",
        composer
            .compute_code(wire, line, 1, 1, 0.25, 0.75, false)
            .unwrap()
    )
}

fn native_row<'a>(expected: &'a str, label: &str) -> &'a str {
    expected
        .lines()
        .find(|row| row.starts_with(label))
        .expect("native oracle row exists")
}

#[test]
fn split_wire_rectangular_loop_matches_native_patch_and_vertex_counts() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_compose_shell.expected");
    let composer = ShapeFixComposeShell::new();
    composer.set_context(&Handle::new(ShapeBuildReShape::new()));
    composer
        .init(
            &two_patch_plane_grid(),
            &Location::default(),
            &face_fixture(),
            1.0e-7,
        )
        .unwrap();
    let segments = composer.load_wires().unwrap();
    let mut wire = segments.value(1).unwrap().clone();
    let mut indices = sequence(&[1, 3]);
    let values = sequence(&[1.0, 1.0]);
    let codes = sequence(&[2, 1]);
    let mut vertices = Sequence::default();
    let result = composer
        .split_wire(
            &mut wire,
            &mut indices,
            &values,
            &mut vertices,
            &codes,
            true,
            1,
        )
        .unwrap();
    assert_eq!(
        split_wire_summary(&result, vertices.length()),
        native_row(expected, "split_wire ")
    );
}

#[test]
fn split_by_line_rectangular_loop_matches_native_intersections_and_codes() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_compose_shell.expected");
    let composer = initialized_composer(&face_fixture(), &Handle::new(ShapeBuildReShape::new()));
    let segments = composer.load_wires().unwrap();
    let mut wire = segments.value(1).unwrap().clone();
    let line = Line2::new(Point2::new(1.0, 0.0), Direction2::new(0.0, 1.0).unwrap());
    let mut parameters = Sequence::default();
    let mut codes = Sequence::default();
    let mut vertices = Sequence::default();
    let done = composer
        .split_by_line(
            &mut wire,
            line,
            true,
            1,
            &mut parameters,
            &mut codes,
            &mut vertices,
        )
        .unwrap();
    assert!(done);
    assert_eq!(
        split_by_line_summary(&wire, &parameters, &codes, &vertices),
        native_row(expected, "split_by_line ")
    );
}

#[test]
fn split_by_line_vertex_segment_matches_native_projection_and_replacement() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_compose_shell.expected");
    assert_eq!(
        split_vertex_segment_summary(),
        native_row(expected, "split_by_line_vertex ")
    );
}

fn split_vertex_segment_summary() -> String {
    let face = bounded_plane_face();
    let composer = initialized_composer(&face, &Handle::new(ShapeBuildReShape::new()));
    let mut segment = ShapeFixWireSegment::new();
    let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 1.0, 0.0))
        .unwrap()
        .vertex()
        .unwrap();
    segment.set_vertex(&vertex);
    let mut parameters = Sequence::default();
    let mut codes = Sequence::default();
    let mut vertices = Sequence::default();
    let line = Line2::new(Point2::new(1.0, 0.0), Direction2::new(0.0, 1.0).unwrap());
    let done = composer
        .split_by_line(
            &mut segment,
            line,
            true,
            1,
            &mut parameters,
            &mut codes,
            &mut vertices,
        )
        .unwrap();
    let point = BRepTool::pnt(&topods::vertex(vertices.value(1).unwrap()).unwrap()).unwrap();
    format!(
        "split_by_line_vertex {} {} {} {} {}:{}",
        done as u8,
        segment.is_vertex() as u8,
        parameters.length(),
        codes.value(1).unwrap(),
        point.x(),
        point.y()
    )
}

#[test]
fn split_wire_projects_and_preserves_an_internal_edge_vertex() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_compose_shell.expected");
    assert_eq!(
        split_internal_vertex_summary(),
        native_row(expected, "split_wire_internal ")
    );
}

fn split_internal_vertex_summary() -> String {
    let (face, edge) = internal_edge_fixture();
    let mut wire = ShapeFixWireSegment::new();
    wire.add_edge(0, &edge).unwrap();
    let composer = initialized_composer(&face, &Handle::new(ShapeBuildReShape::new()));
    let mut indices = sequence(&[1]);
    let values = sequence(&[1.0]);
    let codes = sequence(&[2]);
    let mut vertices = Sequence::default();
    let split = composer
        .split_wire(
            &mut wire,
            &mut indices,
            &values,
            &mut vertices,
            &codes,
            true,
            1,
        )
        .unwrap();
    internal_vertex_result(&split, vertices.length())
}

fn internal_edge_fixture() -> (rustycad_tkbrep::topods::Face, rustycad_tkbrep::topods::Edge) {
    let points = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 2.0, 0.0),
        Point3::new(0.0, 2.0, 0.0),
    ];
    let edges: Vec<_> = (0..4)
        .map(|index| {
            BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % 4])
                .unwrap()
                .edge()
                .unwrap()
        })
        .collect();
    let edge = edge_with_internal_vertex(edges[0].clone());
    let mut wire_builder = BRepBuilderAPIMakeWire::new();
    wire_builder.add_edge(&edge).unwrap();
    for other in edges.iter().skip(1) {
        wire_builder.add_edge(other).unwrap();
    }
    let wire = wire_builder.wire().unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_wire(
        rustycad_tkmath::gp::Plane3::default(),
        &wire,
        true,
    )
    .unwrap()
    .face()
    .unwrap();
    (face, edge)
}

fn bounded_plane_face() -> rustycad_tkbrep::topods::Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(
        rustycad_tkmath::gp::Plane3::default(),
        (0.0, 2.0, 0.0, 2.0),
    )
    .unwrap()
    .face()
    .unwrap()
}

fn edge_with_internal_vertex(
    mut edge: rustycad_tkbrep::topods::Edge,
) -> rustycad_tkbrep::topods::Edge {
    let mut internal = BRepBuilderAPIMakeVertex::new(Point3::new(0.5, 0.0, 0.0))
        .unwrap()
        .vertex()
        .unwrap();
    internal.set_orientation(rustycad_tkg3d::top_abs::Orientation::Internal);
    let mut edge_shape = edge.clone().into_shape();
    BRepBuilder::default()
        .add(&mut edge_shape, &internal.into_shape())
        .unwrap();
    edge = topods::edge(&edge_shape).unwrap();
    edge
}

fn internal_vertex_result(segment: &ShapeFixWireSegment, vertex_count: i32) -> String {
    let counts: Vec<usize> = (1..=segment.nb_edges())
        .map(|index| internal_vertices(&segment.edge(index).unwrap()))
        .collect();
    format!(
        "split_wire_internal {} {vertex_count} {}",
        segment.nb_edges(),
        counts
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn internal_vertices(edge: &rustycad_tkbrep::topods::Edge) -> usize {
    let shape = edge.clone().into_shape();
    let mut iterator = rustycad_tkbrep::topods::ShapeIterator::new(&shape, false, true);
    let mut count = 0;
    while iterator.more() {
        count += usize::from(
            iterator.value().unwrap().orientation()
                == rustycad_tkg3d::top_abs::Orientation::Internal,
        );
        iterator.next();
    }
    count
}

fn sequence<T: Clone>(values: &[T]) -> Sequence<T> {
    let mut sequence = Sequence::default();
    for value in values {
        sequence.append(value.clone());
    }
    sequence
}

fn two_patch_plane_grid() -> Handle<ShapeExtendCompositeSurface> {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let cells = HArray2::with_value(1, 2, 1, 1, Handle::<dyn GeomSurface>::default()).unwrap();
    cells
        .change_array2()
        .set_value(1, 1, plane.clone())
        .unwrap();
    cells.change_array2().set_value(2, 1, plane).unwrap();
    let patches = Handle::new(cells);
    let grid = Handle::new(
        ShapeExtendCompositeSurface::with_grid(&patches, ShapeExtendParametrisation::Natural)
            .unwrap(),
    );
    let joints = HArray1::with_value(1, 3, 0.0).unwrap();
    joints.change_array1().set_value(2, 1.0).unwrap();
    joints.change_array1().set_value(3, 2.0).unwrap();
    assert!(
        grid.get()
            .unwrap()
            .set_u_joint_values(&joints.array1())
            .unwrap()
    );
    grid
}

fn split_wire_summary(segment: &ShapeFixWireSegment, vertex_count: i32) -> String {
    let mut summary = format!("split_wire {} {vertex_count}", segment.nb_edges());
    for edge in 1..=segment.nb_edges() {
        let patch = segment.patch_index(edge).unwrap();
        summary.push_str(&format!(
            " {}:{}:{}:{}@{}:{}",
            patch.u_min,
            patch.u_max,
            patch.v_min,
            patch.v_max,
            BRepTool::range(&segment.edge(edge).unwrap()).unwrap().0,
            BRepTool::range(&segment.edge(edge).unwrap()).unwrap().1
        ));
    }
    summary
}

fn split_by_line_summary(
    wire: &ShapeFixWireSegment,
    parameters: &Sequence<f64>,
    codes: &Sequence<i32>,
    vertices: &Sequence<rustycad_tkbrep::topods::Shape>,
) -> String {
    let mut summary = format!(
        "split_by_line 1 {} {} {} {}",
        wire.nb_edges(),
        parameters.length(),
        codes.length(),
        vertices.length()
    );
    for index in 1..=parameters.length() {
        let vertex =
            rustycad_tkbrep::topods::vertex(vertices.value(index as usize).unwrap()).unwrap();
        let point = BRepTool::pnt(&vertex).unwrap();
        summary.push_str(&format!(
            " {}:{}:{}:{}",
            parameters.value(index as usize).unwrap(),
            codes.value(index as usize).unwrap(),
            point.x(),
            point.y()
        ));
    }
    summary
}
