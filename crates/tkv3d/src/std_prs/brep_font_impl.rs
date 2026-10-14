//! `StdPrs_BRepFont::Impl`: the shared font state behind a `StdPrs_BRepFont` handle — the
//! FreeType font, its metrics and source, and the glyph region cache.
use super::brep_font_glyph_cache::StdPrsBRepFontGlyphCache;
use crate::brep_font::{
    BRepFontBuilder, BRepFontBuilderGlyphOptions, BRepFontBuilderTextPlan,
    BRepFontBuilderTextPlanItem, BRepFontPlanarRegion, BRepFontRegularizer,
    BRepFontRegularizerOptions,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::precision;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Xy;
use rustycad_tkservice::font::{
    FontFTFont, FontFTFontParams, FontFontAspect, FontGlyphOutlineGlyph, FontRect, FontStrictLevel,
    FontTextFormatter, FontTextFormatterIterationFilter, FontTextFormatterIterator,
};
use rustycad_tkservice::graphic3d::{
    Graphic3dHorizontalTextAlignment, Graphic3dVerticalTextAlignment,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

// A 72-point font is one inch high. The high resolution reduces rounding in FreeType text metrics.
const POINTS_PER_INCH: u32 = 72;
const FONT_POINT_SIZE: u32 = 72;
const FONT_RESOLUTION: u32 = 4800;
const FONT_METRIC_SIZE: f64 =
    FONT_POINT_SIZE as f64 * FONT_RESOLUTION as f64 / POINTS_PER_INCH as f64;

/// `THE_FONT_PARAMETERS`: 72 points at 4800 dpi.
pub(super) fn font_parameters() -> FontFTFontParams {
    FontFTFontParams {
        point_size: FONT_POINT_SIZE,
        resolution: FONT_RESOLUTION,
        ..FontFTFontParams::default()
    }
}

/// `Impl::Source`: how the font was opened, to reopen it in a copy.
#[derive(Clone, Debug, Default)]
pub(super) enum StdPrsBRepFontSource {
    /// `std::monostate`.
    #[default]
    None,
    /// `FileSource`.
    File { path: AsciiString, face_id: i32 },
    /// `NamedSource`.
    Named {
        name: AsciiString,
        aspect: FontFontAspect,
        strict_level: FontStrictLevel,
    },
}

/// `Impl::Metrics`: the font metrics scaled to the requested size.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct StdPrsBRepFontMetrics {
    pub(super) size: f64,
    pub(super) scale: f64,
    pub(super) ascender: f64,
    pub(super) descender: f64,
    pub(super) line_spacing: f64,
    pub(super) point_size: f64,
}

/// `Impl::Request`: a resolved glyph at a size.
#[derive(Clone, Debug)]
pub(super) struct StdPrsBRepFontRequest {
    pub(super) glyph: FontGlyphOutlineGlyph,
    pub(super) size: f64,
}

/// `StdPrs_BRepFont::Impl`. The font lock serializes the FreeType face as natively.
pub(super) struct StdPrsBRepFontImpl {
    pub(super) font: FontFTFont,
    glyphs: RwLock<StdPrsBRepFontGlyphCache>,
    font_lock: Mutex<()>,
    pub(super) tolerance: f64,
    pub(super) metrics: StdPrsBRepFontMetrics,
    pub(super) source: StdPrsBRepFontSource,
}

impl StdPrsBRepFontImpl {
    /// `Impl()`: a new invalid font with the confusion tolerance; `StdPrsBRepFontImpl::new()`.
    pub(super) fn new() -> Self {
        Self {
            font: FontFTFont::default(),
            glyphs: RwLock::new(StdPrsBRepFontGlyphCache::default()),
            font_lock: Mutex::new(()),
            tolerance: precision::confusion(),
            metrics: StdPrsBRepFontMetrics {
                scale: 1.0,
                ..StdPrsBRepFontMetrics::default()
            },
            source: StdPrsBRepFontSource::None,
        }
    }

    /// The font lock (`std::lock_guard<std::mutex>` on `FontMutex`).
    pub(super) fn lock_font(&self) -> MutexGuard<'_, ()> {
        self.font_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    /// `ClearCache`; `inner.clear_cache()`.
    pub(super) fn clear_cache(&self) {
        self.glyphs
            .write()
            .unwrap_or_else(|poison| poison.into_inner())
            .clear();
    }

    fn find_region(&self, glyph: &FontGlyphOutlineGlyph) -> Option<Arc<BRepFontPlanarRegion>> {
        self.glyphs
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .find_region(glyph)
    }

    /// `Copy`: a new implementation reopening the same source with the same options;
    /// None when reopening fails. Example: `inner.copy()`.
    pub(super) fn copy(&self) -> Option<Self> {
        let _lock = self.lock_font();
        let mut copy = Self::new();
        copy.font
            .set_use_unicode_subset_fallback(self.font.to_use_unicode_subset_fallback());
        let initialized = match &self.source {
            StdPrsBRepFontSource::File { path, face_id } => {
                copy.font.init(path, &font_parameters(), *face_id)
            }
            StdPrsBRepFontSource::Named {
                name,
                aspect,
                strict_level,
            } => copy
                .font
                .find_and_init(name, *aspect, &font_parameters(), *strict_level),
            StdPrsBRepFontSource::None => !self.font.is_valid(),
        };
        if !initialized {
            return None;
        }
        copy.font
            .set_single_stroke_font(self.font.is_single_stroke_font());
        if !copy.font.set_width_scaling(self.font.width_scaling()) {
            return None;
        }
        copy.set_size(self.metrics.size);
        copy.source = self.source.clone();
        Some(copy)
    }

    /// `SetSize`: the metrics of the font scaled to `size`; `inner.set_size(12.0)`.
    pub(super) fn set_size(&mut self, size: f64) {
        let scale = size / FONT_METRIC_SIZE;
        self.metrics = StdPrsBRepFontMetrics {
            size,
            scale,
            ascender: scale * f64::from(self.font.ascender()),
            descender: scale * f64::from(self.font.descender()),
            line_spacing: scale * f64::from(self.font.line_spacing()),
            point_size: scale * f64::from(self.font.point_size()),
        };
    }

    /// `MakeRequest`: the resolved glyph of a character at the font size.
    pub(super) fn make_request(&self, character: u32) -> Option<StdPrsBRepFontRequest> {
        let _lock = self.lock_font();
        if self.metrics.size <= 0.0 || !self.font.is_valid() {
            return None;
        }
        let glyph = self.font.resolve_glyph(character)?;
        Some(StdPrsBRepFontRequest {
            glyph,
            size: self.metrics.size,
        })
    }

    /// `LoadRegion`: the cached region of a glyph, else its regularized outline (cached
    /// when the glyph is still valid). Example: `inner.load_region(&request)`.
    pub(super) fn load_region(
        &self,
        request: &StdPrsBRepFontRequest,
    ) -> Option<Arc<BRepFontPlanarRegion>> {
        if let Some(cached) = self.find_region(&request.glyph) {
            return Some(cached);
        }
        let outline = {
            let _lock = self.lock_font();
            self.font.load_glyph_outline(&request.glyph)?
        };
        let tolerance = self.tolerance * outline.units_per_em() / request.size;
        let mut result =
            BRepFontRegularizer.build_with(&outline, &BRepFontRegularizerOptions { tolerance });
        if !result.is_done() {
            return None;
        }
        let region = Arc::new(result.change_planar_region().take()?);
        let _font_lock = self.lock_font();
        if !self.font.is_glyph_valid(&request.glyph) {
            return None;
        }
        let mut cache = self
            .glyphs
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(existing) = cache.find_region(&request.glyph) {
            return Some(existing);
        }
        cache.bind_region(&request.glyph, &region);
        Some(region)
    }

    /// `RenderGlyph(theRequest, theToConcatenateContours)`: a new glyph shape, null when
    /// the glyph is invalid. Example: `inner.render_glyph(&request, false)`.
    pub(super) fn render_glyph(&self, request: &StdPrsBRepFontRequest, concatenate: bool) -> Shape {
        let Some(region) = self.load_region(request) else {
            return Shape::default();
        };
        let options = BRepFontBuilderGlyphOptions {
            size: request.size,
            tolerance: self.tolerance,
            to_concatenate_contours: concatenate,
        };
        let shape = BRepFontBuilder::build_glyph(&region, &options);
        let _lock = self.lock_font();
        if self.font.is_glyph_valid(&request.glyph) {
            shape
        } else {
            Shape::default()
        }
    }

    /// `RenderRegion`: rebuilds a plan region from its cached glyph; None when the region
    /// is not one of this font's cached regions. Example: `inner.render_region(&region, 12.0, false)`.
    pub(super) fn render_region(
        &self,
        region: &Arc<BRepFontPlanarRegion>,
        size: f64,
        concatenate: bool,
    ) -> Option<Shape> {
        let glyph = self
            .glyphs
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .find_glyph(region)?;
        Some(self.render_glyph(&StdPrsBRepFontRequest { glyph, size }, concatenate))
    }

    /// `PlanText`: the formatted layout of a text with one shared region per glyph.
    /// A formatter exception (a string of command symbols only) gives no plan.
    /// Example: `inner.plan_text(&text, Graphic3dHorizontalTextAlignment::Left, Graphic3dVerticalTextAlignment::Bottom)`.
    pub(super) fn plan_text(
        &self,
        text: &String8,
        horizontal: Graphic3dHorizontalTextAlignment,
        vertical: Graphic3dVerticalTextAlignment,
    ) -> Option<BRepFontBuilderTextPlan> {
        if text.is_empty() {
            return None;
        }
        let layout = self.layout_text(text, horizontal, vertical)?;
        let mut indices: HashMap<usize, Vec<(FontGlyphOutlineGlyph, usize)>> = HashMap::new();
        let (mut regions, mut items) = (Vec::new(), Vec::new());
        for (glyph, position) in &layout.pending {
            let key = rustycad_tkservice::font::FontGlyphOutlineGlyphHasher.hash(glyph);
            let found = indices
                .get(&key)
                .and_then(|bucket| bucket.iter().find(|(known, _)| known == glyph))
                .map(|(_, index)| *index);
            let index = match found {
                Some(index) => index,
                None => {
                    let Some(region) = self.load_region(&StdPrsBRepFontRequest {
                        glyph: glyph.clone(),
                        size: layout.size,
                    }) else {
                        continue;
                    };
                    regions.push(region);
                    indices
                        .entry(key)
                        .or_default()
                        .push((glyph.clone(), regions.len() - 1));
                    regions.len() - 1
                }
            };
            items.push(BRepFontBuilderTextPlanItem::new(index, *position));
        }
        BRepFontBuilder::create_text_plan(
            layout.size,
            self.tolerance,
            layout.lower_left,
            layout.upper_right,
            regions,
            items,
        )
    }

    // The formatter pass of PlanText, under the font lock.
    fn layout_text(
        &self,
        text: &String8,
        horizontal: Graphic3dHorizontalTextAlignment,
        vertical: Graphic3dVerticalTextAlignment,
    ) -> Option<StdPrsBRepFontLayout> {
        let _lock = self.lock_font();
        if self.metrics.size <= 0.0 || !self.font.is_valid() {
            return None;
        }
        let scale = self.metrics.scale;
        let formatter = FontTextFormatter::default();
        formatter.setup_alignment(horizontal, vertical);
        formatter.append(text, &self.font).ok()?;
        formatter.format().ok()?;
        let mut bounds = FontRect::default();
        formatter.bnd_box(&mut bounds);
        let (left, right) = (
            f64::from(bounds.left) * scale,
            f64::from(bounds.right) * scale,
        );
        let (bottom, top) = (
            f64::from(bounds.bottom) * scale,
            f64::from(bounds.top) * scale,
        );
        let mut pending = Vec::with_capacity(64);
        let mut iterator = FontTextFormatterIterator::new(
            &formatter,
            FontTextFormatterIterationFilter::ExcludeInvisible,
        );
        while iterator.more() {
            if let Some(glyph) = self.font.resolve_glyph(iterator.symbol()) {
                let corner = formatter.bottom_left(iterator.symbol_position()).ok()?;
                pending.push((
                    glyph,
                    Xy::new(f64::from(corner.x()) * scale, f64::from(corner.y()) * scale),
                ));
            }
            iterator.next();
        }
        Some(StdPrsBRepFontLayout {
            size: self.metrics.size,
            lower_left: Xy::new(left.min(right), bottom.min(top)),
            upper_right: Xy::new(left.max(right), bottom.max(top)),
            pending,
        })
    }
}

/// The formatted bounds and pending glyph placements of a text.
struct StdPrsBRepFontLayout {
    size: f64,
    lower_left: Xy,
    upper_right: Xy,
    pending: Vec<(FontGlyphOutlineGlyph, Xy)>,
}
