//! The glyph outlines of `Font_FTFont`: `FT_Outline_Decompose` into line and Bezier segments
//! grouped in contours (`decomposeOutline` and its callbacks).
#![allow(unsafe_code)]
use super::freetype_ffi::{
    FT_ERR_ARRAY_TOO_LARGE, FT_ERR_INVALID_ARGUMENT, FT_ERR_INVALID_OUTLINE, FT_ERR_OK,
    FT_FACE_FLAG_TRICKY, FT_GLYPH_FORMAT_OUTLINE, FT_LOAD_NO_BITMAP, FT_LOAD_NO_HINTING,
    FT_LOAD_NO_SCALE, FT_OUTLINE_EVEN_ODD_FILL, FT_OUTLINE_OVERLAP, FtFaceRec, FtOutline,
    FtOutlineFuncs, FtVector, freetype,
};
use super::ft_font::{FontFTFont, LoadedGlyph, face_rec};
use super::glyph_outline::{
    FontGlyphOutline, FontGlyphOutlineContour, FontGlyphOutlineFillRule, FontGlyphOutlineGlyph,
    FontGlyphOutlineSegment,
};
use rustycad_tkmath::gp::Xy;
use std::ffi::{c_int, c_void};

/// `OutlineContext`: the decomposition state shared with the callbacks.
struct OutlineContext {
    segments: Vec<FontGlyphOutlineSegment>,
    contours: Vec<FontGlyphOutlineContour>,
    nb_segments: usize,
    nb_contours: usize,
    current: Xy,
    contour_start: Xy,
    first_segment: u32,
    coordinate_scale: f64,
    width_scale: f64,
    has_contour: bool,
    is_single_stroke: bool,
}

impl OutlineContext {
    /// `fromOutlinePoint`: font units scaled, X also by the width scaling.
    fn point(&self, v: &FtVector) -> Xy {
        Xy::new(
            v.x as f64 * self.coordinate_scale * self.width_scale,
            v.y as f64 * self.coordinate_scale,
        )
    }
    /// Appends a segment; false when the preallocated storage is full.
    fn push(&mut self, segment: FontGlyphOutlineSegment) -> bool {
        let Some(slot) = self.segments.get_mut(self.nb_segments) else {
            return false;
        };
        *slot = segment;
        self.nb_segments += 1;
        self.current = segment.end();
        true
    }
    /// `finishOutlineContour`: closes a filled contour with a line and records it.
    fn finish_contour(&mut self) -> bool {
        if !self.has_contour {
            return true;
        }
        let gap = self.current - self.contour_start;
        if !self.is_single_stroke && gap.square_modulus() > 0.0 {
            let (start, end) = (self.current, self.contour_start);
            if !self.push(FontGlyphOutlineSegment::line(start, end)) {
                return false;
            }
            self.current = start;
        }
        let first = self.first_segment as usize;
        let Some(count) = self
            .nb_segments
            .checked_sub(first)
            .and_then(|n| u32::try_from(n).ok())
        else {
            return false;
        };
        if count > 0 {
            let Some(slot) = self.contours.get_mut(self.nb_contours) else {
                return false;
            };
            *slot = FontGlyphOutlineContour::new(self.first_segment, count);
            self.nb_contours += 1;
        }
        self.has_contour = false;
        true
    }
}

/// The context of a callback's user pointer.
///
/// # Safety
/// `user` is the `OutlineContext` that `decompose` passes to `FT_Outline_Decompose`.
unsafe fn context<'a>(user: *mut c_void) -> Option<&'a mut OutlineContext> {
    // SAFETY: per the contract, `user` is null or the live context of this decomposition.
    unsafe { user.cast::<OutlineContext>().as_mut() }
}

/// `moveToOutline`.
unsafe extern "C" fn move_to(point: *const FtVector, user: *mut c_void) -> c_int {
    // SAFETY: FreeType passes the context given to FT_Outline_Decompose and a valid point.
    let (Some(ctx), Some(point)) = (unsafe { context(user) }, unsafe { point.as_ref() }) else {
        return FT_ERR_INVALID_ARGUMENT;
    };
    if !ctx.finish_contour() {
        return FT_ERR_INVALID_OUTLINE;
    }
    let Ok(first) = u32::try_from(ctx.nb_segments) else {
        return FT_ERR_ARRAY_TOO_LARGE;
    };
    ctx.current = ctx.point(point);
    (ctx.contour_start, ctx.first_segment, ctx.has_contour) = (ctx.current, first, true);
    FT_ERR_OK
}

/// `lineToOutline`.
unsafe extern "C" fn line_to(point: *const FtVector, user: *mut c_void) -> c_int {
    // SAFETY: as in `move_to`.
    let (Some(ctx), Some(point)) = (unsafe { context(user) }, unsafe { point.as_ref() }) else {
        return FT_ERR_INVALID_ARGUMENT;
    };
    let segment = FontGlyphOutlineSegment::line(ctx.current, ctx.point(point));
    if ctx.push(segment) {
        FT_ERR_OK
    } else {
        FT_ERR_INVALID_OUTLINE
    }
}

/// `conicToOutline`.
unsafe extern "C" fn conic_to(
    control: *const FtVector,
    point: *const FtVector,
    user: *mut c_void,
) -> c_int {
    // SAFETY: as in `move_to`.
    let (Some(ctx), Some(control), Some(point)) = (
        unsafe { context(user) },
        unsafe { control.as_ref() },
        unsafe { point.as_ref() },
    ) else {
        return FT_ERR_INVALID_ARGUMENT;
    };
    let segment =
        FontGlyphOutlineSegment::quadratic(ctx.current, ctx.point(control), ctx.point(point));
    if ctx.push(segment) {
        FT_ERR_OK
    } else {
        FT_ERR_INVALID_OUTLINE
    }
}

/// `cubicToOutline`.
unsafe extern "C" fn cubic_to(
    c1: *const FtVector,
    c2: *const FtVector,
    point: *const FtVector,
    user: *mut c_void,
) -> c_int {
    // SAFETY: as in `move_to`.
    let refs = unsafe { (context(user), c1.as_ref(), c2.as_ref(), point.as_ref()) };
    let (Some(ctx), Some(c1), Some(c2), Some(point)) = refs else {
        return FT_ERR_INVALID_ARGUMENT;
    };
    let segment =
        FontGlyphOutlineSegment::cubic(ctx.current, ctx.point(c1), ctx.point(c2), ctx.point(point));
    if ctx.push(segment) {
        FT_ERR_OK
    } else {
        FT_ERR_INVALID_OUTLINE
    }
}

/// The load flags and coordinate scale of the outline of a face (tricky faces load scaled).
fn outline_load(face: &FtFaceRec, regular_flags: i32) -> Option<(i32, f64)> {
    if face.face_flags & FT_FACE_FLAG_TRICKY == 0 {
        return Some((
            FT_LOAD_NO_SCALE | FT_LOAD_NO_HINTING | FT_LOAD_NO_BITMAP,
            1.0,
        ));
    }
    // SAFETY: the size of an open face is null or valid.
    let size = unsafe { face.size.as_ref() }.filter(|s| s.metrics.y_ppem != 0)?;
    let scale = f64::from(face.units_per_em) / (f64::from(size.metrics.y_ppem) * 64.0);
    Some((regular_flags | FT_LOAD_NO_BITMAP, scale))
}

/// `decomposeOutline`: the outline of a glyph index, `None` when FreeType cannot give it.
pub(crate) fn decompose(
    face_address: usize,
    index: u32,
    (single_stroke, regular_flags): (bool, i32),
    width_scale: f64,
) -> Option<FontGlyphOutline> {
    let face = face_rec(face_address).filter(|f| f.units_per_em != 0)?;
    let api = freetype()?;
    let (flags, coordinate_scale) = outline_load(face, regular_flags)?;
    // SAFETY: an open face; FT_Load_Glyph fills its glyph slot.
    if unsafe { (api.FT_Load_Glyph)(face_address as *mut FtFaceRec, index, flags) } != 0 {
        return None;
    }
    // SAFETY: the glyph slot of an open face is null or valid until the next load.
    let slot = unsafe { face.glyph.as_mut() }.filter(|g| g.format == FT_GLYPH_FORMAT_OUTLINE)?;
    let outline: *mut FtOutline = &mut slot.outline;
    // SAFETY: the outline of the loaded slot.
    if unsafe { (api.FT_Outline_Check)(outline) } != 0 {
        return None;
    }
    // SAFETY: as above; the outline is only read.
    let record = unsafe { &*outline };
    let mut ctx = new_context(record, coordinate_scale, width_scale, single_stroke);
    let funcs = FtOutlineFuncs {
        move_to,
        line_to,
        conic_to,
        cubic_to,
        shift: 0,
        delta: 0,
    };
    // SAFETY: the callbacks receive `ctx`, which outlives the call.
    let error = unsafe {
        (api.FT_Outline_Decompose)(outline, &funcs, (&mut ctx as *mut OutlineContext).cast())
    };
    if error != 0 || !ctx.finish_contour() {
        return None;
    }
    Some(finish_outline(
        ctx,
        f64::from(face.units_per_em),
        record.flags,
        single_stroke,
    ))
}

/// The preallocated context of an outline (points plus one closing line per contour).
fn new_context(
    outline: &FtOutline,
    coordinate_scale: f64,
    width_scale: f64,
    is_single_stroke: bool,
) -> OutlineContext {
    let nb_contours = usize::try_from(outline.n_contours.max(0)).unwrap_or(0);
    let nb_points = usize::try_from(outline.n_points).unwrap_or(0);
    let segments = if nb_points > 0 {
        vec![FontGlyphOutlineSegment::default(); nb_points + nb_contours]
    } else {
        Vec::new()
    };
    OutlineContext {
        segments,
        contours: vec![FontGlyphOutlineContour::default(); nb_contours],
        nb_segments: 0,
        nb_contours: 0,
        current: Xy::default(),
        contour_start: Xy::default(),
        first_segment: 0,
        coordinate_scale,
        width_scale,
        has_contour: false,
        is_single_stroke,
    }
}

fn finish_outline(
    mut ctx: OutlineContext,
    units_per_em: f64,
    flags: c_int,
    is_single_stroke: bool,
) -> FontGlyphOutline {
    ctx.segments.truncate(ctx.nb_segments);
    ctx.contours.truncate(ctx.nb_contours);
    let fill_rule = if flags & FT_OUTLINE_EVEN_ODD_FILL != 0 {
        FontGlyphOutlineFillRule::EvenOdd
    } else {
        FontGlyphOutlineFillRule::NonZero
    };
    let overlaps = flags & FT_OUTLINE_OVERLAP != 0;
    FontGlyphOutline::create(
        units_per_em,
        fill_rule,
        overlaps,
        is_single_stroke,
        ctx.segments,
        ctx.contours,
    )
}

impl FontFTFont {
    /// `LoadGlyphOutline(theUChar)`: the outline of the glyph that draws the character.
    /// Example: `font.load_glyph_outline_char('A' as u32)`.
    pub fn load_glyph_outline_char(&self, character: u32) -> Option<FontGlyphOutline> {
        let glyph = self.resolve_glyph(character)?;
        self.load_glyph_outline(&glyph)
    }
    /// `LoadGlyphOutline(theGlyph)`: the outline of a resolved glyph (scaled in X by the
    /// width scaling), `None` for a stale or invalid glyph; the loaded bitmap is cleared.
    /// Example: `font.load_glyph_outline(&glyph)`.
    pub fn load_glyph_outline(&self, glyph: &FontGlyphOutlineGlyph) -> Option<FontGlyphOutline> {
        self.glyph_img.clear();
        let width_scale = {
            let mut state = self.state.write();
            state.loaded_glyph = LoadedGlyph::default();
            f64::from(state.width_scaling)
        };
        let font = self.font_for_glyph(glyph)?;
        let face = font.face(self);
        let record = face_rec(face).filter(|f| f.units_per_em != 0)?;
        if i128::from(glyph.index()) >= i128::from(record.num_glyphs) {
            return None;
        }
        decompose(face, glyph.index(), font.outline_flags(self), width_scale)
    }
}
