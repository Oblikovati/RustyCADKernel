//! `Graphic3d_PresentationAttributes`: the highlight method, color, transparency, Z layer,
//! display mode and fill aspect of a presentation. The setters are virtual natively
//! (`Prs3d_Drawer` overrides them), so the class is a trait over an embedded base.
use super::Graphic3dAspectsSource as _;
use super::{GRAPHIC3D_ZLAYER_ID_DEFAULT, Graphic3dAspectFillArea3d, Graphic3dZLayerId};
use crate::aspect::AspectTypeOfHighlightMethod;
use crate::json_dump::{field_dumped, field_integer};
use rustycad_tkernel::quantity::{Color, ColorRgba, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, Transient};
use std::io;

/// The protected fields of the presentation attributes.
#[derive(Clone)]
pub struct Graphic3dPresentationAttributesFields {
    /// `myBasicFillAreaAspect`.
    pub basic_fill_area_aspect: Handle<Graphic3dAspectFillArea3d>,
    /// `myBasicColor`.
    pub basic_color: ColorRgba,
    /// `myHiMethod`.
    pub hi_method: AspectTypeOfHighlightMethod,
    /// `myZLayer`.
    pub z_layer: Graphic3dZLayerId,
    /// `myDispMode`.
    pub disp_mode: i32,
}

impl Default for Graphic3dPresentationAttributesFields {
    /// White, color highlighting, the default Z layer, display mode 0.
    fn default() -> Self {
        Self {
            basic_fill_area_aspect: Handle::default(),
            basic_color: ColorRgba::from_rgb(Color::named(NamedColor::WHITE), 1.0),
            hi_method: AspectTypeOfHighlightMethod::Color,
            z_layer: GRAPHIC3D_ZLAYER_ID_DEFAULT,
            disp_mode: 0,
        }
    }
}

/// The embedded state of the presentation attributes classes.
#[derive(Default)]
pub struct Graphic3dPresentationAttributesBase {
    /// The fields, shared by every handle.
    pub fields: SharedState<Graphic3dPresentationAttributesFields>,
}

/// A presentation attributes class; the setters may be overridden.
pub trait Graphic3dPresentationAttributesSource: Transient {
    /// The embedded base; `attributes.presentation_attributes_base()`.
    fn presentation_attributes_base(&self) -> &Graphic3dPresentationAttributesBase;
    /// Sets the highlight method; `attributes.set_method(AspectTypeOfHighlightMethod::Boundbox)`.
    fn set_method(&self, method: AspectTypeOfHighlightMethod) {
        self.presentation_attributes_base().fields.write().hi_method = method;
    }
    /// Sets the color (keeping the alpha); `attributes.set_color(red)`.
    fn set_color(&self, color: Color) {
        *self
            .presentation_attributes_base()
            .fields
            .write()
            .basic_color
            .change_rgb() = color;
    }
    /// Sets the transparency (alpha `1 - t`, unchecked); `attributes.set_transparency(0.5)`.
    fn set_transparency(&self, transparency: f32) {
        let mut fields = self.presentation_attributes_base().fields.write();
        fields.basic_color.set_alpha(1.0 - transparency);
    }
    /// Sets the Z layer; `attributes.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOP)`.
    fn set_z_layer(&self, layer: Graphic3dZLayerId) {
        self.presentation_attributes_base().fields.write().z_layer = layer;
    }
    /// Sets the display mode; `attributes.set_display_mode(1)`.
    fn set_display_mode(&self, mode: i32) {
        self.presentation_attributes_base().fields.write().disp_mode = mode;
    }
    /// Sets the fill area aspect; `attributes.set_basic_fill_area_aspect(aspect)`.
    fn set_basic_fill_area_aspect(&self, aspect: Handle<Graphic3dAspectFillArea3d>) {
        self.presentation_attributes_base()
            .fields
            .write()
            .basic_fill_area_aspect = aspect;
    }
    /// `DumpJson`; `attributes.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        graphic3d_presentation_attributes_dump_json(
            self.presentation_attributes_base(),
            stream,
            depth,
        )
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dPresentationAttributesSource,
    "Graphic3d_PresentationAttributes",
    StandardTransient
);

/// The body of `Graphic3d_PresentationAttributes::DumpJson` (no class name, as natively).
/// Example: `graphic3d_presentation_attributes_dump_json(base, &mut stream, -1)?`.
pub fn graphic3d_presentation_attributes_dump_json(
    base: &Graphic3dPresentationAttributesBase,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let f = base.fields.read().clone();
    if let Some(aspect) = f.basic_fill_area_aspect.get() {
        field_dumped(stream, depth, "myBasicFillAreaAspect.get()", &|o, d| {
            aspect.dump_json(o, d)
        })?;
    }
    let color = f.basic_color;
    field_dumped(stream, depth, "&myBasicColor", &|o, _| color.dump_json(o))?;
    field_integer(stream, "myHiMethod", f.hi_method as i32)?;
    field_integer(stream, "myZLayer", f.z_layer)?;
    field_integer(stream, "myDispMode", f.disp_mode)
}

/// The non-virtual getters of the presentation attributes, for every implementer.
pub trait Graphic3dPresentationAttributesMembers: Graphic3dPresentationAttributesSource {
    /// The highlight method; `attributes.method()`.
    fn method(&self) -> AspectTypeOfHighlightMethod {
        self.presentation_attributes_base().fields.read().hi_method
    }
    /// The color with alpha; `attributes.color_rgba()`.
    fn color_rgba(&self) -> ColorRgba {
        self.presentation_attributes_base()
            .fields
            .read()
            .basic_color
    }
    /// The color; `attributes.color()`.
    fn color(&self) -> Color {
        *self.color_rgba().rgb()
    }
    /// The transparency `1 - alpha`; `attributes.transparency()`.
    fn transparency(&self) -> f32 {
        1.0 - self.color_rgba().alpha()
    }
    /// The Z layer; `attributes.z_layer()`.
    fn z_layer(&self) -> Graphic3dZLayerId {
        self.presentation_attributes_base().fields.read().z_layer
    }
    /// The display mode; `attributes.display_mode()`.
    fn display_mode(&self) -> i32 {
        self.presentation_attributes_base().fields.read().disp_mode
    }
    /// The fill area aspect; `attributes.basic_fill_area_aspect()`.
    fn basic_fill_area_aspect(&self) -> Handle<Graphic3dAspectFillArea3d> {
        self.presentation_attributes_base()
            .fields
            .read()
            .basic_fill_area_aspect
            .clone()
    }
}
impl<T: Graphic3dPresentationAttributesSource + ?Sized> Graphic3dPresentationAttributesMembers
    for T
{
}

/// The native `Graphic3d_PresentationAttributes`; `Graphic3dPresentationAttributes::default()`.
#[derive(Default)]
pub struct Graphic3dPresentationAttributes {
    transient: StandardTransient,
    base: Graphic3dPresentationAttributesBase,
}
impl Graphic3dPresentationAttributesSource for Graphic3dPresentationAttributes {
    fn presentation_attributes_base(&self) -> &Graphic3dPresentationAttributesBase {
        &self.base
    }
}
rustycad_tkernel::impl_transient!(Graphic3dPresentationAttributes,
    "Graphic3d_PresentationAttributes", StandardTransient, transient,
    interfaces: [dyn Graphic3dPresentationAttributesSource]);
