//! Native fill factories differ in their null/missing-input preconditions and material source.
use super::{
    MeshVSTool,
    values::{DrawerValues, native_enum, required_value},
};
use crate::{MeshVSDrawer, MeshVSDrawerAttribute as Key, MeshVSDrawerSource};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, HandleReference, exceptions::program_error},
};
use rustycad_tkservice::{
    aspect::{AspectHatchStyle, AspectInteriorStyle, AspectTypeOfLine},
    graphic3d::{
        Graphic3dAspectFillArea3d, Graphic3dAspectsEdgeMembers, Graphic3dAspectsMembers,
        Graphic3dMaterialAspect, Graphic3dNameOfMaterial,
    },
};
struct FillValues {
    interior: Color,
    back: Color,
    edge: Color,
    width: f64,
    style: i32,
    line: i32,
    hatch: i32,
}
impl MeshVSTool {
    /// Native explicit-material fill factory; `Self::create_aspect_fill_area3d_with_material(&drawer,&material,true)?`.
    pub fn create_aspect_fill_area3d_with_material(
        drawer: &dyn HandleReference<dyn MeshVSDrawerSource>,
        material: &Graphic3dMaterialAspect,
        use_defaults: bool,
    ) -> Result<Handle<Graphic3dAspectFillArea3d>, Failure> {
        if drawer.is_null() {
            return Ok(Handle::default());
        }
        let Some(values) = fill_values(&DrawerValues::new(drawer, use_defaults))? else {
            return Ok(Handle::default());
        };
        Ok(Handle::new(make_fill(values, material)?))
    }
    /// Native drawer-material fill factory; `Self::create_aspect_fill_area3d(&drawer,true)?`.
    /// Null drawer and late missing inputs violate native dereference preconditions.
    pub fn create_aspect_fill_area3d(
        drawer: &dyn HandleReference<dyn MeshVSDrawerSource>,
        use_defaults: bool,
    ) -> Result<Handle<Graphic3dAspectFillArea3d>, Failure> {
        let values = DrawerValues::new(drawer, use_defaults);
        let Some((front, back)) = drawer_materials(&values)? else {
            return Ok(Handle::default());
        };
        let result = Self::create_aspect_fill_area3d_with_material(drawer, &front, use_defaults)?;
        let aspect = result.get().ok_or_else(|| {
            program_error("MeshVS_Tool fill=null: expected all fill keys after both material keys")
        })?;
        aspect.set_back_material(back);
        Ok(result)
    }
}
fn fill_values(values: &DrawerValues<'_>) -> Result<Option<FillValues>, Failure> {
    let interior = required_value!(values.read(
        Key::InteriorColor,
        Color::named(NamedColor::CYAN1),
        MeshVSDrawer::get_color
    ));
    let back =
        required_value!(values.read(Key::BackInteriorColor, interior, MeshVSDrawer::get_color));
    let edge = required_value!(values.read(
        Key::EdgeColor,
        Color::named(NamedColor::WHITE),
        MeshVSDrawer::get_color
    ));
    fill_style_values(values, fill_parameters(interior, back, edge))
}
fn fill_style_values(
    values: &DrawerValues<'_>,
    mut fill: FillValues,
) -> Result<Option<FillValues>, Failure> {
    fill.width = required_value!(values.read(Key::EdgeWidth, 1., MeshVSDrawer::get_double));
    fill.style = required_value!(values.read(Key::InteriorStyle, -1, MeshVSDrawer::get_integer));
    fill.line = required_value!(values.read(Key::EdgeType, 0, MeshVSDrawer::get_integer));
    fill.hatch = required_value!(values.read(Key::HatchStyle, 7, MeshVSDrawer::get_integer));
    Ok(Some(fill))
}
fn make_fill(
    values: FillValues,
    material: &Graphic3dMaterialAspect,
) -> Result<Graphic3dAspectFillArea3d, Failure> {
    let style = interior_style(values.style)?;
    let line = line_type(values.line)?;
    let hatch = hatch_type(values.hatch)?;
    let result = construct_fill(&values, material, style, line)?;
    result.set_back_interior_color(values.back);
    result.set_hatch_style_type(hatch);
    Ok(result)
}
fn drawer_materials(
    values: &DrawerValues<'_>,
) -> Result<Option<(Graphic3dMaterialAspect, Graphic3dMaterialAspect)>, Failure> {
    let front = required_value!(values.read(Key::FrontMaterial, 0, MeshVSDrawer::get_integer));
    let front = material_from_integer(front)?;
    let back = required_value!(values.read(Key::BackMaterial, 0, MeshVSDrawer::get_integer));
    Ok(Some((front, material_from_integer(back)?)))
}
fn material_from_integer(value: i32) -> Result<Graphic3dMaterialAspect, Failure> {
    let name = MATERIALS.get(value as usize).copied().ok_or_else(|| {
        program_error(&format!(
            "MeshVS_Tool material={value}: expected native material table index 0..=25"
        ))
    })?;
    Ok(Graphic3dMaterialAspect::new(name))
}
const MATERIALS: &[Graphic3dNameOfMaterial] = &[
    Graphic3dNameOfMaterial::Brass,
    Graphic3dNameOfMaterial::Bronze,
    Graphic3dNameOfMaterial::Copper,
    Graphic3dNameOfMaterial::Gold,
    Graphic3dNameOfMaterial::Pewter,
    Graphic3dNameOfMaterial::Plastered,
    Graphic3dNameOfMaterial::Plastified,
    Graphic3dNameOfMaterial::Silver,
    Graphic3dNameOfMaterial::Steel,
    Graphic3dNameOfMaterial::Stone,
    Graphic3dNameOfMaterial::ShinyPlastified,
    Graphic3dNameOfMaterial::Satin,
    Graphic3dNameOfMaterial::Metalized,
    Graphic3dNameOfMaterial::Ionized,
    Graphic3dNameOfMaterial::Chrome,
    Graphic3dNameOfMaterial::Aluminum,
    Graphic3dNameOfMaterial::Obsidian,
    Graphic3dNameOfMaterial::Neon,
    Graphic3dNameOfMaterial::Jade,
    Graphic3dNameOfMaterial::Charcoal,
    Graphic3dNameOfMaterial::Water,
    Graphic3dNameOfMaterial::Glass,
    Graphic3dNameOfMaterial::Diamond,
    Graphic3dNameOfMaterial::Transparent,
    Graphic3dNameOfMaterial::Default,
    Graphic3dNameOfMaterial::UserDefined,
];

fn fill_parameters(interior: Color, back: Color, edge: Color) -> FillValues {
    FillValues {
        interior,
        back,
        edge,
        width: 1.,
        style: -1,
        line: 0,
        hatch: 7,
    }
}
fn interior_style(value: i32) -> Result<AspectInteriorStyle, Failure> {
    native_enum(
        value,
        AspectInteriorStyle::from_native_value(value),
        "Aspect_InteriorStyle",
    )
}
fn line_type(value: i32) -> Result<AspectTypeOfLine, Failure> {
    native_enum(
        value,
        AspectTypeOfLine::from_native_value(value),
        "Aspect_TypeOfLine",
    )
}
fn hatch_type(value: i32) -> Result<AspectHatchStyle, Failure> {
    native_enum(
        value,
        AspectHatchStyle::from_native_value(value),
        "Aspect_HatchStyle",
    )
}
fn construct_fill(
    values: &FillValues,
    material: &Graphic3dMaterialAspect,
    style: AspectInteriorStyle,
    line: AspectTypeOfLine,
) -> Result<Graphic3dAspectFillArea3d, Failure> {
    Graphic3dAspectFillArea3d::new(
        style,
        values.interior,
        values.edge,
        line,
        values.width,
        material.clone(),
        material.clone(),
    )
}
