//! `Font_TextFormatter::Format` and the line and glyph queries over the formatted corners.
use super::text_formatter::{TextFormatterState, corner};
use super::{
    FontRect, FontTextFormatter, FontTextFormatterIterationFilter, FontTextFormatterIterator,
};
use crate::cast::int_from_f32;
use crate::graphic3d::{
    Graphic3dHorizontalTextAlignment as AlignX, Graphic3dVerticalTextAlignment as AlignY,
};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// `std::max(float, float)`.
fn max_f32(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

/// `LineWidth`: the pen advance of a line (0 outside the lines).
fn line_width(state: &TextFormatterState, index: i32) -> f32 {
    let Ok(i) = usize::try_from(index) else {
        return 0.0;
    };
    let lines = &state.new_lines;
    if i < lines.len() {
        return if i == 0 {
            lines[0]
        } else {
            lines[i] - lines[i - 1]
        };
    }
    if i == lines.len() {
        return if i == 0 {
            state.pen.x
        } else {
            state.pen.x - lines[i - 1]
        };
    }
    0.0
}

/// `LineIndex`: the line of a glyph from its baseline (0 for a negative spacing).
fn line_index(state: &TextFormatterState, index: i32) -> Result<i32, Failure> {
    if state.line_spacing < 0.0 {
        return Ok(0);
    }
    let y = corner(state, index)?.y;
    Ok(int_from_f32(
        ((y + state.ascender) / state.line_spacing).abs(),
    ))
}

/// `GlyphBoundingBox`: the glyph cell up to the next glyph (or to the line end by the
/// alignment); false for an index out of range.
fn glyph_bounding_box(
    state: &TextFormatterState,
    index: i32,
    rect: &mut FontRect,
) -> Result<bool, Failure> {
    if index < 0 || index as usize >= state.corners.len() {
        return Ok(false);
    }
    let left = corner(state, index)?;
    (rect.left, rect.right) = (left.x, left.x + state.last_symbol_width);
    (rect.bottom, rect.top) = (left.y, left.y + state.line_spacing);
    if index as usize + 1 >= state.corners.len() {
        return Ok(true);
    }
    let next = corner(state, index + 1)?;
    if f64::from((left.y - next.y).abs()) < precision::confusion() {
        rect.right = next.x;
        return Ok(true);
    }
    let width = line_width(state, line_index(state, index)?);
    rect.left = left.x;
    rect.right = match state.align_x {
        AlignX::Left => width,
        AlignX::Right => state.bnd_width,
        AlignX::Center => 0.5 * (state.bnd_width + width),
    };
    Ok(true)
}

/// `newLine`: aligns the glyphs of the line ending at `last_rect` and moves to the next line.
fn new_line(
    state: &mut TextFormatterState,
    last_rect: i32,
    max_line_width: f32,
) -> Result<(), Failure> {
    let first = state.rect_line_start;
    if first as usize >= state.corners.len() {
        state.lines_nb += 1;
        state.pen_curr_line -= state.line_spacing;
        return Ok(());
    }
    let x_min = corner(state, first)?.x;
    let mut bounds = FontRect::default();
    glyph_bounding_box(state, last_rect, &mut bounds)?;
    let x_max = bounds.right;
    state.move_vec.y = state.pen_curr_line;
    state.move_vec.x = match state.align_x {
        AlignX::Left => -x_min,
        AlignX::Right => -x_min + (max_line_width - (x_max - x_min)) - max_line_width,
        AlignX::Center => -x_min + 0.5 * (max_line_width - (x_max - x_min)) - 0.5 * max_line_width,
    };
    let shift = state.move_vec;
    for k in first..=last_rect {
        let c = &mut state.corners[k as usize];
        *c = Vec2::new(c.x + shift.x, c.y + shift.y);
    }
    state.lines_nb += 1;
    state.pen_curr_line -= state.line_spacing;
    state.rect_line_start = last_rect + 1;
    Ok(())
}

/// The maximum line width of `Format`: the wrapping width (at least the widest symbol), or
/// the widest line.
fn max_line_width(state: &TextFormatterState) -> f32 {
    let wrapping = state.wrapping_width;
    if wrapping > 0.0 {
        return max_f32(wrapping, state.max_symbol_width);
    }
    if state.new_lines.is_empty() {
        return state.pen.x;
    }
    let count = state.new_lines.len() as i32;
    let widest = (0..count).fold(wrapping, |acc, k| max_f32(acc, line_width(state, k)));
    max_f32(widest, line_width(state, count))
}

/// Whether the word starting at the iterator still fits on the line.
fn word_fits(
    state: &TextFormatterState,
    from: &FontTextFormatterIterator,
    rect_iter: i32,
    next_x: f32,
    max_width: f32,
) -> bool {
    let mut word = from.clone();
    while word.more() {
        if FontTextFormatter::is_separator_symbol(word.symbol()) {
            return true;
        }
        let width =
            state.corners[word.symbol_position() as usize].x - state.corners[rect_iter as usize].x;
        if next_x + width > max_width {
            return false;
        }
        word.next();
    }
    true
}

/// The wrapping test of one glyph of `Format`.
fn wrap_glyph(
    state: &mut TextFormatterState,
    it: &FontTextFormatterIterator,
    char_prev: u32,
    max_width: f32,
) -> Result<(), Failure> {
    let rect_iter = it.symbol_position();
    let mut bounds = FontRect::default();
    glyph_bounding_box(state, rect_iter, &mut bounds)?;
    let next_x = bounds.right - corner(state, state.rect_line_start)?.x;
    let fits = !(state.is_word_wrapping && FontTextFormatter::is_separator_symbol(char_prev))
        || word_fits(state, it, rect_iter, next_x, max_width);
    if next_x > max_width || !fits {
        new_line(state, rect_iter - 1, max_width)?;
    }
    Ok(())
}

impl FontTextFormatter {
    /// `Format`: splits the glyphs into lines (line feeds and wrapping), aligns them and
    /// computes the box; nothing for an empty or already formatted text.
    /// Example: `formatter.format()?`.
    pub fn format(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.corners.is_empty() || state.is_formatted {
            return Ok(());
        }
        state.is_formatted = true;
        (state.lines_nb, state.rect_line_start, state.new_line_nb) = (0, 0, 0);
        (state.bnd_top, state.bnd_width) = (0.0, 0.0);
        state.move_vec = Vec2::new(0.0, 0.0);
        state.pen_curr_line = -state.ascender;
        let max_width = max_line_width(&state);
        let mut char_prev = 0;
        let mut it = FontTextFormatterIterator::over(
            state.text.clone(),
            FontTextFormatterIterationFilter::None,
        );
        while it.more() {
            let this = it.symbol();
            if this == 0x0A {
                new_line(&mut state, it.symbol_position(), max_width)?;
                state.new_line_nb += 1;
                it.next();
                continue;
            }
            if state.wrapping_width > 0.0 {
                wrap_glyph(&mut state, &it, char_prev, max_width)?;
            }
            char_prev = this;
            it.next();
        }
        state.bnd_width = max_width;
        let last = state.corners.len() as i32 - 1;
        new_line(&mut state, last, max_width)?;
        align_vertically(&mut state);
        Ok(())
    }
}

/// The vertical alignment of `Format`.
fn align_vertically(state: &mut TextFormatterState) {
    state.bnd_top = match state.align_y {
        AlignY::Bottom => -state.line_spacing - state.pen_curr_line,
        AlignY::Center => 0.5 * (state.line_spacing * state.lines_nb as f32),
        AlignY::Topfirstline => state.ascender,
        AlignY::Top => state.bnd_top,
    };
    if state.align_y != AlignY::Top {
        let top = state.bnd_top;
        state.corners.iter_mut().for_each(|c| c.y += top);
    }
}

impl FontTextFormatter {
    /// `GlyphBoundingBox`; `formatter.glyph_bounding_box(0, &mut rect)?`.
    pub fn glyph_bounding_box(&self, index: i32, rect: &mut FontRect) -> Result<bool, Failure> {
        glyph_bounding_box(&self.state.read(), index, rect)
    }
    /// `LineWidth`; `formatter.line_width(0)`.
    pub fn line_width(&self, index: i32) -> f32 {
        line_width(&self.state.read(), index)
    }
    /// `IsLFSymbol`: a glyph cell of zero width; `formatter.is_lf_symbol(3)?`.
    pub fn is_lf_symbol(&self, index: i32) -> Result<bool, Failure> {
        let mut rect = FontRect::default();
        if !self.glyph_bounding_box(index, &mut rect)? {
            return Ok(false);
        }
        Ok(f64::from((rect.right - rect.left).abs()) < precision::confusion())
    }
    /// `FirstPosition`: the X of the line start by the alignment; `formatter.first_position()`.
    pub fn first_position(&self) -> f32 {
        let state = self.state.read();
        match state.align_x {
            AlignX::Left => 0.0,
            AlignX::Right => state.bnd_width,
            AlignX::Center => 0.5 * state.bnd_width,
        }
    }
    /// `LinePositionIndex`: the glyph's position in its line; `formatter.line_position_index(4)?`.
    pub fn line_position_index(&self, index: i32) -> Result<i32, Failure> {
        let state = self.state.read();
        let height = corner(&state, index)?.y;
        let mut position = 0;
        for previous in (0..index).rev() {
            if corner(&state, previous)?.y > height {
                break;
            }
            position += 1;
        }
        Ok(position)
    }
    /// `LineIndex`; `formatter.line_index(4)?`.
    pub fn line_index(&self, index: i32) -> Result<i32, Failure> {
        line_index(&self.state.read(), index)
    }
}
