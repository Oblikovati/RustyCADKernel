mod classify;
mod orient;
mod periodic;

use super::{ShapeFixFace, ShapeFixFaceWireMap, apply_context};
use crate::shape_analysis::{ShapeAnalysis, ShapeAnalysisEdge};
use crate::shape_extend::ShapeExtendWireData;
use crate::shape_fix::{FixRoot, need_fix};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

#[derive(Default)]
struct FaceWireSet {
    oriented: Vec<Wire>,
    all: Vec<Shape>,
    changed: bool,
}

pub(super) fn fix_orientation_empty_map(fixer: &ShapeFixFace) -> Result<bool, Failure> {
    fix_orientation(fixer, &mut ShapeFixFaceWireMap::default())
}

pub(super) fn fix_orientation(
    fixer: &ShapeFixFace,
    map: &mut ShapeFixFaceWireMap,
) -> Result<bool, Failure> {
    let face = current_face(fixer)?;
    store_current_face(fixer, &face);
    let wires = collect_oriented_wires(&face)?;
    if wires.oriented.is_empty() {
        return Ok(false);
    }
    let natural = is_need_add_natural_bound(fixer, &face, &wires.oriented)?;
    let (oriented, reversed, changed) = orientation_decision(fixer, &face, &wires, natural, map)?;
    finish_orientation(fixer, &face, &wires, oriented, reversed, changed, natural)
}

fn store_current_face(fixer: &ShapeFixFace, face: &Face) {
    let shape = face.clone().into_shape();
    *fixer.face_state.write() = shape;
}

fn orientation_decision(
    fixer: &ShapeFixFace,
    face: &Face,
    wires: &FaceWireSet,
    natural: bool,
    map: &mut ShapeFixFaceWireMap,
) -> Result<(Vec<Wire>, Vec<usize>, bool), Failure> {
    if wires.oriented.len() == 1 {
        return orient_single_wire(face, wires, natural, fixer);
    }
    if fixer.analysis_surface.read().is_null() {
        return Ok((wires.oriented.clone(), Vec::new(), wires.changed));
    }
    classify::orient_multiple_wires(fixer, face, wires, map)
}

fn current_face(fixer: &ShapeFixFace) -> Result<Face, Failure> {
    let shape = fixer.face_state.read().clone();
    let context = fixer.context();
    let applied = apply_context(&shape, &context)?;
    topods::face(&applied)
}

fn collect_oriented_wires(face: &Face) -> Result<FaceWireSet, Failure> {
    let mut wires = FaceWireSet::default();
    for child in ShapeIterator::new(&face.clone().into_shape(), false, true) {
        collect_face_child(&mut wires, child)?;
    }
    Ok(wires)
}

fn collect_face_child(wires: &mut FaceWireSet, child: Shape) -> Result<(), Failure> {
    if is_orientation_candidate(&child)? {
        let wire = topods::wire(&child)?;
        if wire_length(&wire)? > rustycad_tkernel::precision::confusion() {
            wires.oriented.push(wire);
            wires.all.push(child);
        } else {
            wires.changed = true;
        }
    } else {
        wires.all.push(child);
    }
    Ok(())
}

fn wire_length(wire: &Wire) -> Result<f64, Failure> {
    let edges: Vec<Shape> = ShapeIterator::new(&wire.clone().into_shape(), false, true).collect();
    if edges.len() > 1 {
        return Ok(f64::MAX);
    }
    let Some(edge) = edges.first() else {
        return Ok(0.0);
    };
    sampled_edge_length(&topods::edge(edge)?)
}

fn sampled_edge_length(edge: &rustycad_tkbrep::topods::Edge) -> Result<f64, Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().curve_3d(edge, true)?;
    if curve.is_null() {
        return Ok(0.0);
    }
    let mut previous = curve.value(first)?;
    let mut length = 0.0;
    for index in 1..10 {
        let parameter = ((9 - index) as f64 * first + index as f64 * last) / 9.0;
        let point = curve.value(parameter)?;
        length += point.distance(&previous);
        previous = point;
    }
    Ok(length)
}

fn is_orientation_candidate(shape: &Shape) -> Result<bool, Failure> {
    if shape.shape_type()? != ShapeEnum::Wire {
        return Ok(false);
    }
    Ok(matches!(
        shape.orientation(),
        Orientation::Forward | Orientation::Reversed
    ))
}

pub(super) fn is_need_add_natural_bound(
    fixer: &ShapeFixFace,
    face: &Face,
    wires: &[Wire],
) -> Result<bool, Failure> {
    if !need_fix(fixer.fix_add_natural_bound_mode(), true) {
        return Ok(false);
    }
    if !has_double_closed_surface(fixer)? || ShapeAnalysis::is_outer_bound(face)? {
        return Ok(false);
    }
    no_wire_seam_or_degenerated_edge(face, wires)
}

fn has_double_closed_surface(fixer: &ShapeFixFace) -> Result<bool, Failure> {
    let surface = fixer.analysis_surface.read().clone();
    let Some(surface) = surface.get() else {
        return Ok(true);
    };
    let adaptor = surface.adaptor3d();
    Ok((adaptor.is_u_periodic()? && adaptor.is_v_periodic()?)
        || adaptor.get_type()? == rustycad_tkmath::geom_abs::SurfaceType::Sphere)
}

fn no_wire_seam_or_degenerated_edge(face: &Face, wires: &[Wire]) -> Result<bool, Failure> {
    for wire in wires {
        for child in ShapeIterator::new(&wire.clone().into_shape(), false, true) {
            let edge = topods::edge(&child)?;
            if BRepTool::degenerated(&edge)? || BRepTool::is_closed_on_face(&edge, face)? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn orient_single_wire(
    face: &Face,
    wires: &FaceWireSet,
    natural: bool,
    fixer: &ShapeFixFace,
) -> Result<(Vec<Wire>, Vec<usize>, bool), Failure> {
    let mut oriented = wires.oriented.clone();
    if oriented.len() != 1 || natural || single_wire_is_outer(face, &oriented[0])? {
        return Ok((oriented, Vec::new(), false));
    }
    let fixed = reverse_wire(face, &oriented[0])?;
    send_reversed_warning(fixer, &fixed);
    oriented[0] = fixed;
    Ok((oriented, vec![1], true))
}

fn single_wire_is_outer(face: &Face, wire: &Wire) -> Result<bool, Failure> {
    let mut single = face.empty_copied()?;
    single.set_orientation(Orientation::Forward);
    rustycad_tkbrep::topods::Builder.add(&mut single, &wire.clone().into_shape())?;
    ShapeAnalysis::is_outer_bound(&topods::face(&single)?)
}

fn reverse_wire(face: &Face, wire: &Wire) -> Result<Wire, Failure> {
    let data = Handle::new(ShapeExtendWireData::from_wire(wire, true, true)?);
    data.reverse_on_face(face)?;
    data.wire()
}

fn send_reversed_warning(fixer: &ShapeFixFace, wire: &Wire) {
    let msg = Msg::from_key(&rustycad_tkernel::tcollection::AsciiString::from(
        "FixAdvFace.FixOrientation.MSG5",
    ));
    fixer.send_warning(&wire.clone().into_shape(), &msg);
}

fn finish_orientation(
    fixer: &ShapeFixFace,
    face: &Face,
    wires: &FaceWireSet,
    oriented: Vec<Wire>,
    reversed: Vec<usize>,
    changed: bool,
    natural: bool,
) -> Result<bool, Failure> {
    if !orientation_changed(changed, wires, &reversed, natural) {
        return Ok(false);
    }
    let result = rebuild_face(face, &wires.all, &oriented, fixer)?;
    store_oriented_result(fixer, face, &result)?;
    Ok(true)
}

fn orientation_changed(
    changed: bool,
    wires: &FaceWireSet,
    reversed: &[usize],
    natural: bool,
) -> bool {
    (changed || wires.changed) && !(natural && reversed.len() == wires.oriented.len())
}

fn store_oriented_result(fixer: &ShapeFixFace, face: &Face, result: &Shape) -> Result<(), Failure> {
    replace_oriented_face_in_context(fixer, face, result)?;
    fixer.face_state.write().clone_from(result);
    rustycad_tkbrep::brep_tools::update_face(&topods::face(result)?)
}

fn replace_oriented_face_in_context(
    fixer: &ShapeFixFace,
    face: &Face,
    result: &Shape,
) -> Result<(), Failure> {
    let context = fixer.context();
    let Some(context) = context.get() else {
        return Ok(());
    };
    rustycad_tkbrep::brep_tools::ReShape::replace(context, &face.clone().into_shape(), result)
}

fn rebuild_face(
    face: &Face,
    other_children: &[Shape],
    wires: &[Wire],
    fixer: &ShapeFixFace,
) -> Result<Shape, Failure> {
    let mut result = face.empty_copied()?;
    result.set_orientation(Orientation::Forward);
    append_oriented_wires(&mut result, wires)?;
    append_nonwire_children(&mut result, other_children)?;
    preserve_face_orientation(&mut result, fixer);
    Ok(result)
}

fn append_oriented_wires(result: &mut Shape, wires: &[Wire]) -> Result<(), Failure> {
    let builder = rustycad_tkbrep::topods::Builder;
    for wire in wires {
        builder.add(result, &wire.clone().into_shape())?;
    }
    Ok(())
}

fn append_nonwire_children(result: &mut Shape, children: &[Shape]) -> Result<(), Failure> {
    let builder = rustycad_tkbrep::topods::Builder;
    for child in children {
        if is_orientation_candidate(child)? {
            continue;
        }
        builder.add(result, child)?;
    }
    Ok(())
}

fn preserve_face_orientation(result: &mut Shape, fixer: &ShapeFixFace) {
    if !*fixer.forward.read() {
        result.set_orientation(Orientation::Reversed);
    }
}
