//! Genuine grouping replay awaits native fixture capture after the shared timing hold.
#[path = "textured_face_fixture.rs"]
mod face_fixture;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools,
    topods::{CompSolid, Compound, Shape, ShapeIterator, Shell, Solid, Vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{
    gp::{Point3, Transform3},
    toploc::Location,
};
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkv3d::std_prs::{StdPrsShadedShape, StdPrsToolTriangulatedShape};

pub fn replay_textured_volume_case(scenario: i32, ignore: bool) {
    let shape = textured_volume_shape(scenario).unwrap();
    let builder = BRepBuilder::default();
    let mut closed = Compound::default();
    let mut opened = Compound::default();
    builder.make_compound(&mut closed);
    builder.make_compound(&mut opened);
    StdPrsShadedShape::explore_solids(&shape, &builder, &mut closed, &mut opened, ignore).unwrap();
    let mut values = vec![
        f64::from(i32::from(
            StdPrsToolTriangulatedShape::is_triangulated(&shape).unwrap(),
        )),
        f64::from(i32::from(
            StdPrsToolTriangulatedShape::is_closed(&shape).unwrap(),
        )),
    ];
    append_volume_children(&mut values, &closed);
    append_volume_children(&mut values, &opened);
    assert_eq!(values, native_volume_values(scenario, ignore));
}
fn append_volume_children(values: &mut Vec<f64>, compound: &Compound) {
    values.push(compound.nb_children() as f64);
    for child in ShapeIterator::new(compound, true, true) {
        let position = child.location().transformation().translation_part();
        values.extend([
            f64::from(child.shape_type().unwrap() as i32),
            f64::from(child.orientation() as i32),
            position.x(),
            position.y(),
            position.z(),
        ]);
    }
}
fn native_volume_values(scenario: i32, ignore: bool) -> Vec<f64> {
    let prefix = format!("MARTENSITE_VOLUMES {scenario} {} ", i32::from(ignore));
    include_str!("../fixtures/ais_textured_shape/native-volumes.txt")
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect()
}
fn textured_volume_shape(scenario: i32) -> Result<Shape, Failure> {
    match scenario {
        0 => Ok(Shape::default()),
        1 => Ok(face_fixture::textured_face_fixture(1)?.into_shape()),
        2 => Ok(textured_volume_shell(true)?.into_shape()),
        3 => Ok(textured_volume_solid(true)?.into_shape()),
        4 => Ok(textured_volume_solid(false)?.into_shape()),
        5 => Ok(textured_comp_solid()?.into_shape()),
        6 => textured_volume_compound(),
        7 => textured_box_volume(true, false),
        8 => textured_box_volume(false, false),
        _ => textured_box_volume(true, true),
    }
}
fn textured_volume_shell(mesh: bool) -> Result<Shell, Failure> {
    let builder = BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let face = face_fixture::textured_face_fixture(1)?;
    if !mesh {
        brep_tools::clean(&face, false)?;
    }
    builder.add(&mut shell, &face)?;
    shell.set_closed(true)?;
    Ok(shell)
}
fn textured_volume_solid(mesh: bool) -> Result<Solid, Failure> {
    let builder = BRepBuilder::default();
    let mut solid = Solid::default();
    builder.make_solid(&mut solid);
    builder.add(&mut solid, &textured_volume_shell(mesh)?.into_shape())?;
    Ok(solid)
}
fn textured_comp_solid() -> Result<CompSolid, Failure> {
    let builder = BRepBuilder::default();
    let mut solid = CompSolid::default();
    builder.make_comp_solid(&mut solid);
    builder.add(&mut solid, &textured_volume_solid(true)?.into_shape())?;
    builder.add(&mut solid, &textured_volume_solid(false)?.into_shape())?;
    Ok(solid)
}
fn textured_volume_compound() -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &textured_comp_solid()?.into_shape())?;
    let mut vertex = Vertex::default();
    builder.make_vertex_at(&mut vertex, Point3::new(1., 2., 3.), 1e-7)?;
    builder.add(&mut compound, &vertex)?;
    compound.reversed().located(
        &Location::new(Transform3::translation([0.5, 1.5, 2.5])),
        false,
    )
}

fn textured_box_volume(mesh: bool, closed: bool) -> Result<Shape, Failure> {
    let mut builder = BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)?;
    let solid = builder.solid()?;
    if mesh {
        BRepMeshIncrementalMesh::with_shape(&solid, 0.1, false, 0.5, false)?;
    }
    if closed {
        solid.set_closed(true)?;
    }
    Ok(solid.into_shape())
}
