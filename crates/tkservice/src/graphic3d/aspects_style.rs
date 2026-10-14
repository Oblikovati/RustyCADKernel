//! The line, marker, text, edge and hatch members of `Graphic3d_Aspects`, and `IsEqual`.
use super::{
    Graphic3dAspectsSource, Graphic3dHatchStyle, Graphic3dMarkerImage,
    Graphic3dTypeOfBackfacingModel,
};
use crate::aspect::{
    AspectHatchStyle, AspectTypeOfDisplayText, AspectTypeOfLine, AspectTypeOfMarker,
    AspectTypeOfStyleText,
};
use crate::font::FontFontAspect;
use rustycad_tkernel::quantity::{Color, ColorRgba};
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::HAsciiString;

/// `DefaultLinePatternForType`: the stipple pattern of a line type.
///
/// Example: `graphic3d_default_line_pattern_for_type(AspectTypeOfLine::Dash) == 0xFFC0`.
pub fn graphic3d_default_line_pattern_for_type(kind: AspectTypeOfLine) -> u16 {
    match kind {
        AspectTypeOfLine::Dash => 0xFFC0,
        AspectTypeOfLine::Dot => 0xCCCC,
        AspectTypeOfLine::Dotdash => 0xFF18,
        AspectTypeOfLine::Empty => 0x0000,
        AspectTypeOfLine::Solid => 0xFFFF,
        AspectTypeOfLine::Userdefined => 0xFF24,
        // Pinned Graphic3d_Aspects.hxx returns this after the switch for unnamed values.
        _ => 0xFFFF,
    }
}

/// `DefaultLineTypeForPattern`: the line type of a stipple pattern, `USERDEFINED` otherwise.
///
/// Example: `graphic3d_default_line_type_for_pattern(0xCCCC) == AspectTypeOfLine::Dot`.
pub fn graphic3d_default_line_type_for_pattern(pattern: u16) -> AspectTypeOfLine {
    match pattern {
        0x0000 => AspectTypeOfLine::Empty,
        0xFFC0 => AspectTypeOfLine::Dash,
        0xCCCC => AspectTypeOfLine::Dot,
        0xFF18 => AspectTypeOfLine::Dotdash,
        0xFFFF => AspectTypeOfLine::Solid,
        _ => AspectTypeOfLine::Userdefined,
    }
}

/// Reads one field of the aspects.
macro_rules! read_field {
    ($self:ident . $field:ident) => {
        $self.aspects_base().fields.read().$field
    };
}

/// The line and marker members of `Graphic3d_Aspects`.
pub trait Graphic3dAspectsLineMembers: Graphic3dAspectsSource {
    /// The line type (`LineType`, `EdgeLineType`); `aspects.line_type()`.
    fn line_type(&self) -> AspectTypeOfLine {
        read_field!(self.line_type)
    }
    /// Sets the line type and its default pattern (`SetLineType`, `SetEdgeLineType`).
    /// Example: `aspects.set_line_type(AspectTypeOfLine::Dash)`.
    fn set_line_type(&self, kind: AspectTypeOfLine) {
        let mut fields = self.aspects_base().fields.write();
        (fields.line_type, fields.line_pattern) =
            (kind, graphic3d_default_line_pattern_for_type(kind));
    }
    /// The stipple pattern; `aspects.line_pattern()`.
    fn line_pattern(&self) -> u16 {
        read_field!(self.line_pattern)
    }
    /// Sets the pattern and the line type it matches; `aspects.set_line_pattern(0xCCCC)`.
    fn set_line_pattern(&self, pattern: u16) {
        let mut fields = self.aspects_base().fields.write();
        (fields.line_type, fields.line_pattern) =
            (graphic3d_default_line_type_for_pattern(pattern), pattern);
    }
    /// The stipple factor; `aspects.line_stipple_factor()`.
    fn line_stipple_factor(&self) -> u16 {
        read_field!(self.line_factor)
    }
    /// Sets the stipple factor; 0 or above 256 raises `Standard_OutOfRange`.
    /// Example: `aspects.set_line_stipple_factor(2)?`.
    fn set_line_stipple_factor(&self, factor: u16) -> Result<(), Failure> {
        if factor == 0 || factor > 256 {
            return Err(out_of_range(&format!(
                "Graphic3d_Aspects::SetLineStippleFactor(), bad factor value; got {factor}, expected [1, 256]"
            )));
        }
        self.aspects_base().fields.write().line_factor = factor;
        Ok(())
    }
    /// The line width (`LineWidth`, `EdgeWidth`); `aspects.line_width()`.
    fn line_width(&self) -> f32 {
        read_field!(self.line_width)
    }
    /// Sets the line width; a width <= 0 raises `Standard_OutOfRange` (a NaN passes, as natively).
    /// Example: `aspects.set_line_width(2.0)?`.
    fn set_line_width(&self, width: f32) -> Result<(), Failure> {
        if width <= 0.0 {
            return Err(out_of_range(&format!(
                "Bad value for EdgeLineWidth; got {width}, expected > 0"
            )));
        }
        self.aspects_base().fields.write().line_width = width;
        Ok(())
    }
    /// `SetEdgeWidth(double)`: the width narrowed to float; `aspects.set_edge_width(2.0)?`.
    fn set_edge_width(&self, width: f64) -> Result<(), Failure> {
        self.set_line_width(width as f32)
    }
    /// The marker type; `aspects.marker_type()`.
    fn marker_type(&self) -> AspectTypeOfMarker {
        read_field!(self.marker_type)
    }
    /// Sets the marker type; `aspects.set_marker_type(AspectTypeOfMarker::Plus)`.
    fn set_marker_type(&self, kind: AspectTypeOfMarker) {
        self.aspects_base().fields.write().marker_type = kind;
    }
    /// The marker scale; `aspects.marker_scale()`.
    fn marker_scale(&self) -> f32 {
        read_field!(self.marker_scale)
    }
    /// Sets the marker scale; a scale <= 0 raises `Standard_OutOfRange`. `aspects.set_marker_scale(2.0)?`.
    fn set_marker_scale(&self, scale: f32) -> Result<(), Failure> {
        if scale <= 0.0 {
            return Err(out_of_range(&format!(
                "Bad value for MarkerScale; got {scale}, expected > 0"
            )));
        }
        self.aspects_base().fields.write().marker_scale = scale;
        Ok(())
    }
    /// The marker image; `aspects.marker_image()`.
    fn marker_image(&self) -> Handle<Graphic3dMarkerImage> {
        read_field!(self.marker_image).clone()
    }
    /// Sets the marker image; `aspects.set_marker_image(image)`.
    fn set_marker_image(&self, image: Handle<Graphic3dMarkerImage>) {
        self.aspects_base().fields.write().marker_image = image;
    }
    /// Whether the marker is drawn as a sprite (not a point, not empty, not a user marker
    /// without image); `aspects.is_marker_sprite()`.
    fn is_marker_sprite(&self) -> bool {
        let fields = self.aspects_base().fields.read();
        if matches!(
            fields.marker_type,
            AspectTypeOfMarker::Point | AspectTypeOfMarker::Empty
        ) {
            return false;
        }
        fields.marker_type != AspectTypeOfMarker::Userdefined || fields.marker_image.is_some()
    }
}
impl<T: Graphic3dAspectsSource + ?Sized> Graphic3dAspectsLineMembers for T {}

/// The text members of `Graphic3d_Aspects`.
pub trait Graphic3dAspectsTextMembers: Graphic3dAspectsSource {
    /// The font name; `aspects.text_font()`.
    fn text_font(&self) -> Handle<HAsciiString> {
        read_field!(self.text_font).clone()
    }
    /// Sets the font name; `aspects.set_text_font(font)`.
    fn set_text_font(&self, font: Handle<HAsciiString>) {
        self.aspects_base().fields.write().text_font = font;
    }
    /// The font aspect; `aspects.text_font_aspect()`.
    fn text_font_aspect(&self) -> FontFontAspect {
        read_field!(self.text_font_aspect)
    }
    /// Sets the font aspect; `aspects.set_text_font_aspect(FontFontAspect::Bold)`.
    fn set_text_font_aspect(&self, aspect: FontFontAspect) {
        self.aspects_base().fields.write().text_font_aspect = aspect;
    }
    /// The text display type; `aspects.text_display_type()`.
    fn text_display_type(&self) -> AspectTypeOfDisplayText {
        read_field!(self.text_display_type)
    }
    /// Sets the text display type; `aspects.set_text_display_type(AspectTypeOfDisplayText::Subtitle)`.
    fn set_text_display_type(&self, kind: AspectTypeOfDisplayText) {
        self.aspects_base().fields.write().text_display_type = kind;
    }
    /// Whether the text scales with the view; `aspects.is_text_zoomable()`.
    fn is_text_zoomable(&self) -> bool {
        read_field!(self.is_text_zoomable)
    }
    /// Sets text zooming; `aspects.set_text_zoomable(true)`.
    fn set_text_zoomable(&self, zoomable: bool) {
        self.aspects_base().fields.write().is_text_zoomable = zoomable;
    }
    /// The text style; `aspects.text_style()`.
    fn text_style(&self) -> AspectTypeOfStyleText {
        read_field!(self.text_style)
    }
    /// Sets the text style; `aspects.set_text_style(AspectTypeOfStyleText::Annotation)`.
    fn set_text_style(&self, style: AspectTypeOfStyleText) {
        self.aspects_base().fields.write().text_style = style;
    }
    /// The text angle in degrees; `aspects.text_angle()`.
    fn text_angle(&self) -> f32 {
        read_field!(self.text_angle)
    }
    /// Sets the text angle; `aspects.set_text_angle(30.0)`.
    fn set_text_angle(&self, angle: f32) {
        self.aspects_base().fields.write().text_angle = angle;
    }
}
impl<T: Graphic3dAspectsSource + ?Sized> Graphic3dAspectsTextMembers for T {}

/// The edge, hatch and deprecated members of `Graphic3d_Aspects`.
pub trait Graphic3dAspectsEdgeMembers: Graphic3dAspectsSource {
    /// The edge (and subtitle) color with alpha (`EdgeColorRGBA`, `ColorSubTitleRGBA`); `aspects.edge_color_rgba()`.
    fn edge_color_rgba(&self) -> ColorRgba {
        read_field!(self.edge_color)
    }
    /// The edge (and subtitle) color (`EdgeColor`, `ColorSubTitle`); `aspects.edge_color()`.
    fn edge_color(&self) -> Color {
        *read_field!(self.edge_color).rgb()
    }
    /// Sets the edge RGB, keeping alpha (`SetEdgeColor`, `SetColorSubTitle`); `aspects.set_edge_color(red)`.
    fn set_edge_color(&self, color: Color) {
        self.aspects_base().fields.write().edge_color.set_rgb(color);
    }
    /// Sets the edge color with alpha; `aspects.set_edge_color_rgba(rgba)`.
    fn set_edge_color_rgba(&self, color: ColorRgba) {
        self.aspects_base().fields.write().edge_color = color;
    }
    /// Whether edges are drawn (an EMPTY line type draws none); `aspects.to_draw_edges()`.
    fn to_draw_edges(&self) -> bool {
        let fields = self.aspects_base().fields.read();
        fields.to_draw_edges && fields.line_type != AspectTypeOfLine::Empty
    }
    /// Sets edge drawing (`SetEdgeOn/Off`); an EMPTY line type becomes SOLID. `aspects.set_draw_edges(true)`.
    fn set_draw_edges(&self, to_draw: bool) {
        let mut fields = self.aspects_base().fields.write();
        fields.to_draw_edges = to_draw;
        if fields.line_type == AspectTypeOfLine::Empty {
            fields.line_type = AspectTypeOfLine::Solid;
        }
    }
    /// Whether the first edge is skipped; `aspects.to_skip_first_edge()`.
    fn to_skip_first_edge(&self) -> bool {
        read_field!(self.to_skip_first_edge)
    }
    /// Sets first-edge skipping; `aspects.set_skip_first_edge(true)`.
    fn set_skip_first_edge(&self, to_skip: bool) {
        self.aspects_base().fields.write().to_skip_first_edge = to_skip;
    }
    /// Whether the silhouette is drawn; `aspects.to_draw_silhouette()`.
    fn to_draw_silhouette(&self) -> bool {
        read_field!(self.to_draw_silhouette)
    }
    /// Sets silhouette drawing; `aspects.set_draw_silhouette(true)`.
    fn set_draw_silhouette(&self, to_draw: bool) {
        self.aspects_base().fields.write().to_draw_silhouette = to_draw;
    }
    /// The hatch style; `aspects.hatch_style()`.
    fn hatch_style(&self) -> Handle<Graphic3dHatchStyle> {
        read_field!(self.hatch_style).clone()
    }
    /// Sets the hatch style; `aspects.set_hatch_style(style)`.
    fn set_hatch_style(&self, style: Handle<Graphic3dHatchStyle>) {
        self.aspects_base().fields.write().hatch_style = style;
    }
    /// Sets a predefined hatch style; SOLID removes it. `aspects.set_hatch_style_type(AspectHatchStyle::Grid)`.
    fn set_hatch_style_type(&self, style: AspectHatchStyle) {
        let hatch = if style == AspectHatchStyle::Solid {
            Handle::default()
        } else {
            Handle::new(Graphic3dHatchStyle::predefined(style))
        };
        self.aspects_base().fields.write().hatch_style = hatch;
    }
    /// Deprecated: whether back faces are culled (`ToSuppressBackFaces`, `BackFace`).
    #[deprecated(note = "Deprecated method, FaceCulling() should be used instead")]
    fn to_suppress_back_faces(&self) -> bool {
        matches!(
            read_field!(self.face_culling),
            Graphic3dTypeOfBackfacingModel::BackCulled | Graphic3dTypeOfBackfacingModel::Auto
        )
    }
    /// Deprecated: AUTO culling when true, else DOUBLE_SIDED (`SetSuppressBackFaces`).
    #[deprecated(note = "Deprecated method, SetFaceCulling() should be used instead")]
    fn set_suppress_back_faces(&self, to_suppress: bool) {
        let culling = if to_suppress {
            Graphic3dTypeOfBackfacingModel::Auto
        } else {
            Graphic3dTypeOfBackfacingModel::DoubleSided
        };
        self.aspects_base().fields.write().face_culling = culling;
    }
    /// Deprecated: DOUBLE_SIDED culling (`AllowBackFace`).
    #[deprecated(note = "Deprecated method, SetFaceCulling() should be used instead")]
    fn allow_back_face(&self) {
        self.aspects_base().fields.write().face_culling =
            Graphic3dTypeOfBackfacingModel::DoubleSided;
    }
    /// Deprecated: AUTO culling (`SuppressBackFace`).
    #[deprecated(note = "Deprecated method, SetFaceCulling() should be used instead")]
    fn suppress_back_face(&self) {
        self.aspects_base().fields.write().face_culling = Graphic3dTypeOfBackfacingModel::Auto;
    }
}
impl<T: Graphic3dAspectsSource + ?Sized> Graphic3dAspectsEdgeMembers for T {}
