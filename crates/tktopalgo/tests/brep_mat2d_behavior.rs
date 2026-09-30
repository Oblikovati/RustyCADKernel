//! Behavior tests of BRepMAT2d (no upstream suite uses it): the explorer, the bisecting
//! locus and the links on the rectangle face [0, 4] x [0, 2] of BRepBuilderAPI_MakeFace.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::Plane3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use rustycad_tktopalgo::brep_mat2d::{
    BRepMAT2dBisectingLocus, BRepMAT2dExplorer, BRepMAT2dLinkTopoBilo,
};
use rustycad_tktopalgo::mat::MatSide;

fn rectangle() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 4.0, 0.0, 2.0))
        .unwrap()
        .face()
        .unwrap()
}

fn locus_of(explorer: &mut BRepMAT2dExplorer, side: MatSide) -> BRepMAT2dBisectingLocus {
    let mut locus = BRepMAT2dBisectingLocus::new();
    locus
        .compute(explorer, 1, side, JoinType::Arc, false)
        .unwrap();
    locus
}

#[test]
fn explorer_reads_one_closed_contour_of_four_curves() {
    let face = rectangle();
    let mut explorer = BRepMAT2dExplorer::with_face(&face).unwrap();
    assert_eq!(explorer.number_of_contours(), 1);
    assert_eq!(explorer.number_of_curves(1).unwrap(), 4);
    assert!(*explorer.get_is_closed().value(1).unwrap());
    explorer.init(1);
    let mut count = 0;
    while explorer.more().unwrap() {
        count += 1;
        explorer.next();
    }
    assert_eq!(count, 4);
    // The face is replaced by the new face; its connected edges are not modified.
    let shape: Shape = face.clone().into();
    assert!(explorer.is_modified(&shape));
    for edge in Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        assert!(!explorer.is_modified(&edge));
        assert!(explorer.modified_shape(&edge).is_same(&edge));
    }
}

#[test]
fn inner_locus_nodes_lie_at_most_half_the_width_from_the_contour() {
    let mut explorer = BRepMAT2dExplorer::with_face(&rectangle()).unwrap();
    let locus = locus_of(&mut explorer, MatSide::Left);
    assert!(locus.is_done());
    let graph = locus.graph();
    assert!(graph.number_of_arcs() > 0);
    assert_eq!(graph.number_of_infinite_nodes(), 0);
    let widest = (1..=graph.number_of_nodes())
        .map(|i| graph.node(i).unwrap().distance())
        .fold(0.0_f64, f64::max);
    assert!((widest - 1.0).abs() <= 1.0e-9, "{widest}");
    assert_eq!(
        locus.number_of_elts(1).unwrap(),
        graph.number_of_basic_elts()
    );
    assert_eq!(locus.number_of_sections(1, 1).unwrap(), 1);
    assert!(locus.number_of_sections(1, 5).is_err());
}

#[test]
fn outer_locus_reaches_infinity() {
    let mut explorer = BRepMAT2dExplorer::with_face(&rectangle()).unwrap();
    let locus = locus_of(&mut explorer, MatSide::Right);
    assert!(locus.is_done());
    assert!(locus.graph().number_of_infinite_nodes() > 0);
}

#[test]
fn every_edge_generates_basic_elements_that_link_back_to_it() {
    let face = rectangle();
    let mut explorer = BRepMAT2dExplorer::with_face(&face).unwrap();
    let locus = locus_of(&mut explorer, MatSide::Left);
    let mut link = BRepMAT2dLinkTopoBilo::with_locus(&explorer, &locus).unwrap();
    let shape: Shape = face.into();
    for edge in Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        link.init(&edge);
        assert!(link.more().unwrap());
        while link.more().unwrap() {
            let element = link.value().unwrap();
            assert!(link.generating_shape(&element).unwrap().is_same(&edge));
            link.next();
        }
    }
}

#[test]
fn a_face_without_wires_has_no_contour_and_no_locus() {
    let mut explorer = BRepMAT2dExplorer::new();
    let locus = locus_of(&mut explorer, MatSide::Left);
    assert!(!locus.is_done());
    assert_eq!(locus.number_of_contours(), 0);
    assert!(BRepMAT2dLinkTopoBilo::with_locus(&explorer, &locus).is_err());
}
