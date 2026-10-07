//! `Font_TextFormatter`: lays out a UTF-8 string with a font into glyph corners (pen
//! positions) and lines, with tabulations, wrapping and alignment.
use super::{FontFTFont, FontRect};
use crate::graphic3d::{Graphic3dHorizontalTextAlignment, Graphic3dVerticalTextAlignment};
use rustycad_tkernel::ncollection::{String8, UtfIterator, Vec2};
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};

/// `Font_TextFormatter::IterationFilter`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontTextFormatterIterationFilter {
    /// `IterationFilter_None`.
    None = 0x0000,
    /// `IterationFilter_ExcludeInvisible`: skips `' '`, `'\t'` and `'\n'`.
    ExcludeInvisible = 0x0002,
}

/// The formatter fields, shared by every handle.
#[derive(Clone, Debug)]
pub(crate) struct TextFormatterState {
    pub(crate) align_x: Graphic3dHorizontalTextAlignment,
    pub(crate) align_y: Graphic3dVerticalTextAlignment,
    pub(crate) tab_size: i32,
    pub(crate) wrapping_width: f32,
    pub(crate) is_word_wrapping: bool,
    pub(crate) last_symbol_width: f32,
    pub(crate) max_symbol_width: f32,
    /// `myString` as code points (the UTF-8 text decoded up to its terminator).
    pub(crate) text: Vec<u32>,
    pub(crate) pen: Vec2<f32>,
    pub(crate) corners: Vec<Vec2<f32>>,
    pub(crate) new_lines: Vec<f32>,
    pub(crate) line_spacing: f32,
    pub(crate) ascender: f32,
    pub(crate) is_formatted: bool,
    pub(crate) lines_nb: i32,
    pub(crate) rect_line_start: i32,
    pub(crate) new_line_nb: i32,
    pub(crate) pen_curr_line: f32,
    pub(crate) bnd_top: f32,
    pub(crate) bnd_width: f32,
    pub(crate) move_vec: Vec2<f32>,
}

/// A text formatter; `let f = FontTextFormatter::default(); f.append(&text, &font)?; f.format()?;`.
pub struct FontTextFormatter {
    transient: StandardTransient,
    pub(crate) state: SharedState<TextFormatterState>,
}
rustycad_tkernel::impl_transient!(
    FontTextFormatter,
    "Font_TextFormatter",
    StandardTransient,
    transient
);

impl Default for FontTextFormatter {
    /// Left/top alignment, tabulations of 8 spaces, word wrapping on, no wrapping width.
    fn default() -> Self {
        let state = TextFormatterState {
            align_x: Graphic3dHorizontalTextAlignment::Left,
            align_y: Graphic3dVerticalTextAlignment::Top,
            tab_size: 8,
            wrapping_width: 0.0,
            is_word_wrapping: true,
            last_symbol_width: 0.0,
            max_symbol_width: 0.0,
            text: Vec::new(),
            pen: Vec2::new(0.0, 0.0),
            corners: Vec::new(),
            new_lines: Vec::new(),
            line_spacing: 0.0,
            ascender: 0.0,
            is_formatted: false,
            lines_nb: 0,
            rect_line_start: 0,
            new_line_nb: 0,
            pen_curr_line: 0.0,
            bnd_top: 0.0,
            bnd_width: 0.0,
            move_vec: Vec2::new(0.0, 0.0),
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
}

/// The code points of a UTF-8 string up to its terminator.
pub(crate) fn decode(text: &String8) -> Vec<u32> {
    let mut chars = Vec::new();
    let mut it = UtfIterator::new(text.to_c_string());
    while it.current() != 0 {
        chars.push(it.current());
        it.advance();
    }
    chars
}

impl FontTextFormatter {
    /// `IsCommandSymbol`: CR, BEL, FF, BS and VT; `FontTextFormatter::is_command_symbol(13)`.
    pub fn is_command_symbol(symbol: u32) -> bool {
        matches!(symbol, 0x0D | 0x07 | 0x0C | 0x08 | 0x0B)
    }
    /// `IsSeparatorSymbol`: LF, space and tab; `FontTextFormatter::is_separator_symbol(32)`.
    pub fn is_separator_symbol(symbol: u32) -> bool {
        matches!(symbol, 0x0A | 0x20 | 0x09)
    }
    /// Sets the alignments; `formatter.setup_alignment(Graphic3dHorizontalTextAlignment::Center, Graphic3dVerticalTextAlignment::Bottom)`.
    pub fn setup_alignment(
        &self,
        align_x: Graphic3dHorizontalTextAlignment,
        align_y: Graphic3dVerticalTextAlignment,
    ) {
        let mut state = self.state.write();
        (state.align_x, state.align_y) = (align_x, align_y);
    }
    /// `Reset`: forgets the text, glyphs and lines (the options stay); `formatter.reset()`.
    pub fn reset(&self) {
        let mut state = self.state.write();
        state.is_formatted = false;
        state.text.clear();
        state.pen = Vec2::new(0.0, 0.0);
        (state.line_spacing, state.ascender) = (0.0, 0.0);
        state.corners.clear();
        state.new_lines.clear();
        (state.last_symbol_width, state.max_symbol_width) = (0.0, 0.0);
    }
    /// `Append`: adds the string's glyph corners laid out with the font (from the start of the
    /// whole text, as natively); a string of command symbols only raises `Standard_OutOfRange`
    /// from `Last()` of the empty corner list. Example: `formatter.append(&text, &font)?`.
    pub fn append(&self, text: &String8, font: &FontFTFont) -> Result<(), Failure> {
        if text.is_empty() {
            return Ok(());
        }
        let mut state = self.state.write();
        state.ascender = state.ascender.max(font.ascender());
        state.line_spacing = state.line_spacing.max(font.line_spacing());
        state.text.extend(decode(text));
        layout_glyphs(&mut state, font);
        let last = state.corners.last().map(|c| c.x).ok_or_else(|| {
            out_of_range(
                "NCollection_DynamicArray::Last - no glyph corner; expected a printable symbol",
            )
        })?;
        state.last_symbol_width = state.pen.x - last;
        Ok(())
    }
}

/// `std::max(float, float)` as the native code computes it (`a < b ? b : a`).
fn max_f32(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

/// The advance loop of `Append` over the whole text.
fn layout_glyphs(state: &mut TextFormatterState, font: &FontFTFont) {
    let mut symbols_counter = 0;
    let mut it =
        FontTextFormatterIterator::over(state.text.clone(), FontTextFormatterIterationFilter::None);
    while it.more() {
        let (this, next) = (it.symbol(), it.symbol_next());
        it.next();
        if FontTextFormatter::is_command_symbol(this) {
            continue;
        }
        let advance = match this {
            0x0A => {
                symbols_counter = 0;
                state.new_lines.push(state.pen.x);
                0.0
            }
            0x20 => font.advance_x_of(0x20, next),
            0x09 => {
                let spaces = state.tab_size - (symbols_counter - 1) % state.tab_size;
                symbols_counter += spaces;
                font.advance_x_of(0x20, next) * spaces as f32
            }
            _ => font.advance_x_of(this, next),
        };
        symbols_counter += 1;
        state.corners.push(state.pen);
        state.pen.x += advance;
        state.max_symbol_width = max_f32(state.max_symbol_width, advance);
    }
}

/// `Font_TextFormatter::Iterator`: the non-command symbols of the text with their positions
/// (filtered symbols keep their position). Example: `FontTextFormatterIterator::new(&formatter, FontTextFormatterIterationFilter::None)`.
#[derive(Clone, Debug)]
pub struct FontTextFormatterIterator {
    filter: FontTextFormatterIterationFilter,
    text: Vec<u32>,
    cursor: usize,
    symbol_position: i32,
    symbol_char: u32,
    symbol_next: i32,
    symbol_char_next: u32,
}

impl FontTextFormatterIterator {
    /// An iterator over the formatter text; `FontTextFormatterIterator::new(&formatter, filter)`.
    pub fn new(formatter: &FontTextFormatter, filter: FontTextFormatterIterationFilter) -> Self {
        Self::over(formatter.state.read().text.clone(), filter)
    }
    pub(crate) fn over(text: Vec<u32>, filter: FontTextFormatterIterationFilter) -> Self {
        let mut it = Self {
            filter,
            text,
            cursor: 0,
            symbol_position: -1,
            symbol_char: 0,
            symbol_next: -1,
            symbol_char_next: 0,
        };
        let (position, symbol) = it.read_next_symbol(-1);
        (it.symbol_position, it.symbol_char) = (position, symbol.unwrap_or(0));
        let (next, symbol) = it.read_next_symbol(it.symbol_position);
        (it.symbol_next, it.symbol_char_next) = (next, symbol.unwrap_or(0));
        it
    }
    /// `readNextSymbol`: the position and symbol after `from`, (-1, None) at the end; a
    /// symbol not found keeps the previous symbol value natively.
    fn read_next_symbol(&mut self, from: i32) -> (i32, Option<u32>) {
        let mut position = from;
        while let Some(&c) = self.text.get(self.cursor) {
            if FontTextFormatter::is_command_symbol(c) {
                self.cursor += 1;
                continue;
            }
            position += 1;
            let excluded = self.filter == FontTextFormatterIterationFilter::ExcludeInvisible
                && matches!(c, 0x0A | 0x20 | 0x09);
            self.cursor += 1;
            if !excluded {
                return (position, Some(c));
            }
        }
        (-1, None)
    }
    /// Whether a current symbol exists; `it.more()`.
    pub fn more(&self) -> bool {
        self.symbol_position >= 0
    }
    /// Whether a next symbol exists; `it.has_next()`.
    pub fn has_next(&self) -> bool {
        self.symbol_next >= 0
    }
    /// The current symbol; `it.symbol()`.
    pub fn symbol(&self) -> u32 {
        self.symbol_char
    }
    /// The next symbol; `it.symbol_next()`.
    pub fn symbol_next(&self) -> u32 {
        self.symbol_char_next
    }
    /// The position of the current symbol; `it.symbol_position()`.
    pub fn symbol_position(&self) -> i32 {
        self.symbol_position
    }
    /// The position of the next symbol; `it.symbol_position_next()`.
    pub fn symbol_position_next(&self) -> i32 {
        self.symbol_next
    }
    /// Moves to the next symbol; `it.next()`.
    pub fn next(&mut self) {
        (self.symbol_position, self.symbol_char) = (self.symbol_next, self.symbol_char_next);
        let (next, symbol) = self.read_next_symbol(self.symbol_position);
        self.symbol_next = next;
        if let Some(symbol) = symbol {
            self.symbol_char_next = symbol;
        }
    }
}

impl FontTextFormatter {
    /// `BottomLeft`: a glyph corner; an index out of range raises `Standard_OutOfRange`.
    /// Example: `formatter.bottom_left(0)?`.
    pub fn bottom_left(&self, index: i32) -> Result<Vec2<f32>, Failure> {
        corner(&self.state.read(), index)
    }
    /// Deprecated `TopLeft`: the bottom-left corner; `formatter.top_left(0)?`.
    #[deprecated(note = "BottomLeft should be used instead")]
    pub fn top_left(&self, index: i32) -> Result<Vec2<f32>, Failure> {
        self.bottom_left(index)
    }
    /// The UTF-8 text; `formatter.string()`.
    pub fn string(&self) -> String8 {
        String8::from_unicode_units(&self.state.read().text, -1)
    }
    /// The glyph corners; `formatter.corners()`.
    pub fn corners(&self) -> Vec<Vec2<f32>> {
        self.state.read().corners.clone()
    }
    /// The pen positions at the line feeds; `formatter.new_lines()`.
    pub fn new_lines(&self) -> Vec<f32> {
        self.state.read().new_lines.clone()
    }
}

/// `myCorners.Value(index)` with the native range check.
pub(crate) fn corner(state: &TextFormatterState, index: i32) -> Result<Vec2<f32>, Failure> {
    usize::try_from(index)
        .ok()
        .and_then(|i| state.corners.get(i))
        .copied()
        .ok_or_else(|| {
            out_of_range(&format!(
                "NCollection_DynamicArray::Value - index {index}; expected 0..{}",
                state.corners.len()
            ))
        })
}

impl FontTextFormatter {
    /// The horizontal tabulation width in spaces; `formatter.tab_size()`.
    pub fn tab_size(&self) -> i32 {
        self.state.read().tab_size
    }
    /// The horizontal alignment; `formatter.horizontal_text_alignment()`.
    pub fn horizontal_text_alignment(&self) -> Graphic3dHorizontalTextAlignment {
        self.state.read().align_x
    }
    /// The vertical alignment; `formatter.vertical_text_alignment()`.
    pub fn vertical_text_alignment(&self) -> Graphic3dVerticalTextAlignment {
        self.state.read().align_y
    }
    /// Sets the wrapping width (0 for none); `formatter.set_wrapping(200.0)`.
    pub fn set_wrapping(&self, width: f32) {
        self.state.write().wrapping_width = width;
    }
    /// Whether a wrapping width is set; `formatter.has_wrapping()`.
    pub fn has_wrapping(&self) -> bool {
        self.state.read().wrapping_width > 0.0
    }
    /// The wrapping width; `formatter.wrapping()`.
    pub fn wrapping(&self) -> f32 {
        self.state.read().wrapping_width
    }
    /// Whether words are kept whole when wrapping; `formatter.word_wrapping()`.
    pub fn word_wrapping(&self) -> bool {
        self.state.read().is_word_wrapping
    }
    /// Keeps words whole when wrapping; `formatter.set_word_wrapping(false)`.
    pub fn set_word_wrapping(&self, is_word_wrapping: bool) {
        self.state.write().is_word_wrapping = is_word_wrapping;
    }
    /// The formatted width; `formatter.result_width()`.
    pub fn result_width(&self) -> f32 {
        self.state.read().bnd_width
    }
    /// The formatted height (line spacing times lines); `formatter.result_height()`.
    pub fn result_height(&self) -> f32 {
        let state = self.state.read();
        state.line_spacing * state.lines_nb as f32
    }
    /// The widest symbol advance; `formatter.maximum_symbol_width()`.
    pub fn maximum_symbol_width(&self) -> f32 {
        self.state.read().max_symbol_width
    }
    /// `LineHeight`: the ascender for the first line, else the line spacing.
    /// Example: `formatter.line_height(0)`.
    pub fn line_height(&self, index: i32) -> f32 {
        let state = self.state.read();
        if index == 0 {
            state.ascender
        } else {
            state.line_spacing
        }
    }
    /// `BndBox`: the box of the formatted text by the alignments; `formatter.bnd_box(&mut rect)`.
    pub fn bnd_box(&self, rect: &mut FontRect) {
        let state = self.state.read();
        rect.left = 0.0;
        match state.align_x {
            Graphic3dHorizontalTextAlignment::Left => rect.right = state.bnd_width,
            Graphic3dHorizontalTextAlignment::Right => rect.right = -state.bnd_width,
            Graphic3dHorizontalTextAlignment::Center => {
                (rect.left, rect.right) = (-0.5 * state.bnd_width, 0.5 * state.bnd_width);
            }
        }
        rect.top = state.bnd_top;
        rect.bottom = rect.top - state.line_spacing * state.lines_nb as f32;
    }
}
