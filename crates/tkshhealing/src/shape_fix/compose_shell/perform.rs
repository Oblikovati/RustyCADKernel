//! Full ComposeShell orchestration from native `Perform`.
use super::dispatch_wires::{PatchLoopGroup, dispatch_wires};
use super::{ShapeFixComposeShell, ShapeFixWireSegment};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::{FixRoot, face::make_faces_on_patch::make_faces_on_patch};
use rustycad_tkbrep::topods::{Builder, Face, Shape, Shell};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;

pub(super) fn perform(composer: &ShapeFixComposeShell) -> Result<bool, Failure> {
    initialize_perform(composer);
    let mut segments = composer.load_wires()?;
    if segments.is_empty() {
        mark_status(composer, ShapeExtendStatus::Fail6);
        return Ok(false);
    }
    split_and_collect(composer, &mut segments)?;
    let faces = compose_patch_faces(composer, &mut segments)?;
    store_result(composer, faces)?;
    mark_status(composer, ShapeExtendStatus::Done1);
    Ok(true)
}

fn initialize_perform(composer: &ShapeFixComposeShell) {
    let mut fields = composer.fields.write();
    fields.status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
    fields.invert_edge_status = false;
}

fn split_and_collect(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
) -> Result<(), Failure> {
    super::split_by_grid::split_by_grid(composer, segments)?;
    composer.break_wires(segments)?;
    let mut loops = Sequence::default();
    composer.collect_wires(&mut loops, segments)?;
    *segments = loops;
    Ok(())
}

fn compose_patch_faces(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
) -> Result<Vec<Face>, Failure> {
    let groups = dispatch_wires(composer, segments)?;
    let (_, original, location, _) = composer.collection_inputs();
    let context = composer.context();
    let invert_edge_status = composer.fields.read().invert_edge_status;
    assemble_patch_groups(groups, &original, &location, &context, invert_edge_status)
}

fn assemble_patch_groups(
    groups: Vec<PatchLoopGroup>,
    original_face: &Face,
    location: &rustycad_tkmath::toploc::Location,
    context: &rustycad_tkernel::standard::Handle<ShapeBuildReShape>,
    invert_edge_status: bool,
) -> Result<Vec<Face>, Failure> {
    let mut faces = Vec::new();
    for group in groups {
        faces.extend(assemble_patch_group(
            group,
            original_face,
            location,
            context,
            invert_edge_status,
        )?);
    }
    Ok(faces)
}

fn assemble_patch_group(
    mut group: PatchLoopGroup,
    original_face: &Face,
    location: &rustycad_tkmath::toploc::Location,
    context: &rustycad_tkernel::standard::Handle<ShapeBuildReShape>,
    invert_edge_status: bool,
) -> Result<Vec<Face>, Failure> {
    let mut loops = Sequence::default();
    for shape in group.loops.drain(..) {
        loops.append(shape);
    }
    make_faces_on_patch(
        &group.surface,
        &mut loops,
        original_face,
        location,
        context,
        invert_edge_status,
    )
}

fn store_result(composer: &ShapeFixComposeShell, faces: Vec<Face>) -> Result<(), Failure> {
    let mut result = if faces.len() == 1 {
        faces[0].clone().into_shape()
    } else {
        make_result_shell(faces)?
    };
    result.set_orientation(composer.fields.read().orientation);
    composer.fields.write().result = result;
    Ok(())
}

fn make_result_shell(faces: Vec<Face>) -> Result<Shape, Failure> {
    let mut shell = Shell::default();
    Builder.make_shell(&mut shell);
    for face in faces {
        Builder.add(&mut shell, &face.clone().into_shape())?;
    }
    Ok(shell.into_shape())
}

fn mark_status(composer: &ShapeFixComposeShell, status: ShapeExtendStatus) {
    composer.fields.write().status |= ShapeExtend::encode_status(status);
}

#[cfg(test)]
#[path = "perform/tests.rs"]
mod tests;
