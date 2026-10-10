//! Open numeric selection-image kind, including native unknown-value handling.
/// Native image mode; `StdSelectTypeOfSelectionImage::NORMALIZED_DEPTH`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StdSelectTypeOfSelectionImage(pub i32);
impl StdSelectTypeOfSelectionImage {
    /// Normalized depth, zero at the nearest hit.
    pub const NORMALIZED_DEPTH: Self = Self(0);
    /// Inverted normalized depth.
    pub const NORMALIZED_DEPTH_INVERTED: Self = Self(1);
    /// Raw depth converted to RGB.
    pub const UNNORMALIZED_DEPTH: Self = Self(2);
    /// Detected selectable object's drawer color.
    pub const COLORED_DETECTED_OBJECT: Self = Self(3);
    /// Stable generated color for each sensitive entity.
    pub const COLORED_ENTITY: Self = Self(4);
    /// Stable generated color for each sensitive runtime type.
    pub const COLORED_ENTITY_TYPE: Self = Self(5);
    /// Stable generated color for each owner.
    pub const COLORED_OWNER: Self = Self(6);
    /// Selection mode's standard or generated color.
    pub const COLORED_SELECTION_MODE: Self = Self(7);
    /// Normal direction remapped into RGB.
    pub const SURFACE_NORMAL: Self = Self(8);
}
