use super::super::{ShapeExtendStatus, ShapeFixFace};
use crate::shape_fix::{FixRoot, ShapeFix};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topods::{self, Face, Shape, ShapeIterator, Wire},
};
use rustycad_tkernel::{message::ProgressRange, standard::Failure};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn run(
    fixer: &ShapeFixFace,
    progress: &ProgressRange<'_>,
    saved_precision: f64,
    reordered_wires: &mut Vec<(Shape, Shape)>,
) -> Result<bool, Failure> {
    let source = apply_current_face(fixer)?;
    configure_first_pass(fixer, &source, saved_precision)?;
    let mut rebuilt = empty_forward_face(&source)?;
    let modes = disable_first_modes(fixer);
    let copied = copy_first_pass_children(fixer, &source, &mut rebuilt, progress, reordered_wires);
    restore_first_modes(fixer, modes);
    let outcome = copied?;
    finish_first_pass(fixer, &source, rebuilt, outcome)
}

fn apply_current_face(fixer: &ShapeFixFace) -> Result<Shape, Failure> {
    super::super::apply_context(&fixer.face_state.read(), &fixer.context())
}

fn configure_first_pass(
    fixer: &ShapeFixFace,
    source: &Shape,
    saved_precision: f64,
) -> Result<(), Failure> {
    let face = topods::face(source)?;
    set_wire_face(fixer, &face)?;
    update_auto_precision(fixer, source, saved_precision)?;
    Ok(())
}

fn set_wire_face(fixer: &ShapeFixFace, face: &Face) -> Result<(), Failure> {
    let analysis = fixer.analysis_surface.read().clone();
    if analysis.is_null() {
        fixer.wire_tool.set_face(face)
    } else {
        fixer.wire_tool.set_face_with_analysis(face, &analysis);
        Ok(())
    }
}

fn update_auto_precision(
    fixer: &ShapeFixFace,
    source: &Shape,
    saved_precision: f64,
) -> Result<(), Failure> {
    if fixer.auto_correct_precision_mode() == 0 {
        return Ok(());
    }
    let candidate = ShapeFix::least_edge_size(source)? / 2. * 1.00001;
    if saved_precision > candidate && candidate > rustycad_tkernel::precision::confusion() {
        fixer.set_precision(candidate);
        fixer.wire_tool.set_precision(candidate);
    }
    Ok(())
}

fn empty_forward_face(source: &Shape) -> Result<Face, Failure> {
    let mut face = topods::face(&source.empty_copied()?)?;
    face.set_orientation(Orientation::Forward);
    Ok(face)
}

struct FirstPassOutcome {
    fixed: bool,
    reordered: bool,
}

#[derive(Clone, Copy)]
struct SavedFirstModes {
    lacking: i32,
    self_intersection: i32,
}

fn copy_first_pass_children(
    fixer: &ShapeFixFace,
    source: &Shape,
    rebuilt: &mut Face,
    progress: &ProgressRange<'_>,
    reordered_wires: &mut Vec<(Shape, Shape)>,
) -> Result<FirstPassOutcome, Failure> {
    let mut fixed = false;
    let mut reordered = false;
    for child in ShapeIterator::new(source, false, true) {
        if child.shape_type()? != ShapeEnum::Wire {
            BRepBuilder::default().add(rebuilt, &child)?;
            continue;
        }
        let (child_fixed, child_reordered) =
            perform_first_wire(fixer, &child, rebuilt, progress, reordered_wires)?;
        fixed = child_fixed;
        reordered |= child_reordered;
    }
    Ok(FirstPassOutcome { fixed, reordered })
}

fn disable_first_modes(fixer: &ShapeFixFace) -> SavedFirstModes {
    let wire = &fixer.wire_tool;
    let modes = SavedFirstModes {
        lacking: wire.fix_lacking_mode(),
        self_intersection: wire.fix_self_intersection_mode(),
    };
    wire.set_fix_lacking_mode(0);
    wire.set_fix_self_intersection_mode(0);
    modes
}

fn perform_first_wire(
    fixer: &ShapeFixFace,
    child: &Shape,
    rebuilt: &mut Face,
    progress: &ProgressRange<'_>,
    reordered_wires: &mut Vec<(Shape, Shape)>,
) -> Result<(bool, bool), Failure> {
    let wire = topods::wire(child)?;
    fixer.wire_tool.load(&wire)?;
    if fixer.wire_tool.nb_edges() == 0 {
        return preserve_or_drop_empty_wire(fixer, &wire, rebuilt);
    }
    if !fixer.wire_tool.perform_with_progress(progress)? {
        BRepBuilder::default().add(rebuilt, &wire.clone().into_shape())?;
        return Ok((false, false));
    }
    append_first_performed_wire(fixer, &wire, rebuilt, reordered_wires)
}

fn preserve_or_drop_empty_wire(
    fixer: &ShapeFixFace,
    wire: &Wire,
    rebuilt: &mut Face,
) -> Result<(bool, bool), Failure> {
    if fixer.wire_tool.wire_data()?.nb_non_manifold_edges() > 0 {
        BRepBuilder::default().add(rebuilt, &wire.clone().into_shape())?;
        return Ok((false, false));
    }
    fixer.record_wire_face_stage(ShapeExtendStatus::Done5);
    Ok((true, false))
}

fn append_first_performed_wire(
    fixer: &ShapeFixFace,
    source: &Wire,
    rebuilt: &mut Face,
    reordered_wires: &mut Vec<(Shape, Shape)>,
) -> Result<(bool, bool), Failure> {
    let wire_tool = &fixer.wire_tool;
    let fixed = first_wire_was_fixed(fixer);
    let output = wire_tool.wire()?;
    if fixed && wire_tool.nb_edges() == 0 {
        fixer.record_wire_face_stage(ShapeExtendStatus::Done5);
        replace_wire_context(fixer, source, &output)?;
        return Ok((true, wire_tool.status_reorder(ShapeExtendStatus::Done)));
    }
    if fixed {
        replace_wire_context(fixer, source, &output)?;
    } else {
        remember_reordered_wire(source, &output, reordered_wires);
    }
    BRepBuilder::default().add(rebuilt, &output.clone().into_shape())?;
    Ok((fixed, wire_tool.status_reorder(ShapeExtendStatus::Done)))
}

fn replace_wire_context(fixer: &ShapeFixFace, source: &Wire, output: &Wire) -> Result<(), Failure> {
    let context = fixer.context();
    if !context.is_null() {
        context.replace(&source.clone().into_shape(), &output.clone().into_shape())?;
    }
    Ok(())
}

fn first_wire_was_fixed(fixer: &ShapeFixFace) -> bool {
    let wire = &fixer.wire_tool;
    [
        wire.status_small(ShapeExtendStatus::Done),
        wire.status_connected(ShapeExtendStatus::Done),
        wire.status_edge_curves(ShapeExtendStatus::Done),
        wire.status_notches(ShapeExtendStatus::Done),
        wire.status_fix_tails(ShapeExtendStatus::Done),
        wire.status_degenerated(ShapeExtendStatus::Done),
        wire.status_closed(ShapeExtendStatus::Done),
    ]
    .into_iter()
    .any(|changed| changed)
}

fn restore_first_modes(fixer: &ShapeFixFace, modes: SavedFirstModes) {
    fixer.wire_tool.set_fix_lacking_mode(modes.lacking);
    fixer
        .wire_tool
        .set_fix_self_intersection_mode(modes.self_intersection);
}

fn finish_first_pass(
    fixer: &ShapeFixFace,
    source: &Shape,
    mut rebuilt: Face,
    outcome: FirstPassOutcome,
) -> Result<bool, Failure> {
    if !*fixer.forward.read() {
        rebuilt.set_orientation(Orientation::Reversed);
    }
    if outcome.fixed {
        replace_first_pass_context(fixer, source, &rebuilt)?;
    }
    if outcome.fixed || outcome.reordered {
        *fixer.face_state.write() = rebuilt.into_shape();
        if !fixer.wire_tool.status_reorder(ShapeExtendStatus::Done5) {
            fixer.record_wire_face_stage(ShapeExtendStatus::Done1);
        }
    }
    Ok(outcome.fixed)
}

fn remember_reordered_wire(
    original: &Wire,
    replacement: &Wire,
    reordered_wires: &mut Vec<(Shape, Shape)>,
) {
    let source = original.clone().into_shape();
    let target = replacement.clone().into_shape();
    if !source.is_same(&target) {
        reordered_wires.push((source, target));
    }
}

fn replace_first_pass_context(
    fixer: &ShapeFixFace,
    source: &Shape,
    rebuilt: &Face,
) -> Result<(), Failure> {
    let context = fixer.context();
    if !context.is_null() {
        context.replace(source, &rebuilt.clone().into_shape())?;
    }
    Ok(())
}
