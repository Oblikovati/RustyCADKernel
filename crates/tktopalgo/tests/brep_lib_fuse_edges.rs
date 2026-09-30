//! Behavior tests of BRepLib_FuseEdges (docs/tktopalgo-progress.md). The numeric oracle
//! compares the same class with OCCT bit for bit (`blp_fuseedges` requests).
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Circle3, Point3};
use rustycad_tktopalgo::brep_lib::{
    BRepLibFuseEdges, BRepLibMakeEdge, BRepLibMakeFace, BRepLibMakePolygon,
};

fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}

/// The face of the rectangle [0, 2] x [0, 1] whose bottom side is split at x = 1.
fn split_face() -> Result<Shape, Failure> {
    let mut polygon = BRepLibMakePolygon::new();
    for (x, y) in [(0., 0.), (1., 0.), (2., 0.), (2., 1.), (0., 1.)] {
        polygon.add_point(Point3::new(x, y, 0.))?;
    }
    polygon.close()?;
    Ok(BRepLibMakeFace::from_wire(&polygon.wire()?, true)?
        .face()?
        .into_shape())
}

#[test]
fn fuse_edges_merges_the_collinear_sides_of_a_face() -> Result<(), Failure> {
    let face = split_face()?;
    let mut fuser = BRepLibFuseEdges::new(&face, false)?;
    let chains = fuser.edges()?;
    assert_eq!(chains.extent(), 1);
    assert_eq!(chains.find(&1)?.extent(), 2);
    assert_eq!(fuser.nb_vertices()?, 1);
    let fused = fuser.shape()?.clone();
    assert_eq!(count(&fused, ShapeEnum::Edge)?, 4);
    assert_eq!(fuser.faces()?.extent(), 1);
    Ok(())
}

#[test]
fn fuse_edges_perform_first_finds_no_chain() -> Result<(), Failure> {
    let face = split_face()?;
    let mut fuser = BRepLibFuseEdges::new(&face, false)?;
    fuser.perform()?;
    assert_eq!(count(fuser.shape()?, ShapeEnum::Edge)?, 5);
    Ok(())
}

#[test]
fn fuse_edges_respects_avoided_edges() -> Result<(), Failure> {
    let face = split_face()?;
    let mut avoided = IndexedMapOfShape::default();
    for edge in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        avoided.add(edge);
    }
    let mut fuser = BRepLibFuseEdges::new(&face, false)?;
    fuser.avoid_edges(&avoided);
    assert_eq!(fuser.nb_vertices()?, 0);
    assert!(BRepLibFuseEdges::new(&Shape::default(), true).is_err());
    Ok(())
}

#[test]
fn fuse_edges_closes_a_circle_of_arcs() -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let circle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::from_circ(Circle3::new(Axis2::default(), 1.)?)).cast();
    let params = [0., 2., 4.];
    let vertices: Vec<Vertex> = params
        .iter()
        .map(|&u| {
            let mut v = Vertex::default();
            builder.make_vertex_at(&mut v, circle.get().expect("a circle").value(u)?, 1e-7)?;
            Ok(v)
        })
        .collect::<Result<_, Failure>>()?;
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let arcs = [
        (0, 1, 0., 2.),
        (1, 2, 2., 4.),
        (2, 0, 4., std::f64::consts::TAU),
    ];
    for (a, b, f, l) in arcs {
        let edge =
            BRepLibMakeEdge::from_curve_vertices_params(&circle, &vertices[a], &vertices[b], f, l)?
                .edge()?;
        builder.add(&mut wire, &edge.into_shape())?;
    }
    let mut fuser = BRepLibFuseEdges::new(&wire.into_shape(), false)?;
    assert_eq!(fuser.nb_vertices()?, 2);
    assert_eq!(count(fuser.shape()?, ShapeEnum::Edge)?, 1);
    Ok(())
}
