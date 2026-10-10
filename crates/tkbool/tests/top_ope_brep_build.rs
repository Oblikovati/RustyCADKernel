//! TopOpeBRepBuild face chain: the wire edge classifier, the face builder with its
//! unclosed wire and pseudo internal edge detections, Builder::MakeFaces and
//! WireToFace on a planar face.
use rustycad_tkbool::top_ope_brep_build::{
    CLOSEDW, GCLOSEDW, ISVERTEX, ShapeToShapeMap, TopOpeBRepBuildBlockBuilder,
    TopOpeBRepBuildBuilder, TopOpeBRepBuildCompositeClassifier, TopOpeBRepBuildFaceBuilder,
    TopOpeBRepBuildShapeSet, TopOpeBRepBuildWireEdgeClassifier, TopOpeBRepBuildWireEdgeSet,
    TopOpeBRepBuildWireToFace, UNCLOSEDW, fun_analyzemap_von1e, fun_detect_vertices_on1_edge,
};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSBuildTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib::{
    BRepLibMakeEdge, BRepLibMakeFace, BRepLibMakePolygon, BRepLibMakeVertex,
};

/// The closed polygon wire of points of the plane z = 0.
fn polygon(points: &[(f64, f64)]) -> Result<Wire, Failure> {
    let mut mp = BRepLibMakePolygon::new();
    for (x, y) in points {
        mp.add_point(Point3::new(*x, *y, 0.))?;
    }
    mp.close()?;
    mp.wire()
}

/// The square [x0, x1]^2 counter-clockwise, or clockwise.
fn square(x0: f64, x1: f64, ccw: bool) -> Result<Wire, Failure> {
    let mut points = vec![(x0, x0), (x1, x0), (x1, x1), (x0, x1)];
    if !ccw {
        points.reverse();
    }
    polygon(&points)
}

/// The face of the square [0, 10]^2 and its wire.
fn base_face() -> Result<(Face, Wire), Failure> {
    let outer = square(0., 10., true)?;
    Ok((BRepLibMakeFace::from_wire(&outer, true)?.face()?, outer))
}

/// The sub-shapes of a type.
fn subs(s: &Shape, t: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(s, t, ShapeEnum::Shape)?.collect())
}

/// The wire count of each face of a list.
fn wire_counts(lf: &List<Shape>) -> Result<Vec<usize>, Failure> {
    lf.iter()
        .map(|f| Ok(subs(f, ShapeEnum::Wire)?.len()))
        .collect()
}

#[test]
fn wire_to_face_keeps_a_hole_in_its_face() -> Result<(), Failure> {
    let (face, outer) = base_face()?;
    let mut wtf = TopOpeBRepBuildWireToFace::new();
    wtf.add_wire(&outer);
    wtf.add_wire(&square(3., 5., false)?);
    let mut lf = List::default();
    wtf.make_faces(&face, &mut lf)?;
    assert_eq!(wire_counts(&lf)?, vec![2]);
    wtf.init();
    wtf.make_faces(&face, &mut lf)?;
    assert!(lf.is_empty());
    Ok(())
}

#[test]
fn wire_to_face_splits_disjoint_wires_into_faces() -> Result<(), Failure> {
    let (face, _) = base_face()?;
    let mut wtf = TopOpeBRepBuildWireToFace::new();
    wtf.add_wire(&square(1., 3., true)?);
    wtf.add_wire(&square(5., 8., true)?);
    let mut lf = List::default();
    wtf.make_faces(&face, &mut lf)?;
    assert_eq!(wire_counts(&lf)?, vec![1, 1]);
    Ok(())
}

#[test]
fn the_classifier_puts_a_hole_inside_its_outer_wire() -> Result<(), Failure> {
    let (face, outer) = base_face()?;
    let hole = square(3., 5., false)?;
    let bb = TopOpeBRepBuildBlockBuilder::new();
    let mut wec = TopOpeBRepBuildWireEdgeClassifier::new(face.as_ref(), &bb)?;
    assert_eq!(
        wec.compare_shapes(hole.as_ref(), outer.as_ref())?,
        State::In
    );
    // A clockwise hole bounds the region outside it, so the outer wire is IN it too (as
    // the native classifier answers, tb_wec).
    assert_eq!(
        wec.compare_shapes(outer.as_ref(), hole.as_ref())?,
        State::In
    );
    let away = square(12., 14., true)?;
    assert_eq!(
        wec.compare_shapes(away.as_ref(), outer.as_ref())?,
        State::Out
    );
    Ok(())
}

/// A face builder on the edges of the outer square (start elements) and `extra`.
fn edge_builder(
    face: &Face,
    outer: &Wire,
    extra: &[Shape],
) -> Result<TopOpeBRepBuildFaceBuilder, Failure> {
    let mut wes = TopOpeBRepBuildWireEdgeSet::new(face.as_ref())?;
    for e in subs(outer.as_ref(), ShapeEnum::Edge)?.iter().chain(extra) {
        wes.add_start_element(e)?;
    }
    TopOpeBRepBuildFaceBuilder::with_wire_edge_set(&mut wes, face.as_ref(), true)
}

/// The first vertex of the wire.
fn first_vertex(w: &Wire) -> Result<Vertex, Failure> {
    let edges = subs(w.as_ref(), ShapeEnum::Edge)?;
    Ok(topexp::first_vertex(
        &rustycad_tkbrep::topods::edge(&edges[0])?,
        false,
    ))
}

#[test]
fn an_edge_given_both_ways_is_a_pseudo_internal_edge() -> Result<(), Failure> {
    let (face, outer) = base_face()?;
    let inner = BRepLibMakeVertex::new(Point3::new(3., 1., 0.))?.vertex()?;
    let e = BRepLibMakeEdge::from_vertices(&first_vertex(&outer)?, &inner)?
        .edge()?
        .into_shape();
    let mut fb = edge_builder(&face, &outer, &[e.clone(), e.reversed()])?;
    let mut map_e: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
    fb.detect_pseudo_internal_edge(&mut map_e)?;
    assert_eq!(map_e.len(), 1);
    assert!(map_e.find_key(1)?.is_same(&e));
    Ok(())
}

#[test]
fn an_open_chain_is_removed_from_the_built_faces() -> Result<(), Failure> {
    let (face, outer) = base_face()?;
    let chain = polygon(&[(3., 3.), (5., 3.), (5., 5.)])?;
    let open: Vec<Shape> = subs(chain.as_ref(), ShapeEnum::Edge)?
        .into_iter()
        .take(2)
        .collect();
    let mut fb = edge_builder(&face, &outer, &open)?;
    let (mut vv, mut v1e) = (ShapeToShapeMap::default(), ShapeToShapeMap::default());
    fb.detect_unclosed_wire(&mut vv, &mut v1e)?;
    assert_eq!((vv.len(), v1e.len()), (0, 0));
    let mut lf = List::default();
    TopOpeBRepBuildBuilder::new(&TopOpeBRepDSBuildTool::new()).make_faces(
        face.as_ref(),
        &mut fb,
        &mut lf,
    )?;
    let edges: usize = lf
        .iter()
        .map(|f| subs(f, ShapeEnum::Edge).map(|e| e.len()))
        .sum::<Result<usize, Failure>>()?;
    assert_eq!(edges, 4);
    Ok(())
}

#[test]
fn free_vertices_classify_wires() -> Result<(), Failure> {
    let (mut von1e, mut vv) = (ShapeToShapeMap::default(), ShapeToShapeMap::default());
    assert_eq!(fun_analyzemap_von1e(&von1e, &mut vv)?, CLOSEDW);
    let open = polygon(&[(0., 0.), (1., 0.), (1., 1.)])?;
    let edge = subs(open.as_ref(), ShapeEnum::Edge)?.remove(0);
    fun_detect_vertices_on1_edge(&edge, &mut von1e)?;
    assert_eq!(von1e.len(), 2);
    assert_eq!(fun_analyzemap_von1e(&von1e, &mut vv)?, UNCLOSEDW);
    assert_ne!(GCLOSEDW, ISVERTEX);
    Ok(())
}
