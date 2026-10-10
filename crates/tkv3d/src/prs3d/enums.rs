//! The Prs3d enumerations, with their native discriminants. The legacy aliases of the native
//! enumerators (`Prs3d_DA_XAxisLength`...) are associated constants.

/// `Prs3d_DatumAttribute`: the numeric attributes of a datum.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Prs3dDatumAttribute {
    /// `Prs3d_DatumAttribute_XAxisLength`.
    XAxisLength = 0,
    /// `Prs3d_DatumAttribute_YAxisLength`.
    YAxisLength = 1,
    /// `Prs3d_DatumAttribute_ZAxisLength`.
    ZAxisLength = 2,
    /// `Prs3d_DatumAttribute_ShadingTubeRadiusPercent`.
    ShadingTubeRadiusPercent = 3,
    /// `Prs3d_DatumAttribute_ShadingConeRadiusPercent`.
    ShadingConeRadiusPercent = 4,
    /// `Prs3d_DatumAttribute_ShadingConeLengthPercent`.
    ShadingConeLengthPercent = 5,
    /// `Prs3d_DatumAttribute_ShadingOriginRadiusPercent`.
    ShadingOriginRadiusPercent = 6,
    /// `Prs3d_DatumAttribute_ShadingNumberOfFacettes`.
    ShadingNumberOfFacettes = 7,
}

impl Prs3dDatumAttribute {
    /// `Prs3d_DatumAttribute_NB`: the number of attributes.
    pub const NB: usize = 8;
    /// `Prs3d_DA_XAxisLength`.
    pub const DA_X_AXIS_LENGTH: Self = Self::XAxisLength;
    /// `Prs3d_DA_YAxisLength`.
    pub const DA_Y_AXIS_LENGTH: Self = Self::YAxisLength;
    /// `Prs3d_DA_ZAxisLength`.
    pub const DA_Z_AXIS_LENGTH: Self = Self::ZAxisLength;
    /// `Prs3d_DP_ShadingTubeRadiusPercent`.
    pub const DP_SHADING_TUBE_RADIUS_PERCENT: Self = Self::ShadingTubeRadiusPercent;
    /// `Prs3d_DP_ShadingConeRadiusPercent`.
    pub const DP_SHADING_CONE_RADIUS_PERCENT: Self = Self::ShadingConeRadiusPercent;
    /// `Prs3d_DP_ShadingConeLengthPercent`.
    pub const DP_SHADING_CONE_LENGTH_PERCENT: Self = Self::ShadingConeLengthPercent;
    /// `Prs3d_DP_ShadingOriginRadiusPercent`.
    pub const DP_SHADING_ORIGIN_RADIUS_PERCENT: Self = Self::ShadingOriginRadiusPercent;
    /// `Prs3d_DP_ShadingNumberOfFacettes`.
    pub const DP_SHADING_NUMBER_OF_FACETTES: Self = Self::ShadingNumberOfFacettes;
}

/// `Prs3d_DatumAxes`: the axes of a datum, a bit mask of X (1), Y (2) and Z (4).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prs3dDatumAxes {
    /// `Prs3d_DatumAxes_XAxis`.
    XAxis = 0x01,
    /// `Prs3d_DatumAxes_YAxis`.
    YAxis = 0x02,
    /// `Prs3d_DatumAxes_XYAxes`.
    XYAxes = 0x03,
    /// `Prs3d_DatumAxes_ZAxis`.
    ZAxis = 0x04,
    /// `Prs3d_DatumAxes_XZAxes`.
    XZAxes = 0x05,
    /// `Prs3d_DatumAxes_YZAxes`.
    YZAxes = 0x06,
    /// `Prs3d_DatumAxes_XYZAxes`.
    XYZAxes = 0x07,
}

impl Prs3dDatumAxes {
    /// `Prs3d_DA_XAxis`.
    pub const DA_X_AXIS: Self = Self::XAxis;
    /// `Prs3d_DA_YAxis`.
    pub const DA_Y_AXIS: Self = Self::YAxis;
    /// `Prs3d_DA_ZAxis`.
    pub const DA_Z_AXIS: Self = Self::ZAxis;
    /// `Prs3d_DA_XYAxis`.
    pub const DA_XY_AXIS: Self = Self::XYAxes;
    /// `Prs3d_DA_YZAxis`.
    pub const DA_YZ_AXIS: Self = Self::YZAxes;
    /// `Prs3d_DA_XZAxis`.
    pub const DA_XZ_AXIS: Self = Self::XZAxes;
    /// `Prs3d_DA_XYZAxis`.
    pub const DA_XYZ_AXIS: Self = Self::XYZAxes;
}

/// `Prs3d_DatumMode`: how a datum is drawn.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prs3dDatumMode {
    /// `Prs3d_DM_WireFrame`.
    WireFrame = 0,
    /// `Prs3d_DM_Shaded`.
    Shaded = 1,
}

/// `Prs3d_DatumParts`: the parts of a datum.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Prs3dDatumParts {
    /// `Prs3d_DatumParts_Origin`.
    Origin = 0,
    /// `Prs3d_DatumParts_XAxis`.
    XAxis = 1,
    /// `Prs3d_DatumParts_YAxis`.
    YAxis = 2,
    /// `Prs3d_DatumParts_ZAxis`.
    ZAxis = 3,
    /// `Prs3d_DatumParts_XArrow`.
    XArrow = 4,
    /// `Prs3d_DatumParts_YArrow`.
    YArrow = 5,
    /// `Prs3d_DatumParts_ZArrow`.
    ZArrow = 6,
    /// `Prs3d_DatumParts_XOYAxis`.
    XOYAxis = 7,
    /// `Prs3d_DatumParts_YOZAxis`.
    YOZAxis = 8,
    /// `Prs3d_DatumParts_XOZAxis`.
    XOZAxis = 9,
    /// `Prs3d_DatumParts_None`.
    None = 10,
}

impl Prs3dDatumParts {
    /// `Prs3d_DatumParts_NB`: the number of parts.
    pub const NB: usize = 11;
    /// Every part from `Origin` to `XOZAxis`, in native order.
    pub const DRAWN: [Self; 10] = [
        Self::Origin,
        Self::XAxis,
        Self::YAxis,
        Self::ZAxis,
        Self::XArrow,
        Self::YArrow,
        Self::ZArrow,
        Self::XOYAxis,
        Self::YOZAxis,
        Self::XOZAxis,
    ];
    /// `Prs3d_DP_Origin`.
    pub const DP_ORIGIN: Self = Self::Origin;
    /// `Prs3d_DP_XAxis`.
    pub const DP_X_AXIS: Self = Self::XAxis;
    /// `Prs3d_DP_YAxis`.
    pub const DP_Y_AXIS: Self = Self::YAxis;
    /// `Prs3d_DP_ZAxis`.
    pub const DP_Z_AXIS: Self = Self::ZAxis;
    /// `Prs3d_DP_XArrow`.
    pub const DP_X_ARROW: Self = Self::XArrow;
    /// `Prs3d_DP_YArrow`.
    pub const DP_Y_ARROW: Self = Self::YArrow;
    /// `Prs3d_DP_ZArrow`.
    pub const DP_Z_ARROW: Self = Self::ZArrow;
    /// `Prs3d_DP_XOYAxis`.
    pub const DP_XOY_AXIS: Self = Self::XOYAxis;
    /// `Prs3d_DP_YOZAxis`.
    pub const DP_YOZ_AXIS: Self = Self::YOZAxis;
    /// `Prs3d_DP_XOZAxis`.
    pub const DP_XOZ_AXIS: Self = Self::XOZAxis;
    /// `Prs3d_DP_None`.
    pub const DP_NONE: Self = Self::None;
}

/// `Prs3d_DimensionArrowOrientation`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prs3dDimensionArrowOrientation {
    /// `Prs3d_DAO_Internal`.
    Internal = 0,
    /// `Prs3d_DAO_External`.
    External = 1,
    /// `Prs3d_DAO_Fit`.
    Fit = 2,
}

/// `Prs3d_DimensionTextHorizontalPosition`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prs3dDimensionTextHorizontalPosition {
    /// `Prs3d_DTHP_Left`.
    Left = 0,
    /// `Prs3d_DTHP_Right`.
    Right = 1,
    /// `Prs3d_DTHP_Center`.
    Center = 2,
    /// `Prs3d_DTHP_Fit`.
    Fit = 3,
}

/// `Prs3d_DimensionTextVerticalPosition`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prs3dDimensionTextVerticalPosition {
    /// `Prs3d_DTVP_Above`.
    Above = 0,
    /// `Prs3d_DTVP_Below`.
    Below = 1,
    /// `Prs3d_DTVP_Center`.
    Center = 2,
}

/// `Prs3d_TypeOfHighlight`: the highlight styles of an interactive context.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Prs3dTypeOfHighlight {
    /// `Prs3d_TypeOfHighlight_None`.
    None = 0,
    /// `Prs3d_TypeOfHighlight_Selected`.
    Selected = 1,
    /// `Prs3d_TypeOfHighlight_Dynamic`.
    Dynamic = 2,
    /// `Prs3d_TypeOfHighlight_LocalSelected`.
    LocalSelected = 3,
    /// `Prs3d_TypeOfHighlight_LocalDynamic`.
    LocalDynamic = 4,
    /// `Prs3d_TypeOfHighlight_SubIntensity`.
    SubIntensity = 5,
    /// `Prs3d_TypeOfHighlight_NB`: the number of styles.
    Nb = 6,
}

/// `Prs3d_TypeOfHLR`: the hidden line removal algorithm.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prs3dTypeOfHlr {
    /// `Prs3d_TOH_NotSet`.
    NotSet = 0,
    /// `Prs3d_TOH_PolyAlgo`.
    PolyAlgo = 1,
    /// `Prs3d_TOH_Algo`.
    Algo = 2,
}

/// `Prs3d_TypeOfLinePicking`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prs3dTypeOfLinePicking {
    /// `Prs3d_TOLP_Point`.
    Point = 0,
    /// `Prs3d_TOLP_Segment`.
    Segment = 1,
}

/// `Prs3d_VertexDrawMode`: which vertices of a shape are drawn.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prs3dVertexDrawMode {
    /// `Prs3d_VDM_Isolated`: only the isolated vertices.
    Isolated = 0,
    /// `Prs3d_VDM_All`: every vertex.
    All = 1,
    /// `Prs3d_VDM_Inherited`: the mode of the link drawer.
    Inherited = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_and_aliases_match_the_native_values() {
        assert_eq!(
            Prs3dDatumAttribute::ShadingNumberOfFacettes as usize + 1,
            Prs3dDatumAttribute::NB
        );
        assert_eq!(Prs3dDatumParts::None as usize + 1, Prs3dDatumParts::NB);
        assert_eq!(Prs3dDatumAxes::XYZAxes as i32, 7);
        assert_eq!(Prs3dDatumAxes::DA_XZ_AXIS as i32, 5);
        assert_eq!(Prs3dDatumParts::DP_XOZ_AXIS, Prs3dDatumParts::XOZAxis);
        assert_eq!(Prs3dTypeOfHighlight::Nb as i32, 6);
        assert_eq!(Prs3dVertexDrawMode::Inherited as i32, 2);
        assert_eq!(Prs3dDimensionTextHorizontalPosition::Fit as i32, 3);
    }
}
