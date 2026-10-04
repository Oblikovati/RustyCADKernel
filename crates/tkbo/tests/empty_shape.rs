//! Native geometry presence, nested topology and representation dispatch.
use rustycad_tkbo::bop_tools::algo_tools3d::is_empty_shape;
use rustycad_tkbrep::{
    brep::{
        BRepBuilder, BRepCurveOn2Surfaces, BRepCurveRepresentation, BRepPolygon3D,
        BRepPolygonOnSurface, BRepTEdge,
    },
    topods::{Compound, Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::{
    geom_abs::Shape as Continuity,
    gp::{Axis3, Point3},
    poly::{Polygon3D, Triangulation},
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
#[test]
fn native_geometry_presence_and_polygon_dispatch_match() {
    for line in include_str!("../../../tools/oracle/tkbo_empty_shape.expected").lines() {
        let (kind, expected) = line.split_once(' ').unwrap();
        let status = match is_empty_shape(&fixture(kind.parse().unwrap())) {
            Ok(empty) => i32::from(empty),
            Err(error) => {
                assert_eq!(error.exception_type(), "Standard_DomainError");
                -1
            }
        };
        assert_eq!(status, expected.parse::<i32>().unwrap(), "{line}");
    }
}
fn fixture(kind: usize) -> Shape {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let mut vertex = Vertex::default();
    builder.make_vertex(&mut vertex);
    match kind {
        0 => Shape::default(),
        2 => vertex.into(),
        3 => edge.into(),
        4 => BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(1., 0., 0.))
            .unwrap()
            .edge()
            .unwrap()
            .into(),
        5 | 6 | 13 => face_fixture(kind),
        9..=12 => represented_edge(edge, kind),
        _ => {
            let mut compound = Compound::default();
            builder.make_compound(&mut compound);
            if kind == 7 {
                builder.add(&mut compound, &edge).unwrap();
            }
            if kind == 8 {
                builder.add(&mut compound, &vertex).unwrap();
            }
            compound.into()
        }
    }
}
fn face_fixture(kind: usize) -> Shape {
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder.make_face(&mut face);
    if kind == 6 {
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
        builder
            .update_face(&face, &surface, &Location::default(), 1e-7)
            .unwrap();
    }
    if kind == 13 {
        builder
            .update_face_triangulation(
                &face,
                &Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap()),
                true,
            )
            .unwrap();
    }
    face.into()
}
fn represented_edge(edge: Edge, kind: usize) -> Shape {
    let location = Location::default();
    let representation: Handle<dyn BRepCurveRepresentation> = match kind {
        9 | 10 => {
            let polygon = if kind == 9 {
                Handle::new(Polygon3D::new(2, false).unwrap())
            } else {
                Handle::default()
            };
            Handle::new(BRepPolygon3D::new(&polygon, &location)).cast()
        }
        11 => Handle::new(BRepPolygonOnSurface::new(
            &Handle::default(),
            &Handle::default(),
            &location,
        ))
        .cast(),
        _ => Handle::new(BRepCurveOn2Surfaces::new(
            &Handle::default(),
            &Handle::default(),
            &location,
            &location,
            Continuity::G1,
        ))
        .cast(),
    };
    let storage: Handle<BRepTEdge> = edge.tshape().cast();
    storage.change_curves(|curves| curves.push(representation));
    edge.into()
}
