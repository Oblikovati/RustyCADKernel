//! The PrsDim enumerations.

/// `PrsDim_KindOfDimension`: the kind of a measured value.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimKindOfDimension {
    /// `PrsDim_KOD_NONE`.
    #[default]
    None = 0,
    /// `PrsDim_KOD_LENGTH`.
    Length,
    /// `PrsDim_KOD_PLANEANGLE`.
    PlaneAngle,
    /// `PrsDim_KOD_SOLIDANGLE`.
    SolidAngle,
    /// `PrsDim_KOD_AREA`.
    Area,
    /// `PrsDim_KOD_VOLUME`.
    Volume,
    /// `PrsDim_KOD_MASS`.
    Mass,
    /// `PrsDim_KOD_TIME`.
    Time,
    /// `PrsDim_KOD_RADIUS`.
    Radius,
    /// `PrsDim_KOD_DIAMETER`.
    Diameter,
    /// `PrsDim_KOD_CHAMF2D`.
    Chamf2d,
    /// `PrsDim_KOD_CHAMF3D`.
    Chamf3d,
    /// `PrsDim_KOD_OFFSET`.
    Offset,
    /// `PrsDim_KOD_ELLIPSERADIUS`.
    EllipseRadius,
}

/// `PrsDim_KindOfRelation`: the kind of a relation.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimKindOfRelation {
    /// `PrsDim_KOR_NONE`.
    #[default]
    None = 0,
    /// `PrsDim_KOR_CONCENTRIC`.
    Concentric,
    /// `PrsDim_KOR_EQUALDISTANCE`.
    EqualDistance,
    /// `PrsDim_KOR_EQUALRADIUS`.
    EqualRadius,
    /// `PrsDim_KOR_FIX`.
    Fix,
    /// `PrsDim_KOR_IDENTIC`.
    Identic,
    /// `PrsDim_KOR_OFFSET`.
    Offset,
    /// `PrsDim_KOR_PARALLEL`.
    Parallel,
    /// `PrsDim_KOR_PERPENDICULAR`.
    Perpendicular,
    /// `PrsDim_KOR_TANGENT`.
    Tangent,
    /// `PrsDim_KOR_SYMMETRIC`.
    Symmetric,
}

/// `PrsDim_KindOfSurface`: the kind of a measured surface.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimKindOfSurface {
    /// `PrsDim_KOS_Plane`.
    Plane = 0,
    /// `PrsDim_KOS_Cylinder`.
    Cylinder,
    /// `PrsDim_KOS_Cone`.
    Cone,
    /// `PrsDim_KOS_Sphere`.
    Sphere,
    /// `PrsDim_KOS_Torus`.
    Torus,
    /// `PrsDim_KOS_Revolution`.
    Revolution,
    /// `PrsDim_KOS_Extrusion`.
    Extrusion,
    /// `PrsDim_KOS_OtherSurface`.
    #[default]
    OtherSurface,
}

/// `PrsDim_DimensionSelectionMode`: the selection modes of a dimension.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimDimensionSelectionMode {
    /// `PrsDim_DimensionSelectionMode_All`: the whole dimension.
    #[default]
    All = 0,
    /// `PrsDim_DimensionSelectionMode_Line`: the dimension line and arrows.
    Line,
    /// `PrsDim_DimensionSelectionMode_Text`: the label.
    Text,
}
impl PrsDimDimensionSelectionMode {
    /// The mode of a selection mode number (`All` for unknown numbers, as the native switch).
    /// Example: `PrsDimDimensionSelectionMode::from_i32(2)`.
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::All),
            1 => Some(Self::Line),
            2 => Some(Self::Text),
            _ => None,
        }
    }
}

/// `PrsDim_DisplaySpecialSymbol`: where the special symbol of a value is shown.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimDisplaySpecialSymbol {
    /// `PrsDim_DisplaySpecialSymbol_No`.
    #[default]
    No = 0,
    /// `PrsDim_DisplaySpecialSymbol_Before`.
    Before,
    /// `PrsDim_DisplaySpecialSymbol_After`.
    After,
}

/// `PrsDim_TypeOfAngleArrowVisibility`: the visible arrows of an angle.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimTypeOfAngleArrowVisibility {
    /// `PrsDim_TypeOfAngleArrowVisibility_Both`.
    #[default]
    Both = 0,
    /// `PrsDim_TypeOfAngleArrowVisibility_First`.
    First,
    /// `PrsDim_TypeOfAngleArrowVisibility_Second`.
    Second,
    /// `PrsDim_TypeOfAngleArrowVisibility_None`.
    None,
}

/// `PrsDim_TypeOfAngle`: the interior angle or its complement to a full turn.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimTypeOfAngle {
    /// `PrsDim_TypeOfAngle_Interior`.
    #[default]
    Interior = 0,
    /// `PrsDim_TypeOfAngle_Exterior`.
    Exterior,
}

/// `PrsDim_TypeOfDist`: the direction of an offset distance.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimTypeOfDist {
    /// `PrsDim_TypeOfDist_Unknown`.
    #[default]
    Unknown = 0,
    /// `PrsDim_TypeOfDist_Horizontal`.
    Horizontal,
    /// `PrsDim_TypeOfDist_Vertical`.
    Vertical,
}
