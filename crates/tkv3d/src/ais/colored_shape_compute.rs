//! Native ColoredShape Compute prepares shared triangulation before live per-subshape dispatch and grouped emission.
use super::*;
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, std_prs::StdPrsToolTriangulatedShape};
use rustycad_tkbrep::topods::TopoDSShapeReadSource;
use rustycad_tkernel::standard::{Failure, Handle, HandleReference, catch_native_failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkservice::graphic3d::*;
/// Compute native ColoredShape modes without clearing prior groups; `ais_colored_shape_compute(&object,&prs,1)?`.
pub fn ais_colored_shape_compute<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    catch_native_failure(|| compute_colored_shape_modes(object, prs, mode))
}
fn compute_colored_shape_modes<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    let source = object.shape_ref();
    if source.read_current_shape(|shape| Ok(shape.is_null()))? {
        return Ok(());
    }
    if object.is_infinite() {
        prs.observe().set_infinite_state(true);
    }
    match mode {
        0 => prepare_colored_wireframe(object)?,
        1 => prepare_colored_shading(object)?,
        2 => return super::shape_compute_bounds::ais_shape_compute_bounds_mode(object, prs),
        _ => return Ok(()),
    }
    dispatch_colored_presentation(object, prs, mode)
}
fn prepare_colored_wireframe<S: AisColoredShapeSource + ?Sized>(object: &S) -> Result<(), Failure> {
    let source = object.shape_ref();
    let drawer = object.attributes();
    source.read_current_shape(|shape| {
        StdPrsToolTriangulatedShape::clear_on_own_deflection_change(shape, &drawer, true)
    })?;
    source
        .read_current_shape(|shape| StdPrsToolTriangulatedShape::get_deflection(shape, &drawer))?;
    Ok(())
}
fn prepare_colored_shading<S: AisColoredShapeSource + ?Sized>(object: &S) -> Result<(), Failure> {
    let drawer = object.attributes();
    if !drawer.is_auto_triangulation() {
        return Ok(());
    }
    let source = object.shape_ref();
    source.read_current_shape(|shape| {
        StdPrsToolTriangulatedShape::clear_on_own_deflection_change(shape, &drawer, true)
    })?;
    let rebuilt = source
        .read_current_shape(|shape| StdPrsToolTriangulatedShape::tessellate(shape, &drawer))?;
    if drawer.iso_on_triangulation() && rebuilt {
        object.set_to_update(0);
    }
    Ok(())
}
fn dispatch_colored_presentation<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    let styles = AisShapeDrawerMap::default();
    object.fill_subshape_drawer_map(&styles)?;
    let source = object.shape_ref();
    let base = source.read_current_shape(|shape| colored_shape_base_drawer(object, shape))?;
    let opened: AisColoredOpenedCompounds = std::array::from_fn(|_| Default::default());
    let closed = AisColoredDrawerCompounds::default();
    let closed_output = colored_closed_dispatch_output(mode, &opened, &closed);
    source.read_current_shape(|shape| {
        dispatch_colored_root(&base, shape, &styles, &opened, closed_output)
    })?;
    ais_colored_shape_add_custom_props(object, prs, &opened, &closed, mode)
}
fn colored_closed_dispatch_output<'a>(
    mode: i32,
    opened: &'a AisColoredOpenedCompounds,
    closed: &'a AisColoredDrawerCompounds,
) -> &'a AisColoredDrawerCompounds {
    if mode == 1 {
        return closed;
    }
    &opened[ShapeEnum::Face as usize]
}
pub(super) fn colored_shape_base_drawer<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    shape: &rustycad_tkbrep::topods::Shape,
) -> Result<Handle<AisColoredDrawer>, Failure> {
    let nodes = object.colored_shape_base().shape_colors.nodes.read();
    let Some(node) = super::shape_drawer_map::find_shape_drawer_node(&nodes, shape) else {
        return Ok(Handle::default());
    };
    node.read_fields(|fields| fields.drawer.clone())
}

fn dispatch_colored_root(
    base: &Handle<AisColoredDrawer>,
    shape: &rustycad_tkbrep::topods::Shape,
    styles: &AisShapeDrawerMap,
    opened: &AisColoredOpenedCompounds,
    closed: &AisColoredDrawerCompounds,
) -> Result<bool, Failure> {
    let parent = ShapeEnum::Compound;
    ais_colored_shape_dispatch_colors(base, shape, styles, parent, false, opened, closed)
}
