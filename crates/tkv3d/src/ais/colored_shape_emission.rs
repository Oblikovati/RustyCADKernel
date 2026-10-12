//! Native per-style emission keeps shared shaded groups and restores deflection only after successful rendering.
use super::{
    AisColoredCompoundRef, AisColoredDrawerCompounds, AisColoredOpenedCompounds,
    AisColoredShapeSource, colored_compound_cursor::ColoredCompoundNativeCursor,
};
use crate::{
    prs_mgr::PrsMgrPresentableObjectMembers,
    prs3d::Prs3dDrawer,
    std_prs::{StdPrsShadedShape, StdPrsWfShape},
};
use rustycad_tkbrep::topods::TopoDSShapeReadSource;
use rustycad_tkernel::standard::{Failure, Handle, HandleReference, catch_native_failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkservice::{aspect::AspectTypeOfDeflection, graphic3d::*};
pub(super) struct ColoredShapeEmission<'a, S: AisColoredShapeSource + ?Sized> {
    pub object: &'a S,
    pub prs: &'a dyn HandleReference<dyn Graphic3dStructureSource>,
    pub mode: i32,
    pub opened: Handle<dyn Graphic3dGroup>,
    pub closed: Handle<dyn Graphic3dGroup>,
    pub boundary: Handle<dyn Graphic3dGroup>,
}
/// Emit native grouped custom topology without replacing map ownership; `ais_colored_shape_add_custom_props(&object,&prs,&opened,&closed,1)?`.
pub fn ais_colored_shape_add_custom_props<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
    opened: &AisColoredOpenedCompounds,
    closed: &AisColoredDrawerCompounds,
    mode: i32,
) -> Result<(), Failure> {
    let mut emission = ColoredShapeEmission {
        object,
        prs,
        mode,
        opened: Default::default(),
        closed: Default::default(),
        boundary: Default::default(),
    };
    catch_native_failure(|| emit_colored_shape_maps(opened, closed, &mut emission))
}
fn emit_colored_shape_maps<S: AisColoredShapeSource + ?Sized>(
    opened: &AisColoredOpenedCompounds,
    closed: &AisColoredDrawerCompounds,
    emission: &mut ColoredShapeEmission<'_, S>,
) -> Result<(), Failure> {
    for (index, map) in opened.iter().chain(std::iter::once(closed)).enumerate() {
        let mut cursor = ColoredCompoundNativeCursor::new(map);
        while let Some(reference) = cursor.current()? {
            emit_colored_shape_entry(&reference, index == 8, emission)?;
            cursor.next();
        }
    }
    Ok(())
}
fn colored_entry_drawer<S: AisColoredShapeSource + ?Sized>(
    reference: &AisColoredCompoundRef<'_>,
    object: &S,
) -> Result<Option<Handle<Prs3dDrawer>>, Failure> {
    let node = reference.live_node()?;
    if node.drawer.is_null() {
        return Ok(Some(object.attributes()));
    }
    let custom = node.drawer.observe();
    if custom.is_hidden() {
        return Ok(None);
    }
    Ok(Some(custom.cast()))
}
fn emit_colored_shape_entry<S: AisColoredShapeSource + ?Sized>(
    reference: &AisColoredCompoundRef<'_>,
    closed: bool,
    emission: &mut ColoredShapeEmission<'_, S>,
) -> Result<(), Failure> {
    let Some(drawer) = colored_entry_drawer(reference, emission.object)? else {
        return Ok(());
    };
    let previous = drawer.type_of_deflection();
    drawer.set_type_of_deflection(AspectTypeOfDeflection::Absolute);
    emit_colored_shape_geometry(reference, &drawer, closed, emission)?;
    // OCCT 3d097a0 intentionally leaves Absolute on any failure; a cleanup guard would change behavior.
    drawer.set_type_of_deflection(previous);
    Ok(())
}
fn emit_colored_shape_geometry<S: AisColoredShapeSource + ?Sized>(
    reference: &AisColoredCompoundRef<'_>,
    drawer: &Handle<Prs3dDrawer>,
    closed: bool,
    emission: &mut ColoredShapeEmission<'_, S>,
) -> Result<(), Failure> {
    let kind = reference.read_current_shape(|shape| shape.shape_type())?;
    if emission.mode != 1 || kind > ShapeEnum::Face || emission.object.is_infinite() {
        return StdPrsWfShape::add_reference(emission.prs, reference, drawer);
    }
    StdPrsShadedShape::add_free_elements_reference(emission.prs, reference, drawer)?;
    StdPrsShadedShape::add_unmeshed_faces_reference(emission.prs, reference, drawer)?;
    super::colored_shape_emission_shading::emit_colored_shading(reference, drawer, closed, emission)
}
