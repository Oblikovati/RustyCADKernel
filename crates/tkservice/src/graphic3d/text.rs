//! `Graphic3d_Text`: a text primitive (string or formatter, position or 3D orientation,
//! height and alignments).
use super::{Graphic3dHorizontalTextAlignment, Graphic3dVerticalTextAlignment};
use crate::font::FontTextFormatter;
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::{Axis2, Point3};

/// The fields of a text, shared by every handle.
#[derive(Clone)]
struct TextState {
    formatter: Handle<FontTextFormatter>,
    text: Vec<u8>,
    orientation: Axis2,
    height: f32,
    h_align: Graphic3dHorizontalTextAlignment,
    v_align: Graphic3dVerticalTextAlignment,
    has_plane: bool,
    has_own_anchor: bool,
}

/// A text primitive; `Graphic3dText::new(16.0)`.
pub struct Graphic3dText {
    transient: StandardTransient,
    state: SharedState<TextState>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dText,
    "Graphic3d_Text",
    StandardTransient,
    transient
);

impl Graphic3dText {
    /// A text of a height, left/bottom aligned, anchored at its position, without a plane.
    /// Example: `Graphic3dText::new(16.0)`.
    pub fn new(height: f32) -> Self {
        let state = TextState {
            formatter: Handle::default(),
            text: Vec::new(),
            orientation: Axis2::default(),
            height,
            h_align: Graphic3dHorizontalTextAlignment::Left,
            v_align: Graphic3dVerticalTextAlignment::Bottom,
            has_plane: false,
            has_own_anchor: true,
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// The UTF-8 text; `text.text()`.
    pub fn text(&self) -> String8 {
        let mut bytes = self.state.read().text.clone();
        bytes.push(0);
        String8::from_unicode_units(&bytes, -1)
    }
    /// Sets the text (`SetText(NCollection_String)`); `text.set_text(&string)`.
    pub fn set_text(&self, text: &String8) {
        let units = text.to_c_string();
        let end = units.iter().position(|&b| b == 0).unwrap_or(units.len());
        self.state.write().text = units[..end].to_vec();
    }
    /// Sets the text (`SetText(TCollection_AsciiString)`, up to its first NUL).
    /// Example: `text.set_text_ascii(&"label".into())`.
    pub fn set_text_ascii(&self, text: &AsciiString) {
        let bytes = text.as_bytes();
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        self.state.write().text = bytes[..end].to_vec();
    }
    /// Sets the text (`SetText(const char*)`); `text.set_text_str("label")`.
    pub fn set_text_str(&self, text: &str) {
        self.set_text_ascii(&AsciiString::from(text));
    }
    /// The text formatter (null for the default one); `text.text_formatter()`.
    pub fn text_formatter(&self) -> Handle<FontTextFormatter> {
        self.state.read().formatter.clone()
    }
    /// Sets the text formatter; `text.set_text_formatter(formatter)`.
    pub fn set_text_formatter(&self, formatter: Handle<FontTextFormatter>) {
        self.state.write().formatter = formatter;
    }
    /// The position (the orientation origin); `text.position()`.
    pub fn position(&self) -> Point3 {
        self.state.read().orientation.location()
    }
    /// Moves the text; `text.set_position(&p)`.
    pub fn set_position(&self, point: &Point3) {
        self.state.write().orientation.set_location(*point);
    }
    /// The 3D orientation; `text.orientation()`.
    pub fn orientation(&self) -> Axis2 {
        self.state.read().orientation
    }
    /// Whether the text has a 3D plane; `text.has_plane()`.
    pub fn has_plane(&self) -> bool {
        self.state.read().has_plane
    }
    /// Places the text in a 3D plane; `text.set_orientation(&axis)`.
    pub fn set_orientation(&self, orientation: &Axis2) {
        let mut state = self.state.write();
        (state.orientation, state.has_plane) = (*orientation, true);
    }
    /// Forgets the plane (default orientation, so the position resets too).
    /// Example: `text.reset_orientation()`.
    pub fn reset_orientation(&self) {
        let mut state = self.state.write();
        (state.orientation, state.has_plane) = (Axis2::default(), false);
    }
    /// Whether the position is the attach point; `text.has_own_anchor_point()`.
    pub fn has_own_anchor_point(&self) -> bool {
        self.state.read().has_own_anchor
    }
    /// Sets whether the position is the attach point; `text.set_own_anchor_point(false)`.
    pub fn set_own_anchor_point(&self, has_own_anchor: bool) {
        self.state.write().has_own_anchor = has_own_anchor;
    }
    /// The height; `text.height()`.
    pub fn height(&self) -> f32 {
        self.state.read().height
    }
    /// Sets the height; `text.set_height(20.0)`.
    pub fn set_height(&self, height: f32) {
        self.state.write().height = height;
    }
    /// The horizontal alignment; `text.horizontal_alignment()`.
    pub fn horizontal_alignment(&self) -> Graphic3dHorizontalTextAlignment {
        self.state.read().h_align
    }
    /// Sets the horizontal alignment; `text.set_horizontal_alignment(Graphic3dHorizontalTextAlignment::Center)`.
    pub fn set_horizontal_alignment(&self, alignment: Graphic3dHorizontalTextAlignment) {
        self.state.write().h_align = alignment;
    }
    /// The vertical alignment; `text.vertical_alignment()`.
    pub fn vertical_alignment(&self) -> Graphic3dVerticalTextAlignment {
        self.state.read().v_align
    }
    /// Sets the vertical alignment; `text.set_vertical_alignment(Graphic3dVerticalTextAlignment::Top)`.
    pub fn set_vertical_alignment(&self, alignment: Graphic3dVerticalTextAlignment) {
        self.state.write().v_align = alignment;
    }
}
