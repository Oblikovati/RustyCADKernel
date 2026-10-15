// Rust port of the datum and tolerance enumeration headers of DataExchange/TKXCAF/
// XCAFDimTolObjects (DatumModifWithValue, DatumSingleModif, DatumTargetType, GeomToleranceMatReqModif,
// GeomToleranceModif, GeomToleranceType, GeomToleranceTypeValue, GeomToleranceZoneModif,
// ToleranceZoneAffectedPlane) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The enumerations of the datum and geometric tolerance objects, in native order and values.

/// `XCAFDimTolObjects_DatumModifWithValue`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsDatumModifWithValue {
    /// `XCAFDimTolObjects_DatumModifWithValue_None`.
    #[default]
    None = 0,
    /// `XCAFDimTolObjects_DatumModifWithValue_CircularOrCylindrical`.
    CircularOrCylindrical = 1,
    /// `XCAFDimTolObjects_DatumModifWithValue_Distance`.
    Distance = 2,
    /// `XCAFDimTolObjects_DatumModifWithValue_Projected`.
    Projected = 3,
    /// `XCAFDimTolObjects_DatumModifWithValue_Spherical`.
    Spherical = 4,
}

impl crate::NativeEnum for XCAFDimTolObjectsDatumModifWithValue {
    const ALL: &'static [Self] = &[
        Self::None,
        Self::CircularOrCylindrical,
        Self::Distance,
        Self::Projected,
        Self::Spherical,
    ];
}

/// `XCAFDimTolObjects_DatumSingleModif`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsDatumSingleModif {
    /// `XCAFDimTolObjects_DatumSingleModif_AnyCrossSection`.
    #[default]
    AnyCrossSection = 0,
    /// `XCAFDimTolObjects_DatumSingleModif_Any_LongitudinalSection`.
    AnyLongitudinalSection = 1,
    /// `XCAFDimTolObjects_DatumSingleModif_Basic`.
    Basic = 2,
    /// `XCAFDimTolObjects_DatumSingleModif_ContactingFeature`.
    ContactingFeature = 3,
    /// `XCAFDimTolObjects_DatumSingleModif_DegreeOfFreedomConstraintU`.
    DegreeOfFreedomConstraintU = 4,
    /// `XCAFDimTolObjects_DatumSingleModif_DegreeOfFreedomConstraintV`.
    DegreeOfFreedomConstraintV = 5,
    /// `XCAFDimTolObjects_DatumSingleModif_DegreeOfFreedomConstraintW`.
    DegreeOfFreedomConstraintW = 6,
    /// `XCAFDimTolObjects_DatumSingleModif_DegreeOfFreedomConstraintX`.
    DegreeOfFreedomConstraintX = 7,
    /// `XCAFDimTolObjects_DatumSingleModif_DegreeOfFreedomConstraintY`.
    DegreeOfFreedomConstraintY = 8,
    /// `XCAFDimTolObjects_DatumSingleModif_DegreeOfFreedomConstraintZ`.
    DegreeOfFreedomConstraintZ = 9,
    /// `XCAFDimTolObjects_DatumSingleModif_DistanceVariable`.
    DistanceVariable = 10,
    /// `XCAFDimTolObjects_DatumSingleModif_FreeState`.
    FreeState = 11,
    /// `XCAFDimTolObjects_DatumSingleModif_LeastMaterialRequirement`.
    LeastMaterialRequirement = 12,
    /// `XCAFDimTolObjects_DatumSingleModif_Line`.
    Line = 13,
    /// `XCAFDimTolObjects_DatumSingleModif_MajorDiameter`.
    MajorDiameter = 14,
    /// `XCAFDimTolObjects_DatumSingleModif_MaximumMaterialRequirement`.
    MaximumMaterialRequirement = 15,
    /// `XCAFDimTolObjects_DatumSingleModif_MinorDiameter`.
    MinorDiameter = 16,
    /// `XCAFDimTolObjects_DatumSingleModif_Orientation`.
    Orientation = 17,
    /// `XCAFDimTolObjects_DatumSingleModif_PitchDiameter`.
    PitchDiameter = 18,
    /// `XCAFDimTolObjects_DatumSingleModif_Plane`.
    Plane = 19,
    /// `XCAFDimTolObjects_DatumSingleModif_Point`.
    Point = 20,
    /// `XCAFDimTolObjects_DatumSingleModif_Translation`.
    Translation = 21,
}

impl crate::NativeEnum for XCAFDimTolObjectsDatumSingleModif {
    const ALL: &'static [Self] = &[
        Self::AnyCrossSection,
        Self::AnyLongitudinalSection,
        Self::Basic,
        Self::ContactingFeature,
        Self::DegreeOfFreedomConstraintU,
        Self::DegreeOfFreedomConstraintV,
        Self::DegreeOfFreedomConstraintW,
        Self::DegreeOfFreedomConstraintX,
        Self::DegreeOfFreedomConstraintY,
        Self::DegreeOfFreedomConstraintZ,
        Self::DistanceVariable,
        Self::FreeState,
        Self::LeastMaterialRequirement,
        Self::Line,
        Self::MajorDiameter,
        Self::MaximumMaterialRequirement,
        Self::MinorDiameter,
        Self::Orientation,
        Self::PitchDiameter,
        Self::Plane,
        Self::Point,
        Self::Translation,
    ];
}

/// `XCAFDimTolObjects_DatumTargetType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsDatumTargetType {
    /// `XCAFDimTolObjects_DatumTargetType_Point`.
    #[default]
    Point = 0,
    /// `XCAFDimTolObjects_DatumTargetType_Line`.
    Line = 1,
    /// `XCAFDimTolObjects_DatumTargetType_Rectangle`.
    Rectangle = 2,
    /// `XCAFDimTolObjects_DatumTargetType_Circle`.
    Circle = 3,
    /// `XCAFDimTolObjects_DatumTargetType_Area`.
    Area = 4,
}

impl crate::NativeEnum for XCAFDimTolObjectsDatumTargetType {
    const ALL: &'static [Self] = &[
        Self::Point,
        Self::Line,
        Self::Rectangle,
        Self::Circle,
        Self::Area,
    ];
}

/// `XCAFDimTolObjects_GeomToleranceMatReqModif`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsGeomToleranceMatReqModif {
    /// `XCAFDimTolObjects_GeomToleranceMatReqModif_None`.
    #[default]
    None = 0,
    /// `XCAFDimTolObjects_GeomToleranceMatReqModif_M`.
    M = 1,
    /// `XCAFDimTolObjects_GeomToleranceMatReqModif_L`.
    L = 2,
}

impl crate::NativeEnum for XCAFDimTolObjectsGeomToleranceMatReqModif {
    const ALL: &'static [Self] = &[Self::None, Self::M, Self::L];
}

/// `XCAFDimTolObjects_GeomToleranceModif`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsGeomToleranceModif {
    /// `XCAFDimTolObjects_GeomToleranceModif_Any_Cross_Section`.
    #[default]
    AnyCrossSection = 0,
    /// `XCAFDimTolObjects_GeomToleranceModif_Common_Zone`.
    CommonZone = 1,
    /// `XCAFDimTolObjects_GeomToleranceModif_Each_Radial_Element`.
    EachRadialElement = 2,
    /// `XCAFDimTolObjects_GeomToleranceModif_Free_State`.
    FreeState = 3,
    /// `XCAFDimTolObjects_GeomToleranceModif_Least_Material_Requirement`.
    LeastMaterialRequirement = 4,
    /// `XCAFDimTolObjects_GeomToleranceModif_Line_Element`.
    LineElement = 5,
    /// `XCAFDimTolObjects_GeomToleranceModif_Major_Diameter`.
    MajorDiameter = 6,
    /// `XCAFDimTolObjects_GeomToleranceModif_Maximum_Material_Requirement`.
    MaximumMaterialRequirement = 7,
    /// `XCAFDimTolObjects_GeomToleranceModif_Minor_Diameter`.
    MinorDiameter = 8,
    /// `XCAFDimTolObjects_GeomToleranceModif_Not_Convex`.
    NotConvex = 9,
    /// `XCAFDimTolObjects_GeomToleranceModif_Pitch_Diameter`.
    PitchDiameter = 10,
    /// `XCAFDimTolObjects_GeomToleranceModif_Reciprocity_Requirement`.
    ReciprocityRequirement = 11,
    /// `XCAFDimTolObjects_GeomToleranceModif_Separate_Requirement`.
    SeparateRequirement = 12,
    /// `XCAFDimTolObjects_GeomToleranceModif_Statistical_Tolerance`.
    StatisticalTolerance = 13,
    /// `XCAFDimTolObjects_GeomToleranceModif_Tangent_Plane`.
    TangentPlane = 14,
    /// `XCAFDimTolObjects_GeomToleranceModif_All_Around`.
    AllAround = 15,
    /// `XCAFDimTolObjects_GeomToleranceModif_All_Over`.
    AllOver = 16,
}

impl crate::NativeEnum for XCAFDimTolObjectsGeomToleranceModif {
    const ALL: &'static [Self] = &[
        Self::AnyCrossSection,
        Self::CommonZone,
        Self::EachRadialElement,
        Self::FreeState,
        Self::LeastMaterialRequirement,
        Self::LineElement,
        Self::MajorDiameter,
        Self::MaximumMaterialRequirement,
        Self::MinorDiameter,
        Self::NotConvex,
        Self::PitchDiameter,
        Self::ReciprocityRequirement,
        Self::SeparateRequirement,
        Self::StatisticalTolerance,
        Self::TangentPlane,
        Self::AllAround,
        Self::AllOver,
    ];
}

/// `XCAFDimTolObjects_GeomToleranceType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsGeomToleranceType {
    /// `XCAFDimTolObjects_GeomToleranceType_None`.
    #[default]
    None = 0,
    /// `XCAFDimTolObjects_GeomToleranceType_Angularity`.
    Angularity = 1,
    /// `XCAFDimTolObjects_GeomToleranceType_CircularRunout`.
    CircularRunout = 2,
    /// `XCAFDimTolObjects_GeomToleranceType_CircularityOrRoundness`.
    CircularityOrRoundness = 3,
    /// `XCAFDimTolObjects_GeomToleranceType_Coaxiality`.
    Coaxiality = 4,
    /// `XCAFDimTolObjects_GeomToleranceType_Concentricity`.
    Concentricity = 5,
    /// `XCAFDimTolObjects_GeomToleranceType_Cylindricity`.
    Cylindricity = 6,
    /// `XCAFDimTolObjects_GeomToleranceType_Flatness`.
    Flatness = 7,
    /// `XCAFDimTolObjects_GeomToleranceType_Parallelism`.
    Parallelism = 8,
    /// `XCAFDimTolObjects_GeomToleranceType_Perpendicularity`.
    Perpendicularity = 9,
    /// `XCAFDimTolObjects_GeomToleranceType_Position`.
    Position = 10,
    /// `XCAFDimTolObjects_GeomToleranceType_ProfileOfLine`.
    ProfileOfLine = 11,
    /// `XCAFDimTolObjects_GeomToleranceType_ProfileOfSurface`.
    ProfileOfSurface = 12,
    /// `XCAFDimTolObjects_GeomToleranceType_Straightness`.
    Straightness = 13,
    /// `XCAFDimTolObjects_GeomToleranceType_Symmetry`.
    Symmetry = 14,
    /// `XCAFDimTolObjects_GeomToleranceType_TotalRunout`.
    TotalRunout = 15,
}

impl crate::NativeEnum for XCAFDimTolObjectsGeomToleranceType {
    const ALL: &'static [Self] = &[
        Self::None,
        Self::Angularity,
        Self::CircularRunout,
        Self::CircularityOrRoundness,
        Self::Coaxiality,
        Self::Concentricity,
        Self::Cylindricity,
        Self::Flatness,
        Self::Parallelism,
        Self::Perpendicularity,
        Self::Position,
        Self::ProfileOfLine,
        Self::ProfileOfSurface,
        Self::Straightness,
        Self::Symmetry,
        Self::TotalRunout,
    ];
}

/// `XCAFDimTolObjects_GeomToleranceTypeValue`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsGeomToleranceTypeValue {
    /// `XCAFDimTolObjects_GeomToleranceTypeValue_None`.
    #[default]
    None = 0,
    /// `XCAFDimTolObjects_GeomToleranceTypeValue_Diameter`.
    Diameter = 1,
    /// `XCAFDimTolObjects_GeomToleranceTypeValue_SphericalDiameter`.
    SphericalDiameter = 2,
}

impl crate::NativeEnum for XCAFDimTolObjectsGeomToleranceTypeValue {
    const ALL: &'static [Self] = &[Self::None, Self::Diameter, Self::SphericalDiameter];
}

/// `XCAFDimTolObjects_GeomToleranceZoneModif`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsGeomToleranceZoneModif {
    /// `XCAFDimTolObjects_GeomToleranceZoneModif_None`.
    #[default]
    None = 0,
    /// `XCAFDimTolObjects_GeomToleranceZoneModif_Projected`.
    Projected = 1,
    /// `XCAFDimTolObjects_GeomToleranceZoneModif_Runout`.
    Runout = 2,
    /// `XCAFDimTolObjects_GeomToleranceZoneModif_NonUniform`.
    NonUniform = 3,
}

impl crate::NativeEnum for XCAFDimTolObjectsGeomToleranceZoneModif {
    const ALL: &'static [Self] = &[Self::None, Self::Projected, Self::Runout, Self::NonUniform];
}

/// `XCAFDimTolObjects_ToleranceZoneAffectedPlane`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsToleranceZoneAffectedPlane {
    /// `XCAFDimTolObjects_ToleranceZoneAffectedPlane_None`.
    #[default]
    None = 0,
    /// `XCAFDimTolObjects_ToleranceZoneAffectedPlane_Intersection`.
    Intersection = 1,
    /// `XCAFDimTolObjects_ToleranceZoneAffectedPlane_Orientation`.
    Orientation = 2,
}

impl crate::NativeEnum for XCAFDimTolObjectsToleranceZoneAffectedPlane {
    const ALL: &'static [Self] = &[Self::None, Self::Intersection, Self::Orientation];
}
