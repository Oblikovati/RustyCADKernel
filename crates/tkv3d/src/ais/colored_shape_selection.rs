//! Native ColoredShape selection retains inherited fast paths and skips hidden styles during depth-first sensitive construction.
use super::*;
use crate::{
    prs_mgr::PrsMgrPresentableObjectMembers,
    select_mgr::*,
    std_prs::StdPrsToolTriangulatedShape,
    std_select::{StdSelectBRepOwner, StdSelectBrepSelectionTool},
};
use rustycad_tkbrep::{
    brep_tools,
    topods::{Shape, ShapeIterator, TopoDSShapeReadSource},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, catch_native_failure},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
/// Execute actual native whole/per-subshape selection policy; `ais_colored_shape_compute_selection(object,&selection,4)?`.
pub fn ais_colored_shape_compute_selection<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    catch_native_failure(|| compute_colored_shape_selection(object, selection, mode))
}
fn compute_colored_shape_selection<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    let source = object.shape_ref();
    if source.read_current_shape(|shape| Ok(shape.is_null()))? {
        return Ok(());
    }
    if object.is_shape_entirely_visible()? {
        return ais_shape_compute_selection(object, selection, mode);
    }
    let kind = ais_shape_selection_type(mode);
    let drawer = object.attributes();
    let deflection = source
        .read_current_shape(|shape| StdPrsToolTriangulatedShape::get_deflection(shape, &drawer))?;
    let angle = drawer.deviation_angle();
    let priority = source.read_current_shape(|shape| {
        StdSelectBrepSelectionTool::get_standard_priority(shape, kind)
    })?;
    prepare_colored_selection_mesh(object, deflection, angle)?;
    let styles = AisShapeDrawerMap::default();
    object.fill_subshape_drawer_map(&styles)?;
    let context = ColoredSubshapeSelection {
        object,
        selection,
        styles: &styles,
        kind,
        priority,
        deflection,
        angle,
    };
    begin_colored_subshape_selection(&context)?;
    let selectable = object.presentable_base().this().cast();
    crate::std_select::brep_selection_load::set_brep_selection_origin(selection, &selectable);
    Ok(())
}
fn prepare_colored_selection_mesh<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    deflection: f64,
    angle: f64,
) -> Result<(), Failure> {
    if !object.attributes().is_auto_triangulation() {
        return Ok(());
    }
    object.shape_ref().read_current_shape(|shape| {
        if !brep_tools::triangulation(shape, precision::infinite(), false)? {
            BRepMeshIncrementalMesh::with_shape(shape, deflection, false, angle, false)?;
        }
        Ok(())
    })
}
struct ColoredSubshapeSelection<'a, S: AisColoredShapeSource + ?Sized> {
    object: &'a S,
    selection: &'a Handle<SelectMgrSelection>,
    styles: &'a AisShapeDrawerMap,
    kind: ShapeEnum,
    priority: i32,
    deflection: f64,
    angle: f64,
}
fn begin_colored_subshape_selection<S: AisColoredShapeSource + ?Sized>(
    context: &ColoredSubshapeSelection<'_, S>,
) -> Result<(), Failure> {
    let source = context.object.shape_ref();
    let mut owner = source.read_current_shape(|shape| {
        Ok(StdSelectBRepOwner::with_shape(
            shape,
            context.priority,
            false,
        ))
    })?;
    if context.kind == ShapeEnum::Shape {
        // Native constructs the root owner twice for mode zero; do not collapse its retain/release sequence.
        owner = source.read_current_shape(|shape| {
            Ok(StdSelectBRepOwner::with_shape(
                shape,
                context.priority,
                false,
            ))
        })?;
    }
    let base = source.read_current_shape(|shape| {
        super::colored_shape_compute::colored_shape_base_drawer(context.object, shape)
    })?;
    source.read_current_shape(|shape| {
        compute_colored_subshape_selection(&base, shape, &owner, context)
    })
}
fn compute_colored_subshape_selection<S: AisColoredShapeSource + ?Sized>(
    parent: &Handle<AisColoredDrawer>,
    shape: &Shape,
    owner: &Handle<StdSelectBRepOwner>,
    context: &ColoredSubshapeSelection<'_, S>,
) -> Result<(), Failure> {
    let (drawer, _) =
        super::colored_shape_dispatch::dispatch_colored_drawer(parent, shape, context.styles)?;
    if !drawer.is_null() && drawer.is_hidden() {
        return Ok(());
    }
    let kind = shape.shape_type()?;
    if context.kind == ShapeEnum::Shape && kind >= ShapeEnum::Face {
        return build_colored_sensitive(shape, owner, context);
    }
    if kind == context.kind {
        let decomposed = !context.object.shape_ref().is_equal(shape);
        let owner = StdSelectBRepOwner::with_shape(shape, context.priority, decomposed);
        return build_colored_sensitive(shape, &owner, context);
    }
    let mut children = ShapeIterator::new(shape, true, true);
    while children.more() {
        compute_colored_subshape_selection(&drawer, children.value()?, owner, context)?;
        children.next();
    }
    Ok(())
}
fn build_colored_sensitive<S: AisColoredShapeSource + ?Sized>(
    shape: &Shape,
    owner: &Handle<StdSelectBRepOwner>,
    context: &ColoredSubshapeSelection<'_, S>,
) -> Result<(), Failure> {
    let owner = owner.cast();
    StdSelectBrepSelectionTool::compute_sensitive(
        shape,
        &owner,
        context.selection,
        context.deflection,
        context.angle,
        9,
        500.,
        context.object.attributes().is_auto_triangulation(),
    )
}
