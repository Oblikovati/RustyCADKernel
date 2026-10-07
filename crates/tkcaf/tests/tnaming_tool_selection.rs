//! 1:1 translation of the selection cases of
//! src/ApplicationFramework/TKCAF/GTests/TNaming_Tool_Test.cxx (TNaming_Selector and
//! TNaming_Naming::Name); the other cases are in `tnaming_tool.rs`.
#![allow(non_snake_case)]
mod support;

use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkcaf::tnaming::{
    TNamingBuilder, TNamingNaming, TNamingNamingTool, TNamingSelector, TNamingTool,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{
    TdfAttributeMap, TdfAttributeState, TdfChildIterator, TdfCopyLabel, TdfData, TdfLabel,
    TdfLabelMap, TdfTool,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Plane3, Point3, Transform3, Vector3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeOneAxis, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeVertex, BRepBuilderAPIMakeWire, BRepBuilderAPITransform,
};
use support::{find_named_shape, make_box, shape_at};

fn child(label: &TdfLabel, tag: i32) -> TdfLabel {
    label.find_child(tag, true).unwrap()
}

fn point(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn direction(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

/// `AddNamingPrimitives(label, &shape, 1)`.
fn add_primitive(label: &TdfLabel, shape: &Shape) {
    TNamingBuilder::new(label)
        .unwrap()
        .generated(shape)
        .unwrap();
}

/// The triangle face of A3 and A5 with its three edges.
fn triangle() -> (Shape, [Shape; 3]) {
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

/// `TNaming_Tool::GetShape(aSelector.NamedShape())`.
fn selected(selector: &TNamingSelector) -> Shape {
    TNamingTool::get_shape(&selector.named_shape().unwrap()).unwrap()
}

// caf/nam/A5: SelectShape command flows resolve geometry and reject an insufficient
// valid-label map when solving an intersection selection.
#[test]
fn TNaming_Tool_Test__CafNamedShape_A5_SelectorCommandFlow() {
    let data = TdfData::new();
    let test_label = child(&data.root(), 1);
    let face_label = child(&test_label, 1);
    let edge_label = child(&face_label, 1);
    let (face, edges) = triangle();
    let solid = make_box(100.0, 100.0, 100.0);
    let first_face_vertex = shape_at(&face, ShapeEnum::Vertex, 1);
    let first_box_edge = shape_at(&solid, ShapeEnum::Edge, 1);
    let first_box_vertex = shape_at(&solid, ShapeEnum::Vertex, 1);
    for shape in [&first_face_vertex, &first_box_edge, &first_box_vertex] {
        assert!(!shape.is_null());
    }
    add_primitive(&face_label, &face);
    for (tag, edge) in (1..=3).zip(&edges) {
        add_primitive(&child(&face_label, tag), edge);
    }
    add_primitive(&edge_label, &edges[0]);

    let selection_root = child(&test_label, 10);
    let selectors: Vec<TNamingSelector> = (1..=6)
        .map(|tag| TNamingSelector::new(&child(&selection_root, tag)))
        .collect();
    assert!(selectors[0].select_alone(&face, false, false).unwrap());
    assert!(selectors[1].select(&edges[0], &face, false, false).unwrap());
    assert!(selectors[2].select_alone(&edges[1], false, false).unwrap());
    assert!(
        selectors[3]
            .select(&first_face_vertex, &face, false, false)
            .unwrap()
    );
    assert!(
        selectors[4]
            .select(&first_box_edge, &solid, false, false)
            .unwrap()
    );
    assert!(
        selectors[5]
            .select(&first_box_vertex, &solid, false, false)
            .unwrap()
    );
    let expected = [
        &face,
        &edges[0],
        &edges[1],
        &first_face_vertex,
        &first_box_edge,
        &first_box_vertex,
    ];
    for (selector, shape) in selectors.iter().zip(expected) {
        assert!(selected(selector).is_same(shape));
    }
    let mut valid = TdfLabelMap::default();
    valid.add(edge_label.clone());
    assert!(!selectors[3].solve(&mut valid).unwrap());
}

/// The `TNaming_ToolTest` fixture: a bare data framework.
fn fixture_root() -> (Handle<TdfData>, TdfLabel) {
    let data = TdfData::new();
    let root = data.root();
    (data, root)
}

// caf/bugs/buc60847: a generated box is available through TNaming_Naming.
#[test]
fn TNaming_ToolTest__Buc_CafBug_60847_GeneratedBoxNaming() {
    let (_data, root) = fixture_root();
    let label = child(&root, 2);
    let solid = make_box(10.0, 20.0, 30.0);
    add_primitive(&label, &solid);
    let named = TNamingNaming::name(&label, &solid, &solid, false, false, false).unwrap();
    assert!(!named.is_null());
    assert!(!named.is_empty());
    assert!(TNamingTool::get_shape(&named).unwrap().is_same(&solid));
}

// caf/bugs/buc60862: a generated edge follows the same naming path as a box.
#[test]
fn TNaming_ToolTest__Buc_CafBug_60862_GeneratedEdgeNaming() {
    let (_data, root) = fixture_root();
    let label = child(&root, 2);
    let edge = BRepBuilderAPIMakeEdge::from_points(point(10.0, 20.0, 0.0), point(20.0, 10.0, 0.0))
        .unwrap()
        .edge()
        .unwrap()
        .into_shape();
    assert!(!edge.is_null());
    add_primitive(&label, &edge);
    let named = TNamingNaming::name(&label, &edge, &edge, false, false, false).unwrap();
    assert!(!named.is_null());
    assert!(!named.is_empty());
    assert!(TNamingTool::get_shape(&named).unwrap().is_same(&edge));
}

/// `BRepBuilderAPI_Transform(theShape, theTrsf).Shape()`.
fn transformed(shape: &Shape, transform: Transform3) -> Shape {
    let mut transformer =
        BRepBuilderAPITransform::with_shape(shape, transform, false, false).unwrap();
    transformer.shape().unwrap().clone()
}

// bugs/caf/bug26061: selecting a translated mirrored face does not throw and resolves to
// the current shape of the modified named-shape history.
#[test]
fn TNaming_Tool_Test__CafBug_26061_TranslatedMirroredSelection() {
    let data = TdfData::new();
    let root = data.root();
    let plane = Plane3::new(point(0.0, 0.0, 0.0), direction(0.0, 0.0, 1.0)).unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(plane, (-10.0, 10.0, -10.0, 10.0))
        .unwrap()
        .face()
        .unwrap()
        .into_shape();
    let mut mirror = Transform3::default();
    mirror.set_mirror_plane(Axis2::new(point(0.0, 0.0, 10.0), direction(0.0, 0.0, 1.0)).unwrap());
    let mirrored = transformed(&face, mirror);
    let mut translation = Transform3::default();
    translation.set_translation(Vector3::new(0.0, 0.0, -5.0));
    let translated = transformed(&face, translation);
    let translated_mirrored = transformed(&mirrored, translation);

    add_primitive(&child(&root, 900), &face);
    TNamingBuilder::new(&child(&root, 901))
        .unwrap()
        .generated_from(&face, &mirrored)
        .unwrap();
    TNamingBuilder::new(&child(&root, 902))
        .unwrap()
        .modify(&face, &translated)
        .unwrap();
    TNamingBuilder::new(&child(&root, 903))
        .unwrap()
        .modify(&mirrored, &translated_mirrored)
        .unwrap();
    let modified = find_named_shape(&child(&root, 903)).unwrap();
    let current = TNamingTool::get_shape(&modified).unwrap();
    assert!(!current.is_null());
    let selector = TNamingSelector::new(&child(&root, 904));
    assert!(selector.select_alone(&current, false, false).unwrap());
    let selection = selector.named_shape().unwrap();
    assert!(!selection.is_null());
    assert!(
        TNamingTool::get_shape(&selection)
            .unwrap()
            .is_same(&translated_mirrored)
    );
}

/// The file-static `FillSelectionValidMap(theLabel, theValidMap)`: the labels outside the
/// selection that its sub-labels refer to, with the descendants of their named shapes (the
/// referenced attributes accumulate over the sub-labels, as natively).
fn fill_selection_valid_map(label: &TdfLabel, valid: &mut TdfLabelMap) {
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

/// The D1-D3 flow: every distinct sub-shape of a type of a loaded box is selected with
/// geometry and solved back.
fn select_geometry(kind: ShapeEnum, first_tag: i32) -> usize {
    let data = TdfData::new();
    let root = data.root();
    let solid = make_box(100.0, 200.0, 300.0);
    support::dnaming::load_imported_shape(&child(&root, 1), &solid);
    let mut seen = MapOfShape::default();
    seen.add(solid.clone());
    let mut index = 0;
    for sub_shape in support::explore(&solid, kind) {
        if !seen.add(sub_shape.clone()) {
            continue;
        }
        let selection_label = child(&root, first_tag + index);
        index += 1;
        let selector = TNamingSelector::new(&selection_label);
        assert!(selector.select(&sub_shape, &solid, true, false).unwrap());
        let mut valid = TdfLabelMap::default();
        fill_selection_valid_map(&selection_label, &mut valid);
        assert!(selector.solve(&mut valid).unwrap());
        assert!(selected(&selector).is_same(&sub_shape));
    }
    usize::try_from(index).unwrap()
}

// caf/named_shape/D1: SelectGeometry resolves every box edge.
#[test]
fn TNaming_Tool_Test__CafNamedShape_D1_SelectGeometryEdges() {
    assert_eq!(select_geometry(ShapeEnum::Edge, 1001), 12);
}

// caf/named_shape/D2: SelectGeometry resolves every box face.
#[test]
fn TNaming_Tool_Test__CafNamedShape_D2_SelectGeometryFaces() {
    assert_eq!(select_geometry(ShapeEnum::Face, 2001), 6);
}

// caf/named_shape/D3: SelectGeometry resolves every box vertex.
#[test]
fn TNaming_Tool_Test__CafNamedShape_D3_SelectGeometryVertices() {
    assert_eq!(select_geometry(ShapeEnum::Vertex, 3001), 8);
}

// caf/bugs/bug23799: copying a selection remains safe after its imported source label has
// forgotten its own attributes.
#[test]
fn TNaming_ToolTest__CafBug_23799_CopySelectionAfterSourceForget() {
    let (_data, root) = fixture_root();
    let sphere = BRepPrimAPIMakeSphere::new(100.0)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape();
    assert!(!sphere.is_null());
    let source = child(&root, 1);
    support::dnaming::load_imported_shape(&source, &sphere);
    let face = support::explore(&sphere, ShapeEnum::Face)[0].clone();
    let selection_label = child(&root, 2);
    let selector = TNamingSelector::new(&selection_label);
    assert!(selector.select_alone(&face, true, false).unwrap());
    source.forget_all_attributes(true).unwrap();
    let copy_label = child(&root, 3);
    let mut copier = TdfCopyLabel::new();
    copier.load(&selection_label, &copy_label);
    copier.perform().unwrap();
    assert!(copier.is_done());
}

// bugs/caf/bug25153: retrieving selection arguments remains safe after the imported source
// label has forgotten its attributes.
#[test]
fn TNaming_Tool_Test__CafBug_25153_ArgumentsAfterSourceForget() {
    let data = TdfData::new();
    let root = data.root();
    let source = child(&root, 1);
    let selection_label = child(&root, 2);
    let plane = Plane3::new(point(0.0, 0.0, 0.0), direction(0.0, 0.0, 1.0)).unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(plane, (-100.0, 100.0, -100.0, 100.0))
        .unwrap()
        .face()
        .unwrap()
        .into_shape();
    support::dnaming::load_imported_shape(&source, &face);
    let selector = TNamingSelector::new(&selection_label);
    assert!(selector.select_alone(&face, false, false).unwrap());
    source.forget_all_attributes(true).unwrap();
    let mut arguments = TdfAttributeMap::default();
    assert!(selector.arguments(&mut arguments).is_ok());
}
