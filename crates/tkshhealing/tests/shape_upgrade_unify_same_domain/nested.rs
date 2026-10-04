use super::*;

const NATIVE_NESTED: &str =
    include_str!("../../../../tools/oracle/tkshhealing_unify_nested_compound.expected");

#[test]
fn planar_wire_edges_merge_inside_a_compound_face() {
    let input = compound_with_face(face_with_split_boundary());
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    let result = unifier.shape();
    assert_nested_native_counts(&result);
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

fn compound_with_face(face: Shape) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = rustycad_tkbrep::topods::Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &face).unwrap();
    compound.into_shape()
}

fn assert_nested_native_counts(result: &Shape) {
    let edges = edge_shapes(result).len();
    let wires = Explorer::new(result, ShapeEnum::Wire, ShapeEnum::Shape)
        .unwrap()
        .count();
    let vertices = Explorer::new(result, ShapeEnum::Vertex, ShapeEnum::Shape)
        .unwrap()
        .count();
    assert_eq!(
        NATIVE_NESTED.trim(),
        format!("nested_compound faces 1 edges {edges} wires {wires} vertices {vertices} valid 1")
    );
    assert_eq!(face_count(result), 1);
    assert_eq!(edges, 4);
    assert_eq!(wires, 1);
    assert_eq!(vertices, 8);
}
