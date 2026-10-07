//! 1:1 translation of the Boolean cases of
//! src/ApplicationFramework/TKCAF/GTests/TNaming_Tool_Test.cxx: selections solved after a
//! fuse or cut is recomputed (F2, F3, E9, F8, F9) and the naming graph of a prism cut (A3);
//! the other cases are in `tnaming_tool.rs` and `tnaming_tool_selection.rs`.
#![allow(non_snake_case)]
mod support;

use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICut, BRepAlgoAPIFuse};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingSelector, TNamingTool};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfData, TdfLabel, TdfLabelMap};
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeOneAxis, BRepPrimAPIMakePrism, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use support::naming::{
    add_naming_generated, add_naming_modified, add_primitive, check_selector_command_flow, child,
    collect_naming_sub_shapes, expect_naming_current_shape, expect_naming_get_shape,
    expect_naming_shape, fill_selection_valid_map, selected, triangle,
};
use support::{find_named_shape, make_box, make_box_at, shape_at};

/// `BRepAlgoAPI_Fuse(S1, S2).Shape()`.
fn fuse(object: &Shape, tool: &Shape) -> Shape {
    BRepAlgoAPIFuse::from_shapes(object, tool)
        .unwrap()
        .shape()
        .clone()
}

/// `BRepAlgoAPI_Cut(S1, S2).Shape()`.
fn cut(object: &Shape, tool: &Shape) -> Shape {
    BRepAlgoAPICut::from_shapes(object, tool)
        .unwrap()
        .shape()
        .clone()
}

/// `BRepPrimAPI_MakeSphere(gp_Pnt(x, y, z), radius).Shape()`.
fn make_sphere_at(center: [f64; 3], radius: f64) -> Shape {
    let center = Point3::new(center[0], center[1], center[2]);
    BRepPrimAPIMakeSphere::at_center(center, radius)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape()
}

/// The file-static `FirstSubShape(theShape, theType)`: the first explored sub-shape, or a null
/// shape.
fn first_sub_shape(shape: &Shape, shape_type: ShapeEnum) -> Shape {
    support::explore(shape, shape_type)
        .into_iter()
        .next()
        .unwrap_or_default()
}

/// The file-static `SolveSelectionAfterModification(theInitial, theModified)`: the first face
/// of an imported shape is selected with geometry, the shape is modified into `modified`, and
/// the selection is solved against its valid-label map; a null shape on any failure.
fn solve_selection_after_modification(initial: &Shape, modified: &Shape) -> Shape {
    let data = TdfData::new();
    let root = data.root();
    let context_label = child(&root, 1);
    let selection_label = child(&root, 2);
    support::dnaming::load_imported_shape(&context_label, initial);

    let selection = first_sub_shape(initial, ShapeEnum::Face);
    let selector = TNamingSelector::new(&selection_label);
    if selection.is_null() || !selector.select(&selection, initial, true, false).unwrap() {
        return Shape::default();
    }
    TNamingBuilder::new(&context_label)
        .unwrap()
        .modify(initial, modified)
        .unwrap();

    let mut valid = TdfLabelMap::default();
    fill_selection_valid_map(&selection_label, &mut valid);
    if !selector.solve(&mut valid).unwrap() {
        return Shape::default();
    }
    selected(&selector)
}

/// The shared tail of F2..F9: the solved selection is a face.
fn assert_selected_face(initial: &Shape, modified: &Shape) {
    let selection = solve_selection_after_modification(initial, modified);
    assert!(!selection.is_null());
    assert_eq!(selection.shape_type().unwrap(), ShapeEnum::Face);
}

#[test]
fn TNaming_Tool_Test__CafNamedShape_F2_FusedSelectionAfterRecompute() {
    let first = make_box(130.0, 140.0, 150.0);
    let second = make_box_at([80.0, 20.0, 20.0], 80.0, 100.0, 120.0);
    let fused = fuse(&first, &second);
    let changed_second = make_box_at([80.0, 20.0, 20.0], 80.0, 100.0, 140.0);
    let changed_fused = fuse(&first, &changed_second);
    assert_selected_face(&fused, &changed_fused);
}

#[test]
fn TNaming_Tool_Test__CafNamedShape_F3_CutSelectionAfterRecompute() {
    let first = make_box(130.0, 140.0, 150.0);
    let second = make_box_at([20.0, 20.0, 20.0], 30.0, 30.0, 30.0);
    let cut_shape = cut(&first, &second);
    let changed_second = make_box_at([30.0, 20.0, 20.0], 30.0, 30.0, 30.0);
    let changed_cut = cut(&first, &changed_second);
    assert_selected_face(&cut_shape, &changed_cut);
}

#[test]
fn TNaming_Tool_Test__CafNamedShape_E9_SelectionAfterMultipleFusions() {
    let solid = make_box(130.0, 140.0, 150.0);
    let sphere1 = make_sphere_at([20.0, 20.0, 20.0], 25.0);
    let sphere2 = make_sphere_at([70.0, 70.0, 70.0], 25.0);
    let fused = fuse(&fuse(&solid, &sphere1), &sphere2);
    let changed_sphere = make_sphere_at([70.0, 70.0, 80.0], 25.0);
    let changed_fused = fuse(&fuse(&solid, &sphere1), &changed_sphere);
    assert_selected_face(&fused, &changed_fused);
}

/// The F8/F9 flow: three boxes fused, recomputed with a second box of height `changed_dz`.
fn assert_three_box_fusion_selection(changed_dz: f64) {
    let box1 = make_box(100.0, 200.0, 350.0);
    let box2 = make_box_at([80.0, 80.0, 20.0], 330.0, 330.0, 90.0);
    let box3 = make_box_at([40.0, 40.0, 40.0], 60.0, 450.0, 150.0);
    let fused = fuse(&fuse(&box1, &box2), &box3);
    let changed_box2 = make_box_at([80.0, 80.0, 20.0], 330.0, 330.0, changed_dz);
    let changed_fused = fuse(&fuse(&box1, &changed_box2), &box3);
    assert_selected_face(&fused, &changed_fused);
}

#[test]
fn TNaming_Tool_Test__CafNamedShape_F8_SelectShapeAfterFusion() {
    assert_three_box_fusion_selection(120.0);
}

#[test]
fn TNaming_Tool_Test__CafNamedShape_F9_AttachedSelectionAfterFusion() {
    assert_three_box_fusion_selection(125.0);
}

/// The file-static `NamingVertexAverage`: the mean of the explored vertex points (a vertex
/// shared by several edges counts once per occurrence, as natively).
fn naming_vertex_average(shape: &Shape) -> Point3 {
    let vertices = support::explore(shape, ShapeEnum::Vertex);
    if vertices.is_empty() {
        return Point3::default();
    }
    let mut sum = [0.0; 3];
    for vertex in &vertices {
        let point = BRepTool::pnt(&topods::vertex(vertex).unwrap()).unwrap();
        sum[0] += point.x();
        sum[1] += point.y();
        sum[2] += point.z();
    }
    let count = vertices.len() as f64;
    Point3::new(sum[0] / count, sum[1] / count, sum[2] / count)
}

/// The file-static `NamingNear`.
fn naming_near(value: f64, expected: f64) -> bool {
    (value - expected).abs() <= 1.0e-7 // tol: the upstream literal of NamingNear
}

/// `ExpectNamingLabel(theShape, theExpectedLabel, theAccessLabel)`.
fn expect_naming_label(shape: &Shape, expected: &TdfLabel, access: &TdfLabel) {
    let named = TNamingTool::named_shape(shape, access).unwrap();
    assert!(!named.is_null());
    assert_eq!(&named.get().unwrap().label(), expected);
}

/// `ExpectNamingCompoundFaces(theLabel, theCurrentCount, theShapeCount)`.
fn expect_naming_compound_faces(label: &TdfLabel, current_count: usize, shape_count: usize) {
    let named = find_named_shape(label).expect("the label has a named shape");
    let current = TNamingTool::current_shape(&named).unwrap();
    let stored = TNamingTool::get_shape(&named).unwrap();
    assert_eq!(
        collect_naming_sub_shapes(&current, ShapeEnum::Face).len(),
        current_count
    );
    assert_eq!(
        collect_naming_sub_shapes(&stored, ShapeEnum::Face).len(),
        shape_count
    );
}

/// (old, new) evolution pairs, split into the two arrays of `AddNamingGenerated` and
/// `AddNamingModified`.
fn unzip_pairs<'a>(pairs: impl Iterator<Item = &'a (Shape, Shape)>) -> (Vec<Shape>, Vec<Shape>) {
    pairs.cloned().unzip()
}

/// The A3 prism faces: a side face (four edges) is generated from the triangle edges among its
/// first three edges, under `label2`/1; a cap face is a primitive under `label2`/2, `label2`/3.
fn name_prism_faces(prism_faces: &[Shape], edges: &[Shape; 3], label2: &TdfLabel) {
    let mut generated: Vec<(Shape, Shape)> = Vec::new();
    let mut primitive_tag = 2;
    for face in prism_faces {
        let face_edges = collect_naming_sub_shapes(face, ShapeEnum::Edge);
        if face_edges.len() != 4 {
            add_primitive(&child(label2, primitive_tag), face);
            primitive_tag += 1;
            continue;
        }
        let shared = edges.iter().flat_map(|old| {
            face_edges[..3]
                .iter()
                .filter(|new| old.is_same(new))
                .map(move |_| old)
        });
        for old in shared.take(8 - generated.len().min(8)) {
            generated.push((old.clone(), face.clone()));
        }
    }
    assert_eq!(generated.len(), 3);
    let (olds, news) = unzip_pairs(generated.iter());
    add_naming_generated(&child(label2, 1), &olds, &news);
}

/// The prism faces centred on x = 50 (`PX`), y = 50 (`PY`) or both (`PXY`).
fn prism_side_faces(prism_faces: &[Shape]) -> [Shape; 3] {
    let (mut px, mut py, mut pxy) = (Shape::default(), Shape::default(), Shape::default());
    for face in prism_faces {
        let center = naming_vertex_average(face);
        match (naming_near(center.x(), 50.0), naming_near(center.y(), 50.0)) {
            (true, true) => pxy = face.clone(),
            (true, false) => px = face.clone(),
            (false, true) => py = face.clone(),
            (false, false) => {}
        }
    }
    for side in [&px, &py, &pxy] {
        assert!(!side.is_null());
    }
    [px, py, pxy]
}

/// The origin of an A3 cut face: `Some((true, side))` for a face generated from a prism side
/// `[PX, PY, PXY]`, `Some((false, box face))` for a face at z = 20 or z = 40 modified from the
/// box faces 5 and 6.
fn cut_face_origin<'a>(
    face: &Shape,
    sides: &'a [Shape; 3],
    box_faces: &'a [Shape],
) -> Option<(bool, &'a Shape)> {
    let [px, py, pxy] = sides;
    let center = naming_vertex_average(face);
    if naming_near(center.x(), 50.0) {
        return Some((
            true,
            if naming_near(center.y(), 50.0) {
                pxy
            } else {
                px
            },
        ));
    }
    if naming_near(center.y(), 50.0) {
        return Some((true, py));
    }
    if naming_near(center.z(), 20.0) {
        return Some((false, &box_faces[4]));
    }
    naming_near(center.z(), 40.0).then_some((false, &box_faces[5]))
}

/// The (old, new) generated and modified pairs of the A3 cut faces, in cut-face order.
fn cut_face_history(
    cut_faces: &[Shape],
    sides: &[Shape; 3],
    box_faces: &[Shape],
) -> [Vec<(Shape, Shape)>; 2] {
    let mut history: [Vec<(Shape, Shape)>; 2] = [Vec::new(), Vec::new()];
    for face in cut_faces {
        if let Some((generated, old)) = cut_face_origin(face, sides, box_faces) {
            history[usize::from(!generated)].push((old.clone(), face.clone()));
        }
    }
    history
}

/// The expected label of an A3 cut face: `[root, label1, label2/3, label41, label42]`.
fn expect_cut_face_label(face: &Shape, prism_faces: &[Shape], labels: [&TdfLabel; 5]) {
    let [root, label1, label23, label41, label42] = labels;
    let center = naming_vertex_average(face);
    if face.is_same(&prism_faces[3]) {
        expect_naming_label(face, label1, root);
    } else if face.is_same(&prism_faces[4]) {
        expect_naming_label(face, label23, root);
    } else if naming_near(center.x(), 50.0) || naming_near(center.y(), 50.0) {
        expect_naming_label(face, label41, root);
    } else if naming_near(center.z(), 20.0) || naming_near(center.z(), 40.0) {
        expect_naming_label(face, label42, root);
    } else {
        panic!("Unclassified cut face in A3 naming graph");
    }
}

/// The A3 labels under the test label: 1, 1/1..1/3, 2, 2/1, 3, 4, 4/1, 4/2.
struct A3Labels {
    label1: TdfLabel,
    edges: [TdfLabel; 3],
    label2: TdfLabel,
    label21: TdfLabel,
    label3: TdfLabel,
    label4: TdfLabel,
    label41: TdfLabel,
    label42: TdfLabel,
}

impl A3Labels {
    fn new(test_label: &TdfLabel) -> Self {
        let label1 = child(test_label, 1);
        let edges = [child(&label1, 1), child(&label1, 2), child(&label1, 3)];
        let label2 = child(test_label, 2);
        let label21 = child(&label2, 1);
        let label3 = child(test_label, 3);
        let label4 = child(test_label, 4);
        let (label41, label42) = (child(&label4, 1), child(&label4, 2));
        Self {
            label1,
            edges,
            label2,
            label21,
            label3,
            label4,
            label41,
            label42,
        }
    }
}

/// The A3 shapes: the triangle face and edges, the prism, the box and the cut, with the
/// distinct faces of the prism, box and cut.
struct A3Shapes {
    face: Shape,
    edges: [Shape; 3],
    prism: Shape,
    prism_faces: Vec<Shape>,
    solid: Shape,
    box_faces: Vec<Shape>,
    cut: Shape,
    cut_faces: Vec<Shape>,
}

/// The A3 graph up to the cut: the triangle (1, 1/1..1/3), the prism and its faces (2, 2/n),
/// the box and its faces (3, 3/1..3/6) and the cut modification (4).
fn build_a3_primitives(labels: &A3Labels) -> A3Shapes {
    let (face, edges) = triangle();
    add_primitive(&labels.label1, &face);
    for (label, edge) in labels.edges.iter().zip(&edges) {
        add_primitive(label, edge);
    }
    let prism = BRepPrimAPIMakePrism::new(&face, Vector3::new(0.0, 0.0, 100.0), false, true)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let prism_faces = collect_naming_sub_shapes(&prism, ShapeEnum::Face);
    assert_eq!(prism_faces.len(), 5);
    add_primitive(&labels.label2, &prism);
    name_prism_faces(&prism_faces, &edges, &labels.label2);

    let solid = make_box_at([0.0, 0.0, 20.0], 100.0, 100.0, 20.0);
    let box_faces = collect_naming_sub_shapes(&solid, ShapeEnum::Face);
    assert_eq!(box_faces.len(), 6);
    add_primitive(&labels.label3, &solid);
    for (tag, box_face) in (1..).zip(&box_faces) {
        add_primitive(&child(&labels.label3, tag), box_face);
    }

    let cut = cut(&prism, &solid);
    assert!(!cut.is_null());
    let cut_faces = collect_naming_sub_shapes(&cut, ShapeEnum::Face);
    assert_eq!(cut_faces.len(), 10);
    add_naming_modified(
        &labels.label4,
        std::slice::from_ref(&prism),
        std::slice::from_ref(&cut),
    );
    A3Shapes {
        face,
        edges,
        prism,
        prism_faces,
        solid,
        box_faces,
        cut,
        cut_faces,
    }
}

/// The A3 cut history: the side faces modified under 4/1 and the box faces generated under 4/2,
/// both in reverse cut-face order, 4/2 closed by the first box face onto itself.
fn name_a3_cut_faces(labels: &A3Labels, shapes: &A3Shapes) {
    let sides = prism_side_faces(&shapes.prism_faces);
    let [generated, modified] = cut_face_history(&shapes.cut_faces, &sides, &shapes.box_faces);
    assert_eq!(generated.len(), 6);
    assert_eq!(modified.len(), 2);
    let (olds, news) = unzip_pairs(generated.iter().rev());
    add_naming_modified(&labels.label41, &olds, &news);
    let first_box_face = (shapes.box_faces[0].clone(), shapes.box_faces[0].clone());
    let (olds, news) = unzip_pairs(modified.iter().rev().chain([&first_box_face]));
    add_naming_generated(&labels.label42, &olds, &news);
}

/// The A3 `ExpectNamingLabel` block: each shape is found on its label from the root.
fn expect_a3_labels(root: &TdfLabel, labels: &A3Labels, shapes: &A3Shapes) {
    for (edge, label) in shapes.edges.iter().zip(&labels.edges) {
        expect_naming_label(edge, label, root);
    }
    expect_naming_label(&shapes.prism, &labels.label2, root);
    for prism_face in &shapes.prism_faces[0..3] {
        expect_naming_label(prism_face, &labels.label21, root);
    }
    expect_naming_label(&shapes.prism_faces[4], &child(&labels.label2, 3), root);
    expect_naming_label(&shapes.solid, &labels.label3, root);
    for (tag, box_face) in (1..).zip(&shapes.box_faces) {
        expect_naming_label(box_face, &child(&labels.label3, tag), root);
    }
    expect_naming_label(&shapes.cut, &labels.label4, root);
    let label23 = child(&labels.label2, 3);
    let cut_labels = [
        root,
        &labels.label1,
        &label23,
        &labels.label41,
        &labels.label42,
    ];
    for cut_face in &shapes.cut_faces {
        expect_cut_face_label(cut_face, &shapes.prism_faces, cut_labels);
    }
}

/// The A3 `ExpectNamingShape` block: the current and stored shapes of every label.
fn expect_a3_shapes(labels: &A3Labels, shapes: &A3Shapes) {
    expect_naming_shape(&labels.label1, &shapes.face);
    for (label, edge) in labels.edges.iter().zip(&shapes.edges) {
        expect_naming_shape(label, edge);
    }
    expect_naming_current_shape(&labels.label2, &shapes.cut);
    expect_naming_get_shape(&labels.label2, &shapes.prism);
    expect_naming_shape(&child(&labels.label2, 2), &shapes.prism_faces[3]);
    expect_naming_shape(&child(&labels.label2, 3), &shapes.prism_faces[4]);
    expect_naming_shape(&labels.label3, &shapes.solid);
    for (tag, box_face) in (1..).zip(&shapes.box_faces) {
        expect_naming_shape(&child(&labels.label3, tag), box_face);
    }
    expect_naming_shape(&labels.label4, &shapes.cut);
    expect_naming_compound_faces(&labels.label21, 6, 3);
    expect_naming_compound_faces(&labels.label41, 6, 6);
    expect_naming_compound_faces(&labels.label42, 2, 2);
}

// caf/nam/A3: NamedShape, CurrentShape, GetShape, and selector resolution on a triangle prism
// cut preserve the complete DRAW naming graph.
#[test]
fn TNaming_Tool_Test__CafNamedShape_A3_NamingToolGraph() {
    let data = TdfData::new();
    let root = data.root();
    let test_label = child(&root, 1);
    let labels = A3Labels::new(&test_label);
    let shapes = build_a3_primitives(&labels);
    name_a3_cut_faces(&labels, &shapes);
    expect_a3_labels(&root, &labels, &shapes);
    expect_a3_shapes(&labels, &shapes);

    let first_face_vertex = shape_at(&shapes.face, ShapeEnum::Vertex, 1);
    let first_box_edge = shape_at(&shapes.solid, ShapeEnum::Edge, 1);
    let first_box_vertex = shape_at(&shapes.solid, ShapeEnum::Vertex, 1);
    let sub_shapes = [&first_face_vertex, &first_box_edge, &first_box_vertex];
    let (face, edges, solid) = (&shapes.face, &shapes.edges, &shapes.solid);
    check_selector_command_flow(
        &test_label,
        face,
        edges,
        solid,
        sub_shapes,
        &labels.edges[0],
    );
}
