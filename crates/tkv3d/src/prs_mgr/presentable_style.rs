//! The native bodies of the `PrsMgr_PresentableObject` virtual members for the bounding box,
//! the material, the transparency, the polygon offsets and `UnsetAttributes`.
use super::{PrsMgrPresentableObject, PrsMgrPresentation};
use crate::prs3d::{Prs3dDrawer, Prs3dShadingAspect};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkservice::aspect::AspectTypeOfFacingModel;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsMembers as _, Graphic3dAspectsRenderMembers as _, Graphic3dMaterialAspect,
    Graphic3dNameOfMaterial, Graphic3dPresentationAttributesMembers as _,
    Graphic3dPresentationAttributesSource as _, Graphic3dStructureMembers as _,
};

/// Copies the driver box of a presentation into `bounds`, or makes `bounds` void when the
/// driver box is invalid.
fn presentation_box(prs: &Handle<PrsMgrPresentation>, bounds: &mut BoundingBox3) {
    let Some(cstructure) = prs.get().map(|p| p.cstructure()).filter(|c| !c.is_null()) else {
        return;
    };
    let bvh = cstructure.cstructure_base().fields.read().bnd_box;
    if !bvh.is_valid() {
        bounds.set_void();
        return;
    }
    let (min, max) = (bvh.corner_min(), bvh.corner_max());
    bounds.update_bounds([min.x, min.y, min.z], [max.x, max.y, max.z]);
}

/// `PrsMgr_PresentableObject::BoundingBox`; `prs_mgr_bounding_box(&*object, &mut bounds)`.
pub fn prs_mgr_bounding_box<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    bounds: &mut BoundingBox3,
) {
    let fields = object.presentable_base().read();
    let mode = fields.drawer.display_mode();
    if mode != -1 {
        if let Some(prs) = fields
            .presentations
            .iter()
            .find(|p| p.get().is_some_and(|v| v.mode() == mode))
        {
            presentation_box(prs, bounds);
        }
        return;
    }
    if let Some(first) = fields.presentations.first() {
        presentation_box(first, bounds);
        return;
    }
    for child in fields.children.iter().filter_map(Handle::get) {
        let mut child_box = BoundingBox3::default();
        child.bounding_box(&mut child_box);
        bounds.add_box(&child_box);
    }
}

/// The shading aspect of the drawer (own or linked).
fn shading_of(drawer: &Handle<Prs3dDrawer>) -> Handle<Prs3dShadingAspect> {
    drawer
        .get()
        .map(Prs3dDrawer::shading_aspect)
        .unwrap_or_default()
}

/// `PrsMgr_PresentableObject::Material`: the name of the front material; a drawer without
/// shading aspect gives the default material name. Example: `prs_mgr_material(&*object)`.
pub fn prs_mgr_material<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
) -> Graphic3dNameOfMaterial {
    let shading = shading_of(&object.presentable_base().read().drawer);
    let material = shading
        .get()
        .map(|s| s.material(AspectTypeOfFacingModel::FrontSide))
        .unwrap_or_default();
    material.name()
}

/// `PrsMgr_PresentableObject::SetMaterial`; `prs_mgr_set_material(&*object, &material)?`.
pub fn prs_mgr_set_material<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    material: &Graphic3dMaterialAspect,
) -> Result<(), Failure> {
    let drawer = object.presentable_base().read().drawer;
    drawer.setup_own_shading_aspect(&Handle::default())?;
    if let Some(shading) = shading_of(&drawer).get() {
        shading.set_material(material, AspectTypeOfFacingModel::BothSide);
    }
    object.presentable_base().fields.write().has_own_material = true;
    Ok(())
}

/// The own color and transparency flags of an object.
fn color_and_transparency<O: PrsMgrPresentableObject + ?Sized>(object: &O) -> (bool, bool) {
    let fields = object.presentable_base().read();
    (fields.has_own_color, fields.drawer.transparency() > 0.005)
}

/// `PrsMgr_PresentableObject::UnsetMaterial`: restores the link material and re-applies the
/// own color and transparency, or forgets the own shading aspect.
/// Example: `prs_mgr_unset_material(&*object)?`.
pub fn prs_mgr_unset_material<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    let fields = object.presentable_base().read();
    if !fields.has_own_material {
        return Ok(());
    }
    let (has_color, is_transparent) = color_and_transparency(object);
    let drawer = fields.drawer;
    if !(has_color || is_transparent) {
        drawer.set_shading_aspect(Handle::default());
        object.presentable_base().fields.write().has_own_material = false;
        return Ok(());
    }
    let link_shading = shading_of(&drawer.link());
    if let (Some(own), Some(link)) = (
        shading_of(&drawer).get(),
        link_shading.get().filter(|_| drawer.has_link()),
    ) {
        own.set_material(
            &link.aspect().back_material(),
            AspectTypeOfFacingModel::BothSide,
        );
    }
    if has_color {
        object.set_color(drawer.color())?;
    }
    // The native code asks IsTransparent() again after the (virtual) SetColor.
    if color_and_transparency(object).1 {
        object.set_transparency(f64::from(drawer.transparency()))?;
    }
    object.presentable_base().fields.write().has_own_material = false;
    Ok(())
}

/// Sets the transparency of both materials of the drawer shading aspect.
fn set_materials_transparency(drawer: &Handle<Prs3dDrawer>, value: f32) -> Result<(), Failure> {
    let Some(shading) = shading_of(drawer).get().map(Prs3dShadingAspect::aspect) else {
        return Ok(());
    };
    shading.change_front_material(|m| m.set_transparency(value))?;
    shading.change_back_material(|m| m.set_transparency(value))
}

/// `PrsMgr_PresentableObject::SetTransparency`; a value outside [0, 1] raises the
/// `Standard_OutOfRange` of the material. Example: `prs_mgr_set_transparency(&*object, 0.6)?`.
pub fn prs_mgr_set_transparency<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    value: f64,
) -> Result<(), Failure> {
    let drawer = object.presentable_base().read().drawer;
    drawer.setup_own_shading_aspect(&Handle::default())?;
    set_materials_transparency(&drawer, value as f32)?;
    drawer.set_transparency(value as f32);
    Ok(())
}

/// `PrsMgr_PresentableObject::UnsetTransparency`; `prs_mgr_unset_transparency(&*object)?`.
pub fn prs_mgr_unset_transparency<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    let fields = object.presentable_base().read();
    if fields.has_own_color || fields.has_own_material {
        set_materials_transparency(&fields.drawer, 0.0)?;
    } else {
        fields.drawer.set_shading_aspect(Handle::default());
    }
    fields.drawer.set_transparency(0.0);
    Ok(())
}

/// `PrsMgr_PresentableObject::HasPolygonOffsets`: false with an own shading aspect or the
/// link's one. Example: `prs_mgr_has_polygon_offsets(&*object)`.
pub fn prs_mgr_has_polygon_offsets<O: PrsMgrPresentableObject + ?Sized>(object: &O) -> bool {
    let drawer = object.presentable_base().read().drawer;
    let shares_link = drawer.has_link() && shading_of(&drawer) == shading_of(&drawer.link());
    !(drawer.has_own_shading_aspect() || shares_link)
}

/// `PrsMgr_PresentableObject::PolygonOffsets`; `prs_mgr_polygon_offsets(&*object)`.
pub fn prs_mgr_polygon_offsets<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
) -> Option<(i32, f32, f32)> {
    if !object.has_polygon_offsets() {
        return None;
    }
    let drawer = object.presentable_base().read().drawer;
    shading_of(&drawer)
        .get()
        .map(|s| s.aspect().polygon_offsets())
}

/// `PrsMgr_PresentableObject::SetPolygonOffsets`; `prs_mgr_set_polygon_offsets(&*object, 1, 1.0, 0.0)?`.
pub fn prs_mgr_set_polygon_offsets<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    mode: i32,
    factor: f32,
    units: f32,
) -> Result<(), Failure> {
    let drawer = object.presentable_base().read().drawer;
    drawer.setup_own_shading_aspect(&Handle::default())?;
    if let Some(shading) = shading_of(&drawer).get() {
        shading.aspect().set_polygon_offsets(mode, factor, units);
    }
    super::PrsMgrPresentableObjectMembers::synchronize_aspects(object);
    Ok(())
}

/// `PrsMgr_PresentableObject::UnsetAttributes`: a new drawer keeping the link, no own color,
/// material or width, no transparency. Example: `prs_mgr_unset_attributes(&*object)`.
pub fn prs_mgr_unset_attributes<O: PrsMgrPresentableObject + ?Sized>(object: &O) {
    let old = object.presentable_base().read().drawer;
    let drawer = Prs3dDrawer::new();
    if old.has_link() {
        drawer.set_link(&old.link());
    }
    drawer.set_transparency(0.0);
    let mut fields = object.presentable_base().fields.write();
    fields.drawer = Handle::new(drawer);
    (
        fields.has_own_color,
        fields.has_own_material,
        fields.own_width,
    ) = (false, false, 0.0);
}
