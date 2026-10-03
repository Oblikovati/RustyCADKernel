use super::ShapeFixFixSmallFace;
use crate::shape_analysis::{SplittingVertexEdges, SplittingVertexParameters};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Compound, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn fix_split_face(fixer: &ShapeFixFixSmallFace) -> Result<Shape, Failure> {
    let input = fixer.input.read().clone();
    if input.is_null() {
        return Ok(input);
    }
    let done = if split_container_supported(&input)? {
        scan_split_faces(fixer, &input)?
    } else {
        false
    };
    let output = apply_context(fixer, &input)?;
    *fixer.input.write() = output.clone();
    *fixer.result.write() = output.clone();
    if done {
        *fixer.status.write() = ShapeExtend::encode_status(ShapeExtendStatus::Done3);
    }
    Ok(output)
}

pub(super) fn split_one_face(
    fixer: &ShapeFixFixSmallFace,
    face: &mut Face,
    _split_faces: &mut Compound,
) -> Result<bool, Failure> {
    let applied = apply_context(fixer, &face.clone().into_shape())?;
    *face = topods::face(&applied)?;
    let mut edge_map = SplittingVertexEdges::default();
    let mut parameter_map = SplittingVertexParameters::default();
    let mut all_vertices = Compound::default();
    Builder.make_compound(&mut all_vertices);
    let found = fixer.analyzer.write().check_splitting_vertices(
        face,
        &mut edge_map,
        &mut parameter_map,
        &mut all_vertices,
    )?;
    if found == 0 {
        return Ok(false);
    }
    Err(Failure::with_message(
        "Standard_NotImplemented",
        "ShapeFix_FixSmallFace::SplitOneFace found splitting vertices; recursive split construction is not ported",
    ))
}

fn split_container_supported(shape: &Shape) -> Result<bool, Failure> {
    Ok(matches!(
        shape.shape_type()?,
        ShapeEnum::Compound
            | ShapeEnum::CompSolid
            | ShapeEnum::Solid
            | ShapeEnum::Shell
            | ShapeEnum::Face
    ))
}

fn scan_split_faces(fixer: &ShapeFixFixSmallFace, shape: &Shape) -> Result<bool, Failure> {
    let context = context(fixer)?;
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    let mut done = false;
    while faces.more() {
        let mut face = topods::face(faces.current())?;
        let mut split_faces = Compound::default();
        Builder.make_compound(&mut split_faces);
        if split_one_face(fixer, &mut face, &mut split_faces)? {
            ReShape::replace(
                context.get().unwrap(),
                &face.clone().into_shape(),
                &split_faces.into_shape(),
            )?;
            done = true;
        }
        faces.next()?;
    }
    Ok(done)
}

fn apply_context(fixer: &ShapeFixFixSmallFace, shape: &Shape) -> Result<Shape, Failure> {
    let context = context(fixer)?;
    ReShape::apply(context.get().unwrap(), shape, ShapeEnum::Shape)
}

fn context(fixer: &ShapeFixFixSmallFace) -> Result<Handle<ShapeBuildReShape>, Failure> {
    let context = fixer.context();
    if context.get().is_some() {
        return Ok(context);
    }
    Err(Failure::new(
        "ShapeFix_FixSmallFace requires Init(shape) before splitting",
    ))
}
