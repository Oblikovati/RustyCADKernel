use super::*;

#[test]
fn long_overlap_reuses_one_wire_segment_for_the_full_overlap() {
    assert_full_long_overlap_replacement(2.0, 1.0);
}

#[test]
fn reversed_long_overlap_reuses_one_wire_segment_for_the_full_overlap() {
    assert_full_long_overlap_replacement(8.0, -1.0);
}

#[test]
fn contained_first_long_overlap_is_inserted_into_the_second_wire() {
    let mut face = planar_face();
    let first = Handle::new(ShapeExtendWireData::new());
    first
        .add_edge(&line_edge_range(&face, 2.0, 0.0, 0.0, 1.0, 0.0, 6.0), 0)
        .unwrap();
    first
        .add_edge(&line_edge(&face, 30.0, 30.0, 1.0, 0.0), 0)
        .unwrap();
    let second_edge = line_edge_range(&face, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0);
    let second_endpoint = ShapeAnalysisEdge::new().first_vertex(&second_edge);
    BRepBuilder::default()
        .update_vertex_tolerance(&second_endpoint, 1.0e-5)
        .unwrap();
    let second = Handle::new(ShapeExtendWireData::new());
    second.add_edge(&second_edge, 0).unwrap();
    second
        .add_edge(&line_edge(&face, -30.0, -30.0, 0.0, 1.0), 0)
        .unwrap();
    add_wires(&mut face, &first, &second);
    let old_face = face.clone().into_shape();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    assert_eq!([wires[0].nb_children(), wires[1].nb_children()], [2, 4]);
    assert_eq!(
        wires
            .iter()
            .flat_map(|wire| direct_edge_ranges(wire, &face))
            .collect::<Vec<_>>(),
        [0.0, 6.0, 0.0, 2.0, 0.0, 2.0, 0.0, 6.0, 8.0, 10.0, 0.0, 2.0]
    );
    let first_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[0]).unwrap(),
        true,
        true,
    )
    .unwrap();
    let second_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[1]).unwrap(),
        true,
        true,
    )
    .unwrap();
    assert!(
        first_data
            .edge(1)
            .unwrap()
            .is_same(&second_data.edge(2).unwrap())
    );
    let applied = context
        .apply(&old_face, rustycad_tkg3d::top_abs::ShapeEnum::Shape)
        .unwrap();
    assert!(applied.is_same(&face.clone().into_shape()));
}

#[test]
fn partial_long_overlap_inserts_the_source_middle_and_keeps_target_tail() {
    assert_partial_long_overlap(
        0.0,
        1.0,
        -2.0,
        1.0,
        1,
        2,
        [0.0, 8.0, 8.0, 10.0, 0.0, 2.0, 0.0, 2.0, 0.0, 8.0, 0.0, 2.0],
    );
}

#[test]
fn partial_long_overlap_keeps_source_head_and_inserts_target_middle() {
    assert_partial_long_overlap(
        0.0,
        1.0,
        2.0,
        1.0,
        2,
        1,
        [
            0.0, 2.0, 2.0, 10.0, 0.0, 2.0, 2.0, 10.0, 8.0, 10.0, 0.0, 2.0,
        ],
    );
}

#[test]
fn reversed_partial_long_overlap_replaces_the_native_split_slot() {
    assert_partial_long_overlap(
        0.0,
        1.0,
        8.0,
        -1.0,
        1,
        2,
        [0.0, 8.0, 8.0, 10.0, 0.0, 2.0, 0.0, 8.0, 0.0, 8.0, 0.0, 2.0],
    );
}

#[test]
fn reversed_source_partial_long_overlap_uses_native_target_slot() {
    assert_partial_long_overlap(
        8.0,
        -1.0,
        2.0,
        1.0,
        1,
        2,
        [0.0, 6.0, 6.0, 10.0, 0.0, 2.0, 0.0, 6.0, 0.0, 6.0, 0.0, 2.0],
    );
}

fn assert_partial_long_overlap(
    first_start: f64,
    first_direction: f64,
    second_start: f64,
    second_direction: f64,
    source_middle_index: i32,
    target_middle_index: i32,
    expected_ranges: [f64; 12],
) {
    let mut face = planar_face();
    let first = Handle::new(ShapeExtendWireData::new());
    first
        .add_edge(
            &line_edge_range(&face, first_start, 0.0, 0.0, first_direction, 0.0, 10.0),
            0,
        )
        .unwrap();
    first
        .add_edge(&line_edge(&face, 30.0, 30.0, 1.0, 0.0), 0)
        .unwrap();
    let second_edge = line_edge_range(&face, second_start, 0.0, 0.0, second_direction, 0.0, 10.0);
    let endpoint = ShapeAnalysisEdge::new().first_vertex(&second_edge);
    BRepBuilder::default()
        .update_vertex_tolerance(&endpoint, 1.0e-5)
        .unwrap();
    let second = Handle::new(ShapeExtendWireData::new());
    second.add_edge(&second_edge, 0).unwrap();
    second
        .add_edge(&line_edge(&face, -30.0, -30.0, 0.0, 1.0), 0)
        .unwrap();
    add_wires(&mut face, &first, &second);
    let old_face = face.clone().into_shape();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    assert_eq!([wires[0].nb_children(), wires[1].nb_children()], [3, 3]);
    assert_eq!(
        wires
            .iter()
            .flat_map(|wire| direct_edge_ranges(wire, &face))
            .collect::<Vec<_>>(),
        expected_ranges
    );
    let first_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[0]).unwrap(),
        true,
        true,
    )
    .unwrap();
    let second_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[1]).unwrap(),
        true,
        true,
    )
    .unwrap();
    assert!(
        first_data
            .edge(source_middle_index)
            .unwrap()
            .is_same(&second_data.edge(target_middle_index).unwrap())
    );
    let applied = context
        .apply(&old_face, rustycad_tkg3d::top_abs::ShapeEnum::Shape)
        .unwrap();
    assert!(applied.is_same(&face.clone().into_shape()));
}

fn assert_full_long_overlap_replacement(second_start: f64, second_direction: f64) {
    let mut face = planar_face();
    let first_edge = line_edge_range(&face, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0);
    let first_vertex = ShapeAnalysisEdge::new().first_vertex(&first_edge);
    BRepBuilder::default()
        .update_vertex_tolerance(&first_vertex, 1.0e-5)
        .unwrap();
    let first = Handle::new(ShapeExtendWireData::new());
    first.add_edge(&first_edge, 0).unwrap();
    first
        .add_edge(&line_edge(&face, 30.0, 30.0, 1.0, 0.0), 0)
        .unwrap();
    let second = Handle::new(ShapeExtendWireData::new());
    second
        .add_edge(
            &line_edge_range(&face, second_start, 0.0, 0.0, second_direction, 0.0, 6.0),
            0,
        )
        .unwrap();
    second
        .add_edge(&line_edge(&face, -30.0, -30.0, 0.0, 1.0), 0)
        .unwrap();
    add_wires(&mut face, &first, &second);
    let old_face = face.clone().into_shape();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    assert_eq!([wires[0].nb_children(), wires[1].nb_children()], [4, 2]);
    let first_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[0]).unwrap(),
        true,
        true,
    )
    .unwrap();
    let second_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[1]).unwrap(),
        true,
        true,
    )
    .unwrap();
    assert!(
        first_data
            .edge(2)
            .unwrap()
            .is_same(&second_data.edge(1).unwrap())
    );
    let applied = context
        .apply(&old_face, rustycad_tkg3d::top_abs::ShapeEnum::Shape)
        .unwrap();
    assert!(applied.is_same(&face.clone().into_shape()));
    assert_eq!(
        wires
            .iter()
            .flat_map(|wire| direct_edge_ranges(wire, &face))
            .collect::<Vec<_>>(),
        [0.0, 2.0, 2.0, 8.0, 8.0, 10.0, 0.0, 2.0, 2.0, 8.0, 0.0, 2.0]
    );
}

#[test]
fn two_sided_endpoint_cut_trims_long_edge_without_replacing_face() {
    assert_endpoint_cut(0.5, (1.5, 1000.0), false, true);
}

#[test]
fn far_endpoint_cut_trims_long_edge_without_replacing_face() {
    assert_endpoint_cut(999.0, (0.0, 999.0), true, true);
}

#[test]
fn failed_short_edge_cut_skips_long_edge_when_short_is_first() {
    assert_endpoint_cut(0.0, (0.0, 1000.0), false, false);
}

#[test]
fn one_sided_endpoint_cut_trims_edge_without_replacing_face() {
    assert_endpoint_cut(0.0, (1.0, 1000.0), false, true);
}

fn assert_endpoint_cut(
    second_start: f64,
    expected_long_range: (f64, f64),
    use_last_vertex_tolerance: bool,
    long_edge_first: bool,
) {
    let mut face = planar_face();
    let long_edge = line_edge_range(&face, 0.0, 0.0, 0.0, 1.0, 0.0, 1000.0);
    let analysis = ShapeAnalysisEdge::new();
    let endpoint = if use_last_vertex_tolerance {
        analysis.last_vertex(&long_edge)
    } else {
        analysis.first_vertex(&long_edge)
    };
    BRepBuilder::default()
        .update_vertex_tolerance(&endpoint, 2.0)
        .unwrap();
    let short_edge = line_edge_range(&face, second_start, 0.0, 0.0, 1.0, 0.0, 1.0);
    let (first_edge, second_edge) = if long_edge_first {
        (&long_edge, &short_edge)
    } else {
        (&short_edge, &long_edge)
    };
    let first = endpoint_cut_wire(&face, first_edge, (3000.0, 3000.0, 1.0, 0.0));
    let second = endpoint_cut_wire(&face, second_edge, (-3000.0, -3000.0, 0.0, 1.0));
    add_wires(&mut face, &first, &second);
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(!tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    assert_eq!([wires[0].nb_children(), wires[1].nb_children()], [2, 2]);
    assert_eq!(
        wires
            .iter()
            .flat_map(|wire| direct_edge_ranges(wire, &face))
            .collect::<Vec<_>>(),
        if long_edge_first {
            [
                expected_long_range.0,
                expected_long_range.1,
                0.0,
                2.0,
                0.0,
                1.0,
                0.0,
                2.0,
            ]
        } else {
            [
                0.0,
                1.0,
                0.0,
                2.0,
                expected_long_range.0,
                expected_long_range.1,
                0.0,
                2.0,
            ]
        }
    );
}

fn endpoint_cut_wire(
    face: &Face,
    edge: &rustycad_tkbrep::topods::Edge,
    far: (f64, f64, f64, f64),
) -> Handle<ShapeExtendWireData> {
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(edge, 0).unwrap();
    wire.add_edge(&line_edge(face, far.0, far.1, far.2, far.3), 0)
        .unwrap();
    wire
}
