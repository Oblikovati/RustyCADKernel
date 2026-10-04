use super::*;

#[test]
fn dropped_edge_replaces_a_small_neighbor_in_face_maps() {
    assert_eq!(
        drop_small_neighbor_map_row(),
        expected("drop_small_neighbor_map")
    );
}

#[test]
fn linked_face_wire_order_gate_matches_native_forward_and_reversed_wires() {
    assert_eq!(
        linked_face_wire_order_row(false),
        expected("linked_wire_forward")
    );
    assert_eq!(
        linked_face_wire_order_row(true),
        expected("linked_wire_reversed")
    );
}

fn drop_small_neighbor_map_row() -> Vec<f64> {
    let face_a = plane_face(&tiny_wire());
    let face_b = plane_face(&unit_wire());
    let face_c = plane_face(&unit_wire());
    let shape = face_a.clone().into_shape();
    let fixer = configured_wireframe(&shape);
    fixer.set_precision(0.1);
    let edges = edge_shapes(&shape);
    let small_edge = edges[0].clone();
    let small_neighbor = edges[1].clone();
    let before = edges[3].clone();
    let mut small = shape_map(&small_edge);
    small.add(small_neighbor.clone());
    let mut multiple = shape_map(&small_neighbor);
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    edge_faces.bind(
        small_edge.clone(),
        List::from_iter([face_a.clone().into_shape()]),
    );
    edge_faces.bind(
        small_neighbor.clone(),
        List::from_iter([face_c.clone().into_shape()]),
    );
    edge_faces.bind(before, List::from_iter([face_b.into_shape()]));
    let mut face_small = ShapeFixWireframeShapeLists::default();
    face_small.bind(
        face_a.clone().into_shape(),
        List::from_iter([small_edge.clone()]),
    );
    face_small.bind(
        face_c.clone().into_shape(),
        List::from_iter([small_neighbor.clone()]),
    );
    let changed = merge(
        &fixer,
        &mut small,
        &mut edge_faces,
        &mut face_small,
        &mut multiple,
        true,
    );
    let applied_face = applied_shape(&fixer, &shape);
    let applied_neighbor = applied_shape(&fixer, &small_neighbor);
    let neighbor_list = face_small.find(&face_c.clone().into_shape()).unwrap();
    let original_face_list = face_small.find(&face_a.into_shape()).unwrap();
    vec![
        f64::from(changed),
        small.extent() as f64,
        f64::from(small.contains(&applied_neighbor)),
        edge_faces.extent() as f64,
        face_small.extent() as f64,
        f64::from(
            neighbor_list
                .iter()
                .any(|edge| edge.is_same(&applied_neighbor)),
        ),
        f64::from(
            original_face_list
                .iter()
                .any(|edge| edge.is_same(&small_edge)),
        ),
        edge_count(&applied_face),
        f64::from(fixer.status_small_edges(ShapeExtendStatus::Done3)),
    ]
}

fn linked_face_wire_order_row(reverse_linked_wire: bool) -> Vec<f64> {
    let wire = tiny_wire();
    let linked_wire = linked_wire(&wire, reverse_linked_wire);
    let face = plane_face(&wire);
    let linked_face = plane_face(&linked_wire);
    let shape = face.clone().into_shape();
    let fixer = configured_wireframe(&shape);
    fixer.set_precision(0.1);
    let edges = edge_shapes(&shape);
    let small_edge = edges[0].clone();
    let linked_faces = List::from_iter([face.clone().into_shape(), linked_face.into_shape()]);
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    for edge in &edges {
        edge_faces.bind(edge.clone(), linked_faces.clone());
    }
    let mut face_small = ShapeFixWireframeShapeLists::default();
    face_small.bind(
        face.clone().into_shape(),
        List::from_iter([small_edge.clone()]),
    );
    let mut small = shape_map(&small_edge);
    let mut multiple = MapOfShape::default();
    let changed = merge(
        &fixer,
        &mut small,
        &mut edge_faces,
        &mut face_small,
        &mut multiple,
        false,
    );
    linked_order_result(&fixer, &shape, changed)
}

fn linked_wire(wire: &Wire, reversed: bool) -> Wire {
    if reversed {
        return topods::wire(&wire.clone().into_shape().reversed()).unwrap();
    }
    wire.clone()
}

fn linked_order_result(fixer: &ShapeFixWireframe, shape: &Shape, changed: bool) -> Vec<f64> {
    let result = applied_shape(fixer, shape);
    vec![
        f64::from(changed),
        edge_count(&result),
        done_status(fixer),
        f64::from(fixer.status_small_edges(ShapeExtendStatus::Fail1)),
    ]
}
