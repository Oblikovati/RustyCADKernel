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
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, catch_native_failure},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
/// Execute actual native whole/per-subshape selection policy; `ais_colored_shape_compute_selection(object,&selection,4)?`.
pub fn ais_colored_shape_compute_selection<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    selection: &dyn HandleReference<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    catch_native_failure(|| compute_colored_shape_selection(object, selection, mode))
}
/// Construct native subshape selection using the supplied style map and owner.
/// Example: `ais_colored_shape_compute_subshape_selection(object,&parent,&styles,&shape,&owner,&selection,ShapeEnum::Face,42,0.1,0.5)?`.
#[allow(clippy::too_many_arguments)] // Pinned protected native overload.
pub fn ais_colored_shape_compute_subshape_selection<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    parent: &Handle<AisColoredDrawer>,
    styles: &AisShapeDrawerMap,
    shape: &Shape,
    owner: &dyn HandleReference<StdSelectBRepOwner>,
    selection: &dyn HandleReference<SelectMgrSelection>,
    kind: ShapeEnum,
    priority: i32,
    deflection: f64,
    angle: f64,
) -> Result<(), Failure> {
    let references = (selection, styles);
    let criteria = (priority, deflection, angle);
    let context = ColoredSubshapeSelection::new(object, references, kind, criteria);
    catch_native_failure(|| compute_colored_subshape_selection(parent, shape, owner, &context))
}
fn compute_colored_shape_selection<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    selection: &dyn HandleReference<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    let source = object.shape_ref();
    if source.read_current_shape(|shape| Ok(shape.is_null()))? {
        return Ok(());
    }
    if object.is_shape_entirely_visible()? {
        return ais_shape_compute_selection(object, selection, mode);
    }
    compute_hidden_colored_selection(object, selection, mode)
}
fn compute_hidden_colored_selection<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    selection: &dyn HandleReference<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    let kind = ais_shape_selection_type(mode);
    let criteria = prepare_colored_selection_geometry(object, kind)?;
    let styles = AisShapeDrawerMap::default();
    object.fill_subshape_drawer_map(&styles)?;
    let context = ColoredSubshapeSelection::new(object, (selection, &styles), kind, criteria);
    begin_colored_subshape_selection(&context)?;
    let selectable = object.presentable_base().this().cast();
    crate::std_select::brep_selection_load::set_brep_selection_origin(
        &selection.observe(),
        &selectable,
    );
    Ok(())
}
fn prepare_colored_selection_geometry<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    kind: ShapeEnum,
) -> Result<(i32, f64, f64), Failure> {
    let source = object.shape_ref();
    let drawer = object.attributes();
    let deflection = source
        .read_current_shape(|shape| StdPrsToolTriangulatedShape::get_deflection(shape, &drawer))?;
    let angle = drawer.deviation_angle();
    let priority = source.read_current_shape(|shape| {
        StdSelectBrepSelectionTool::get_standard_priority(shape, kind)
    })?;
    prepare_colored_selection_mesh(object, deflection, angle)?;
    Ok((priority, deflection, angle))
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
    selection: &'a dyn HandleReference<SelectMgrSelection>,
    styles: &'a AisShapeDrawerMap,
    kind: ShapeEnum,
    priority: i32,
    deflection: f64,
    angle: f64,
}
type ColoredSelectionReferences<'a> = (
    &'a dyn HandleReference<SelectMgrSelection>,
    &'a AisShapeDrawerMap,
);
impl<'a, S: AisColoredShapeSource + ?Sized> ColoredSubshapeSelection<'a, S> {
    fn new(
        object: &'a S,
        references: ColoredSelectionReferences<'a>,
        kind: ShapeEnum,
        criteria: (i32, f64, f64),
    ) -> Self {
        let (selection, styles) = references;
        let (priority, deflection, angle) = criteria;
        Self {
            object,
            selection,
            styles,
            kind,
            priority,
            deflection,
            angle,
        }
    }
}
fn begin_colored_subshape_selection<S: AisColoredShapeSource + ?Sized>(
    context: &ColoredSubshapeSelection<'_, S>,
) -> Result<(), Failure> {
    let source = context.object.shape_ref();
    let mut owner = new_colored_root_owner(context.object, context.priority)?;
    if context.kind == ShapeEnum::Shape {
        // Native constructs the root owner twice for mode zero; do not collapse its retain/release sequence.
        owner = new_colored_root_owner(context.object, context.priority)?;
    }
    let base = source.read_current_shape(|shape| {
        super::colored_shape_compute::colored_shape_base_drawer(context.object, shape)
    })?;
    source.read_current_shape(|shape| {
        compute_colored_subshape_selection(&base, shape, &owner, context)
    })
}
fn new_colored_root_owner<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    priority: i32,
) -> Result<Handle<StdSelectBRepOwner>, Failure> {
    object
        .shape_ref()
        .read_current_shape(|shape| Ok(StdSelectBRepOwner::with_shape(shape, priority, false)))
}

fn compute_colored_subshape_selection<S: AisColoredShapeSource + ?Sized>(
    parent: &Handle<AisColoredDrawer>,
    shape: &Shape,
    owner: &dyn HandleReference<StdSelectBRepOwner>,
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
        return build_colored_decomposed_sensitive(shape, context);
    }
    select_colored_children(&drawer, shape, owner, context)
}
fn build_colored_decomposed_sensitive<S: AisColoredShapeSource + ?Sized>(
    shape: &Shape,
    context: &ColoredSubshapeSelection<'_, S>,
) -> Result<(), Failure> {
    let decomposed = !context.object.shape_ref().is_equal(shape);
    let owner = StdSelectBRepOwner::with_shape(shape, context.priority, decomposed);
    build_colored_sensitive(shape, &owner, context)
}
fn select_colored_children<S: AisColoredShapeSource + ?Sized>(
    drawer: &Handle<AisColoredDrawer>,
    shape: &Shape,
    owner: &dyn HandleReference<StdSelectBRepOwner>,
    context: &ColoredSubshapeSelection<'_, S>,
) -> Result<(), Failure> {
    let mut children = ShapeIterator::new(shape, true, true);
    while children.more() {
        compute_colored_subshape_selection(drawer, children.value()?, owner, context)?;
        children.next();
    }
    Ok(())
}

fn build_colored_sensitive<S: AisColoredShapeSource + ?Sized>(
    shape: &Shape,
    owner: &dyn HandleReference<StdSelectBRepOwner>,
    context: &ColoredSubshapeSelection<'_, S>,
) -> Result<(), Failure> {
    let owner = rustycad_tkernel::standard::ProjectedHandleReference::new(owner);
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
