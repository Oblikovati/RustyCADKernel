//! Texture unit, texture kind and texture filtering enumerations of Graphic3d.

/// Texture unit.
///
/// Native enumeration `Graphic3d_TextureUnit` (`Graphic3d_TextureUnit.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTextureUnit {
    /// `Graphic3d_TextureUnit_0`.
    Unit0 = 0,
    /// `Graphic3d_TextureUnit_1`.
    Unit1 = 1,
    /// `Graphic3d_TextureUnit_2`.
    Unit2 = 2,
    /// `Graphic3d_TextureUnit_3`.
    Unit3 = 3,
    /// `Graphic3d_TextureUnit_4`.
    Unit4 = 4,
    /// `Graphic3d_TextureUnit_5`.
    Unit5 = 5,
    /// `Graphic3d_TextureUnit_6`.
    Unit6 = 6,
    /// `Graphic3d_TextureUnit_7`.
    Unit7 = 7,
    /// `Graphic3d_TextureUnit_8`.
    Unit8 = 8,
    /// `Graphic3d_TextureUnit_9`.
    Unit9 = 9,
    /// `Graphic3d_TextureUnit_10`.
    Unit10 = 10,
    /// `Graphic3d_TextureUnit_11`.
    Unit11 = 11,
    /// `Graphic3d_TextureUnit_12`.
    Unit12 = 12,
    /// `Graphic3d_TextureUnit_13`.
    Unit13 = 13,
    /// `Graphic3d_TextureUnit_14`.
    Unit14 = 14,
    /// `Graphic3d_TextureUnit_15`.
    Unit15 = 15,
    /// `Graphic3d_TextureUnit_DepthPeelingDepth`.
    DepthPeelingDepth = -6,
    /// `Graphic3d_TextureUnit_DepthPeelingFrontColor`.
    DepthPeelingFrontColor = -5,
    /// `Graphic3d_TextureUnit_ShadowMap`.
    ShadowMap = -4,
    /// `Graphic3d_TextureUnit_PbrEnvironmentLUT`.
    PbrEnvironmentLUT = -3,
    /// `Graphic3d_TextureUnit_PbrIblDiffuseSH`.
    PbrIblDiffuseSH = -2,
    /// `Graphic3d_TextureUnit_PbrIblSpecular`.
    PbrIblSpecular = -1,
}
impl Graphic3dTextureUnit {
    /// Obsolete alias `Graphic3d_TextureUnit_BaseColor` of [`Self::Unit0`].
    pub const BASE_COLOR: Self = Self::Unit0;
    /// Obsolete alias `Graphic3d_TextureUnit_Emissive` of [`Self::Unit1`].
    pub const EMISSIVE: Self = Self::Unit1;
    /// Obsolete alias `Graphic3d_TextureUnit_Occlusion` of [`Self::Unit2`].
    pub const OCCLUSION: Self = Self::Unit2;
    /// Obsolete alias `Graphic3d_TextureUnit_Normal` of [`Self::Unit3`].
    pub const NORMAL: Self = Self::Unit3;
    /// Obsolete alias `Graphic3d_TextureUnit_MetallicRoughness` of [`Self::Unit4`].
    pub const METALLIC_ROUGHNESS: Self = Self::Unit4;
    /// Obsolete alias `Graphic3d_TextureUnit_EnvMap` of [`Self::Unit0`].
    pub const ENV_MAP: Self = Self::Unit0;
    /// Obsolete alias `Graphic3d_TextureUnit_PointSprite` of [`Self::Unit1`].
    pub const POINT_SPRITE: Self = Self::Unit1;
}
/// `Graphic3d_TextureUnit_NB`. (`Graphic3d_TextureUnit_NB`)
pub const GRAPHIC3D_TEXTURE_UNIT_NB: i32 = 16;

/// Standard texture units combination bits.
///
/// Native enumeration `Graphic3d_TextureSetBits` (`Graphic3d_TextureSetBits.hxx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Graphic3dTextureSetBits(pub i32);
impl Graphic3dTextureSetBits {
    /// `Graphic3d_TextureSetBits_NONE`.
    pub const NONE: Self = Self(0);
    /// `Graphic3d_TextureSetBits_BaseColor`.
    pub const BASE_COLOR: Self = Self(1);
    /// `Graphic3d_TextureSetBits_Emissive`.
    pub const EMISSIVE: Self = Self(2);
    /// `Graphic3d_TextureSetBits_Occlusion`.
    pub const OCCLUSION: Self = Self(4);
    /// `Graphic3d_TextureSetBits_Normal`.
    pub const NORMAL: Self = Self(8);
    /// `Graphic3d_TextureSetBits_MetallicRoughness`.
    pub const METALLIC_ROUGHNESS: Self = Self(16);
}

/// Level of anisotropy filter.
/// Notice that actual quality depends on hardware capabilities!
///
/// Native enumeration `Graphic3d_LevelOfTextureAnisotropy` (`Graphic3d_LevelOfTextureAnisotropy.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dLevelOfTextureAnisotropy {
    /// `Graphic3d_LOTA_OFF`.
    Off = 0,
    /// `Graphic3d_LOTA_FAST`.
    Fast = 1,
    /// `Graphic3d_LOTA_MIDDLE`.
    Middle = 2,
    /// `Graphic3d_LOTA_QUALITY`.
    Quality = 3,
}

/// Types of standard textures.
///
/// Native enumeration `Graphic3d_NameOfTexture1D` (`Graphic3d_NameOfTexture1D.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dNameOfTexture1D {
    /// `Graphic3d_NOT_1D_ELEVATION`.
    Elevation = 0,
    /// `Graphic3d_NOT_1D_UNKNOWN`.
    Unknown = 1,
}

/// Types of standard textures.
///
/// Native enumeration `Graphic3d_NameOfTexture2D` (`Graphic3d_NameOfTexture2D.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dNameOfTexture2D {
    /// `Graphic3d_NOT_2D_MATRA`.
    Matra = 0,
    /// `Graphic3d_NOT_2D_ALIENSKIN`.
    Alienskin = 1,
    /// `Graphic3d_NOT_2D_BLUE_ROCK`.
    BlueRock = 2,
    /// `Graphic3d_NOT_2D_BLUEWHITE_PAPER`.
    BluewhitePaper = 3,
    /// `Graphic3d_NOT_2D_BRUSHED`.
    Brushed = 4,
    /// `Graphic3d_NOT_2D_BUBBLES`.
    Bubbles = 5,
    /// `Graphic3d_NOT_2D_BUMP`.
    Bump = 6,
    /// `Graphic3d_NOT_2D_CAST`.
    Cast = 7,
    /// `Graphic3d_NOT_2D_CHIPBD`.
    Chipbd = 8,
    /// `Graphic3d_NOT_2D_CLOUDS`.
    Clouds = 9,
    /// `Graphic3d_NOT_2D_FLESH`.
    Flesh = 10,
    /// `Graphic3d_NOT_2D_FLOOR`.
    Floor = 11,
    /// `Graphic3d_NOT_2D_GALVNISD`.
    Galvnisd = 12,
    /// `Graphic3d_NOT_2D_GRASS`.
    Grass = 13,
    /// `Graphic3d_NOT_2D_ALUMINUM`.
    Aluminum = 14,
    /// `Graphic3d_NOT_2D_ROCK`.
    Rock = 15,
    /// `Graphic3d_NOT_2D_KNURL`.
    Knurl = 16,
    /// `Graphic3d_NOT_2D_MAPLE`.
    Maple = 17,
    /// `Graphic3d_NOT_2D_MARBLE`.
    Marble = 18,
    /// `Graphic3d_NOT_2D_MOTTLED`.
    Mottled = 19,
    /// `Graphic3d_NOT_2D_RAIN`.
    Rain = 20,
    /// `Graphic3d_NOT_2D_CHESS`.
    Chess = 21,
    /// `Graphic3d_NOT_2D_UNKNOWN`.
    Unknown = 22,
}

/// Types of standard textures.
///
/// Native enumeration `Graphic3d_NameOfTextureEnv` (`Graphic3d_NameOfTextureEnv.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dNameOfTextureEnv {
    /// `Graphic3d_NOT_ENV_CLOUDS`.
    Clouds = 0,
    /// `Graphic3d_NOT_ENV_CV`.
    Cv = 1,
    /// `Graphic3d_NOT_ENV_MEDIT`.
    Medit = 2,
    /// `Graphic3d_NOT_ENV_PEARL`.
    Pearl = 3,
    /// `Graphic3d_NOT_ENV_SKY1`.
    Sky1 = 4,
    /// `Graphic3d_NOT_ENV_SKY2`.
    Sky2 = 5,
    /// `Graphic3d_NOT_ENV_LINES`.
    Lines = 6,
    /// `Graphic3d_NOT_ENV_ROAD`.
    Road = 7,
    /// `Graphic3d_NOT_ENV_UNKNOWN`.
    Unknown = 8,
}

/// Type of the texture projection plane for both S and T texture coordinate.
///
/// Native enumeration `Graphic3d_NameOfTexturePlane` (`Graphic3d_NameOfTexturePlane.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dNameOfTexturePlane {
    /// `Graphic3d_NOTP_XY`.
    Xy = 0,
    /// `Graphic3d_NOTP_YZ`.
    Yz = 1,
    /// `Graphic3d_NOTP_ZX`.
    Zx = 2,
    /// `Graphic3d_NOTP_UNKNOWN`.
    Unknown = 3,
}

/// Type of the texture file format.
///
/// Native enumeration `Graphic3d_TypeOfTexture` (`Graphic3d_TypeOfTexture.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfTexture {
    /// `Graphic3d_TypeOfTexture_1D`.
    Texture1d = 0,
    /// `Graphic3d_TypeOfTexture_2D`.
    Texture2d = 1,
    /// `Graphic3d_TypeOfTexture_3D`.
    Texture3d = 2,
    /// `Graphic3d_TypeOfTexture_CUBEMAP`.
    Cubemap = 3,
    /// `Graphic3d_TOT_2D_MIPMAP`.
    Tot2dMipmap = 4,
}
impl Graphic3dTypeOfTexture {
    /// Obsolete alias `Graphic3d_TOT_1D` of [`Self::Texture1d`].
    pub const TOT_1D: Self = Self::Texture1d;
    /// Obsolete alias `Graphic3d_TOT_2D` of [`Self::Texture2d`].
    pub const TOT_2D: Self = Self::Texture2d;
    /// Obsolete alias `Graphic3d_TOT_CUBEMAP` of [`Self::Cubemap`].
    pub const TOT_CUBEMAP: Self = Self::Cubemap;
}

/// Type of the texture filter.
/// Notice that for textures without mipmaps linear interpolation will be used instead of
/// TOTF_BILINEAR and TOTF_TRILINEAR.
///
/// Native enumeration `Graphic3d_TypeOfTextureFilter` (`Graphic3d_TypeOfTextureFilter.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfTextureFilter {
    /// `Graphic3d_TOTF_NEAREST`.
    Nearest = 0,
    /// `Graphic3d_TOTF_BILINEAR`.
    Bilinear = 1,
    /// `Graphic3d_TOTF_TRILINEAR`.
    Trilinear = 2,
}

/// Type of the texture projection.
///
/// Native enumeration `Graphic3d_TypeOfTextureMode` (`Graphic3d_TypeOfTextureMode.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfTextureMode {
    /// `Graphic3d_TOTM_OBJECT`.
    Object = 0,
    /// `Graphic3d_TOTM_SPHERE`.
    Sphere = 1,
    /// `Graphic3d_TOTM_EYE`.
    Eye = 2,
    /// `Graphic3d_TOTM_MANUAL`.
    Manual = 3,
    /// `Graphic3d_TOTM_SPRITE`.
    Sprite = 4,
}
