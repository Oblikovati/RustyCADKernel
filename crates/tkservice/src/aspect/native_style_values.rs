//! Safe conversions retain unnamed C++ enum values without invalid Rust discriminants.
use super::*;
const ASPECT_INTERIOR_STYLE_NATIVE_VALUES: &[AspectInteriorStyle] = &[
    AspectInteriorStyle::NativeMinus4,
    AspectInteriorStyle::NativeMinus3,
    AspectInteriorStyle::NativeMinus2,
    AspectInteriorStyle::Empty,
    AspectInteriorStyle::Solid,
    AspectInteriorStyle::Hatch,
    AspectInteriorStyle::Hiddenline,
    AspectInteriorStyle::Point,
];
impl AspectInteriorStyle {
    /// Preserve a value in the original C++ enum range; `AspectInteriorStyle::from_native_value(-4)`.
    pub fn from_native_value(value: i32) -> Option<Self> {
        ASPECT_INTERIOR_STYLE_NATIVE_VALUES
            .iter()
            .copied()
            .find(|candidate| *candidate as i32 == value)
    }
}
const ASPECT_TYPE_OF_LINE_NATIVE_VALUES: &[AspectTypeOfLine] = &[
    AspectTypeOfLine::NativeMinus8,
    AspectTypeOfLine::NativeMinus7,
    AspectTypeOfLine::NativeMinus6,
    AspectTypeOfLine::NativeMinus5,
    AspectTypeOfLine::NativeMinus4,
    AspectTypeOfLine::NativeMinus3,
    AspectTypeOfLine::NativeMinus2,
    AspectTypeOfLine::Empty,
    AspectTypeOfLine::Solid,
    AspectTypeOfLine::Dash,
    AspectTypeOfLine::Dot,
    AspectTypeOfLine::Dotdash,
    AspectTypeOfLine::Userdefined,
    AspectTypeOfLine::Native5,
    AspectTypeOfLine::Native6,
    AspectTypeOfLine::Native7,
];
impl AspectTypeOfLine {
    /// Preserve a value in the original C++ enum range; `AspectTypeOfLine::from_native_value(-8)`.
    pub fn from_native_value(value: i32) -> Option<Self> {
        ASPECT_TYPE_OF_LINE_NATIVE_VALUES
            .iter()
            .copied()
            .find(|candidate| *candidate as i32 == value)
    }
}
const ASPECT_TYPE_OF_MARKER_NATIVE_VALUES: &[AspectTypeOfMarker] = &[
    AspectTypeOfMarker::NativeMinus16,
    AspectTypeOfMarker::NativeMinus15,
    AspectTypeOfMarker::NativeMinus14,
    AspectTypeOfMarker::NativeMinus13,
    AspectTypeOfMarker::NativeMinus12,
    AspectTypeOfMarker::NativeMinus11,
    AspectTypeOfMarker::NativeMinus10,
    AspectTypeOfMarker::NativeMinus9,
    AspectTypeOfMarker::NativeMinus8,
    AspectTypeOfMarker::NativeMinus7,
    AspectTypeOfMarker::NativeMinus6,
    AspectTypeOfMarker::NativeMinus5,
    AspectTypeOfMarker::NativeMinus4,
    AspectTypeOfMarker::NativeMinus3,
    AspectTypeOfMarker::NativeMinus2,
    AspectTypeOfMarker::Empty,
    AspectTypeOfMarker::Point,
    AspectTypeOfMarker::Plus,
    AspectTypeOfMarker::Star,
    AspectTypeOfMarker::X,
    AspectTypeOfMarker::O,
    AspectTypeOfMarker::OPoint,
    AspectTypeOfMarker::OPlus,
    AspectTypeOfMarker::OStar,
    AspectTypeOfMarker::OX,
    AspectTypeOfMarker::Ring1,
    AspectTypeOfMarker::Ring2,
    AspectTypeOfMarker::Ring3,
    AspectTypeOfMarker::Ball,
    AspectTypeOfMarker::Userdefined,
    AspectTypeOfMarker::Native14,
    AspectTypeOfMarker::Native15,
];
impl AspectTypeOfMarker {
    /// Preserve a value in the original C++ enum range; `AspectTypeOfMarker::from_native_value(-16)`.
    pub fn from_native_value(value: i32) -> Option<Self> {
        ASPECT_TYPE_OF_MARKER_NATIVE_VALUES
            .iter()
            .copied()
            .find(|candidate| *candidate as i32 == value)
    }
}
const ASPECT_TYPE_OF_DISPLAY_TEXT_NATIVE_VALUES: &[AspectTypeOfDisplayText] = &[
    AspectTypeOfDisplayText::Normal,
    AspectTypeOfDisplayText::Subtitle,
    AspectTypeOfDisplayText::Dekale,
    AspectTypeOfDisplayText::Blend,
    AspectTypeOfDisplayText::Dimension,
    AspectTypeOfDisplayText::Shadow,
    AspectTypeOfDisplayText::Native6,
    AspectTypeOfDisplayText::Native7,
];
impl AspectTypeOfDisplayText {
    /// Preserve a value in the original C++ enum range; `AspectTypeOfDisplayText::from_native_value(0)`.
    pub fn from_native_value(value: i32) -> Option<Self> {
        ASPECT_TYPE_OF_DISPLAY_TEXT_NATIVE_VALUES
            .iter()
            .copied()
            .find(|candidate| *candidate as i32 == value)
    }
}
const ASPECT_HATCH_STYLE_NATIVE_VALUES: &[AspectHatchStyle] = &[
    AspectHatchStyle::Solid,
    AspectHatchStyle::GridDiagonal,
    AspectHatchStyle::GridDiagonalWide,
    AspectHatchStyle::Grid,
    AspectHatchStyle::GridWide,
    AspectHatchStyle::Diagonal45,
    AspectHatchStyle::Diagonal135,
    AspectHatchStyle::Horizontal,
    AspectHatchStyle::Vertical,
    AspectHatchStyle::Diagonal45Wide,
    AspectHatchStyle::Diagonal135Wide,
    AspectHatchStyle::HorizontalWide,
    AspectHatchStyle::VerticalWide,
    AspectHatchStyle::Nb,
    AspectHatchStyle::Native14,
    AspectHatchStyle::Native15,
];
impl AspectHatchStyle {
    /// Preserve a value in the original C++ enum range; `AspectHatchStyle::from_native_value(0)`.
    pub fn from_native_value(value: i32) -> Option<Self> {
        ASPECT_HATCH_STYLE_NATIVE_VALUES
            .iter()
            .copied()
            .find(|candidate| *candidate as i32 == value)
    }
}
const ASPECT_TYPE_OF_STYLE_TEXT_NATIVE_VALUES: &[AspectTypeOfStyleText] = &[
    AspectTypeOfStyleText::Normal,
    AspectTypeOfStyleText::Annotation,
];
impl AspectTypeOfStyleText {
    /// Preserve a value in the original C++ enum range; `AspectTypeOfStyleText::from_native_value(0)`.
    pub fn from_native_value(value: i32) -> Option<Self> {
        ASPECT_TYPE_OF_STYLE_TEXT_NATIVE_VALUES
            .iter()
            .copied()
            .find(|candidate| *candidate as i32 == value)
    }
}
