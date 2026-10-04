//! Existing pcurve transfer from BOPTools_AlgoTools2D_1.cxx.
use crate::{
    bop_tools::is_split_edge_to_reverse,
    int_tools::{Context, Tools},
};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face, ShapeIterator, Vertex},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::{geom::GeomCurve, top_abs::Orientation};
use rustycad_tkgeombase::geom_lib;
use rustycad_tktopalgo::brep_lib;
mod closed;
type RangedPCurve = (Handle<dyn Geom2dCurve>, (f64, f64));
/// Attach a reparameterized copy of an old edge's pcurve; `attach_existing_pcurve(&old,&new,&face,&context)?`.
pub fn attach_existing_pcurve(
    old: &Edge,
    new: &Edge,
    face: &Face,
    context: &Context,
) -> Result<i32, Failure> {
    let (old, new, face) = forward_shapes(old, new, face);
    let Some((pcurve, old_range)) = prepare_pcurve(&old, &new, &face, context)? else {
        return Ok(1);
    };
    let (spatial, first, last) = BRepTool::curve(&new)?;
    let mut adjusted = Handle::default();
    geom_lib::same_range(
        precision::p_confusion(),
        &pcurve,
        old_range,
        (first, last),
        &mut adjusted,
    )?;
    if adjusted.is_null() {
        return Ok(2);
    }
    let status = check_tolerance(&new, &face, &spatial, &adjusted, (first, last))?;
    if status != 0 {
        return Ok(status);
    }
    finish_attachment(
        &old,
        &new,
        &face,
        &spatial,
        &adjusted,
        (first, last),
        context,
    )
}
fn forward_shapes(old: &Edge, new: &Edge, face: &Face) -> (Edge, Edge, Face) {
    let (mut old, mut new, mut face) = (old.clone(), new.clone(), face.clone());
    old.set_orientation(Orientation::Forward);
    new.set_orientation(Orientation::Forward);
    face.set_orientation(Orientation::Forward);
    (old, new, face)
}
fn prepare_pcurve(
    old: &Edge,
    new: &Edge,
    face: &Face,
    context: &Context,
) -> Result<Option<RangedPCurve>, Failure> {
    let stored = BRepTool::curve_on_face(old, face)?;
    if stored.pcurve.is_null() {
        return Ok(None);
    }
    let copy: Handle<dyn Geom2dCurve> = stored.pcurve.copy()?.cast();
    let mut bounds = (stored.first, stored.last);
    if is_split_edge_to_reverse(new, old, context, &mut 0)? {
        copy.reverse()?;
        bounds = (
            copy.reversed_parameter(bounds.1)?,
            copy.reversed_parameter(bounds.0)?,
        );
    }
    Ok(Some((
        Handle::new(Geom2dTrimmedCurve::new(
            &copy, bounds.0, bounds.1, true, true,
        )?)
        .cast(),
        bounds,
    )))
}
fn check_tolerance(
    edge: &Edge,
    face: &Face,
    spatial: &Handle<dyn GeomCurve>,
    pcurve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
) -> Result<i32, Failure> {
    let mut distance = 0.;
    let mut parameter = 0.;
    if !Tools::compute_tolerance(
        spatial,
        pcurve,
        &BRepTool::surface(face)?,
        range.0,
        range.1,
        &mut distance,
        &mut parameter,
        precision::p_confusion(),
        false,
    )? {
        return Ok(3);
    }
    if distance > 10. * BRepTool::edge_tolerance(edge)? && distance > 0.1 {
        return Ok(4);
    }
    Ok(0)
}
fn temporary_edge(
    new: &Edge,
    face: &Face,
    spatial: &Handle<dyn GeomCurve>,
    pcurve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    let tolerance = BRepTool::edge_tolerance(new)?;
    builder.make_edge_with_curve(&mut edge, spatial, tolerance)?;
    builder.range(&edge, range.0, range.1, false)?;
    builder.same_range(&edge, false)?;
    builder.same_parameter(&edge, false)?;
    builder.update_edge_pcurve_on_face(&edge, pcurve, face, tolerance)?;
    Ok(edge)
}
fn finish_attachment(
    old: &Edge,
    new: &Edge,
    face: &Face,
    spatial: &Handle<dyn GeomCurve>,
    pcurve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    context: &Context,
) -> Result<i32, Failure> {
    let temporary = temporary_edge(new, face, spatial, pcurve, range)?;
    if brep_lib::same_parameter_edge(&temporary, 1e-5)
        .and_then(|()| brep_lib::same_range(&temporary, 1e-5))
        .is_err()
    {
        return Ok(6);
    }
    if closed::is_closed(old, face)? && closed::update_pcurve(old, &temporary, face, context)? != 0
    {
        return Ok(5);
    }
    let builder = BRepBuilder::default();
    builder.transfert(&temporary, new)?;
    let tolerance = BRepTool::edge_tolerance(&temporary)?;
    for child in ShapeIterator::new(&new.clone().into_shape(), true, true) {
        builder.update_vertex_tolerance(&Vertex::unchecked(child), tolerance)?;
    }
    Ok(0)
}

#[cfg(test)]
mod tests;
