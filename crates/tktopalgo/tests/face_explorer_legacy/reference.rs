use super::*;

pub(super) fn classify(face: &Face, point: Point2, use_bounds: bool) -> State {
    let mut explorer = BRepClassFaceExplorer::new(face).unwrap();
    explorer.set_use_bnd_box(use_bounds);
    FClassifier::with_point(&mut explorer, &point, precision::confusion())
        .unwrap()
        .state()
}

pub(super) fn shapes_of(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .collect()
}

fn other_edge(edges: &rustycad_tkernel::ncollection::List<Shape>, edge: &Edge) -> Edge {
    let mut next = Edge::default();
    for candidate in edges.iter() {
        if !candidate.is_null() && !candidate.is_same(edge) {
            next = topods::edge(candidate).unwrap();
        }
    }
    next
}

fn reference_next_edge(edge: &Edge, wire: &Shape) -> Edge {
    let (first, last) = topexp::vertices(edge, true);
    if last.is_null() || last.is_same(&first) {
        return Edge::default();
    }
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(wire, ShapeEnum::Vertex, ShapeEnum::Edge, &mut ancestors)
        .unwrap();
    let Some(edges) = ancestors.seek(&last) else {
        return Edge::default();
    };
    if edges.extent() != 2 {
        return Edge::default();
    }
    other_edge(edges, edge)
}

fn assert_reference_edge(explorer: &BRepClassFaceExplorer, edges: &mut Explorer, wire: &Shape) {
    assert!(edges.more());
    let (record, _) = explorer.current_edge().unwrap();
    let expected = topods::edge(edges.current()).unwrap();
    assert!(record.edge().is_equal(&expected));
    let next = reference_next_edge(&expected, wire);
    if next.is_null() {
        assert!(record.next_edge().is_null());
    } else {
        assert!(record.next_edge().is_equal(&next));
    }
    edges.next().unwrap();
}

pub(super) fn expect_next_edges_match_reference(face: &Face) {
    let mut explorer = BRepClassFaceExplorer::new(face).unwrap();
    let mut wires = Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape).unwrap();
    explorer.init_wires();
    while explorer.more_wires() {
        assert!(wires.more());
        let wire = topods::wire(wires.current()).unwrap();
        let mut edges = Explorer::new(&wire, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
        explorer.init_edges();
        while explorer.more_edges() {
            assert_reference_edge(&explorer, &mut edges, &wire);
            explorer.next_edge();
        }
        assert!(!edges.more());
        explorer.next_wire();
        wires.next().unwrap();
    }
    assert!(!wires.more());
}
