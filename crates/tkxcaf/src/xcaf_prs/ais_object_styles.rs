// Rust port of DataExchange/TKXCAF/XCAFPrs/XCAFPrs_AISObject.cxx (setStyleToDrawer, the style
// groups of DispatchStyles, SetMaterial and DisplayText) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The styles of the drawers of `XCAFPrs_AISObject`: the colors and material of a drawer
//! from a style, the custom drawers of the groups of sub-shapes of one style, the material
//! of the object, and the names drawn in the view name mode.
use super::ais_object::{XCAFPrsAISObject, front_material};
use super::{XCAFPrsStyle, XCAFPrsStyleMap};
use crate::xcaf_doc::XCAFDocShapeTool;
use rustycad_tkbrep::topods::{Builder, Compound, Shape, ShapeIterator};
use rustycad_tkernel::quantity::{Color, ColorRgba};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::tdata_std::{TDataStdGenericExtString, TDataStdName};
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsMembers, Graphic3dMaterialAspect, Graphic3dStructureSource,
};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tkv3d::ais::{AisColoredDrawer, AisShapeSource};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkv3d::prs3d::{Prs3dDrawer, Prs3dRoot, Prs3dText, Prs3dTextAspect};

/// The protected static `AIS_Shape::setMaterial(theDrawer, theMaterial, theToKeepColor,
/// theToKeepTransp)`: an own shading aspect with the material, keeping its color and
/// transparency on request.
pub(super) fn set_shape_material<S: AisShapeSource + ?Sized>(
    object: &S,
    drawer: &Handle<Prs3dDrawer>,
    material: &Graphic3dMaterialAspect,
    keep: (bool, bool),
) -> Result<(), Failure> {
    let own = rustycad_tklcaf::live(drawer, "AIS_Shape::setMaterial")?;
    own.setup_own_shading_aspect(&Handle::default())?;
    let facing = object.presentable_base().read().current_facing_model;
    let shading = own.shading_aspect();
    let shading = rustycad_tklcaf::live(&shading, "AIS_Shape::setMaterial")?;
    let (color, transparency) = (shading.color(facing), shading.transparency(facing));
    shading.set_material(material, facing);
    if keep.0 {
        shading.set_color(color, facing)?;
    }
    if keep.1 {
        shading.set_transparency(transparency, facing)?;
    }
    Ok(())
}

/// The surface and curve colors and the material of `setStyleToDrawer`: the default ones,
/// the visualization material's, then the style's colors.
fn style_colors(
    drawer: &Prs3dDrawer,
    style: &XCAFPrsStyle,
    default: &XCAFPrsStyle,
    material: &Graphic3dMaterialAspect,
) -> Result<(ColorRgba, Color, Graphic3dMaterialAspect), Failure> {
    let (mut surface, mut curve, mut material) = (
        default.get_color_surf_rgba(),
        default.get_color_curv(),
        material.clone(),
    );
    let vis_material = if style.material().is_null() {
        default.material()
    } else {
        style.material()
    };
    if let Some(vis) = vis_material.get().filter(|m| !m.is_empty()) {
        let aspect = rustycad_tklcaf::live(&drawer.shading_aspect(), "XCAFPrs_AISObject")?.aspect();
        vis.fill_aspect(&aspect.clone().cast())?;
        material = rustycad_tklcaf::live(&aspect, "XCAFPrs_AISObject")?.front_material();
        surface = ColorRgba::from_rgb(material.color(), material.alpha());
        curve = material.color();
    }
    if style.is_set_color_surf() {
        surface = style.get_color_surf_rgba();
        material.set_color(*surface.rgb())?;
        material.set_alpha(surface.alpha())?;
    }
    if style.is_set_color_curv() {
        curve = style.get_color_curv();
    }
    Ok((surface, curve, material))
}

/// The protected static `setStyleToDrawer(theDrawer, theStyle, theDefStyle, theDefMaterial)`:
/// own shading and line aspects with the colors and material of the style.
pub(super) fn set_style_to_drawer(
    drawer: &Handle<Prs3dDrawer>,
    style: &XCAFPrsStyle,
    default: &XCAFPrsStyle,
    material: &Graphic3dMaterialAspect,
) -> Result<(), Failure> {
    let own = rustycad_tklcaf::live(drawer, "XCAFPrs_AISObject::setStyleToDrawer")?;
    own.setup_own_shading_aspect(&Handle::default())?;
    own.set_own_line_aspects(&Handle::default())?;
    let (surface, curve, material) = style_colors(own, style, default, material)?;
    for line in [
        own.un_free_boundary_aspect(),
        own.free_boundary_aspect(),
        own.wire_aspect(),
    ] {
        rustycad_tklcaf::live(&line, "XCAFPrs_AISObject")?.set_color(curve);
    }
    let aspect = rustycad_tklcaf::live(&own.shading_aspect(), "XCAFPrs_AISObject")?.aspect();
    let aspect = rustycad_tklcaf::live(&aspect, "XCAFPrs_AISObject")?;
    aspect.set_interior_color_rgba(surface);
    aspect.set_front_material(material);
    for iso in [own.u_iso_aspect(), own.v_iso_aspect()] {
        rustycad_tklcaf::live(&iso, "XCAFPrs_AISObject")?
            .line_aspect()
            .set_color(*surface.rgb());
    }
    Ok(())
}

/// The groups of `DispatchStyles`: the shapes of each style in a compound, in the order of
/// their first shape.
fn style_groups(settings: &XCAFPrsStyleMap) -> Result<Vec<(XCAFPrsStyle, Shape)>, Failure> {
    let mut groups: Vec<(XCAFPrsStyle, Shape)> = Vec::new();
    for index in 1..=settings.extent() {
        let (shape, style) = (settings.find_key(index)?, settings.find_from_index(index)?);
        if let Some((_, compound)) = groups.iter_mut().find(|(known, _)| known == style) {
            Builder.add(compound, shape)?;
            continue;
        }
        let mut compound = Compound::default();
        Builder.make_compound(&mut compound);
        let mut compound = compound.into_shape();
        Builder.add(&mut compound, shape)?;
        groups.push((style.clone(), compound));
    }
    Ok(groups)
}

/// The only shape of a compound, else the compound.
fn group_shape(compound: &Shape) -> Result<Shape, Failure> {
    let mut iterator = ShapeIterator::new(compound, true, true);
    let first = iterator.value()?.clone();
    iterator.next();
    Ok(if iterator.more() {
        compound.clone()
    } else {
        first
    })
}

/// Binds one custom drawer per style group, with the visibility, ownership flags and
/// colors of its style.
pub(super) fn bind_style_groups(
    object: &XCAFPrsAISObject,
    settings: &XCAFPrsStyleMap,
    default: &XCAFPrsStyle,
) -> Result<(), Failure> {
    let parent = object.attributes();
    for (style, compound) in style_groups(settings)? {
        let drawer = Handle::new(AisColoredDrawer::new(&parent));
        object
            .colored_shape_base_map()
            .bind(&group_shape(&compound)?, &drawer)?;
        object.remember_drawer(&drawer);
        let own = rustycad_tklcaf::live(&drawer, "XCAFPrs_AISObject::DispatchStyles")?;
        own.set_hidden(!style.is_visible());
        if style.material().get().is_some_and(|m| !m.is_empty()) {
            own.set_own_material();
        }
        if style.is_set_color_surf() || style.is_set_color_curv() {
            own.set_own_color(Color::default());
        }
        set_style_to_drawer(
            &drawer.clone().cast(),
            &style,
            default,
            &front_material(&parent),
        )?;
    }
    Ok(())
}

/// `SetMaterial(theMaterial)` of `XCAFPrs_AISObject`.
pub(super) fn set_object_material(
    object: &XCAFPrsAISObject,
    material: &Graphic3dMaterialAspect,
) -> Result<(), Failure> {
    let default = object.default_style();
    let drawer = object.attributes();
    set_shape_material(
        object,
        &drawer,
        material,
        (object.has_color(), object.is_transparent()),
    )?;
    set_style_to_drawer(&drawer, &default, &default, &front_material(&drawer))?;
    for custom in object.dispatched_drawers() {
        let Some(own) = custom.get().filter(|d| !d.has_own_material()) else {
            continue;
        };
        if !own.has_own_shading_aspect() {
            continue;
        }
        let aspect =
            rustycad_tklcaf::live(&own.shading_aspect(), "XCAFPrs_AISObject::SetMaterial")?
                .aspect();
        let aspect = rustycad_tklcaf::live(&aspect, "XCAFPrs_AISObject::SetMaterial")?;
        let surface = aspect.interior_color_rgba();
        let mut own_material = front_material(&drawer);
        own_material.set_color(*surface.rgb())?;
        own_material.set_alpha(surface.alpha())?;
        aspect.set_interior_color_rgba(surface);
        aspect.set_front_material(own_material);
    }
    object.synchronize_aspects();
    Ok(())
}

/// The file-static `DisplayText`: the name of the label at the center of the bounds of its
/// moved shape, then those of its sub-shapes, components and referred shapes.
pub(super) fn display_text(
    label: &TdfLabel,
    prs: &Handle<dyn Graphic3dStructureSource>,
    aspect: &Handle<Prs3dTextAspect>,
    location: &Location,
) -> Result<(), Failure> {
    draw_name(label, prs, aspect, location)?;
    let mut sub_shapes = Vec::new();
    if XCAFDocShapeTool::get_sub_shapes(label, &mut sub_shapes)? {
        for sub in &sub_shapes {
            display_text(sub, prs, aspect, location)?; // suppose that subshapes do not contain locations
        }
    }
    let mut components = Vec::new();
    if XCAFDocShapeTool::get_components(label, &mut components, false)? {
        for component in &components {
            display_text(component, prs, aspect, location)?;
            let component_location = XCAFDocShapeTool::get_location(component)?;
            let mut referred = TdfLabel::default();
            if XCAFDocShapeTool::get_referred_shape(component, &mut referred)? {
                display_text(&referred, prs, aspect, &component_location)?;
            }
        }
    }
    Ok(())
}

/// The name of one label at the center of the bounds of its moved shape, when it has both.
fn draw_name(
    label: &TdfLabel,
    prs: &Handle<dyn Graphic3dStructureSource>,
    aspect: &Handle<Prs3dTextAspect>,
    location: &Location,
) -> Result<(), Failure> {
    let mut name: Handle<TDataStdName> = Handle::default();
    if !label.find_attribute_as(&TDataStdName::get_id(), &mut name)? {
        return Ok(());
    }
    let mut shape = Shape::default();
    if !XCAFDocShapeTool::get_shape(label, &mut shape)? {
        return Ok(());
    }
    shape.move_by(location, true)?;
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add(&shape, &mut bounds, true)?;
    if bounds.is_void() {
        return Ok(());
    }
    let [x_min, y_min, z_min, x_max, y_max, z_max] = bounds.get()?;
    let center = Point3::new(
        0.5 * (x_min + x_max),
        0.5 * (y_min + y_max),
        0.5 * (z_min + z_max),
    );
    let text = rustycad_tklcaf::live(&name, "XCAFPrs_AISObject")?.get();
    Prs3dText::draw(&Prs3dRoot::current_group(prs)?, aspect, &text, &center);
    Ok(())
}
