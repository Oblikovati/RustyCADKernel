//! The aspects classes of the primitive kinds: `Graphic3d_AspectFillArea3d`,
//! `Graphic3d_AspectLine3d`, `Graphic3d_AspectMarker3d` and `Graphic3d_AspectText3d`.
use super::aspects::{aspects_class, graphic3d_aspects_dump_json};
use super::{
    Graphic3dAlphaMode, Graphic3dAspectsBase, Graphic3dAspectsEdgeMembers,
    Graphic3dAspectsLineMembers, Graphic3dAspectsSource, Graphic3dMarkerImage,
    Graphic3dMaterialAspect, Graphic3dTypeOfShadingModel,
};
use crate::aspect::{
    AspectInteriorStyle, AspectTypeOfDisplayText, AspectTypeOfLine, AspectTypeOfMarker,
    AspectTypeOfStyleText,
};
use crate::image::ImagePixMap;
use crate::json_dump::{base_class, class_begin};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::{AsciiString, HAsciiString};
use std::io;

/// The aspects of filled primitives; `Graphic3dAspectFillArea3d::default()` has an EMPTY interior.
pub struct Graphic3dAspectFillArea3d {
    transient: StandardTransient,
    base: Graphic3dAspectsBase,
}
aspects_class!(Graphic3dAspectFillArea3d, "Graphic3d_AspectFillArea3d");

impl Default for Graphic3dAspectFillArea3d {
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: Graphic3dAspectsBase::with(|f| f.interior_style = AspectInteriorStyle::Empty),
        }
    }
}

impl Graphic3dAspectFillArea3d {
    /// Interior style and color (front and back), edge color, line type and width, materials;
    /// a width <= 0 raises `Standard_OutOfRange`. Example:
    /// `Graphic3dAspectFillArea3d::new(AspectInteriorStyle::Solid, red, white, AspectTypeOfLine::Solid, 1.0, front, back)?`.
    pub fn new(
        interior: AspectInteriorStyle,
        interior_color: Color,
        edge_color: Color,
        edge_line_type: AspectTypeOfLine,
        edge_width: f64,
        front_material: Graphic3dMaterialAspect,
        back_material: Graphic3dMaterialAspect,
    ) -> Result<Self, Failure> {
        let base = Graphic3dAspectsBase::with(|f| {
            (f.front_material, f.back_material) = (front_material, back_material);
            f.interior_color.set_rgb(interior_color);
            f.back_interior_color.set_rgb(interior_color);
            f.edge_color.set_rgb(edge_color);
            (f.interior_style, f.line_type) = (interior, edge_line_type);
        });
        let aspects = Self {
            transient: StandardTransient::default(),
            base,
        };
        aspects.set_edge_width(f64::from(edge_width as f32))?;
        Ok(aspects)
    }
    /// Deprecated: `ToDrawEdges`; `aspects.edge()`.
    #[deprecated(note = "Deprecated method, ToDrawEdges() should be used instead")]
    pub fn edge(&self) -> bool {
        self.to_draw_edges()
    }
}

/// The unlit, yellow defaults of the line, marker and text aspects.
fn unlit_base(edit: impl FnOnce(&mut super::Graphic3dAspectsFields)) -> Graphic3dAspectsBase {
    Graphic3dAspectsBase::with(|f| {
        f.shading_model = Graphic3dTypeOfShadingModel::Unlit;
        f.interior_color.set_rgb(Color::named(NamedColor::YELLOW));
        edit(f);
    })
}

/// The aspects of line primitives; `Graphic3dAspectLine3d::default().width() == 1.0`.
pub struct Graphic3dAspectLine3d {
    transient: StandardTransient,
    base: Graphic3dAspectsBase,
}
aspects_class!(Graphic3dAspectLine3d, "Graphic3d_AspectLine3d");

impl Default for Graphic3dAspectLine3d {
    /// Unlit, yellow, solid, width 1.
    fn default() -> Self {
        let base = unlit_base(|f| (f.line_type, f.line_width) = (AspectTypeOfLine::Solid, 1.0));
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
}

impl Graphic3dAspectLine3d {
    /// A colored line of a type and width; a width <= 0 raises `Standard_OutOfRange`.
    /// Example: `Graphic3dAspectLine3d::new(red, AspectTypeOfLine::Dash, 2.0)?`.
    pub fn new(color: Color, kind: AspectTypeOfLine, width: f64) -> Result<Self, Failure> {
        let aspects = Self {
            transient: StandardTransient::default(),
            base: unlit_base(|f| f.interior_color.set_rgb(color)),
        };
        aspects.set_line_type(kind);
        aspects.set_line_width(width as f32)?;
        Ok(aspects)
    }
    /// The line type; `line.line_kind()`.
    pub fn line_kind(&self) -> AspectTypeOfLine {
        self.line_type()
    }
    /// Sets the line type; `line.set_line_kind(AspectTypeOfLine::Dot)`.
    pub fn set_line_kind(&self, kind: AspectTypeOfLine) {
        self.set_line_type(kind);
    }
    /// The width; `line.width()`.
    pub fn width(&self) -> f32 {
        self.line_width()
    }
    /// `SetWidth(float)`; `line.set_width(2.0)?`.
    pub fn set_width(&self, width: f32) -> Result<(), Failure> {
        self.set_line_width(width)
    }
    /// `SetWidth(double)`: narrowed to float; `line.set_width_f64(2.0)?`.
    pub fn set_width_f64(&self, width: f64) -> Result<(), Failure> {
        self.set_width(width as f32)
    }
}

/// The aspects of marker primitives; `Graphic3dAspectMarker3d::default().marker_kind()` is X.
pub struct Graphic3dAspectMarker3d {
    transient: StandardTransient,
    base: Graphic3dAspectsBase,
}
aspects_class!(Graphic3dAspectMarker3d, "Graphic3d_AspectMarker3d");

impl Default for Graphic3dAspectMarker3d {
    /// Unlit, yellow, X markers of scale 1.
    fn default() -> Self {
        let base = unlit_base(|f| (f.marker_type, f.marker_scale) = (AspectTypeOfMarker::X, 1.0));
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
}

impl Graphic3dAspectMarker3d {
    /// A marker type, color and scale; a scale <= 0 raises `Standard_OutOfRange`.
    /// Example: `Graphic3dAspectMarker3d::new(AspectTypeOfMarker::Plus, red, 2.0)?`.
    pub fn new(kind: AspectTypeOfMarker, color: Color, scale: f64) -> Result<Self, Failure> {
        let base = unlit_base(|f| f.interior_color.set_rgb(color));
        let aspects = Self {
            transient: StandardTransient::default(),
            base,
        };
        aspects.set_marker_type(kind);
        aspects.set_marker_scale(scale as f32)?;
        Ok(aspects)
    }
    /// A user marker from a 1-bit bitmap; `Graphic3dAspectMarker3d::from_bitmap(red, 8, 8, bitmap)`.
    pub fn from_bitmap(
        color: Color,
        width: i32,
        height: i32,
        bit_map: Handle<HArray1<u8>>,
    ) -> Self {
        let image = Handle::new(Graphic3dMarkerImage::from_bitmap(bit_map, width, height));
        let base = unlit_base(|f| {
            (f.marker_image, f.marker_type, f.marker_scale) =
                (image, AspectTypeOfMarker::Userdefined, 1.0);
            f.interior_color.set_rgb(color);
        });
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
    /// A yellow user marker from an image; the marker image checks raise as natively.
    /// Example: `Graphic3dAspectMarker3d::from_image(pixmap)?`.
    pub fn from_image(image: Handle<ImagePixMap>) -> Result<Self, Failure> {
        let marker = Handle::new(Graphic3dMarkerImage::from_images(image, Handle::default())?);
        let base = unlit_base(|f| {
            (f.marker_image, f.marker_type, f.marker_scale) =
                (marker, AspectTypeOfMarker::Userdefined, 1.0)
        });
        Ok(Self {
            transient: StandardTransient::default(),
            base,
        })
    }
    /// The scale; `marker.scale()`.
    pub fn scale(&self) -> f32 {
        self.marker_scale()
    }
    /// `SetScale(float)`; `marker.set_scale(2.0)?`.
    pub fn set_scale(&self, scale: f32) -> Result<(), Failure> {
        self.set_marker_scale(scale)
    }
    /// `SetScale(double)`: narrowed to float; `marker.set_scale_f64(2.0)?`.
    pub fn set_scale_f64(&self, scale: f64) -> Result<(), Failure> {
        self.set_scale(scale as f32)
    }
    /// The marker type; `marker.marker_kind()`.
    pub fn marker_kind(&self) -> AspectTypeOfMarker {
        self.marker_type()
    }
    /// Sets the marker type; `marker.set_marker_kind(AspectTypeOfMarker::Star)`.
    pub fn set_marker_kind(&self, kind: AspectTypeOfMarker) {
        self.set_marker_type(kind);
    }
    /// `GetTextureSize`: the marker image size, `(0, 0)` without image; `marker.get_texture_size()`.
    pub fn get_texture_size(&self) -> (i32, i32) {
        self.marker_image()
            .get()
            .map_or((0, 0), Graphic3dMarkerImage::get_texture_size)
    }
    /// The marker image; `marker.get_marker_image()`.
    pub fn get_marker_image(&self) -> Handle<Graphic3dMarkerImage> {
        self.marker_image()
    }
    /// Replaces the marker image by a bitmap; `marker.set_bit_map(8, 8, bitmap)`.
    pub fn set_bit_map(&self, width: i32, height: i32, bit_map: Handle<HArray1<u8>>) {
        self.set_marker_image(Handle::new(Graphic3dMarkerImage::from_bitmap(
            bit_map, width, height,
        )));
    }
}

/// The aspects of text primitives: masked blending (cutoff 0.285), unlit, yellow on white.
///
/// Example: `Graphic3dAspectText3d::new(red, Some("Arial"), 1.0, 0.0, style, display)`.
pub struct Graphic3dAspectText3d {
    transient: StandardTransient,
    base: Graphic3dAspectsBase,
}
impl Graphic3dAspectsSource for Graphic3dAspectText3d {
    fn aspects_base(&self) -> &Graphic3dAspectsBase {
        &self.base
    }
    /// The class name, then the `Graphic3d_Aspects` dump as the base class.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_AspectText3d")?;
        base_class(stream, depth, &|s, d| {
            graphic3d_aspects_dump_json(&self.base, s, d)
        })
    }
}
rustycad_tkernel::impl_transient!(Graphic3dAspectText3d, "Graphic3d_AspectText3d", dyn Graphic3dAspectsSource, transient,
    interfaces: [dyn Graphic3dAspectsSource]);

/// The text defaults on top of the unlit base.
fn text_base(edit: impl FnOnce(&mut super::Graphic3dAspectsFields)) -> Graphic3dAspectsBase {
    unlit_base(|f| {
        (f.alpha_mode, f.alpha_cutoff) = (Graphic3dAlphaMode::MaskBlend, 0.285);
        f.edge_color.set_rgb(Color::named(NamedColor::WHITE));
        edit(f);
    })
}

impl Default for Graphic3dAspectText3d {
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: text_base(|_| {}),
        }
    }
}

impl Graphic3dAspectText3d {
    /// A colored text with an optional font (an empty name keeps none); the expansion factor
    /// and the space are ignored, as natively.
    /// Example: `Graphic3dAspectText3d::new(red, Some("Arial"), 1.0, 0.0, AspectTypeOfStyleText::Normal, AspectTypeOfDisplayText::Normal)`.
    pub fn new(
        color: Color,
        font: Option<&str>,
        _expansion_factor: f64,
        _space: f64,
        style: AspectTypeOfStyleText,
        display_type: AspectTypeOfDisplayText,
    ) -> Self {
        let font = font
            .filter(|name| !name.is_empty())
            .map_or_else(Handle::default, |name| Handle::new(HAsciiString::new(name)));
        let base = text_base(|f| {
            (f.text_style, f.text_display_type, f.text_font) = (style, display_type, font);
            f.interior_color.set_rgb(color);
        });
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
    /// The font name, empty without font; `text.font()`.
    pub fn font(&self) -> AsciiString {
        let font = self.base.fields.read().text_font.clone();
        font.get()
            .map_or_else(AsciiString::default, |name| name.string().clone())
    }
    /// Sets the font name; an empty name removes the font. `text.set_font(&"Arial".into())`.
    pub fn set_font(&self, font: &AsciiString) {
        let handle = if font.is_empty() {
            Handle::default()
        } else {
            Handle::new(HAsciiString::from_string(font))
        };
        self.base.fields.write().text_font = handle;
    }
    /// `SetTextAngle(double)`: narrowed to float; `text.set_text_angle_f64(30.0)`.
    pub fn set_text_angle_f64(&self, angle: f64) {
        self.base.fields.write().text_angle = angle as f32;
    }
}
