//! Display, rendering and interaction enumerations of the Aspect package.

/// Texture color spaces accepted by XR composer.
///
/// Native enumeration `Aspect_ColorSpace` (`Aspect_ColorSpace.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectColorSpace {
    /// non-linear sRGB color space.
    SRGB = 0,
    /// linear RGB color space.
    Linear = 1,
}

/// Camera eye index within stereoscopic pair.
///
/// Native enumeration `Aspect_Eye` (`Aspect_Eye.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectEye {
    /// `Aspect_Eye_Left`.
    Left = 0,
    /// `Aspect_Eye_Right`.
    Right = 1,
}

/// Defines the fill methods to
/// write bitmaps in a window.
///
/// Native enumeration `Aspect_FillMethod` (`Aspect_FillMethod.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectFillMethod {
    /// `Aspect_FM_NONE`.
    None = 0,
    /// `Aspect_FM_CENTERED`.
    Centered = 1,
    /// `Aspect_FM_TILED`.
    Tiled = 2,
    /// `Aspect_FM_STRETCH`.
    Stretch = 3,
}

/// Defines the fill methods to write gradient background in a window.
///
/// Native enumeration `Aspect_GradientFillMethod` (`Aspect_GradientFillMethod.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectGradientFillMethod {
    /// fill method not specified.
    None = 0,
    /// gradient directed from left (Color1) to right (Color2).
    Horizontal = 1,
    /// gradient directed from top (Color1) to bottom (Color2).
    Vertical = 2,
    /// gradient directed from upper left corner (Color1) to lower right (Color2).
    Diagonal1 = 3,
    /// gradient directed from upper right corner (Color1) to lower left (Color2).
    Diagonal2 = 4,
    /// highlights upper left corner with Color1.
    Corner1 = 5,
    /// highlights upper right corner with Color1.
    Corner2 = 6,
    /// highlights lower right corner with Color1.
    Corner3 = 7,
    /// highlights lower left corner with Color1.
    Corner4 = 8,
    /// gradient directed from center (Color1) in all directions forming an elliptic shape (Color2).
    Elliptical = 9,
}
impl AspectGradientFillMethod {
    /// Obsolete alias `Aspect_GFM_NONE` of [`Self::None`].
    pub const GFM_NONE: Self = Self::None;
    /// Obsolete alias `Aspect_GFM_HOR` of [`Self::Horizontal`].
    pub const GFM_HOR: Self = Self::Horizontal;
    /// Obsolete alias `Aspect_GFM_VER` of [`Self::Vertical`].
    pub const GFM_VER: Self = Self::Vertical;
    /// Obsolete alias `Aspect_GFM_DIAG1` of [`Self::Diagonal1`].
    pub const GFM_DIAG1: Self = Self::Diagonal1;
    /// Obsolete alias `Aspect_GFM_DIAG2` of [`Self::Diagonal2`].
    pub const GFM_DIAG2: Self = Self::Diagonal2;
    /// Obsolete alias `Aspect_GFM_CORNER1` of [`Self::Corner1`].
    pub const GFM_CORNER1: Self = Self::Corner1;
    /// Obsolete alias `Aspect_GFM_CORNER2` of [`Self::Corner2`].
    pub const GFM_CORNER2: Self = Self::Corner2;
    /// Obsolete alias `Aspect_GFM_CORNER3` of [`Self::Corner3`].
    pub const GFM_CORNER3: Self = Self::Corner3;
    /// Obsolete alias `Aspect_GFM_CORNER4` of [`Self::Corner4`].
    pub const GFM_CORNER4: Self = Self::Corner4;
}

/// Graphics API enumeration.
///
/// Native enumeration `Aspect_GraphicsLibrary` (`Aspect_GraphicsLibrary.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectGraphicsLibrary {
    /// `Aspect_GraphicsLibrary_OpenGL`.
    OpenGL = 0,
    /// `Aspect_GraphicsLibrary_OpenGLES`.
    OpenGLES = 1,
}

/// Defines the grid draw mode. The grid may be drawn
/// by using lines or points.
///
/// Native enumeration `Aspect_GridDrawMode` (`Aspect_GridDrawMode.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectGridDrawMode {
    /// `Aspect_GDM_Lines`.
    Lines = 0,
    /// `Aspect_GDM_Points`.
    Points = 1,
    /// `Aspect_GDM_None`.
    None = 2,
}

/// Defines the grid type : Rectangular or Circular.
///
/// Native enumeration `Aspect_GridType` (`Aspect_GridType.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectGridType {
    /// `Aspect_GT_Rectangular`.
    Rectangular = 0,
    /// `Aspect_GT_Circular`.
    Circular = 1,
}

/// Defines the using type of colors and labels
///
/// Native enumeration `Aspect_TypeOfColorScaleData` (`Aspect_TypeOfColorScaleData.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfColorScaleData {
    /// `Aspect_TOCSD_AUTO`.
    Auto = 0,
    /// `Aspect_TOCSD_USER`.
    User = 1,
}

/// Defines the type of color scale orientation
///
/// Native enumeration `Aspect_TypeOfColorScaleOrientation` (`Aspect_TypeOfColorScaleOrientation.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfColorScaleOrientation {
    /// `Aspect_TOCSO_NONE`.
    None = 0,
    /// `Aspect_TOCSO_LEFT`.
    Left = 1,
    /// `Aspect_TOCSO_RIGHT`.
    Right = 2,
    /// `Aspect_TOCSO_CENTER`.
    Center = 3,
}

/// Defines the type of position for color scale labels
///
/// Native enumeration `Aspect_TypeOfColorScalePosition` (`Aspect_TypeOfColorScalePosition.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfColorScalePosition {
    /// `Aspect_TOCSP_NONE`.
    None = 0,
    /// `Aspect_TOCSP_LEFT`.
    Left = 1,
    /// `Aspect_TOCSP_RIGHT`.
    Right = 2,
    /// `Aspect_TOCSP_CENTER`.
    Center = 3,
}

/// Defines the type of Resize Window method applied
/// by the user.
///
/// Native enumeration `Aspect_TypeOfResize` (`Aspect_TypeOfResize.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectTypeOfResize {
    /// `Aspect_TOR_UNKNOWN`.
    Unknown = 0,
    /// `Aspect_TOR_NO_BORDER`.
    NoBorder = 1,
    /// `Aspect_TOR_TOP_BORDER`.
    TopBorder = 2,
    /// `Aspect_TOR_RIGHT_BORDER`.
    RightBorder = 3,
    /// `Aspect_TOR_BOTTOM_BORDER`.
    BottomBorder = 4,
    /// `Aspect_TOR_LEFT_BORDER`.
    LeftBorder = 5,
    /// `Aspect_TOR_TOP_AND_RIGHT_BORDER`.
    TopAndRightBorder = 6,
    /// `Aspect_TOR_RIGHT_AND_BOTTOM_BORDER`.
    RightAndBottomBorder = 7,
    /// `Aspect_TOR_BOTTOM_AND_LEFT_BORDER`.
    BottomAndLeftBorder = 8,
    /// `Aspect_TOR_LEFT_AND_TOP_BORDER`.
    LeftAndTopBorder = 9,
}

/// Definition of the Trihedron position in the views.
/// It is defined as a bitmask to simplify handling vertical and horizontal alignment independently.
///
/// Native enumeration `Aspect_TypeOfTriedronPosition` (`Aspect_TypeOfTriedronPosition.hxx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AspectTypeOfTriedronPosition(pub i32);
impl AspectTypeOfTriedronPosition {
    /// at the center of the view.
    pub const CENTER: Self = Self(0);
    /// at the middle of the top    side.
    pub const TOP: Self = Self(1);
    /// at the middle of the bottom side.
    pub const BOTTOM: Self = Self(2);
    /// at the middle of the left   side.
    pub const LEFT: Self = Self(4);
    /// at the middle of the right  side.
    pub const RIGHT: Self = Self(8);
    /// at the left lower corner.
    pub const LEFT_LOWER: Self = Self(6);
    /// at the left upper corner.
    pub const LEFT_UPPER: Self = Self(5);
    /// at the right lower corner.
    pub const RIGHT_LOWER: Self = Self(10);
    /// at the right upper corner.
    pub const RIGHT_UPPER: Self = Self(9);
}

/// XR action type.
///
/// Native enumeration `Aspect_XRActionType` (`Aspect_XRActionType.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectXRActionType {
    /// boolean input (like button).
    InputDigital = 0,
    /// analog input (1/2/3 axes).
    InputAnalog = 1,
    /// positional input.
    InputPose = 2,
    /// skeletal input.
    InputSkeletal = 3,
    /// haptic output (vibration).
    OutputHaptic = 4,
}

/// Predefined tracked devices.
///
/// Native enumeration `Aspect_XRTrackedDeviceRole` (`Aspect_XRTrackedDeviceRole.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectXRTrackedDeviceRole {
    /// head.
    Head = 0,
    /// left hand.
    LeftHand = 1,
    /// right hand.
    RightHand = 2,
    /// other devices.
    Other = 3,
}

/// Defines custom identifiers(atoms) for X window custom named properties
///
/// Category: Instantiated classes
///
/// Native enumeration `Aspect_XAtom` (`Aspect_XAtom.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AspectXAtom {
    /// `Aspect_XA_DELETE_WINDOW`.
    DeleteWindow = 0,
}
