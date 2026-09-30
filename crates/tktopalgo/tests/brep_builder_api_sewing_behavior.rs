//! Behavior tests of the sewing group of BRepBuilderAPI (Sewing, FastSewing,
//! VertexInspector and BndBoxTreeSelector) on faces built with the BRepLib makers:
//! gaps below and above the tolerance, the options, the queries and their errors, the
//! statuses of the fast sewing and the two helpers. The numeric probes compare the same
//! members with OCCT bit for bit (tools/tktopalgo_corpus_sewing.py).
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::ncollection::{CellAction, CellFilter, TreeSelector, UBTree, UBTreeFiller};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Direction3, Point3, Xyz};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIBndBoxTreeSelector, BRepBuilderAPIFastSewing, BRepBuilderAPISewing,
    BRepBuilderAPIVertexInspector, FastSewingStatus,
};
use rustycad_tktopalgo::brep_lib::{BRepLibMakeFace, BRepLibMakePolygon};

/// The planar face of the rectangle [x0, x1] x [y0, y1] at height z.
fn rectangle(x0: f64, y0: f64, x1: f64, y1: f64, z: f64) -> Shape {
    let polygon = BRepLibMakePolygon::from_four_points(
        Point3::new(x0, y0, z),
        Point3::new(x1, y0, z),
        Point3::new(x1, y1, z),
        Point3::new(x0, y1, z),
        true,
    )
    .unwrap();
    let face = BRepLibMakeFace::from_wire(&polygon.wire().unwrap(), true).unwrap();
    face.face().unwrap().into_shape()
}

/// Sews the faces with the tolerance and the default options.
fn sew(faces: &[Shape], tol: f64) -> BRepBuilderAPISewing {
    let mut sewing = BRepBuilderAPISewing::new(tol, true, true, true, false);
    for face in faces {
        sewing.add(face).unwrap();
    }
    sewing.perform(&ProgressRange::default()).unwrap();
    sewing
}

/// The number of distinct sub-shapes of a kind.
fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut map).unwrap();
    map.extent()
}

#[test]
fn gap_below_tolerance_is_sewn() {
    let faces = [
        rectangle(0., 0., 1., 1., 0.),
        rectangle(1. + 1e-4, 0., 2., 1., 0.),
    ];
    let sewing = sew(&faces, 1e-3);
    assert_eq!(sewing.nb_contigous_edges(), 1);
    assert_eq!(sewing.nb_free_edges(), 6);
    assert_eq!(sewing.sewed_shape().shape_type().unwrap(), ShapeEnum::Shell);
    assert_eq!(count(sewing.sewed_shape(), ShapeEnum::Edge), 7);
    assert!(sewing.is_modified(&faces[1]));
    assert_eq!(sewing.contigous_edge_couple(1).unwrap().extent(), 2);
}

#[test]
fn gap_above_tolerance_stays_free() {
    let faces = [
        rectangle(0., 0., 1., 1., 0.),
        rectangle(1. + 1e-2, 0., 2., 1., 0.),
    ];
    let sewing = sew(&faces, 1e-3);
    assert_eq!(sewing.nb_contigous_edges(), 0);
    assert_eq!(sewing.nb_free_edges(), 8);
    assert_eq!(
        sewing.sewed_shape().shape_type().unwrap(),
        ShapeEnum::Compound
    );
    assert!(!sewing.is_modified(&faces[1]));
}

#[test]
fn cut_boundary_is_sewn_with_its_sections() {
    let faces = [
        rectangle(0., 0., 2., 1., 0.),
        rectangle(0., 1., 1., 2., 0.),
        rectangle(1., 1., 2., 2., 0.),
    ];
    assert_eq!(sew(&faces, 1e-6).nb_contigous_edges(), 3);
    let mut without_cutting = BRepBuilderAPISewing::new(1e-6, true, true, false, false);
    for face in &faces {
        without_cutting.add(face).unwrap();
    }
    without_cutting.perform(&ProgressRange::default()).unwrap();
    assert_eq!(without_cutting.nb_contigous_edges(), 1);
}

#[test]
fn analysis_only_keeps_the_sections_bound() {
    let faces = [
        rectangle(0., 0., 2., 1., 0.),
        rectangle(0., 1., 1., 2., 0.),
        rectangle(1., 1., 2., 2., 0.),
    ];
    let mut sewing = BRepBuilderAPISewing::new(1e-6, false, true, true, false);
    for face in &faces {
        sewing.add(face).unwrap();
    }
    sewing.perform(&ProgressRange::default()).unwrap();
    assert!(sewing.sewed_shape().is_null());
    assert!(sewing.nb_contigous_edges() > 0);
}

#[test]
fn accessors_raise_out_of_range() {
    let sewing = sew(&[rectangle(0., 0., 1., 1., 0.)], 1e-6);
    assert_eq!(sewing.nb_free_edges(), 4);
    assert!(sewing.free_edge(4).is_ok());
    for failure in [
        sewing.free_edge(0).map(|e| e.into_shape()),
        sewing.free_edge(5).map(|e| e.into_shape()),
        sewing.multiple_edge(1).map(|e| e.into_shape()),
        sewing.contigous_edge(1).map(|e| e.into_shape()),
        sewing.deleted_face(1).map(|f| f.into_shape()),
        sewing.degenerated_shape(1).cloned(),
    ] {
        assert_eq!(failure.unwrap_err().exception_type(), "Standard_OutOfRange");
    }
    let edge = sewing.free_edge(1).unwrap();
    assert!(!sewing.is_section_bound(&edge));
    let err: Failure = sewing.section_to_boundary(&edge).unwrap_err();
    assert_eq!(err.exception_type(), "Standard_NoSuchObject");
}

#[test]
fn modes_and_tolerances_round_trip() {
    let mut sewing = BRepBuilderAPISewing::new(0.0, true, true, true, false);
    assert_eq!(sewing.tolerance(), 1e-7);
    assert_eq!(sewing.min_tolerance(), 1e-7);
    sewing.set_min_tolerance(1e-8);
    sewing.set_max_tolerance(2.0);
    sewing.set_face_mode(false);
    sewing.set_floating_edges_mode(true);
    sewing.set_same_parameter_mode(false);
    sewing.set_local_tolerances_mode(true);
    sewing.set_non_manifold_mode(true);
    assert_eq!(
        (sewing.min_tolerance(), sewing.max_tolerance()),
        (1e-8, 2.0)
    );
    assert!(!sewing.face_mode() && sewing.floating_edges_mode() && !sewing.same_parameter_mode());
    assert!(sewing.local_tolerances_mode() && sewing.non_manifold_mode());
}

#[test]
fn context_is_shared_and_dump_counts() {
    let context: Handle<dyn ReShape> = Handle::new(BRepToolsReShape::new()).cast();
    let mut sewing = BRepBuilderAPISewing::default();
    sewing.set_context(&context);
    assert!(*sewing.get_context() == context);
    sewing.add(&rectangle(0., 0., 1., 1., 0.)).unwrap();
    sewing.perform(&ProgressRange::default()).unwrap();
    let mut text: Vec<u8> = Vec::new();
    sewing.dump(&mut text).unwrap();
    let text = String::from_utf8(text).unwrap();
    assert!(text.contains(" Number of Free Edges        : 4\n"));
    assert!(text.contains(" Number of Bounds            : 4\n"));
}

#[test]
fn which_face_and_history_of_sub_shapes() {
    let faces = [
        rectangle(0., 0., 1., 1., 0.),
        rectangle(1. + 1e-4, 0., 2., 1., 0.),
    ];
    let sewing = sew(&faces, 1e-3);
    let edges: Vec<Shape> = Explorer::new(&faces[1], ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .collect();
    let edge = topods::edge(&edges[0]).unwrap();
    assert!(!sewing.which_face(&edge, 1).unwrap().is_null());
    assert!(sewing.which_face(&edge, 3).unwrap().is_null());
    let modified = edges
        .iter()
        .filter(|e| sewing.is_modified_sub_shape(e).unwrap());
    assert!(modified.count() >= 1);
    assert!(!sewing.is_degenerated(&faces[0]).unwrap());
}

/// A degree-1 Bezier patch with the corners (u rows first).
fn patch(corners: [[f64; 3]; 4]) -> Handle<dyn GeomSurface> {
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    let p = |c: [f64; 3]| Point3::new(c[0], c[1], c[2]);
    poles[(1, 1)] = p(corners[0]);
    poles[(1, 2)] = p(corners[1]);
    poles[(2, 1)] = p(corners[2]);
    poles[(2, 2)] = p(corners[3]);
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}

#[test]
fn fast_sewing_of_two_quads() {
    let mut sewing = BRepBuilderAPIFastSewing::new(1e-6);
    let a = patch([[0., 0., 0.], [0., 1., 0.], [1., 0., 0.], [1., 1., 0.]]);
    let b = patch([[1., 0., 0.], [1., 1., 0.], [2., 0., 0.], [2., 1., 0.]]);
    assert!(sewing.add_surface(&a).unwrap() && sewing.add_surface(&b).unwrap());
    sewing.perform();
    assert_eq!(sewing.get_statuses(None).unwrap(), 0);
    assert_eq!(count(sewing.get_result(), ShapeEnum::Face), 2);
    assert_eq!(count(sewing.get_result(), ShapeEnum::Edge), 7);
    let mut text: Vec<u8> = Vec::new();
    sewing.get_statuses(Some(&mut text)).unwrap();
    assert_eq!(text, b"Fast Sewing OK!\n");
}

#[test]
fn fast_sewing_statuses() {
    let mut sewing = BRepBuilderAPIFastSewing::default();
    sewing.perform();
    assert_eq!(
        sewing.get_statuses(None).unwrap(),
        FastSewingStatus::EmptyInput as u32
    );
    let plane: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_point(Point3::new(0., 0., 0.), Direction3::Z).unwrap()).cast();
    assert!(!sewing.add_surface(&plane).unwrap());
    assert!(!sewing.add_surface(&Handle::default()).unwrap());
    let triangle = patch([[0., 0., 0.], [0., 1., 0.], [1., 0., 0.], [1., 0., 0.]]);
    assert!(sewing.add_surface(&triangle).unwrap());
    sewing.perform();
    let expected = FastSewingStatus::EmptyInput as u32
        | FastSewingStatus::InfiniteSurface as u32
        | FastSewingStatus::FaceWithNullSurface as u32
        | FastSewingStatus::Degenerated as u32;
    assert_eq!(sewing.get_statuses(None).unwrap(), expected);
    sewing.set_tolerance(1e-3);
    assert_eq!(sewing.get_tolerance(), 1e-3);
}

#[test]
fn fast_sewing_diagonal_side_is_an_exception() {
    let mut sewing = BRepBuilderAPIFastSewing::new(1e-6);
    let quad = patch([[0., 0., 0.], [0., 1., 0.], [1., 0., 0.], [1., 1., 0.]]);
    let other = patch([[0., 0., 0.], [0.5, 1.5, 0.], [1., 1., 0.], [1.5, 1.5, 0.]]);
    sewing.add_surface(&quad).unwrap();
    sewing.add_surface(&other).unwrap();
    sewing.perform();
    assert_eq!(
        sewing.get_statuses(None).unwrap(),
        FastSewingStatus::Exception as u32
    );
}

#[test]
fn vertex_inspector_finds_points_within_the_tolerance() {
    let tol = 0.1;
    let points = [
        Xyz::new(0.05, 0., 0.),
        Xyz::new(0.2, 0., 0.),
        Xyz::new(0.1, -0.1, 0.1),
    ];
    let mut filter: CellFilter<BRepBuilderAPIVertexInspector> = CellFilter::new(tol);
    let mut inspector = BRepBuilderAPIVertexInspector::new(tol);
    for (i, p) in points.iter().enumerate() {
        filter.add(&(i as i32 + 1), p);
        inspector.add(*p);
    }
    let current = Xyz::new(0., 0., 0.);
    inspector.set_current(current);
    let (low, high) = (
        BRepBuilderAPIVertexInspector::shift(&current, -tol),
        BRepBuilderAPIVertexInspector::shift(&current, tol),
    );
    filter.inspect_range(&low, &high, &mut inspector);
    let mut found: Vec<i32> = inspector.res_ind().iter().copied().collect();
    found.sort();
    assert_eq!(found, vec![1, 3]);
    inspector.clear_res_list();
    assert!(inspector.res_ind().is_empty());
    assert_eq!(inspector.inspect_target(2).unwrap(), CellAction::Keep);
    assert!(inspector.res_ind().is_empty());
    assert!(inspector.inspect_target(4).is_err());
    assert_eq!(
        BRepBuilderAPIVertexInspector::coord(1, &Xyz::new(1., 2., 3.)),
        2.
    );
}

#[test]
fn box_selector_collects_the_overlapping_boxes() {
    let mut tree: UBTree<i32, BoundingBox3> = UBTree::default();
    {
        let mut filler = UBTreeFiller::new(&mut tree, None, true);
        for i in 0..4 {
            let mut b = BoundingBox3::default();
            let x = f64::from(i);
            b.update_bounds([x, 0., 0.], [x + 0.5, 0.5, 0.5]);
            filler.add(i + 1, b);
        }
        filler.fill();
    }
    let mut selector = BRepBuilderAPIBndBoxTreeSelector::new();
    let mut current = BoundingBox3::default();
    current.update_bounds([0.4, 0., 0.], [1.2, 0.5, 0.5]);
    selector.set_current(current.clone());
    assert_eq!(tree.select(&mut selector), 2);
    let mut found: Vec<i32> = selector.res_ind().iter().copied().collect();
    found.sort();
    assert_eq!(found, vec![1, 2]);
    let mut far = BoundingBox3::default();
    far.update_bounds([5., 5., 5.], [6., 6., 6.]);
    assert!(selector.reject_box(&far) && TreeSelector::reject(&selector, &far));
    selector.clear_res_list();
    assert!(selector.accept_object(9));
    assert_eq!(selector.res_ind().extent(), 1);
    assert!(!TreeSelector::<i32, BoundingBox3>::stop(&selector));
}
