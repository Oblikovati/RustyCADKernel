//! `StdPrs_BRepFont`: renders glyphs and text of a FreeType font as BRep faces or wires.
//! Copies made by `StdPrs_BRepFontCache` share one implementation (font, metrics and glyph
//! region cache); a setter on a shared implementation switches its handle to a copy.
use super::brep_font_impl::{StdPrsBRepFontImpl, StdPrsBRepFontSource, font_parameters};
use crate::brep_font::{
    BRepFontBuilder, BRepFontBuilderGlyphOptions, BRepFontBuilderTextOptions,
    BRepFontBuilderTextPlan,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Axis3;
use rustycad_tkservice::font::{FontFontAspect, FontStrictLevel};
use rustycad_tkservice::graphic3d::{
    Graphic3dHorizontalTextAlignment, Graphic3dVerticalTextAlignment,
};
use std::sync::Arc;

/// `StdPrs_BRepFont::GlyphOptions`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StdPrsBRepFontGlyphOptions {
    /// `ToConcatenateContours`: one C0 B-spline edge per contour.
    pub to_concatenate_contours: bool,
}

/// `StdPrs_BRepFont::TextOptions`.
#[derive(Clone, Copy, Debug)]
pub struct StdPrsBRepFontTextOptions {
    /// `Pen`: the position and orientation of the text baseline.
    pub pen: Axis3,
    /// `HorizontalAlignment` (left by default).
    pub horizontal_alignment: Graphic3dHorizontalTextAlignment,
    /// `VerticalAlignment` (bottom by default).
    pub vertical_alignment: Graphic3dVerticalTextAlignment,
    /// `ToConcatenateContours`: one C0 B-spline edge per contour.
    pub to_concatenate_contours: bool,
}

impl Default for StdPrsBRepFontTextOptions {
    fn default() -> Self {
        Self {
            pen: Axis3::default(),
            horizontal_alignment: Graphic3dHorizontalTextAlignment::Left,
            vertical_alignment: Graphic3dVerticalTextAlignment::Bottom,
            to_concatenate_contours: false,
        }
    }
}

/// `StdPrs_BRepFont`: a font that builds BRep glyphs and text.
/// Example: `let font = StdPrsBRepFont::find_and_create(&"Arial".into(), FontFontAspect::Regular, 12.0, FontStrictLevel::Any);`.
pub struct StdPrsBRepFont {
    transient: StandardTransient,
    implementation: SharedState<Arc<StdPrsBRepFontImpl>>,
}
rustycad_tkernel::impl_transient!(
    StdPrsBRepFont,
    "StdPrs_BRepFont",
    StandardTransient,
    transient
);

impl Default for StdPrsBRepFont {
    fn default() -> Self {
        Self::new()
    }
}

/// `isValidPosition`: every coordinate of the location and the three axes is a valid real.
fn is_valid_position(position: &Axis3) -> bool {
    let location = position.location();
    let directions = [
        position.direction(),
        position.x_direction(),
        position.y_direction(),
    ];
    [location.x(), location.y(), location.z()]
        .into_iter()
        .chain(
            directions
                .iter()
                .flat_map(|direction| [direction.x(), direction.y(), direction.z()]),
        )
        .all(|value| !value.is_nan() && !precision::is_infinite(value))
}

/// The UTF-8 text of a native string argument (`ToCString`).
fn ascii_of(text: &String8) -> AsciiString {
    let units = text.to_c_string();
    let end = units
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(units.len());
    AsciiString::from(String::from_utf8_lossy(&units[..end]).as_ref())
}

impl StdPrsBRepFont {
    /// `StdPrs_BRepFont()`: an invalid font; `StdPrsBRepFont::new()`.
    pub fn new() -> Self {
        Self::from_impl(Arc::new(StdPrsBRepFontImpl::new()))
    }

    /// The private constructor of `StdPrs_BRepFontCache`: a handle on a shared implementation.
    pub(super) fn from_impl(implementation: Arc<StdPrsBRepFontImpl>) -> Self {
        Self {
            transient: StandardTransient::default(),
            implementation: SharedState::new(implementation),
        }
    }

    pub(super) fn implementation(&self) -> Arc<StdPrsBRepFontImpl> {
        self.implementation.read().clone()
    }

    /// `StdPrs_BRepFont(theFontPath, theSize, theFaceId)`: opens a font file.
    /// Example: `StdPrsBRepFont::from_path(&path, 12.0, 0)`.
    pub fn from_path(path: &String8, size: f64, face_id: i32) -> Self {
        let font = Self::new();
        font.init(path, size, face_id);
        font
    }

    /// `StdPrs_BRepFont(theFontName, theFontAspect, theSize, theStrictLevel)`: finds a font.
    /// Example: `StdPrsBRepFont::from_name(&name, FontFontAspect::Regular, 12.0, FontStrictLevel::Any)`.
    pub fn from_name(
        name: &String8,
        aspect: FontFontAspect,
        size: f64,
        strict_level: FontStrictLevel,
    ) -> Self {
        let font = Self::new();
        font.find_and_init(&ascii_of(name), aspect, size, strict_level);
        font
    }

    /// `FindAndCreate`: a found font, or a null handle.
    /// Example: `StdPrsBRepFont::find_and_create(&"Arial".into(), FontFontAspect::Regular, 12.0, FontStrictLevel::Any)`.
    pub fn find_and_create(
        name: &AsciiString,
        aspect: FontFontAspect,
        size: f64,
        strict_level: FontStrictLevel,
    ) -> Handle<StdPrsBRepFont> {
        let font = Self::new();
        if font.find_and_init(name, aspect, size, strict_level) {
            Handle::new(font)
        } else {
            Handle::default()
        }
    }

    /// `Release`: an invalid font that keeps the width scaling; `font.release()`.
    pub fn release(&self) {
        let width_scaling = self.width_scaling();
        let fresh = StdPrsBRepFontImpl::new();
        fresh.font.set_width_scaling(width_scaling);
        *self.implementation.write() = Arc::new(fresh);
    }

    fn width_scaling(&self) -> f32 {
        let implementation = self.implementation();
        let _lock = implementation.lock_font();
        implementation.font.width_scaling()
    }

    /// `IsValid`: a positive size and an open font; `font.is_valid()`.
    pub fn is_valid(&self) -> bool {
        let implementation = self.implementation();
        implementation.metrics.size > 0.0 && implementation.font.is_valid()
    }

    /// `Init(theFontPath, theSize, theFaceId)`: reopens from a file; on failure the current
    /// font is kept. Example: `font.init(&path, 12.0, 0)`.
    pub fn init(&self, path: &String8, size: f64, face_id: i32) -> bool {
        let path = ascii_of(path);
        self.reinit(
            size,
            StdPrsBRepFontSource::File {
                path: path.clone(),
                face_id,
            },
            |fresh| fresh.font.init(&path, &font_parameters(), face_id),
        )
    }

    /// `FindAndInit`: reopens a font found by name; on failure the current font is kept.
    /// Example: `font.find_and_init(&"Arial".into(), FontFontAspect::Regular, 12.0, FontStrictLevel::Any)`.
    pub fn find_and_init(
        &self,
        name: &AsciiString,
        aspect: FontFontAspect,
        size: f64,
        strict_level: FontStrictLevel,
    ) -> bool {
        let source = StdPrsBRepFontSource::Named {
            name: name.clone(),
            aspect,
            strict_level,
        };
        self.reinit(size, source, |fresh| {
            fresh
                .font
                .find_and_init(name, aspect, &font_parameters(), strict_level)
        })
    }

    /// `Init(theFontName, theFontAspect, theSize)`: `FindAndInit` with any strict level.
    /// Example: `font.init_by_name(&name, FontFontAspect::Regular, 12.0)`.
    pub fn init_by_name(&self, name: &String8, aspect: FontFontAspect, size: f64) -> bool {
        self.find_and_init(&ascii_of(name), aspect, size, FontStrictLevel::Any)
    }

    // The shared part of Init and FindAndInit: a new implementation, opened and scaled.
    fn reinit(
        &self,
        size: f64,
        source: StdPrsBRepFontSource,
        open: impl FnOnce(&StdPrsBRepFontImpl) -> bool,
    ) -> bool {
        let width_scaling = self.width_scaling();
        let mut fresh = StdPrsBRepFontImpl::new();
        if size <= fresh.tolerance * 100.0 || size.is_nan() || precision::is_infinite(size) {
            return false;
        }
        if !open(&fresh) || !fresh.font.set_width_scaling(width_scaling) {
            return false;
        }
        fresh.set_size(size);
        fresh.source = source;
        *self.implementation.write() = Arc::new(fresh);
        true
    }

    /// `RenderGlyph(theChar)` with default options; `font.render_glyph('A' as u32)`.
    pub fn render_glyph(&self, character: u32) -> Shape {
        self.render_glyph_with(character, &StdPrsBRepFontGlyphOptions::default())
    }

    /// `RenderGlyph(theChar, theOptions)`: a new glyph shape per call, null when the font is
    /// invalid or the glyph unavailable. Example: `font.render_glyph_with('B' as u32, &options)`.
    pub fn render_glyph_with(&self, character: u32, options: &StdPrsBRepFontGlyphOptions) -> Shape {
        let implementation = self.implementation();
        match implementation.make_request(character) {
            Some(request) => implementation.render_glyph(&request, options.to_concatenate_contours),
            None => Shape::default(),
        }
    }

    /// `RenderText(theText)` with default options; `font.render_text(&text)`.
    pub fn render_text(&self, text: &String8) -> Shape {
        self.render_text_with(text, &StdPrsBRepFontTextOptions::default())
    }

    /// `RenderText(theText, theOptions)`: plans and renders a text compound.
    /// Example: `font.render_text_with(&text, &options)`.
    pub fn render_text_with(&self, text: &String8, options: &StdPrsBRepFontTextOptions) -> Shape {
        if !is_valid_position(&options.pen) {
            return Shape::default();
        }
        match self.plan_text(
            text,
            options.horizontal_alignment,
            options.vertical_alignment,
        ) {
            Some(plan) => self.render_plan_with(&plan, options),
            None => Shape::default(),
        }
    }

    /// `PlanText`: the formatted layout of a text; None for empty text or an invalid font.
    /// Example: `font.plan_text(&text, Graphic3dHorizontalTextAlignment::Left, Graphic3dVerticalTextAlignment::Bottom)`.
    pub fn plan_text(
        &self,
        text: &String8,
        horizontal: Graphic3dHorizontalTextAlignment,
        vertical: Graphic3dVerticalTextAlignment,
    ) -> Option<BRepFontBuilderTextPlan> {
        self.implementation().plan_text(text, horizontal, vertical)
    }

    /// `RenderText(thePlan)` with default options; `font.render_plan(&plan)`.
    pub fn render_plan(&self, plan: &BRepFontBuilderTextPlan) -> Shape {
        self.render_plan_with(plan, &StdPrsBRepFontTextOptions::default())
    }

    /// `RenderText(thePlan, theOptions)`: one shape per plan region (rebuilt from this font's
    /// glyph when cached, else from the plan region), placed per item.
    /// Example: `font.render_plan_with(&plan, &options)`.
    pub fn render_plan_with(
        &self,
        plan: &BRepFontBuilderTextPlan,
        options: &StdPrsBRepFontTextOptions,
    ) -> Shape {
        if !is_valid_position(&options.pen) || plan.nb_regions() == 0 {
            return Shape::default();
        }
        let implementation = self.implementation();
        let concatenate = options.to_concatenate_contours;
        let mut glyphs = Vec::with_capacity(plan.nb_regions());
        for index in 0..plan.nb_regions() {
            let glyph = implementation
                .render_region(plan.region_handle(index), plan.size(), concatenate)
                .unwrap_or_else(|| {
                    let glyph_options = BRepFontBuilderGlyphOptions {
                        size: plan.size(),
                        tolerance: plan.tolerance(),
                        to_concatenate_contours: concatenate,
                    };
                    BRepFontBuilder::build_glyph(plan.region(index), &glyph_options)
                });
            if glyph.is_null() && !plan.region(index).is_empty() {
                return Shape::default();
            }
            glyphs.push(glyph);
        }
        let builder_options = BRepFontBuilderTextOptions {
            pen: options.pen,
            to_concatenate_contours: concatenate,
        };
        BRepFontBuilder::build_text_from_glyphs(plan, &glyphs, &builder_options)
    }

    /// `SetWidthScaling`: rejects non-positive or infinite factors; clears the glyph cache.
    /// Example: `font.set_width_scaling(1.25)`.
    pub fn set_width_scaling(&self, factor: f32) -> bool {
        if factor <= 0.0 || factor.is_nan() || precision::is_infinite(f64::from(factor)) {
            return false;
        }
        if self.width_scaling() == factor {
            return true;
        }
        self.modify(|implementation| implementation.font.set_width_scaling(factor))
    }

    /// `SetSingleStrokeFont`: switches the outline interpretation; clears the glyph cache.
    /// Example: `font.set_single_stroke_font(true)`.
    pub fn set_single_stroke_font(&self, is_single_stroke: bool) {
        {
            let implementation = self.implementation();
            let _lock = implementation.lock_font();
            if implementation.font.is_single_stroke_font() == is_single_stroke {
                return;
            }
        }
        self.modify(|implementation| {
            implementation.font.set_single_stroke_font(is_single_stroke);
            true
        });
    }

    // Copy-on-write: change the implementation in place when unshared, else a copy.
    fn modify(&self, change: impl FnOnce(&StdPrsBRepFontImpl) -> bool) -> bool {
        let mut slot = self.implementation.write();
        let implementation = if Arc::strong_count(&slot) == 1 {
            slot.clone()
        } else {
            match slot.copy() {
                Some(copy) => Arc::new(copy),
                None => return false,
            }
        };
        let _lock = implementation.lock_font();
        if !change(&implementation) {
            return false;
        }
        implementation.clear_cache();
        drop(_lock);
        *slot = implementation;
        true
    }

    /// `Ascender`; `font.ascender()`.
    pub fn ascender(&self) -> f64 {
        self.implementation().metrics.ascender
    }
    /// `Descender`; `font.descender()`.
    pub fn descender(&self) -> f64 {
        self.implementation().metrics.descender
    }
    /// `LineSpacing`; `font.line_spacing()`.
    pub fn line_spacing(&self) -> f64 {
        self.implementation().metrics.line_spacing
    }
    /// `PointSize`; `font.point_size()`.
    pub fn point_size(&self) -> f64 {
        self.implementation().metrics.point_size
    }
    /// `AdvanceX(theUCharNext)`; `font.advance_x('A' as u32)`.
    pub fn advance_x(&self, next: u32) -> f64 {
        let implementation = self.implementation();
        let _lock = implementation.lock_font();
        implementation.metrics.scale * f64::from(implementation.font.advance_x(next))
    }
    /// `AdvanceX(theUChar, theUCharNext)`; `font.advance_x_of('A' as u32, 'V' as u32)`.
    pub fn advance_x_of(&self, character: u32, next: u32) -> f64 {
        let implementation = self.implementation();
        let _lock = implementation.lock_font();
        implementation.metrics.scale * f64::from(implementation.font.advance_x_of(character, next))
    }
    /// `AdvanceY(theUCharNext)`; `font.advance_y('A' as u32)`.
    pub fn advance_y(&self, next: u32) -> f64 {
        let implementation = self.implementation();
        let _lock = implementation.lock_font();
        implementation.metrics.scale * f64::from(implementation.font.advance_y(next))
    }
    /// `AdvanceY(theUChar, theUCharNext)`; `font.advance_y_of('A' as u32, 'V' as u32)`.
    pub fn advance_y_of(&self, character: u32, next: u32) -> f64 {
        let implementation = self.implementation();
        let _lock = implementation.lock_font();
        implementation.metrics.scale * f64::from(implementation.font.advance_y_of(character, next))
    }
    /// `Scale`: the size over the internal metric size; `font.scale()`.
    pub fn scale(&self) -> f64 {
        self.implementation().metrics.scale
    }
    /// `Size`; `font.size()`.
    pub fn size(&self) -> f64 {
        self.implementation().metrics.size
    }
}
