//! Shapes for the ShapeAnalysis, ShapeConstruct and ShapeFix behavior tests, built with the
//! ported BRepBuilderAPI makers: planar polygon faces, shells and compounds.
#![allow(dead_code)]
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Shell, Wire};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};

/// The closed polygon wire through the points.
pub fn polygon(points: &[(f64, f64, f64)]) -> Wire {
    let mut maker = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        maker.add_point(Point3::new(x, y, z)).unwrap();
    }
    maker.close().unwrap();
    maker.wire().unwrap()
}

/// The planar face bounded by the polygon.
pub fn polygon_face(points: &[(f64, f64, f64)]) -> Face {
    BRepBuilderAPIMakeFace::from_wire(&polygon(points), true)
        .unwrap()
        .face()
        .unwrap()
}

/// The unit square face at `z`.
pub fn square_face(z: f64) -> Face {
    polygon_face(&[(0., 0., z), (1., 0., z), (1., 1., z), (0., 1., z)])
}

/// The edges of a shape in explorer order.
pub fn edges_of(shape: &Shape) -> Vec<Edge> {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .map(|e| topods::edge(&e).unwrap())
        .collect()
}

/// A shell of the faces, each added as it is.
pub fn shell(faces: &[Shape]) -> Shell {
    let builder = BRepBuilder::default();
    let mut s = Shell::default();
    builder.make_shell(&mut s);
    for f in faces {
        builder.add(&mut s, f).unwrap();
    }
    s
}

/// A compound of the shapes.
pub fn compound(shapes: &[Shape]) -> Compound {
    let builder = BRepBuilder::default();
    let mut c = Compound::default();
    builder.make_compound(&mut c);
    for s in shapes {
        builder.add(&mut c, s).unwrap();
    }
    c
}
