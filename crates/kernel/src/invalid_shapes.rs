// SPDX-License-Identifier: GPL-2.0-only
//! Deliberately invalid bodies, each with one known defect, built from a real box.
use crate::test_shapes::{box_solid, faces};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Compound, Shape, Shell, Solid};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// A shell holding exactly `faces`.
fn shell_of(faces: &[Shape]) -> Shell {
    let mut shell = Shell::default();
    Builder.make_shell(&mut shell);
    for face in faces {
        Builder.add(&mut shell, face).unwrap();
    }
    shell
}

/// A solid whose one shell holds exactly `faces`.
pub(crate) fn solid_of(faces: &[Shape]) -> Shape {
    let shell = shell_of(faces);
    let mut solid = Solid::default();
    Builder.make_solid(&mut solid);
    Builder.add(&mut solid, &shell).unwrap();
    solid.into_shape()
}

fn box_faces() -> Vec<Shape> {
    faces(&box_solid(1.0, 1.0, 1.0))
        .into_iter()
        .map(Shape::from)
        .collect()
}

/// The unit box without its first face: four free edges.
pub(crate) fn open_shell() -> (Shape, Shape) {
    let all = box_faces();
    (solid_of(&all[1..]), all[0].clone())
}

/// The same five faces as a free sheet: its boundary edges are legitimate.
pub(crate) fn open_sheet() -> Shape {
    shell_of(&box_faces()[1..]).into_shape()
}

/// The unit box with its first face used twice: four edges with three face uses.
pub(crate) fn non_manifold() -> (Shape, Shape) {
    let mut all = box_faces();
    all.push(all[0].clone());
    let repeated = all[0].clone();
    (solid_of(&all), repeated)
}

/// The unit box with its first face reversed: its four edges are traversed twice in the
/// same direction.
pub(crate) fn flipped_face() -> (Shape, Shape) {
    let mut all = box_faces();
    all[0] = all[0].reversed();
    let flipped = all[0].clone();
    (solid_of(&all), flipped)
}

/// Two valid boxes that overlap in a unit cube, in one compound.
pub(crate) fn overlapping_boxes() -> Shape {
    let second = BRepPrimAPIMakeBox::at_point(Point3::new(1.0, 1.0, 1.0), 2.0, 2.0, 2.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    Builder
        .add(&mut compound, &box_solid(2.0, 2.0, 2.0))
        .unwrap();
    Builder.add(&mut compound, &second).unwrap();
    compound.into_shape()
}

/// The unit box with its first vertex tolerance raised to `tolerance`; also the vertex.
pub(crate) fn over_tolerance_vertex(tolerance: f64) -> (Shape, Shape) {
    let solid = box_solid(1.0, 1.0, 1.0);
    let vertex = first(&solid, ShapeEnum::Vertex);
    BRepBuilder::default()
        .update_vertex_tolerance(&topods::vertex(&vertex).unwrap(), tolerance)
        .unwrap();
    (solid, vertex)
}

/// The unit box with its first edge tolerance above its vertices'; also the edge.
pub(crate) fn over_tolerance_edge(tolerance: f64) -> (Shape, Shape) {
    let solid = box_solid(1.0, 1.0, 1.0);
    let edge = first(&solid, ShapeEnum::Edge);
    BRepBuilder::default()
        .update_edge_tolerance(&topods::edge(&edge).unwrap(), tolerance)
        .unwrap();
    (solid, edge)
}

fn first(shape: &Shape, kind: ShapeEnum) -> Shape {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone()
}
