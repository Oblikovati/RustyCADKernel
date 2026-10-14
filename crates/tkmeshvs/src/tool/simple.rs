//! Native line and marker factory lookup order precedes constructor validation.
use super::{
    MeshVSTool,
    values::{DrawerValues, native_enum, required_value},
};
use crate::{MeshVSDrawer, MeshVSDrawerAttribute as Key, MeshVSDrawerSource};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, HandleReference},
};
use rustycad_tkservice::{
    aspect::{AspectTypeOfLine, AspectTypeOfMarker},
    graphic3d::{Graphic3dAspectLine3d, Graphic3dAspectMarker3d},
};
impl MeshVSTool {
    /// Native line factory; `Self::create_aspect_line3d(&drawer, true)?`.
    pub fn create_aspect_line3d(
        drawer: &dyn HandleReference<dyn MeshVSDrawerSource>,
        use_defaults: bool,
    ) -> Result<Handle<Graphic3dAspectLine3d>, Failure> {
        if drawer.is_null() {
            return Ok(Handle::default());
        }
        let Some(parameters) = line_values(&DrawerValues::new(drawer, use_defaults))? else {
            return Ok(Handle::default());
        };
        Ok(Handle::new(make_line(parameters)?))
    }
    /// Native marker factory; `Self::create_aspect_marker3d(&drawer, true)?`.
    pub fn create_aspect_marker3d(
        drawer: &dyn HandleReference<dyn MeshVSDrawerSource>,
        use_defaults: bool,
    ) -> Result<Handle<Graphic3dAspectMarker3d>, Failure> {
        if drawer.is_null() {
            return Ok(Handle::default());
        }
        let Some(parameters) = marker_values(&DrawerValues::new(drawer, use_defaults))? else {
            return Ok(Handle::default());
        };
        Ok(Handle::new(make_marker(parameters)?))
    }
}
fn line_values(values: &DrawerValues<'_>) -> Result<Option<(Color, f64, i32)>, Failure> {
    let color = required_value!(values.read(
        Key::BeamColor,
        Color::named(NamedColor::YELLOW),
        MeshVSDrawer::get_color
    ));
    let width = required_value!(values.read(Key::BeamWidth, 1., MeshVSDrawer::get_double));
    let kind = required_value!(values.read(
        Key::BeamType,
        AspectTypeOfLine::Solid as i32,
        MeshVSDrawer::get_integer
    ));
    Ok(Some((color, width, kind)))
}
fn marker_values(values: &DrawerValues<'_>) -> Result<Option<(Color, f64, i32)>, Failure> {
    let color = required_value!(values.read(
        Key::MarkerColor,
        Color::named(NamedColor::YELLOW),
        MeshVSDrawer::get_color
    ));
    let scale = required_value!(values.read(Key::MarkerScale, 1., MeshVSDrawer::get_double));
    let kind = required_value!(values.read(
        Key::MarkerType,
        AspectTypeOfMarker::X as i32,
        MeshVSDrawer::get_integer
    ));
    Ok(Some((color, scale, kind)))
}

fn make_line((color, width, kind): (Color, f64, i32)) -> Result<Graphic3dAspectLine3d, Failure> {
    let kind = native_enum(
        kind,
        AspectTypeOfLine::from_native_value(kind),
        "Aspect_TypeOfLine",
    )?;
    Graphic3dAspectLine3d::new(color, kind, width)
}
fn make_marker(
    (color, scale, kind): (Color, f64, i32),
) -> Result<Graphic3dAspectMarker3d, Failure> {
    let kind = native_enum(
        kind,
        AspectTypeOfMarker::from_native_value(kind),
        "Aspect_TypeOfMarker",
    )?;
    Graphic3dAspectMarker3d::new(kind, color, scale)
}
