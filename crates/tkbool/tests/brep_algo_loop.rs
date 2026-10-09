//! BRepAlgo_Loop: the cut of an edge at vertices and the loops of a square face.
use rustycad_tkbool::brep_algo::{BRepAlgoLoop, VertexSubstitutes};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeFace, BRepLibMakePolygon, BRepLibMakeVertex};

/// The face of the square [0, 10]^2 of z = 0 and its edges in wire order.
fn square() -> Result<(Face, Vec<Edge>), Failure> {
    let mut mp = BRepLibMakePolygon::new();
    for (x, y) in [(0., 0.), (10., 0.), (10., 10.), (0., 10.)] {
        mp.add_point(Point3::new(x, y, 0.))?;
    }
    mp.close()?;
    let face = BRepLibMakeFace::from_wire(&mp.wire()?, true)?.face()?;
    let edges = Explorer::new(face.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?
        .map(|e| topods::edge(&e))
        .collect::<Result<Vec<Edge>, Failure>>()?;
    Ok((face, edges))
}

/// A vertex at the middle of the first edge (parameter 5), REVERSED then FORWARD.
fn middle_vertices(e: &Edge) -> Result<List<Shape>, Failure> {
    let v = BRepLibMakeVertex::new(Point3::new(5., 0., 0.))?.vertex()?;
    BRepBuilder::default().update_vertex_on_edge(&v, 5., e, 1e-7)?;
    Ok([Orientation::Reversed, Orientation::Forward]
        .into_iter()
        .map(|o| v.as_ref().oriented(o))
        .collect())
}

#[test]
fn an_edge_is_cut_at_its_middle_vertex() -> Result<(), Failure> {
    let (face, edges) = square()?;
    let mut lp = BRepAlgoLoop::new();
    lp.init(&face);
    let mut ne = List::default();
    lp.cut_edge(&edges[0], &middle_vertices(&edges[0])?, &mut ne)?;
    let ranges: Vec<(f64, f64)> = ne
        .iter()
        .map(|e| BRepTool::range(&topods::edge(e)?))
        .collect::<Result<_, Failure>>()?;
    assert_eq!(ranges, vec![(0., 5.), (5., 10.)]);
    let mut whole = List::default();
    lp.cut_edge(&edges[0], &List::default(), &mut whole)?;
    assert!(whole.first().is_some_and(|e| e.is_equal(edges[0].as_ref())));
    Ok(())
}

#[test]
fn the_boundary_edges_make_one_closed_wire_and_face() -> Result<(), Failure> {
    let (face, edges) = square()?;
    let mut lp = BRepAlgoLoop::new();
    assert_eq!(lp.get_tol_conf(), 0.001);
    lp.init(&face);
    lp.add_const_edges(&edges.iter().map(|e| e.as_ref().clone()).collect());
    lp.perform()?;
    assert_eq!(lp.new_wires().len(), 1);
    lp.wires_to_faces()?;
    assert_eq!(lp.new_faces().len(), 1);
    let mut vv = VertexSubstitutes::default();
    lp.get_vertices_for_substitute(&mut vv);
    assert!(vv.is_empty());
    let error = lp
        .new_edges(&edges[0])
        .err()
        .map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("Standard_NoSuchObject"));
    Ok(())
}

#[test]
fn a_cut_edge_keeps_its_used_pieces() -> Result<(), Failure> {
    let (face, edges) = square()?;
    let mut lp = BRepAlgoLoop::new();
    lp.init(&face);
    lp.add_edge(&edges[0], &middle_vertices(&edges[0])?);
    lp.add_const_edges(&edges[1..].iter().map(|e| e.as_ref().clone()).collect());
    lp.perform()?;
    assert_eq!(lp.new_wires().len(), 1);
    assert_eq!(lp.new_edges(&edges[0])?.len(), 2);
    Ok(())
}
