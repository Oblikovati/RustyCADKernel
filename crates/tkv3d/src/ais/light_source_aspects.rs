//! Native constructor aspects and enabled/disabled style updates.
use super::{AisLightSourceMembers, AisLightSourceSource};
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, prs3d::*, select3d::Select3DSensitiveSphere};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Direction3, Transform3};
use rustycad_tkservice::{aspect::*, graphic3d::*};
#[derive(Default)]
pub(super) struct LightSourceVisual {
    pub disabled_marker: Handle<Graphic3dAspectMarker3d>,
    pub arrow_shadow: Handle<Graphic3dAspectLine3d>,
    pub sensitive_sphere: Handle<Select3DSensitiveSphere>,
    pub start_transform: Transform3,
}
/// Initialize native aspects for concrete or derived lights; `ais_light_source_initialize(&object)?`.
pub fn ais_light_source_initialize<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    let color = object.light().color();
    object.set_infinite_state(true);
    let drawer = object.attributes();
    initialize_light_markers(object, color)?;
    initialize_light_arrow(object, color)?;
    initialize_light_shading(&drawer, color)?;
    initialize_light_text(&drawer);
    super::ais_light_source_update_persistence(object)?;
    initialize_light_highlight(object);
    Ok(())
}
fn light_material(color: Color) -> Result<Graphic3dMaterialAspect, Failure> {
    let mut material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::UserDefined);
    material.set_color(color)?;
    Ok(material)
}
fn initialize_light_arrow<O: AisLightSourceSource + ?Sized>(
    object: &O,
    color: Color,
) -> Result<(), Failure> {
    let arrow = Handle::new(Prs3dArrowAspect::new()?);
    arrow.set_color(color);
    let aspect = arrow.aspect();
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    aspect.set_front_material(light_material(color)?);
    aspect.set_marker_type(AspectTypeOfMarker::Empty);
    aspect.set_marker_scale(2.0)?;
    object.attributes().set_arrow_aspect(arrow);
    initialize_light_shadow(object)
}
fn initialize_light_shadow<O: AisLightSourceSource + ?Sized>(object: &O) -> Result<(), Failure> {
    let width = if object.light().light_type() == Graphic3dTypeOfLightSource::Ambient {
        1.0
    } else {
        3.0
    };
    object.light_source_base().visual.write().arrow_shadow =
        Handle::new(Graphic3dAspectLine3d::new(
            Color::named(NamedColor::BLACK),
            AspectTypeOfLine::Solid,
            width,
        )?);
    Ok(())
}
fn initialize_light_shading(drawer: &Prs3dDrawer, color: Color) -> Result<(), Failure> {
    drawer.setup_own_shading_aspect(&Handle::default())?;
    let shading = drawer.shading_aspect();
    shading.set_color(color, AspectTypeOfFacingModel::BothSide)?;
    shading.set_material(&light_material(color)?, AspectTypeOfFacingModel::BothSide);
    shading.set_transparency(0.5, AspectTypeOfFacingModel::BothSide)?;
    shading
        .aspect()
        .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    Ok(())
}
fn initialize_light_text(drawer: &Prs3dDrawer) {
    let text = Handle::new(Prs3dTextAspect::new());
    text.aspect()
        .set_text_display_type(AspectTypeOfDisplayText::Shadow);
    text.aspect()
        .set_edge_color(Color::named(NamedColor::BLACK));
    text.set_horizontal_justification(Graphic3dHorizontalTextAlignment::Left);
    text.set_vertical_justification(Graphic3dVerticalTextAlignment::Topfirstline);
    drawer.set_text_aspect(text);
}
fn initialize_light_highlight<O: AisLightSourceSource + ?Sized>(object: &O) {
    let drawer = object.attributes();
    drawer.set_display_mode(0);
    let highlight = Handle::new(Prs3dDrawer::new());
    highlight.link_to(&drawer);
    highlight.set_display_mode(1);
    highlight.set_color(Color::named(NamedColor::CYAN1));
    object.presentable_base().fields.write().dyn_hilight_drawer = highlight.clone();
    initialize_light_overlay(object, &highlight);
}
fn initialize_light_overlay<O: AisLightSourceSource + ?Sized>(object: &O, highlight: &Prs3dDrawer) {
    let drawer = object.attributes();
    let persistence = object.presentable_base().read().transform_persistence;
    if persistence
        .get()
        .is_some_and(|pers| pers.is_trihedron_or_2d())
    {
        drawer.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOPMOST);
        highlight.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOPMOST);
        drawer
            .text_aspect()
            .set_horizontal_justification(Graphic3dHorizontalTextAlignment::Center);
        drawer
            .text_aspect()
            .set_vertical_justification(Graphic3dVerticalTextAlignment::Top);
    }
}
pub(super) fn refresh_light_aspects<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    let color = enabled_light_color(object)?;
    refresh_light_markers(object, color)?;
    let drawer = object.attributes();
    drawer
        .shading_aspect()
        .set_color(color, AspectTypeOfFacingModel::BothSide)?;
    drawer.arrow_aspect().set_color(color);
    drawer
        .arrow_aspect()
        .aspect()
        .change_front_material(|material| material.set_color(color))?;
    refresh_direction_marker(object)?;
    object.synchronize_aspects();
    Ok(())
}
fn enabled_light_color<O: AisLightSourceSource + ?Sized>(object: &O) -> Result<Color, Failure> {
    let light = object.light();
    if light.is_enabled() {
        return Ok(light.color());
    }
    Color::from_vec3(light.color().rgb_vec3() * 0.3)
}
fn refresh_light_markers<O: AisLightSourceSource + ?Sized>(
    object: &O,
    color: Color,
) -> Result<(), Failure> {
    let enabled = object.light().is_enabled();
    let point = object.attributes().point_aspect();
    point.set_color(color);
    point.aspect().set_marker_type(object.marker_type(enabled));
    point
        .aspect()
        .set_marker_image(object.marker_image(enabled));
    refresh_disabled_marker(object, color, point.aspect().marker_scale())
}
fn refresh_disabled_marker<O: AisLightSourceSource + ?Sized>(
    object: &O,
    color: Color,
    scale: f32,
) -> Result<(), Failure> {
    let enabled = object.light().is_enabled();
    let disabled = disabled_marker_handle(object);
    disabled.set_interior_color(color);
    disabled.set_marker_scale(scale)?;
    disabled.set_marker_type(if enabled {
        AspectTypeOfMarker::Empty
    } else {
        object.marker_type(false)
    });
    disabled.set_marker_image(object.marker_image(false));
    Ok(())
}
fn refresh_direction_marker<O: AisLightSourceSource + ?Sized>(object: &O) -> Result<(), Failure> {
    let light = object.light();
    if light.light_type() != Graphic3dTypeOfLightSource::Directional {
        return Ok(());
    }
    let marker = if light.is_enabled() && light.is_headlight() {
        light_direction_marker(light.direction()?)
    } else {
        AspectTypeOfMarker::Empty
    };
    object
        .attributes()
        .arrow_aspect()
        .aspect()
        .set_marker_type(marker);
    Ok(())
}
fn light_direction_marker(direction: Direction3) -> AspectTypeOfMarker {
    let tolerance = 2.0 * std::f64::consts::PI / 180.0;
    if !direction.is_parallel(Direction3::Z, tolerance) {
        return AspectTypeOfMarker::Empty;
    }
    if direction.is_opposite(Direction3::Z.reversed(), tolerance) {
        AspectTypeOfMarker::OPoint
    } else {
        AspectTypeOfMarker::X
    }
}

fn initialize_light_markers<O: AisLightSourceSource + ?Sized>(
    object: &O,
    color: Color,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    drawer.set_point_aspect(Handle::new(Prs3dPointAspect::new(
        object.marker_type(true),
        color,
        3.0,
    )?));
    object.light_source_base().visual.write().disabled_marker = Handle::new(
        Graphic3dAspectMarker3d::new(AspectTypeOfMarker::Empty, color, 3.0)?,
    );
    Ok(())
}

fn disabled_marker_handle<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Handle<Graphic3dAspectMarker3d> {
    object
        .light_source_base()
        .visual
        .read()
        .disabled_marker
        .clone()
}
