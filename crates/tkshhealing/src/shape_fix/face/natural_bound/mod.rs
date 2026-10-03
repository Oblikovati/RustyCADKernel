mod build;
mod intervals;
mod selection;
mod shift;

use super::orientation::is_need_add_natural_bound;
use super::{ShapeFixFace, apply_context, replace_context};
use crate::shape_fix::{FixRoot, ShapeFixEdge};
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator};
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

pub(super) fn fix_add_natural_bound(fixer: &ShapeFixFace) -> Result<bool, Failure> {
    let analysis_handle = fixer.analysis_surface.read().clone();
    let Some(surface) = analysis_handle.get() else {
        return Ok(false);
    };
    let (face, context) = current_face(fixer)?;
    let (wires, other_children) = collect_face_children(&face)?;
    if wires.is_empty() && is_finite_surface(&surface.surface())? {
        return create_natural_face(fixer, &face, &surface.surface(), &context);
    }
    if !is_need_add_natural_bound(fixer, &face, &wires)? {
        return Ok(false);
    }
    build::build_with_natural_bounds(fixer, &face, &context, surface, wires, other_children)
}

fn current_face(
    fixer: &ShapeFixFace,
) -> Result<
    (
        Face,
        rustycad_tkernel::standard::Handle<crate::shape_build::ShapeBuildReShape>,
    ),
    Failure,
> {
    let (shape, context) = fixer
        .fix_root_data()
        .with(|fields| (fixer.face_state.read().clone(), fields.context.clone()));
    let applied = apply_context(&shape, &context)?;
    let face = topods::face(&applied)?;
    *fixer.face_state.write() = applied.clone();
    Ok((face, context))
}

fn collect_face_children(face: &Face) -> Result<(Vec<topods::Wire>, Vec<Shape>), Failure> {
    let mut wires = Vec::new();
    let mut other = Vec::new();
    for child in ShapeIterator::new(&face.clone().into_shape(), false, true) {
        if is_forward_or_reversed_wire(&child)? {
            wires.push(topods::wire(&child)?);
        } else {
            other.push(child);
        }
    }
    Ok((wires, other))
}

fn is_forward_or_reversed_wire(shape: &Shape) -> Result<bool, Failure> {
    if shape.shape_type()? != ShapeEnum::Wire {
        return Ok(false);
    }
    Ok(matches!(
        shape.orientation(),
        Orientation::Forward | Orientation::Reversed
    ))
}

fn is_finite_surface(
    surface: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
) -> Result<bool, Failure> {
    let (umin, umax, vmin, vmax) = surface
        .get()
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "natural face boundary requires a live surface",
            )
        })?
        .bounds();
    Ok(![umin, umax, vmin, vmax]
        .into_iter()
        .any(precision::is_infinite))
}

fn create_natural_face(
    fixer: &ShapeFixFace,
    old_face: &Face,
    surface: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    context: &rustycad_tkernel::standard::Handle<crate::shape_build::ShapeBuildReShape>,
) -> Result<bool, Failure> {
    let mut new_face =
        BRepBuilderAPIMakeFace::from_surface(surface, precision::confusion())?.face()?;
    new_face.set_orientation(old_face.orientation());
    let new_shape = new_face.clone().into_shape();
    replace_context(context, old_face, &new_shape)?;
    *fixer.face_state.write() = new_shape.clone();
    fix_natural_face_edge_tolerances(fixer, &new_face)?;
    send_natural_boundary_warning(fixer, &new_face);
    rustycad_tkbrep::brep_tools::update_face(&new_face)?;
    *fixer.result.write() = new_shape;
    Ok(true)
}

fn fix_natural_face_edge_tolerances(fixer: &ShapeFixFace, face: &Face) -> Result<(), Failure> {
    let edge_fixer = ShapeFixEdge::default();
    edge_fixer.set_context(&fixer.context());
    let mut pending = vec![face.clone().into_shape()];
    while let Some(parent) = pending.pop() {
        for child in ShapeIterator::new(&parent, true, true) {
            if child.shape_type()? == ShapeEnum::Edge {
                edge_fixer.fix_vertex_tolerance_on_face(&topods::edge(&child)?, face)?;
            } else {
                pending.push(child);
            }
        }
    }
    Ok(())
}

fn send_natural_boundary_warning(fixer: &ShapeFixFace, face: &Face) {
    let message = Msg::from_key(&AsciiString::from("FixAdvFace.FixOrientation.MSG0"));
    fixer.send_warning(&face.clone().into_shape(), &message);
}

fn publish_rebuilt_face(
    fixer: &ShapeFixFace,
    old_face: &Face,
    context: &rustycad_tkernel::standard::Handle<crate::shape_build::ShapeBuildReShape>,
    result: &Face,
) -> Result<(), Failure> {
    let shape = result.clone().into_shape();
    replace_context(context, old_face, &shape)?;
    *fixer.face_state.write() = shape;
    rustycad_tkbrep::brep_tools::update_face(result)?;
    send_natural_boundary_warning(fixer, result);
    Ok(())
}
