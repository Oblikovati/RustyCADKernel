//! Real production primitive and boundary fixtures for discrete model construction.
use crate::wire_fixture as wires;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape, Vertex},
};
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
pub fn model_shape(kind: i32) -> Shape {
    if kind == 0 {
        return Shape::default();
    }
    if kind == 5 {
        return BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)
            .unwrap()
            .shape()
            .unwrap()
            .clone();
    }
    if kind == 6 {
        return BRepPrimAPIMakeSphere::new(1.5)
            .unwrap()
            .shape()
            .unwrap()
            .clone();
    }
    let builder = BRepBuilder::default();
    if kind == 1 {
        let mut vertex = Vertex::default();
        builder
            .make_vertex_at(&mut vertex, Point3::new(1., 2., 3.), 1e-7)
            .unwrap();
        return vertex.into_shape();
    }
    let (wire, mut face, ordered) = wires::order_fixture(0, 0, i32::from(kind == 8));
    match kind {
        2 => ordered.edge(1).unwrap().into_shape(),
        3 => wire.into_shape(),
        7 => duplicate_face(&builder, &face),
        9 => {
            let (inner, _, _) = wires::order_fixture(0, 0, 1);
            builder.add(&mut face, &inner).unwrap();
            face.into_shape()
        }
        _ => face.into_shape(),
    }
}
fn duplicate_face(builder: &BRepBuilder, face: &rustycad_tkbrep::topods::Face) -> Shape {
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, face).unwrap();
    let location = Location::new(Transform3::translation_vector(Vector3::new(7., 11., 13.)));
    builder
        .add(&mut compound, &face.located(&location, false).unwrap())
        .unwrap();
    compound.into_shape()
}
