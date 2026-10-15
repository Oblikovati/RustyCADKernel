// Rust port of the dimension enumeration headers of DataExchange/TKXCAF/XCAFDimTolObjects
// (AngularQualifier, DimensionFormVariance, DimensionGrade, DimensionModif, DimensionQualifier,
// DimensionType) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The enumerations of the dimension objects, in native order and values.

/// `XCAFDimTolObjects_AngularQualifier`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsAngularQualifier {
    /// `XCAFDimTolObjects_AngularQualifier_None`.
    #[default]
    None = 0,
    /// `XCAFDimTolObjects_AngularQualifier_Small`.
    Small = 1,
    /// `XCAFDimTolObjects_AngularQualifier_Large`.
    Large = 2,
    /// `XCAFDimTolObjects_AngularQualifier_Equal`.
    Equal = 3,
}

impl crate::NativeEnum for XCAFDimTolObjectsAngularQualifier {
    const ALL: &'static [Self] = &[Self::None, Self::Small, Self::Large, Self::Equal];
}

/// `XCAFDimTolObjects_DimensionFormVariance`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsDimensionFormVariance {
    /// `XCAFDimTolObjects_DimensionFormVariance_None`.
    #[default]
    None = 0,
    /// `XCAFDimTolObjects_DimensionFormVariance_A`.
    A = 1,
    /// `XCAFDimTolObjects_DimensionFormVariance_B`.
    B = 2,
    /// `XCAFDimTolObjects_DimensionFormVariance_C`.
    C = 3,
    /// `XCAFDimTolObjects_DimensionFormVariance_CD`.
    CD = 4,
    /// `XCAFDimTolObjects_DimensionFormVariance_D`.
    D = 5,
    /// `XCAFDimTolObjects_DimensionFormVariance_E`.
    E = 6,
    /// `XCAFDimTolObjects_DimensionFormVariance_EF`.
    EF = 7,
    /// `XCAFDimTolObjects_DimensionFormVariance_F`.
    F = 8,
    /// `XCAFDimTolObjects_DimensionFormVariance_FG`.
    FG = 9,
    /// `XCAFDimTolObjects_DimensionFormVariance_G`.
    G = 10,
    /// `XCAFDimTolObjects_DimensionFormVariance_H`.
    H = 11,
    /// `XCAFDimTolObjects_DimensionFormVariance_JS`.
    JS = 12,
    /// `XCAFDimTolObjects_DimensionFormVariance_J`.
    J = 13,
    /// `XCAFDimTolObjects_DimensionFormVariance_K`.
    K = 14,
    /// `XCAFDimTolObjects_DimensionFormVariance_M`.
    M = 15,
    /// `XCAFDimTolObjects_DimensionFormVariance_N`.
    N = 16,
    /// `XCAFDimTolObjects_DimensionFormVariance_P`.
    P = 17,
    /// `XCAFDimTolObjects_DimensionFormVariance_R`.
    R = 18,
    /// `XCAFDimTolObjects_DimensionFormVariance_S`.
    S = 19,
    /// `XCAFDimTolObjects_DimensionFormVariance_T`.
    T = 20,
    /// `XCAFDimTolObjects_DimensionFormVariance_U`.
    U = 21,
    /// `XCAFDimTolObjects_DimensionFormVariance_V`.
    V = 22,
    /// `XCAFDimTolObjects_DimensionFormVariance_X`.
    X = 23,
    /// `XCAFDimTolObjects_DimensionFormVariance_Y`.
    Y = 24,
    /// `XCAFDimTolObjects_DimensionFormVariance_Z`.
    Z = 25,
    /// `XCAFDimTolObjects_DimensionFormVariance_ZA`.
    ZA = 26,
    /// `XCAFDimTolObjects_DimensionFormVariance_ZB`.
    ZB = 27,
    /// `XCAFDimTolObjects_DimensionFormVariance_ZC`.
    ZC = 28,
}

impl crate::NativeEnum for XCAFDimTolObjectsDimensionFormVariance {
    const ALL: &'static [Self] = &[
        Self::None,
        Self::A,
        Self::B,
        Self::C,
        Self::CD,
        Self::D,
        Self::E,
        Self::EF,
        Self::F,
        Self::FG,
        Self::G,
        Self::H,
        Self::JS,
        Self::J,
        Self::K,
        Self::M,
        Self::N,
        Self::P,
        Self::R,
        Self::S,
        Self::T,
        Self::U,
        Self::V,
        Self::X,
        Self::Y,
        Self::Z,
        Self::ZA,
        Self::ZB,
        Self::ZC,
    ];
}

/// `XCAFDimTolObjects_DimensionGrade`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsDimensionGrade {
    /// `XCAFDimTolObjects_DimensionGrade_IT01`.
    #[default]
    IT01 = 0,
    /// `XCAFDimTolObjects_DimensionGrade_IT0`.
    IT0 = 1,
    /// `XCAFDimTolObjects_DimensionGrade_IT1`.
    IT1 = 2,
    /// `XCAFDimTolObjects_DimensionGrade_IT2`.
    IT2 = 3,
    /// `XCAFDimTolObjects_DimensionGrade_IT3`.
    IT3 = 4,
    /// `XCAFDimTolObjects_DimensionGrade_IT4`.
    IT4 = 5,
    /// `XCAFDimTolObjects_DimensionGrade_IT5`.
    IT5 = 6,
    /// `XCAFDimTolObjects_DimensionGrade_IT6`.
    IT6 = 7,
    /// `XCAFDimTolObjects_DimensionGrade_IT7`.
    IT7 = 8,
    /// `XCAFDimTolObjects_DimensionGrade_IT8`.
    IT8 = 9,
    /// `XCAFDimTolObjects_DimensionGrade_IT9`.
    IT9 = 10,
    /// `XCAFDimTolObjects_DimensionGrade_IT10`.
    IT10 = 11,
    /// `XCAFDimTolObjects_DimensionGrade_IT11`.
    IT11 = 12,
    /// `XCAFDimTolObjects_DimensionGrade_IT12`.
    IT12 = 13,
    /// `XCAFDimTolObjects_DimensionGrade_IT13`.
    IT13 = 14,
    /// `XCAFDimTolObjects_DimensionGrade_IT14`.
    IT14 = 15,
    /// `XCAFDimTolObjects_DimensionGrade_IT15`.
    IT15 = 16,
    /// `XCAFDimTolObjects_DimensionGrade_IT16`.
    IT16 = 17,
    /// `XCAFDimTolObjects_DimensionGrade_IT17`.
    IT17 = 18,
    /// `XCAFDimTolObjects_DimensionGrade_IT18`.
    IT18 = 19,
}

impl crate::NativeEnum for XCAFDimTolObjectsDimensionGrade {
    const ALL: &'static [Self] = &[
        Self::IT01,
        Self::IT0,
        Self::IT1,
        Self::IT2,
        Self::IT3,
        Self::IT4,
        Self::IT5,
        Self::IT6,
        Self::IT7,
        Self::IT8,
        Self::IT9,
        Self::IT10,
        Self::IT11,
        Self::IT12,
        Self::IT13,
        Self::IT14,
        Self::IT15,
        Self::IT16,
        Self::IT17,
        Self::IT18,
    ];
}

/// `XCAFDimTolObjects_DimensionModif`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsDimensionModif {
    /// `XCAFDimTolObjects_DimensionModif_ControlledRadius`.
    #[default]
    ControlledRadius = 0,
    /// `XCAFDimTolObjects_DimensionModif_Square`.
    Square = 1,
    /// `XCAFDimTolObjects_DimensionModif_StatisticalTolerance`.
    StatisticalTolerance = 2,
    /// `XCAFDimTolObjects_DimensionModif_ContinuousFeature`.
    ContinuousFeature = 3,
    /// `XCAFDimTolObjects_DimensionModif_TwoPointSize`.
    TwoPointSize = 4,
    /// `XCAFDimTolObjects_DimensionModif_LocalSizeDefinedBySphere`.
    LocalSizeDefinedBySphere = 5,
    /// `XCAFDimTolObjects_DimensionModif_LeastSquaresAssociationCriterion`.
    LeastSquaresAssociationCriterion = 6,
    /// `XCAFDimTolObjects_DimensionModif_MaximumInscribedAssociation`.
    MaximumInscribedAssociation = 7,
    /// `XCAFDimTolObjects_DimensionModif_MinimumCircumscribedAssociation`.
    MinimumCircumscribedAssociation = 8,
    /// `XCAFDimTolObjects_DimensionModif_CircumferenceDiameter`.
    CircumferenceDiameter = 9,
    /// `XCAFDimTolObjects_DimensionModif_AreaDiameter`.
    AreaDiameter = 10,
    /// `XCAFDimTolObjects_DimensionModif_VolumeDiameter`.
    VolumeDiameter = 11,
    /// `XCAFDimTolObjects_DimensionModif_MaximumSize`.
    MaximumSize = 12,
    /// `XCAFDimTolObjects_DimensionModif_MinimumSize`.
    MinimumSize = 13,
    /// `XCAFDimTolObjects_DimensionModif_AverageSize`.
    AverageSize = 14,
    /// `XCAFDimTolObjects_DimensionModif_MedianSize`.
    MedianSize = 15,
    /// `XCAFDimTolObjects_DimensionModif_MidRangeSize`.
    MidRangeSize = 16,
    /// `XCAFDimTolObjects_DimensionModif_RangeOfSizes`.
    RangeOfSizes = 17,
    /// `XCAFDimTolObjects_DimensionModif_AnyRestrictedPortionOfFeature`.
    AnyRestrictedPortionOfFeature = 18,
    /// `XCAFDimTolObjects_DimensionModif_AnyCrossSection`.
    AnyCrossSection = 19,
    /// `XCAFDimTolObjects_DimensionModif_SpecificFixedCrossSection`.
    SpecificFixedCrossSection = 20,
    /// `XCAFDimTolObjects_DimensionModif_CommonTolerance`.
    CommonTolerance = 21,
    /// `XCAFDimTolObjects_DimensionModif_FreeStateCondition`.
    FreeStateCondition = 22,
    /// `XCAFDimTolObjects_DimensionModif_Between`.
    Between = 23,
}

impl crate::NativeEnum for XCAFDimTolObjectsDimensionModif {
    const ALL: &'static [Self] = &[
        Self::ControlledRadius,
        Self::Square,
        Self::StatisticalTolerance,
        Self::ContinuousFeature,
        Self::TwoPointSize,
        Self::LocalSizeDefinedBySphere,
        Self::LeastSquaresAssociationCriterion,
        Self::MaximumInscribedAssociation,
        Self::MinimumCircumscribedAssociation,
        Self::CircumferenceDiameter,
        Self::AreaDiameter,
        Self::VolumeDiameter,
        Self::MaximumSize,
        Self::MinimumSize,
        Self::AverageSize,
        Self::MedianSize,
        Self::MidRangeSize,
        Self::RangeOfSizes,
        Self::AnyRestrictedPortionOfFeature,
        Self::AnyCrossSection,
        Self::SpecificFixedCrossSection,
        Self::CommonTolerance,
        Self::FreeStateCondition,
        Self::Between,
    ];
}

/// `XCAFDimTolObjects_DimensionQualifier`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsDimensionQualifier {
    /// `XCAFDimTolObjects_DimensionQualifier_None`.
    #[default]
    None = 0,
    /// `XCAFDimTolObjects_DimensionQualifier_Min`.
    Min = 1,
    /// `XCAFDimTolObjects_DimensionQualifier_Max`.
    Max = 2,
    /// `XCAFDimTolObjects_DimensionQualifier_Avg`.
    Avg = 3,
}

impl crate::NativeEnum for XCAFDimTolObjectsDimensionQualifier {
    const ALL: &'static [Self] = &[Self::None, Self::Min, Self::Max, Self::Avg];
}

/// `XCAFDimTolObjects_DimensionType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum XCAFDimTolObjectsDimensionType {
    /// `XCAFDimTolObjects_DimensionType_Location_None`.
    #[default]
    LocationNone = 0,
    /// `XCAFDimTolObjects_DimensionType_Location_CurvedDistance`.
    LocationCurvedDistance = 1,
    /// `XCAFDimTolObjects_DimensionType_Location_LinearDistance`.
    LocationLinearDistance = 2,
    /// `XCAFDimTolObjects_DimensionType_Location_LinearDistance_FromCenterToOuter`.
    LocationLinearDistanceFromCenterToOuter = 3,
    /// `XCAFDimTolObjects_DimensionType_Location_LinearDistance_FromCenterToInner`.
    LocationLinearDistanceFromCenterToInner = 4,
    /// `XCAFDimTolObjects_DimensionType_Location_LinearDistance_FromOuterToCenter`.
    LocationLinearDistanceFromOuterToCenter = 5,
    /// `XCAFDimTolObjects_DimensionType_Location_LinearDistance_FromOuterToOuter`.
    LocationLinearDistanceFromOuterToOuter = 6,
    /// `XCAFDimTolObjects_DimensionType_Location_LinearDistance_FromOuterToInner`.
    LocationLinearDistanceFromOuterToInner = 7,
    /// `XCAFDimTolObjects_DimensionType_Location_LinearDistance_FromInnerToCenter`.
    LocationLinearDistanceFromInnerToCenter = 8,
    /// `XCAFDimTolObjects_DimensionType_Location_LinearDistance_FromInnerToOuter`.
    LocationLinearDistanceFromInnerToOuter = 9,
    /// `XCAFDimTolObjects_DimensionType_Location_LinearDistance_FromInnerToInner`.
    LocationLinearDistanceFromInnerToInner = 10,
    /// `XCAFDimTolObjects_DimensionType_Location_Angular`.
    LocationAngular = 11,
    /// `XCAFDimTolObjects_DimensionType_Location_Oriented`.
    LocationOriented = 12,
    /// `XCAFDimTolObjects_DimensionType_Location_WithPath`.
    LocationWithPath = 13,
    /// `XCAFDimTolObjects_DimensionType_Size_CurveLength`.
    SizeCurveLength = 14,
    /// `XCAFDimTolObjects_DimensionType_Size_Diameter`.
    SizeDiameter = 15,
    /// `XCAFDimTolObjects_DimensionType_Size_SphericalDiameter`.
    SizeSphericalDiameter = 16,
    /// `XCAFDimTolObjects_DimensionType_Size_Radius`.
    SizeRadius = 17,
    /// `XCAFDimTolObjects_DimensionType_Size_SphericalRadius`.
    SizeSphericalRadius = 18,
    /// `XCAFDimTolObjects_DimensionType_Size_ToroidalMinorDiameter`.
    SizeToroidalMinorDiameter = 19,
    /// `XCAFDimTolObjects_DimensionType_Size_ToroidalMajorDiameter`.
    SizeToroidalMajorDiameter = 20,
    /// `XCAFDimTolObjects_DimensionType_Size_ToroidalMinorRadius`.
    SizeToroidalMinorRadius = 21,
    /// `XCAFDimTolObjects_DimensionType_Size_ToroidalMajorRadius`.
    SizeToroidalMajorRadius = 22,
    /// `XCAFDimTolObjects_DimensionType_Size_ToroidalHighMajorDiameter`.
    SizeToroidalHighMajorDiameter = 23,
    /// `XCAFDimTolObjects_DimensionType_Size_ToroidalLowMajorDiameter`.
    SizeToroidalLowMajorDiameter = 24,
    /// `XCAFDimTolObjects_DimensionType_Size_ToroidalHighMajorRadius`.
    SizeToroidalHighMajorRadius = 25,
    /// `XCAFDimTolObjects_DimensionType_Size_ToroidalLowMajorRadius`.
    SizeToroidalLowMajorRadius = 26,
    /// `XCAFDimTolObjects_DimensionType_Size_Thickness`.
    SizeThickness = 27,
    /// `XCAFDimTolObjects_DimensionType_Size_Angular`.
    SizeAngular = 28,
    /// `XCAFDimTolObjects_DimensionType_Size_WithPath`.
    SizeWithPath = 29,
    /// `XCAFDimTolObjects_DimensionType_CommonLabel`.
    CommonLabel = 30,
    /// `XCAFDimTolObjects_DimensionType_DimensionPresentation`.
    DimensionPresentation = 31,
}

impl crate::NativeEnum for XCAFDimTolObjectsDimensionType {
    const ALL: &'static [Self] = &[
        Self::LocationNone,
        Self::LocationCurvedDistance,
        Self::LocationLinearDistance,
        Self::LocationLinearDistanceFromCenterToOuter,
        Self::LocationLinearDistanceFromCenterToInner,
        Self::LocationLinearDistanceFromOuterToCenter,
        Self::LocationLinearDistanceFromOuterToOuter,
        Self::LocationLinearDistanceFromOuterToInner,
        Self::LocationLinearDistanceFromInnerToCenter,
        Self::LocationLinearDistanceFromInnerToOuter,
        Self::LocationLinearDistanceFromInnerToInner,
        Self::LocationAngular,
        Self::LocationOriented,
        Self::LocationWithPath,
        Self::SizeCurveLength,
        Self::SizeDiameter,
        Self::SizeSphericalDiameter,
        Self::SizeRadius,
        Self::SizeSphericalRadius,
        Self::SizeToroidalMinorDiameter,
        Self::SizeToroidalMajorDiameter,
        Self::SizeToroidalMinorRadius,
        Self::SizeToroidalMajorRadius,
        Self::SizeToroidalHighMajorDiameter,
        Self::SizeToroidalLowMajorDiameter,
        Self::SizeToroidalHighMajorRadius,
        Self::SizeToroidalLowMajorRadius,
        Self::SizeThickness,
        Self::SizeAngular,
        Self::SizeWithPath,
        Self::CommonLabel,
        Self::DimensionPresentation,
    ];
}
