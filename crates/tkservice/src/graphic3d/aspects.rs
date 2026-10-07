//! `Graphic3d_Aspects`: the presentation attributes of a group (interior, edges, lines,
//! markers, text, materials, textures, shader program).
//!
//! The class is concrete and has the virtual `DumpJson`, so it is the trait
//! [`Graphic3dAspectsSource`] over the embedded [`Graphic3dAspectsBase`]; [`Graphic3dAspects`] is
//! the native class itself and the four `Graphic3d_Aspect*3d` classes embed the same base. The
//! non-virtual members are the blanket traits of aspects_members.rs and aspects_style.rs.
use super::{
    Graphic3dAlphaMode, Graphic3dHatchStyle, Graphic3dMarkerImage, Graphic3dMaterialAspect,
    Graphic3dPolygonOffset, Graphic3dShaderProgram, Graphic3dTextureSet,
    Graphic3dTypeOfBackfacingModel, Graphic3dTypeOfShadingModel,
};
use crate::aspect::{
    AspectInteriorStyle, AspectTypeOfDisplayText, AspectTypeOfLine, AspectTypeOfMarker,
    AspectTypeOfStyleText,
};
use crate::font::FontFontAspect;
use crate::json_dump::{class_begin, field_dumped, field_integer, field_real, field_string};
use rustycad_tkernel::quantity::{Color, ColorRgba, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, Transient};
use rustycad_tkernel::tcollection::HAsciiString;
use std::io;

/// The protected fields of `Graphic3d_Aspects`.
#[derive(Clone)]
pub struct Graphic3dAspectsFields {
    /// `myProgram`.
    pub program: Handle<Graphic3dShaderProgram>,
    /// `myTextureSet`.
    pub texture_set: Handle<Graphic3dTextureSet>,
    /// `myMarkerImage`.
    pub marker_image: Handle<Graphic3dMarkerImage>,
    /// `myHatchStyle`.
    pub hatch_style: Handle<Graphic3dHatchStyle>,
    /// `myTextFont`.
    pub text_font: Handle<HAsciiString>,
    /// `myFrontMaterial`.
    pub front_material: Graphic3dMaterialAspect,
    /// `myBackMaterial`.
    pub back_material: Graphic3dMaterialAspect,
    /// `myInteriorColor`.
    pub interior_color: ColorRgba,
    /// `myBackInteriorColor`.
    pub back_interior_color: ColorRgba,
    /// `myEdgeColor`.
    pub edge_color: ColorRgba,
    /// `myPolygonOffset`.
    pub polygon_offset: Graphic3dPolygonOffset,
    /// `myInteriorStyle`.
    pub interior_style: AspectInteriorStyle,
    /// `myShadingModel`.
    pub shading_model: Graphic3dTypeOfShadingModel,
    /// `myFaceCulling`.
    pub face_culling: Graphic3dTypeOfBackfacingModel,
    /// `myAlphaMode`.
    pub alpha_mode: Graphic3dAlphaMode,
    /// `myAlphaCutoff`.
    pub alpha_cutoff: f32,
    /// `myLineType`.
    pub line_type: AspectTypeOfLine,
    /// `myLineWidth`.
    pub line_width: f32,
    /// `myLineFactor`.
    pub line_factor: u16,
    /// `myLinePattern`.
    pub line_pattern: u16,
    /// `myMarkerType`.
    pub marker_type: AspectTypeOfMarker,
    /// `myMarkerScale`.
    pub marker_scale: f32,
    /// `myTextStyle`.
    pub text_style: AspectTypeOfStyleText,
    /// `myTextDisplayType`.
    pub text_display_type: AspectTypeOfDisplayText,
    /// `myTextFontAspect`.
    pub text_font_aspect: FontFontAspect,
    /// `myTextAngle`.
    pub text_angle: f32,
    /// `myToSkipFirstEdge`.
    pub to_skip_first_edge: bool,
    /// `myToDistinguishMaterials`.
    pub to_distinguish_materials: bool,
    /// `myToUseVertexColorForBackFaces`.
    pub to_use_vertex_color_for_back_faces: bool,
    /// `myToDrawEdges`.
    pub to_draw_edges: bool,
    /// `myToDrawSilhouette`.
    pub to_draw_silhouette: bool,
    /// `myToMapTexture`.
    pub to_map_texture: bool,
    /// `myIsTextZoomable`.
    pub is_text_zoomable: bool,
}

/// An opaque named color.
fn opaque(name: NamedColor) -> ColorRgba {
    ColorRgba::from_rgb(Color::named(name), 1.0)
}

impl Default for Graphic3dAspectsFields {
    /// The member initialisers of `Graphic3d_Aspects()`.
    fn default() -> Self {
        Self {
            program: Handle::default(),
            texture_set: Handle::default(),
            marker_image: Handle::default(),
            hatch_style: Handle::default(),
            text_font: Handle::default(),
            front_material: Graphic3dMaterialAspect::default(),
            back_material: Graphic3dMaterialAspect::default(),
            interior_color: opaque(NamedColor::CYAN1),
            back_interior_color: opaque(NamedColor::CYAN1),
            edge_color: opaque(NamedColor::WHITE),
            polygon_offset: Graphic3dPolygonOffset::default(),
            interior_style: AspectInteriorStyle::Solid,
            shading_model: Graphic3dTypeOfShadingModel::Default,
            face_culling: Graphic3dTypeOfBackfacingModel::Auto,
            alpha_mode: Graphic3dAlphaMode::BlendAuto,
            alpha_cutoff: 0.5,
            line_type: AspectTypeOfLine::Solid,
            line_width: 1.0,
            line_factor: 1,
            line_pattern: 0xFFFF,
            marker_type: AspectTypeOfMarker::Point,
            marker_scale: 1.0,
            text_style: AspectTypeOfStyleText::Normal,
            text_display_type: AspectTypeOfDisplayText::Normal,
            text_font_aspect: FontFontAspect::Regular,
            text_angle: 0.0,
            to_skip_first_edge: false,
            to_distinguish_materials: false,
            to_use_vertex_color_for_back_faces: true,
            to_draw_edges: false,
            to_draw_silhouette: false,
            to_map_texture: false,
            is_text_zoomable: false,
        }
    }
}

/// The embedded state of every aspects class.
#[derive(Default)]
pub struct Graphic3dAspectsBase {
    /// The protected fields, shared by every handle to the aspects.
    pub fields: SharedState<Graphic3dAspectsFields>,
}

impl Graphic3dAspectsBase {
    /// A base whose fields start from the defaults changed by `edit` (a subclass constructor).
    /// Example: `Graphic3dAspectsBase::with(|f| f.interior_style = AspectInteriorStyle::Empty)`.
    pub fn with(edit: impl FnOnce(&mut Graphic3dAspectsFields)) -> Self {
        let mut fields = Graphic3dAspectsFields::default();
        edit(&mut fields);
        Self {
            fields: SharedState::new(fields),
        }
    }
}

/// An aspects class; implementers return their embedded base.
pub trait Graphic3dAspectsSource: Transient {
    /// The embedded base; `aspects.aspects_base().fields.read().line_width`.
    fn aspects_base(&self) -> &Graphic3dAspectsBase;
    /// `DumpJson`; `aspects.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        graphic3d_aspects_dump_json(self.aspects_base(), stream, depth)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dAspectsSource,
    "Graphic3d_Aspects",
    StandardTransient
);

/// The body of `Graphic3d_Aspects::DumpJson`, for the classes that dump it as their base.
///
/// Example: `graphic3d_aspects_dump_json(aspects.aspects_base(), &mut stream, -1)?`.
pub fn graphic3d_aspects_dump_json(
    base: &Graphic3dAspectsBase,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let f = base.fields.read().clone();
    class_begin(stream, "Graphic3d_Aspects")?;
    dump_colors_and_flags(&f, stream, depth)?;
    dump_alpha_and_text(&f, stream)
}

/// The colors, polygon offset and edge flags of the aspects dump.
fn dump_colors_and_flags(
    f: &Graphic3dAspectsFields,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    for (name, color) in [
        ("&myInteriorColor", f.interior_color),
        ("&myBackInteriorColor", f.back_interior_color),
        ("&myEdgeColor", f.edge_color),
    ] {
        field_dumped(stream, depth, name, &|s, _| color.dump_json(s))?;
    }
    let offset = f.polygon_offset;
    field_dumped(stream, depth, "&myPolygonOffset", &|s, d| {
        offset.dump_json(s, d)
    })?;
    let flags = [
        ("myToSkipFirstEdge", f.to_skip_first_edge),
        ("myToDistinguishMaterials", f.to_distinguish_materials),
        (
            "myToUseVertexColorForBackFaces",
            f.to_use_vertex_color_for_back_faces,
        ),
        ("myToDrawEdges", f.to_draw_edges),
        ("myToDrawSilhouette", f.to_draw_silhouette),
    ];
    for (name, flag) in flags {
        field_integer(stream, name, i32::from(flag))?;
    }
    field_integer(stream, "myFaceCulling", f.face_culling as i32)
}

/// The texture, alpha and text fields of the aspects dump.
fn dump_alpha_and_text(f: &Graphic3dAspectsFields, stream: &mut DumpStream) -> io::Result<()> {
    field_integer(stream, "myToMapTexture", i32::from(f.to_map_texture))?;
    field_integer(stream, "myIsTextZoomable", i32::from(f.is_text_zoomable))?;
    field_integer(stream, "myAlphaMode", f.alpha_mode as i32)?;
    field_real(stream, "myAlphaCutoff", f64::from(f.alpha_cutoff))?;
    if let Some(font) = f.text_font.get() {
        field_string(stream, "myTextFont->String()", font.string().as_bytes())?;
    }
    field_integer(stream, "myTextStyle", f.text_style as i32)?;
    field_integer(stream, "myTextDisplayType", f.text_display_type as i32)?;
    field_integer(stream, "myTextFontAspect", f.text_font_aspect as i32)?;
    field_real(stream, "myTextAngle", f64::from(f.text_angle))
}

/// The native `Graphic3d_Aspects`; `Graphic3dAspects::default().interior_style()`.
#[derive(Default)]
pub struct Graphic3dAspects {
    transient: StandardTransient,
    base: Graphic3dAspectsBase,
}
impl Graphic3dAspectsSource for Graphic3dAspects {
    fn aspects_base(&self) -> &Graphic3dAspectsBase {
        &self.base
    }
}
rustycad_tkernel::impl_transient!(Graphic3dAspects, "Graphic3d_Aspects", dyn Graphic3dAspectsSource, transient,
    interfaces: [dyn Graphic3dAspectsSource]);

/// Implements the aspects traits of a struct with `transient` and `base: Graphic3dAspectsBase`.
macro_rules! aspects_class {
    ($ty:ident, $name:literal) => {
        impl $crate::graphic3d::Graphic3dAspectsSource for $ty {
            fn aspects_base(&self) -> &$crate::graphic3d::Graphic3dAspectsBase {
                &self.base
            }
        }
        rustycad_tkernel::impl_transient!($ty, $name, dyn $crate::graphic3d::Graphic3dAspectsSource, transient,
            interfaces: [dyn $crate::graphic3d::Graphic3dAspectsSource]);
    };
}
pub(crate) use aspects_class;
