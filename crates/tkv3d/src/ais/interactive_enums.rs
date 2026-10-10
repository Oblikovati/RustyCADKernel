//! Native interactive-object categories and drag actions.
/// Fast object categories; `AisKindOfInteractive::Datum as i32 == 1`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AisKindOfInteractive {
    /// `AIS_KindOfInteractive_None`.
    None = 0,
    /// `AIS_KindOfInteractive_Datum`.
    Datum = 1,
    /// `AIS_KindOfInteractive_Shape`.
    Shape = 2,
    /// `AIS_KindOfInteractive_Object`.
    Object = 3,
    /// `AIS_KindOfInteractive_Relation`.
    Relation = 4,
    /// `AIS_KindOfInteractive_Dimension`.
    Dimension = 5,
    /// `AIS_KindOfInteractive_LightSource`.
    LightSource = 6,
}
impl AisKindOfInteractive {
    /// `AIS_KOI_None`.
    pub const KOI_NONE: Self = Self::None;
    /// `AIS_KOI_Datum`.
    pub const KOI_DATUM: Self = Self::Datum;
    /// `AIS_KOI_Shape`.
    pub const KOI_SHAPE: Self = Self::Shape;
    /// `AIS_KOI_Object`.
    pub const KOI_OBJECT: Self = Self::Object;
    /// `AIS_KOI_Relation`.
    pub const KOI_RELATION: Self = Self::Relation;
    /// `AIS_KOI_Dimension`.
    pub const KOI_DIMENSION: Self = Self::Dimension;
}
/// Lifecycle of a viewer drag; `AisDragAction::Abort as i32 == 4`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AisDragAction {
    /// Begin dragging.
    Start = 0,
    /// Confirm the drag interaction.
    Confirmed = 1,
    /// Update the dragged position.
    Update = 2,
    /// Commit the position.
    Stop = 3,
    /// Restore the position.
    Abort = 4,
}
