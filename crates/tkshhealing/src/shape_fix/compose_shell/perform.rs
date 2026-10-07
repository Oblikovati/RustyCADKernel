//! Full ComposeShell orchestration from native `Perform`.
use super::{ShapeFixComposeShell, ShapeFixWireSegment};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::topods::{Builder, Shape, Shell};
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
    let mut faces = Sequence::default();
    composer.dispatch_wires(&mut faces, &mut segments)?;
    store_result(composer, &faces)?;
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
    composer.split_by_grid(segments)?;
    composer.break_wires(segments)?;
    let mut loops = Sequence::default();
    composer.collect_wires(&mut loops, segments)?;
    *segments = loops;
    Ok(())
}

// Native keeps a single face as is and wraps any other count (zero included) in a shell.
fn store_result(composer: &ShapeFixComposeShell, faces: &Sequence<Shape>) -> Result<(), Failure> {
    let mut result = if faces.len() == 1 {
        faces.value(1)?.clone()
    } else {
        make_result_shell(faces)?
    };
    result.set_orientation(composer.fields.read().orientation);
    composer.fields.write().result = result;
    Ok(())
}

fn make_result_shell(faces: &Sequence<Shape>) -> Result<Shape, Failure> {
    let mut shell = Shell::default();
    Builder.make_shell(&mut shell);
    for face in faces.iter() {
        Builder.add(&mut shell, face)?;
    }
    Ok(shell.into_shape())
}

fn mark_status(composer: &ShapeFixComposeShell, status: ShapeExtendStatus) {
    composer.fields.write().status |= ShapeExtend::encode_status(status);
}

#[cfg(test)]
#[path = "perform/tests.rs"]
mod tests;
