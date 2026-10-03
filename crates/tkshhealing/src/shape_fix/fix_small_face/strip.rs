#[path = "strip_shared_edge.rs"]
mod shared_edge;

use super::ShapeFixFixSmallFace;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn fix_strip_face(
    fixer: &ShapeFixFixSmallFace,
    was_done: bool,
) -> Result<Shape, Failure> {
    let input = fixer.input.read().clone();
    if input.is_null() || !strip_shape_supported(&input)? {
        return Ok(input);
    }
    let done = scan_strip_faces(fixer, &input, was_done)?;
    let mut shape = apply_context(fixer, &input)?;
    *fixer.input.write() = shape.clone();
    if shape.is_null() {
        return Ok(shape);
    }
    remove_empty_shells(fixer, &shape)?;
    shape = apply_context(fixer, &shape)?;
    *fixer.input.write() = shape.clone();
    let result = finish_strip_phase(fixer, &shape, done)?;
    *fixer.result.write() = result.clone();
    Ok(result)
}

pub(super) fn replace_in_case_of_strip(
    fixer: &ShapeFixFixSmallFace,
    face: &mut Face,
    first: &mut Edge,
    second: &mut Edge,
    tolerance: f64,
) -> Result<bool, Failure> {
    let context = context(fixer)?;
    replace_strip_face(
        fixer,
        &context,
        face,
        first.clone(),
        second.clone(),
        tolerance,
    )
}

pub(super) fn remove_faces_in_case_of_strip(
    fixer: &ShapeFixFixSmallFace,
    face: &Face,
) -> Result<bool, Failure> {
    remove_faces_with_context(fixer, &context(fixer)?, face)?;
    Ok(true)
}

pub(super) fn compute_shared_edge_for_strip_face(
    fixer: &ShapeFixFixSmallFace,
    first: &Edge,
    second: &Edge,
    support: &Face,
    tolerance: f64,
) -> Result<Edge, Failure> {
    shared_edge::compute(fixer, first, second, support, tolerance)
}

fn strip_shape_supported(shape: &Shape) -> Result<bool, Failure> {
    Ok(matches!(
        shape.shape_type()?,
        ShapeEnum::Compound
            | ShapeEnum::CompSolid
            | ShapeEnum::Solid
            | ShapeEnum::Shell
            | ShapeEnum::Face
    ))
}

fn scan_strip_faces(
    fixer: &ShapeFixFixSmallFace,
    shape: &Shape,
    mut done: bool,
) -> Result<bool, Failure> {
    let context = context(fixer)?;
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        let applied = ReShape::apply(context.get().unwrap(), faces.current(), ShapeEnum::Shape)?;
        if !applied.is_null() {
            done |= fix_strip_candidate(fixer, &context, &applied)?;
        }
        faces.next()?;
    }
    Ok(done)
}

fn fix_strip_candidate(
    fixer: &ShapeFixFixSmallFace,
    context: &Handle<ShapeBuildReShape>,
    shape: &Shape,
) -> Result<bool, Failure> {
    let mut face = topods::face(shape)?;
    let mut first = Edge::default();
    let mut second = Edge::default();
    let found = fixer.analyzer.write().check_strip_face(
        &face,
        &mut first,
        &mut second,
        fixer.precision(),
    )?;
    if !found {
        return Ok(false);
    }
    let tolerance = fixer.precision();
    let replaced = replace_strip_face(fixer, context, &mut face, first, second, tolerance)?;
    if replaced {
        remove_faces_with_context(fixer, context, &face)?;
    }
    *fixer.status.write() = ShapeExtend::encode_status(ShapeExtendStatus::Done2);
    Ok(true)
}

fn replace_strip_face(
    fixer: &ShapeFixFixSmallFace,
    context: &Handle<ShapeBuildReShape>,
    face: &mut Face,
    first: Edge,
    second: Edge,
    tolerance: f64,
) -> Result<bool, Failure> {
    if first.is_null() || second.is_null() {
        return Ok(false);
    }
    let applied = ReShape::apply(
        context.get().unwrap(),
        &face.clone().into_shape(),
        ShapeEnum::Shape,
    )?;
    *face = topods::face(&applied)?;
    let (first_face, second_face) = adjacent_strip_faces(fixer, face, &first, &second)?;
    let replaced = replace_strip_boundary(
        fixer,
        context,
        face,
        &first,
        &second,
        (first_face, second_face),
        tolerance,
    )?;
    Ok(replaced)
}

fn adjacent_strip_faces(
    fixer: &ShapeFixFixSmallFace,
    face: &Face,
    first: &Edge,
    second: &Edge,
) -> Result<(Option<Face>, Option<Face>), Failure> {
    let mut first_face = None;
    let mut second_face = None;
    let input = fixer.input.read().clone();
    let context = context(fixer)?;
    let mut faces = Explorer::new(&input, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        let applied = ReShape::apply(context.get().unwrap(), faces.current(), ShapeEnum::Shape)?;
        if !applied.is_null() {
            find_edge_faces(
                &applied,
                face,
                first,
                second,
                &mut first_face,
                &mut second_face,
            )?;
        }
        faces.next()?;
    }
    Ok((first_face, second_face))
}

fn find_edge_faces(
    candidate: &Shape,
    face: &Face,
    first: &Edge,
    second: &Edge,
    first_face: &mut Option<Face>,
    second_face: &mut Option<Face>,
) -> Result<(), Failure> {
    let other = topods::face(candidate)?;
    if other.is_same(&face.clone().into_shape()) {
        return Ok(());
    }
    let mut edges = Explorer::new(
        &other.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while edges.more() {
        record_matching_edge(
            &topods::edge(edges.current())?,
            first,
            second,
            &other,
            first_face,
            second_face,
        );
        if first_face.is_some() {
            break;
        }
        edges.next()?;
    }
    Ok(())
}

fn record_matching_edge(
    edge: &Edge,
    first: &Edge,
    second: &Edge,
    face: &Face,
    first_face: &mut Option<Face>,
    second_face: &mut Option<Face>,
) {
    if edge.is_same(&first.clone().into_shape()) {
        *first_face = Some(face.clone());
    }
    if edge.is_same(&second.clone().into_shape()) {
        *second_face = Some(face.clone());
    }
}

fn replace_strip_boundary(
    fixer: &ShapeFixFixSmallFace,
    context: &Handle<ShapeBuildReShape>,
    face: &mut Face,
    first: &Edge,
    second: &Edge,
    adjacent_faces: (Option<Face>, Option<Face>),
    tolerance: f64,
) -> Result<bool, Failure> {
    let Some((adjacent, first_edge, second_edge)) =
        select_adjacent(adjacent_faces.0, adjacent_faces.1, first, second)
    else {
        return Ok(true);
    };
    let shared = shared_edge::compute(fixer, &first_edge, &second_edge, &adjacent, tolerance)?;
    if shared.is_null() {
        return Ok(false);
    }
    let first_reversed = first.orientation() == Orientation::Reversed;
    replace_long_edges(
        context,
        face,
        &first_edge,
        &second_edge,
        first_reversed,
        &adjacent,
        &shared,
    )?;
    remove_short_edges(context, face, &first_edge, &second_edge)?;
    Ok(true)
}

fn select_adjacent(
    first_face: Option<Face>,
    second_face: Option<Face>,
    first: &Edge,
    second: &Edge,
) -> Option<(Face, Edge, Edge)> {
    let has_first = first_face.is_some();
    let face = first_face.or(second_face)?;
    let (first_edge, second_edge) = if has_first {
        (first.clone(), second.clone())
    } else {
        (second.clone(), first.clone())
    };
    Some((face, first_edge, second_edge))
}

fn replace_long_edges(
    context: &Handle<ShapeBuildReShape>,
    face: &Face,
    first: &Edge,
    second: &Edge,
    first_reversed: bool,
    adjacent: &Face,
    shared: &Edge,
) -> Result<(), Failure> {
    let first_new = if first_reversed {
        shared.oriented(Orientation::Reversed)
    } else {
        shared.clone().into_shape()
    };
    let same_face_orientation = face.orientation() == adjacent.orientation();
    let second_orientation = if same_face_orientation != first_reversed {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    let second_new = shared.oriented(second_orientation);
    ReShape::replace(
        context.get().unwrap(),
        &first.clone().into_shape(),
        &first_new,
    )?;
    ReShape::replace(
        context.get().unwrap(),
        &second.clone().into_shape(),
        &second_new,
    )
}

fn remove_short_edges(
    context: &Handle<ShapeBuildReShape>,
    face: &Face,
    first: &Edge,
    second: &Edge,
) -> Result<(), Failure> {
    let mut edges = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while edges.more() {
        let edge = topods::edge(edges.current())?;
        if !edge.is_same(&first.clone().into_shape()) && !edge.is_same(&second.clone().into_shape())
        {
            ReShape::remove(context.get().unwrap(), &edge.into_shape())?;
        }
        edges.next()?;
    }
    Ok(())
}

fn remove_faces_with_context(
    fixer: &ShapeFixFixSmallFace,
    context: &Handle<ShapeBuildReShape>,
    face: &Face,
) -> Result<(), Failure> {
    ReShape::remove(context.get().unwrap(), &face.clone().into_shape())?;
    let message = Msg::from_key(&AsciiString::from("FixAdvFace.FixStripFace.MSG0"));
    fixer.send_warning(&face.clone().into_shape(), &message);
    Ok(())
}

fn remove_empty_shells(fixer: &ShapeFixFixSmallFace, shape: &Shape) -> Result<(), Failure> {
    let context = context(fixer)?;
    let mut shells = Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape)?;
    while shells.more() {
        let shell = shells.current().clone();
        if !has_face(&shell)? {
            ReShape::remove(context.get().unwrap(), &shell)?;
        }
        shells.next()?;
    }
    Ok(())
}

fn has_face(shell: &Shape) -> Result<bool, Failure> {
    Ok(Explorer::new(shell, ShapeEnum::Face, ShapeEnum::Shape)?.more())
}

fn finish_strip_phase(
    fixer: &ShapeFixFixSmallFace,
    shape: &Shape,
    done: bool,
) -> Result<Shape, Failure> {
    if !done || shape.is_null() {
        return Ok(shape.clone());
    }
    super::spot::fix_shape(fixer)
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
        "ShapeFix_FixSmallFace requires Init(shape) before fixing a strip",
    ))
}
