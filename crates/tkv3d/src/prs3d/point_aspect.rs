//! `Prs3d_PointAspect`: the marker attributes of points.
use super::basic_aspect::{Prs3dBasicAspect, dump_graphic_aspects};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::aspect::AspectTypeOfMarker;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectMarker3d, Graphic3dAspectsMembers as _, Graphic3dMarkerImage,
};
use rustycad_tkservice::json_dump::class_begin;
use std::io;

/// The marker attributes of point presentations.
///
/// Example: `let aspect = Prs3dPointAspect::new(AspectTypeOfMarker::Plus, yellow, 1.0)?;`.
pub struct Prs3dPointAspect {
    transient: StandardTransient,
    pub(super) aspect: SharedState<Handle<Graphic3dAspectMarker3d>>,
}
rustycad_tkernel::impl_transient!(Prs3dPointAspect, "Prs3d_PointAspect", dyn Prs3dBasicAspect, transient,
    interfaces: [dyn Prs3dBasicAspect]);

impl Prs3dPointAspect {
    /// A marker of a type, color and scale; a scale <= 0 raises `Standard_OutOfRange`.
    /// Example: `Prs3dPointAspect::new(AspectTypeOfMarker::Star, red, 2.0)?`.
    pub fn new(kind: AspectTypeOfMarker, color: Color, scale: f64) -> Result<Self, Failure> {
        let aspect = Graphic3dAspectMarker3d::new(kind, color, scale)?;
        Ok(Self::from_aspect(Handle::new(aspect)))
    }
    /// A user marker from a bitmap texture; `Prs3dPointAspect::with_texture(red, 8, 8, bits)`.
    pub fn with_texture(
        color: Color,
        width: i32,
        height: i32,
        texture: Handle<HArray1<u8>>,
    ) -> Self {
        let aspect = Graphic3dAspectMarker3d::from_bitmap(color, width, height, texture);
        Self::from_aspect(Handle::new(aspect))
    }
    /// Wraps an existing Graphic3d aspect; `Prs3dPointAspect::from_aspect(aspect)`.
    pub fn from_aspect(aspect: Handle<Graphic3dAspectMarker3d>) -> Self {
        Self {
            transient: StandardTransient::default(),
            aspect: SharedState::new(aspect),
        }
    }
    /// Sets the marker color; `aspect.set_color(red)`.
    pub fn set_color(&self, color: Color) {
        if let Some(aspect) = self.aspect().get() {
            aspect.set_interior_color(color);
        }
    }
    /// Sets the marker type; `aspect.set_type_of_marker(AspectTypeOfMarker::Ring1)`.
    pub fn set_type_of_marker(&self, kind: AspectTypeOfMarker) {
        if let Some(aspect) = self.aspect().get() {
            aspect.set_marker_kind(kind);
        }
    }
    /// Sets the scale (narrowed to float); a scale <= 0 raises `Standard_OutOfRange`.
    /// Example: `aspect.set_scale(3.0)?`.
    pub fn set_scale(&self, scale: f64) -> Result<(), Failure> {
        match self.aspect().get() {
            Some(aspect) => aspect.set_scale_f64(scale),
            None => Ok(()),
        }
    }
    /// The Graphic3d aspect; `aspect.aspect()`.
    pub fn aspect(&self) -> Handle<Graphic3dAspectMarker3d> {
        self.aspect.read().clone()
    }
    /// Replaces the Graphic3d aspect; `aspect.set_aspect(other)`.
    pub fn set_aspect(&self, aspect: Handle<Graphic3dAspectMarker3d>) {
        *self.aspect.write() = aspect;
    }
    /// `GetTextureSize`: `(width, height)` of the marker image; `aspect.get_texture_size()`.
    pub fn get_texture_size(&self) -> (i32, i32) {
        self.aspect()
            .get()
            .map_or((0, 0), Graphic3dAspectMarker3d::get_texture_size)
    }
    /// `GetTexture`: the marker image; `aspect.get_texture()`.
    pub fn get_texture(&self) -> Handle<Graphic3dMarkerImage> {
        self.aspect()
            .get()
            .map_or_else(Handle::default, Graphic3dAspectMarker3d::get_marker_image)
    }
}

impl Prs3dBasicAspect for Prs3dPointAspect {
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Prs3d_PointAspect")?;
        dump_graphic_aspects(stream, depth, "myAspect.get()", &self.aspect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::quantity::NamedColor;

    #[test]
    fn marker_setters_and_texture() {
        let aspect = Prs3dPointAspect::new(
            AspectTypeOfMarker::Plus,
            Color::named(NamedColor::YELLOW),
            1.0,
        )
        .unwrap();
        aspect.set_type_of_marker(AspectTypeOfMarker::Star);
        aspect.set_scale(2.0).unwrap();
        aspect.set_color(Color::named(NamedColor::RED));
        let graphic = aspect.aspect();
        assert_eq!(graphic.marker_kind(), AspectTypeOfMarker::Star);
        assert_eq!(graphic.scale(), 2.0);
        assert!(aspect.set_scale(-1.0).is_err());
        assert_eq!(aspect.get_texture_size(), (0, 0));
        assert!(aspect.get_texture().is_null());
        let bits = Handle::new(HArray1::<u8>::with_bounds(1, 8).unwrap());
        bits.change_array1().init(0xFF);
        let textured = Prs3dPointAspect::with_texture(Color::named(NamedColor::RED), 8, 8, bits);
        assert_eq!(textured.get_texture_size(), (8, 8));
    }
}
