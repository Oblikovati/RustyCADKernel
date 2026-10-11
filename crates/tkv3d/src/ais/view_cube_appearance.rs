//! Display-only native appearance; the complete constructor's animation and owners are deferred.
use super::AisInteractiveObjectBase;
use crate::prs3d::{Prs3dDrawer, Prs3dShadingAspect, Prs3dTextAspect};
use rustycad_tkernel::{
    ncollection::Vec2,
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{aspect::*, graphic3d::*};
pub(super) fn initialize_cube_appearance(
    interactive: &AisInteractiveObjectBase,
) -> Result<(Handle<Prs3dShadingAspect>, Handle<Prs3dShadingAspect>), Failure> {
    let drawer = interactive.presentable.fields.read().drawer.clone();
    drawer.set_text_aspect(Handle::new(Prs3dTextAspect::new()));
    drawer.set_shading_aspect(Handle::new(Prs3dShadingAspect::new()?));
    initialize_cube_text(&drawer);
    initialize_cube_fill(&drawer)?;
    initialize_cube_persistence(interactive)?;
    let edge = cube_secondary_aspect(&drawer)?;
    let corner = cube_secondary_aspect(&drawer)?;
    Ok((edge, corner))
}
fn initialize_cube_text(drawer: &Prs3dDrawer) {
    let text = drawer.text_aspect();
    text.set_horizontal_justification(Graphic3dHorizontalTextAlignment::Center);
    text.set_vertical_justification(Graphic3dVerticalTextAlignment::Center);
    text.set_color(Color::named(NamedColor::BLACK));
    text.set_font("sans-serif");
    text.set_height(70.0 * 0.16);
    text.aspect().set_text_zoomable(true);
    text.aspect()
        .set_face_culling(Graphic3dTypeOfBackfacingModel::BackCulled);
}
fn initialize_cube_fill(drawer: &Prs3dDrawer) -> Result<(), Failure> {
    let mut material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::UserDefined);
    material.set_color(Color::named(NamedColor::WHITE))?;
    material.set_ambient_color(Color::named(NamedColor::GRAY60));
    let fill = drawer.shading_aspect().aspect();
    fill.set_interior_style(AspectInteriorStyle::Solid);
    fill.set_face_culling(Graphic3dTypeOfBackfacingModel::BackCulled);
    fill.set_interior_color(material.color());
    fill.set_front_material(material);
    drawer.set_face_boundary_draw(false);
    Ok(())
}
fn cube_secondary_aspect(drawer: &Prs3dDrawer) -> Result<Handle<Prs3dShadingAspect>, Failure> {
    let aspect = Handle::new(Prs3dShadingAspect::new()?);
    *aspect.aspect().aspects_base().fields.write() = drawer
        .shading_aspect()
        .aspect()
        .aspects_base()
        .fields
        .read()
        .clone();
    aspect.set_color(
        Color::named(NamedColor::GRAY30),
        AspectTypeOfFacingModel::BothSide,
    )?;
    Ok(aspect)
}
fn initialize_cube_persistence(interactive: &AisInteractiveObjectBase) -> Result<(), Failure> {
    let persistence = Graphic3dTransformPers::with_corner(
        Graphic3dTransModeFlags::TRIEDRON_PERS,
        AspectTypeOfTriedronPosition::LEFT_LOWER,
        Vec2::new(100, 100),
    )?;
    let mut state = interactive.presentable.fields.write();
    state.infinite_state = true;
    state.is_mutable = true;
    state.drawer.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOPMOST);
    state.transform_persistence = Handle::new(persistence).cast();
    Ok(())
}
