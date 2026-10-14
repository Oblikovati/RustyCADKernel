//! Line, marker, hatch and text style enumerations of the Aspect package.

/// Definition of all available hatch styles.
///
/// Native enumeration `Aspect_HatchStyle` (`Aspect_HatchStyle.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectHatchStyle {
    /// `Aspect_HS_SOLID`.
    Solid = 0,
    /// `Aspect_HS_HORIZONTAL`.
    Horizontal = 7,
    /// `Aspect_HS_HORIZONTAL_WIDE`.
    HorizontalWide = 11,
    /// `Aspect_HS_VERTICAL`.
    Vertical = 8,
    /// `Aspect_HS_VERTICAL_WIDE`.
    VerticalWide = 12,
    /// `Aspect_HS_DIAGONAL_45`.
    Diagonal45 = 5,
    /// `Aspect_HS_DIAGONAL_45_WIDE`.
    Diagonal45Wide = 9,
    /// `Aspect_HS_DIAGONAL_135`.
    Diagonal135 = 6,
    /// `Aspect_HS_DIAGONAL_135_WIDE`.
    Diagonal135Wide = 10,
    /// `Aspect_HS_GRID`.
    Grid = 3,
    /// `Aspect_HS_GRID_WIDE`.
    GridWide = 4,
    /// `Aspect_HS_GRID_DIAGONAL`.
    GridDiagonal = 1,
    /// `Aspect_HS_GRID_DIAGONAL_WIDE`.
    GridDiagonalWide = 2,
    /// `Aspect_HS_NB`.
    Nb = 13,
    /// Unnamed native value `14` within the original C++ enumeration range.
    Native14 = 14,
    /// Unnamed native value `15` within the original C++ enumeration range.
    Native15 = 15,
}

/// Interior types for primitive faces.
///
/// Native enumeration `Aspect_InteriorStyle` (`Aspect_InteriorStyle.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectInteriorStyle {
    /// no interior.
    Empty = -1,
    /// normally filled surface interior.
    Solid = 0,
    /// hatched surface interior.
    Hatch = 1,
    /// interior is filled with viewer background color.
    Hiddenline = 2,
    /// display only vertices of surface (obsolete).
    Point = 3,
    /// Unnamed native value `-4` within the original C++ enumeration range.
    NativeMinus4 = -4,
    /// Unnamed native value `-3` within the original C++ enumeration range.
    NativeMinus3 = -3,
    /// Unnamed native value `-2` within the original C++ enumeration range.
    NativeMinus2 = -2,
}
impl AspectInteriorStyle {
    /// Obsolete alias `Aspect_IS_HOLLOW` of [`Self::Empty`].
    pub const IS_HOLLOW: Self = Self::Empty;
}

/// The values of `Aspect_PolygonOffsetMode`.
///
/// Native enumeration `Aspect_PolygonOffsetMode` (`Aspect_PolygonOffsetMode.hxx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AspectPolygonOffsetMode(pub i32);
impl AspectPolygonOffsetMode {
    /// `Aspect_POM_Off`.
    pub const OFF: Self = Self(0);
    /// `Aspect_POM_Fill`.
    pub const FILL: Self = Self(1);
    /// `Aspect_POM_Line`.
    pub const LINE: Self = Self(2);
    /// `Aspect_POM_Point`.
    pub const POINT: Self = Self(4);
    /// `Aspect_POM_All`.
    pub const ALL: Self = Self(7);
    /// `Aspect_POM_None`.
    pub const NONE: Self = Self(8);
    /// `Aspect_POM_Mask`.
    pub const MASK: Self = Self(15);
}

/// Defines if the maximal chordial deflection used when
/// drawing an object is absolute or relative to the size
/// of the object.
///
/// Native enumeration `Aspect_TypeOfDeflection` (`Aspect_TypeOfDeflection.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfDeflection {
    /// `Aspect_TOD_RELATIVE`.
    Relative = 0,
    /// `Aspect_TOD_ABSOLUTE`.
    Absolute = 1,
}

/// Define the display type of the text.
///
/// Native enumeration `Aspect_TypeOfDisplayText` (`Aspect_TypeOfDisplayText.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfDisplayText {
    /// default display, text only.
    Normal = 0,
    /// there is a subtitle under the text.
    Subtitle = 1,
    /// the text is displayed with a 3D style.
    Dekale = 2,
    /// the text is displayed in XOR.
    Blend = 3,
    /// dimension line under text will be invisible.
    Dimension = 4,
    /// the text will have a shadow at the right-bottom corner.
    Shadow = 5,
    /// Unnamed native value `6` within the original C++ enumeration range.
    Native6 = 6,
    /// Unnamed native value `7` within the original C++ enumeration range.
    Native7 = 7,
}

/// The values of `Aspect_TypeOfFacingModel`.
///
/// Native enumeration `Aspect_TypeOfFacingModel` (`Aspect_TypeOfFacingModel.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfFacingModel {
    /// `Aspect_TOFM_BOTH_SIDE`.
    BothSide = 0,
    /// `Aspect_TOFM_BACK_SIDE`.
    BackSide = 1,
    /// `Aspect_TOFM_FRONT_SIDE`.
    FrontSide = 2,
}

/// Definition of a highlight method
///
/// TOHM_COLOR          drawn in the highlight color
/// (default white)
/// TOHM_BOUNDBOX       enclosed by the boundary box
/// (default white)
///
/// Native enumeration `Aspect_TypeOfHighlightMethod` (`Aspect_TypeOfHighlightMethod.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfHighlightMethod {
    /// `Aspect_TOHM_COLOR`.
    Color = 0,
    /// `Aspect_TOHM_BOUNDBOX`.
    Boundbox = 1,
}

/// Definition of line types
///
/// Native enumeration `Aspect_TypeOfLine` (`Aspect_TypeOfLine.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfLine {
    /// hidden.
    Empty = -1,
    /// continuous.
    Solid = 0,
    /// dashed 2.0,1.0 (MM).
    Dash = 1,
    /// dotted 0.2,0.5 (MM).
    Dot = 2,
    /// mixed  10.0,1.0,2.0,1.0 (MM).
    Dotdash = 3,
    /// defined by Users.
    Userdefined = 4,
    /// Unnamed native value `-8` within the original C++ enumeration range.
    NativeMinus8 = -8,
    /// Unnamed native value `-7` within the original C++ enumeration range.
    NativeMinus7 = -7,
    /// Unnamed native value `-6` within the original C++ enumeration range.
    NativeMinus6 = -6,
    /// Unnamed native value `-5` within the original C++ enumeration range.
    NativeMinus5 = -5,
    /// Unnamed native value `-4` within the original C++ enumeration range.
    NativeMinus4 = -4,
    /// Unnamed native value `-3` within the original C++ enumeration range.
    NativeMinus3 = -3,
    /// Unnamed native value `-2` within the original C++ enumeration range.
    NativeMinus2 = -2,
    /// Unnamed native value `5` within the original C++ enumeration range.
    Native5 = 5,
    /// Unnamed native value `6` within the original C++ enumeration range.
    Native6 = 6,
    /// Unnamed native value `7` within the original C++ enumeration range.
    Native7 = 7,
}

/// Definition of types of markers
///
/// Native enumeration `Aspect_TypeOfMarker` (`Aspect_TypeOfMarker.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfMarker {
    /// hidden.
    Empty = -1,
    /// point   .
    Point = 0,
    /// plus    +.
    Plus = 1,
    /// star    *.
    Star = 2,
    /// cross   x.
    X = 3,
    /// circle  O.
    O = 4,
    /// a point in a circle.
    OPoint = 5,
    /// a plus  in a circle.
    OPlus = 6,
    /// a star  in a circle.
    OStar = 7,
    /// a cross in a circle.
    OX = 8,
    /// a large  ring.
    Ring1 = 9,
    /// a medium ring.
    Ring2 = 10,
    /// a small  ring.
    Ring3 = 11,
    /// a ball with 1 color and different saturations.
    Ball = 12,
    /// defined by Users (custom image).
    Userdefined = 13,
    /// Unnamed native value `-16` within the original C++ enumeration range.
    NativeMinus16 = -16,
    /// Unnamed native value `-15` within the original C++ enumeration range.
    NativeMinus15 = -15,
    /// Unnamed native value `-14` within the original C++ enumeration range.
    NativeMinus14 = -14,
    /// Unnamed native value `-13` within the original C++ enumeration range.
    NativeMinus13 = -13,
    /// Unnamed native value `-12` within the original C++ enumeration range.
    NativeMinus12 = -12,
    /// Unnamed native value `-11` within the original C++ enumeration range.
    NativeMinus11 = -11,
    /// Unnamed native value `-10` within the original C++ enumeration range.
    NativeMinus10 = -10,
    /// Unnamed native value `-9` within the original C++ enumeration range.
    NativeMinus9 = -9,
    /// Unnamed native value `-8` within the original C++ enumeration range.
    NativeMinus8 = -8,
    /// Unnamed native value `-7` within the original C++ enumeration range.
    NativeMinus7 = -7,
    /// Unnamed native value `-6` within the original C++ enumeration range.
    NativeMinus6 = -6,
    /// Unnamed native value `-5` within the original C++ enumeration range.
    NativeMinus5 = -5,
    /// Unnamed native value `-4` within the original C++ enumeration range.
    NativeMinus4 = -4,
    /// Unnamed native value `-3` within the original C++ enumeration range.
    NativeMinus3 = -3,
    /// Unnamed native value `-2` within the original C++ enumeration range.
    NativeMinus2 = -2,
    /// Unnamed native value `14` within the original C++ enumeration range.
    Native14 = 14,
    /// Unnamed native value `15` within the original C++ enumeration range.
    Native15 = 15,
}

/// Define the style of the text.
///
/// TOST_NORMAL
/// Default text. The text is displayed like any other graphic object.
/// This text can be hidden by another object that is nearest from the
/// point of view.
/// TOST_ANNOTATION
/// The text is always visible. The text is displayed
/// over the other object according to the priority.
///
/// Native enumeration `Aspect_TypeOfStyleText` (`Aspect_TypeOfStyleText.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfStyleText {
    /// `Aspect_TOST_NORMAL`.
    Normal = 0,
    /// `Aspect_TOST_ANNOTATION`.
    Annotation = 1,
}

/// Definition of line types
///
/// WOL_THIN            thin line (1 pixel width)
/// WOL_MEDIUM          medium width of 0.5 MM
/// WOL_THICK           thick width of 0.7 MM
/// WOL_VERYTHICK       very thick width of 1.5 MM
/// WOL_USERDEFINED     defined by Users
///
/// Native enumeration `Aspect_WidthOfLine` (`Aspect_WidthOfLine.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectWidthOfLine {
    /// `Aspect_WOL_THIN`.
    Thin = 0,
    /// `Aspect_WOL_MEDIUM`.
    Medium = 1,
    /// `Aspect_WOL_THICK`.
    Thick = 2,
    /// `Aspect_WOL_VERYTHICK`.
    Verythick = 3,
    /// `Aspect_WOL_USERDEFINED`.
    Userdefined = 4,
}
