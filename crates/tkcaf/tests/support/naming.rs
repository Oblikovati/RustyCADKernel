//! The file-static naming helpers of TNaming_Tool_Test.cxx that several test targets share:
//! label children, sub-shape collections, TNaming_Builder evolutions, shape expectations, the
//! A3/A5 triangle and the selection valid-label map.
use super::{explore, find_named_shape};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingNamingTool, TNamingSelector, TNamingTool};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{
    TdfAttributeMap, TdfAttributeState, TdfChildIterator, TdfLabel, TdfLabelMap, TdfTool,
};
use rustycad_tkmath::gp::{Axis1, Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeVertex, BRepBuilderAPIMakeWire,
};

/// `label.FindChild(tag, true)`.
pub fn child(label: &TdfLabel, tag: i32) -> TdfLabel {
    label.find_child(tag, true).unwrap()
}

/// The file-static `CollectNamingSubShapes`: the distinct sub-shapes of a type (the shape
/// itself excluded).
pub fn collect_naming_sub_shapes(shape: &Shape, shape_type: ShapeEnum) -> Vec<Shape> {
    let mut seen = MapOfShape::default();
    seen.add(shape.clone());
    explore(shape, shape_type)
        .into_iter()
        .filter(|current| seen.add(current.clone()))
        .collect()
}

/// `AddNamingPrimitives(label, shapes, count)`.
pub fn add_naming_primitives(label: &TdfLabel, shapes: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for shape in shapes {
        builder.generated(shape).unwrap();
    }
}

/// `AddNamingPrimitives(label, &shape, 1)`.
pub fn add_primitive(label: &TdfLabel, shape: &Shape) {
    add_naming_primitives(label, std::slice::from_ref(shape));
}

/// `AddNamingGenerated(label, olds, news, count)`.
pub fn add_naming_generated(label: &TdfLabel, olds: &[Shape], news: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for (old, new) in olds.iter().zip(news) {
        builder.generated_from(old, new).unwrap();
    }
}

/// `AddNamingModified(label, olds, news, count)`.
pub fn add_naming_modified(label: &TdfLabel, olds: &[Shape], news: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for (old, new) in olds.iter().zip(news) {
        builder.modify(old, new).unwrap();
    }
}

/// `AddNamingDeleted(label, shapes, count)`.
pub fn add_naming_deleted(label: &TdfLabel, shapes: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for shape in shapes {
        builder.delete(shape).unwrap();
    }
}

/// `AddNamingSelected(label, shapes, contexts, count)`.
pub fn add_naming_selected(label: &TdfLabel, shapes: &[Shape], contexts: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for (shape, context) in shapes.iter().zip(contexts) {
        builder.select(shape, context).unwrap();
    }
}

/// `ExpectNamingCurrentShape(label, expected)`.
pub fn expect_naming_current_shape(label: &TdfLabel, expected: &Shape) {
    let named = find_named_shape(label).expect("the label has a named shape");
    let current = TNamingTool::current_shape(&named).unwrap();
    assert!(
        current.is_same(expected),
        "CurrentShape mismatch at {label:?}"
    );
}

/// `ExpectNamingGetShape(label, expected)`.
pub fn expect_naming_get_shape(label: &TdfLabel, expected: &Shape) {
    let named = find_named_shape(label).expect("the label has a named shape");
    let shape = TNamingTool::get_shape(&named).unwrap();
    assert!(shape.is_same(expected), "GetShape mismatch at {label:?}");
}

/// `ExpectNamingShape(label, expected)`: both the current and the stored shape.
pub fn expect_naming_shape(label: &TdfLabel, expected: &Shape) {
    expect_naming_current_shape(label, expected);
    expect_naming_get_shape(label, expected);
}

/// `TNaming_Tool::GetShape(aSelector.NamedShape())`.
pub fn selected(selector: &TNamingSelector) -> Shape {
    TNamingTool::get_shape(&selector.named_shape().unwrap()).unwrap()
}

fn point(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn direction(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

/// The triangle face of A3 and A5 with its three edges.
pub fn triangle() -> (Shape, [Shape; 3]) {
    let points = [
        point(0.0, 0.0, 0.0),
        point(0.0, 100.0, 0.0),
        point(100.0, 0.0, 0.0),
    ];
    let vertices: Vec<_> = points
        .iter()
        .map(|&p| BRepBuilderAPIMakeVertex::new(p).unwrap().vertex().unwrap())
        .collect();
    let directions = [
        direction(0.0, 1.0, 0.0),
        direction(1.0, -1.0, 0.0),
        direction(-1.0, 0.0, 0.0),
    ];
    let mut edges = Vec::new();
    for index in 0..3 {
        let line: Handle<dyn GeomCurve> =
            Handle::new(GeomLine::new(Axis1::new(points[index], directions[index]))).cast();
        let (first, last) = (&vertices[index], &vertices[(index + 1) % 3]);
        let edge = BRepBuilderAPIMakeEdge::from_curve_vertices(&line, first, last)
            .unwrap()
            .edge()
            .unwrap();
        edges.push(edge);
    }
    let mut wire = BRepBuilderAPIMakeWire::new();
    for edge in &edges {
        wire.add_edge(edge).unwrap();
    }
    assert!(wire.is_done());
    let plane = Plane3::new(point(0.0, 0.0, 0.0), direction(0.0, 0.0, 1.0)).unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_wire(plane, &wire.wire().unwrap(), true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape();
    let edges = edges
        .into_iter()
        .map(|e| e.into_shape())
        .collect::<Vec<_>>();
    (face, [edges[0].clone(), edges[1].clone(), edges[2].clone()])
}

/// The file-static `FillSelectionValidMap(theLabel, theValidMap)`: the labels outside the
/// selection that its sub-labels refer to, with the descendants of their named shapes (the
/// referenced attributes accumulate over the sub-labels, as natively).
pub fn fill_selection_valid_map(label: &TdfLabel, valid: &mut TdfLabelMap) {
    let mut external = TdfAttributeMap::default();
    let mut children = TdfChildIterator::new(label, true).unwrap();
    while children.more() {
        let child_label = children.value();
        TdfTool::out_references(&child_label, &mut external).unwrap();
        for attribute in external.iter() {
            let attribute_label = attribute.get().unwrap().label();
            if attribute_label != child_label && !attribute_label.is_descendant(label) {
                valid.add(attribute_label.clone());
                if let Some(named) = find_named_shape(&attribute_label) {
                    TNamingNamingTool::build_descendants(&named, valid).unwrap();
                }
            }
        }
        children.next();
    }
}

/// The SelectShape command flow that closes A3 and A5: the triangle face, its first and second
/// edges, the first face vertex and the first edge and vertex of `solid` (`sub_shapes`, in that
/// order) are selected without geometry under `test_label`/10/1..6 and resolve to themselves;
/// the vertex selection then does not solve with only `valid_label` valid.
pub fn check_selector_command_flow(
    test_label: &TdfLabel,
    face: &Shape,
    edges: &[Shape; 3],
    solid: &Shape,
    sub_shapes: [&Shape; 3],
    valid_label: &TdfLabel,
) {
    let selection_root = child(test_label, 10);
    let selectors: Vec<TNamingSelector> = (1..=6)
        .map(|tag| TNamingSelector::new(&child(&selection_root, tag)))
        .collect();
    let [face_vertex, box_edge, box_vertex] = sub_shapes;
    assert!(selectors[0].select_alone(face, false, false).unwrap());
    assert!(selectors[1].select(&edges[0], face, false, false).unwrap());
    assert!(selectors[2].select_alone(&edges[1], false, false).unwrap());
    assert!(
        selectors[3]
            .select(face_vertex, face, false, false)
            .unwrap()
    );
    assert!(selectors[4].select(box_edge, solid, false, false).unwrap());
    assert!(
        selectors[5]
            .select(box_vertex, solid, false, false)
            .unwrap()
    );
    let expected = [
        face,
        &edges[0],
        &edges[1],
        face_vertex,
        box_edge,
        box_vertex,
    ];
    for (selector, shape) in selectors.iter().zip(expected) {
        assert!(selected(selector).is_same(shape));
    }
    let mut valid = TdfLabelMap::default();
    valid.add(valid_label.clone());
    assert!(!selectors[3].solve(&mut valid).unwrap());
}
