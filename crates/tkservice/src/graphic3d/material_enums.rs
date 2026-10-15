//! Material, shading, light, transform-persistence, text and primitive enumerations of Graphic3d.

/// List of named materials (predefined presets).
/// Each preset defines either physical (having natural color) or generic (mutable color) material
/// (@sa Graphic3d_TypeOfMaterial).
///
/// Native enumeration `Graphic3d_NameOfMaterial` (`Graphic3d_NameOfMaterial.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dNameOfMaterial {
    /// Brass        (Physic).
    Brass = 0,
    /// Bronze       (Physic).
    Bronze = 1,
    /// Copper       (Physic).
    Copper = 2,
    /// Gold         (Physic).
    Gold = 3,
    /// Pewter       (Physic).
    Pewter = 4,
    /// Plastered    (Generic).
    Plastered = 5,
    /// Plastified   (Generic).
    Plastified = 6,
    /// Silver       (Physic).
    Silver = 7,
    /// Steel        (Physic).
    Steel = 8,
    /// Stone        (Physic).
    Stone = 9,
    /// Shiny Plastified (Generic).
    ShinyPlastified = 10,
    /// Satin        (Generic).
    Satin = 11,
    /// Metalized    (Generic).
    Metalized = 12,
    /// Ionized      (Generic).
    Ionized = 13,
    /// Chrome       (Physic).
    Chrome = 14,
    /// Aluminum     (Physic).
    Aluminum = 15,
    /// Obsidian     (Physic).
    Obsidian = 16,
    /// Neon         (Physic).
    Neon = 17,
    /// Jade         (Physic).
    Jade = 18,
    /// Charcoal     (Physic).
    Charcoal = 19,
    /// Water        (Physic).
    Water = 20,
    /// Glass        (Physic).
    Glass = 21,
    /// Diamond      (Physic).
    Diamond = 22,
    /// Transparent  (Physic).
    Transparent = 23,
    /// Default      (Generic);.
    Default = 24,
    /// User-defined (Physic);.
    UserDefined = 25,
}
impl Graphic3dNameOfMaterial {
    /// Obsolete alias `Graphic3d_NOM_BRASS` of [`Self::Brass`].
    pub const NOM_BRASS: Self = Self::Brass;
    /// Obsolete alias `Graphic3d_NOM_BRONZE` of [`Self::Bronze`].
    pub const NOM_BRONZE: Self = Self::Bronze;
    /// Obsolete alias `Graphic3d_NOM_COPPER` of [`Self::Copper`].
    pub const NOM_COPPER: Self = Self::Copper;
    /// Obsolete alias `Graphic3d_NOM_GOLD` of [`Self::Gold`].
    pub const NOM_GOLD: Self = Self::Gold;
    /// Obsolete alias `Graphic3d_NOM_PEWTER` of [`Self::Pewter`].
    pub const NOM_PEWTER: Self = Self::Pewter;
    /// Obsolete alias `Graphic3d_NOM_PLASTER` of [`Self::Plastered`].
    pub const NOM_PLASTER: Self = Self::Plastered;
    /// Obsolete alias `Graphic3d_NOM_PLASTIC` of [`Self::Plastified`].
    pub const NOM_PLASTIC: Self = Self::Plastified;
    /// Obsolete alias `Graphic3d_NOM_SILVER` of [`Self::Silver`].
    pub const NOM_SILVER: Self = Self::Silver;
    /// Obsolete alias `Graphic3d_NOM_STEEL` of [`Self::Steel`].
    pub const NOM_STEEL: Self = Self::Steel;
    /// Obsolete alias `Graphic3d_NOM_STONE` of [`Self::Stone`].
    pub const NOM_STONE: Self = Self::Stone;
    /// Obsolete alias `Graphic3d_NOM_SHINY_PLASTIC` of [`Self::ShinyPlastified`].
    pub const NOM_SHINY_PLASTIC: Self = Self::ShinyPlastified;
    /// Obsolete alias `Graphic3d_NOM_SATIN` of [`Self::Satin`].
    pub const NOM_SATIN: Self = Self::Satin;
    /// Obsolete alias `Graphic3d_NOM_METALIZED` of [`Self::Metalized`].
    pub const NOM_METALIZED: Self = Self::Metalized;
    /// Obsolete alias `Graphic3d_NOM_NEON_GNC` of [`Self::Ionized`].
    pub const NOM_NEON_GNC: Self = Self::Ionized;
    /// Obsolete alias `Graphic3d_NOM_CHROME` of [`Self::Chrome`].
    pub const NOM_CHROME: Self = Self::Chrome;
    /// Obsolete alias `Graphic3d_NOM_ALUMINIUM` of [`Self::Aluminum`].
    pub const NOM_ALUMINIUM: Self = Self::Aluminum;
    /// Obsolete alias `Graphic3d_NOM_OBSIDIAN` of [`Self::Obsidian`].
    pub const NOM_OBSIDIAN: Self = Self::Obsidian;
    /// Obsolete alias `Graphic3d_NOM_NEON_PHC` of [`Self::Neon`].
    pub const NOM_NEON_PHC: Self = Self::Neon;
    /// Obsolete alias `Graphic3d_NOM_JADE` of [`Self::Jade`].
    pub const NOM_JADE: Self = Self::Jade;
    /// Obsolete alias `Graphic3d_NOM_CHARCOAL` of [`Self::Charcoal`].
    pub const NOM_CHARCOAL: Self = Self::Charcoal;
    /// Obsolete alias `Graphic3d_NOM_WATER` of [`Self::Water`].
    pub const NOM_WATER: Self = Self::Water;
    /// Obsolete alias `Graphic3d_NOM_GLASS` of [`Self::Glass`].
    pub const NOM_GLASS: Self = Self::Glass;
    /// Obsolete alias `Graphic3d_NOM_DIAMOND` of [`Self::Diamond`].
    pub const NOM_DIAMOND: Self = Self::Diamond;
    /// Obsolete alias `Graphic3d_NOM_TRANSPARENT` of [`Self::Transparent`].
    pub const NOM_TRANSPARENT: Self = Self::Transparent;
    /// Obsolete alias `Graphic3d_NOM_DEFAULT` of [`Self::Default`].
    pub const NOM_DEFAULT: Self = Self::Default;
    /// Obsolete alias `Graphic3d_NOM_UserDefined` of [`Self::UserDefined`].
    pub const NOM_USER_DEFINED: Self = Self::UserDefined;
    /// Every enumerator in value order (`ALL[i] as i32 == i`).
    pub const ALL: [Self; 26] = [
        Self::Brass,
        Self::Bronze,
        Self::Copper,
        Self::Gold,
        Self::Pewter,
        Self::Plastered,
        Self::Plastified,
        Self::Silver,
        Self::Steel,
        Self::Stone,
        Self::ShinyPlastified,
        Self::Satin,
        Self::Metalized,
        Self::Ionized,
        Self::Chrome,
        Self::Aluminum,
        Self::Obsidian,
        Self::Neon,
        Self::Jade,
        Self::Charcoal,
        Self::Water,
        Self::Glass,
        Self::Diamond,
        Self::Transparent,
        Self::Default,
        Self::UserDefined,
    ];
    /// The enumerator of a value, as the native cast `(Graphic3d_NameOfMaterial)value`
    /// where the value is valid; `None` outside 0..=25.
    ///
    /// ```
    /// use rustycad_tkservice::graphic3d::Graphic3dNameOfMaterial;
    /// assert_eq!(Graphic3dNameOfMaterial::from_i32(18), Some(Graphic3dNameOfMaterial::Jade));
    /// assert_eq!(Graphic3dNameOfMaterial::from_i32(26), None);
    /// ```
    pub fn from_i32(value: i32) -> Option<Self> {
        usize::try_from(value)
            .ok()
            .and_then(|index| Self::ALL.get(index).copied())
    }
}

/// Types of materials specifies if a material can change color.
///
/// Native enumeration `Graphic3d_TypeOfMaterial` (`Graphic3d_TypeOfMaterial.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfMaterial {
    /// aspect   material definition with configurable color (like plastic).
    Aspect = 0,
    /// physical material definition with fixed color (like gold).
    Physic = 1,
}

/// Nature of the reflection of a material.
///
/// Native enumeration `Graphic3d_TypeOfReflection` (`Graphic3d_TypeOfReflection.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfReflection {
    /// `Graphic3d_TOR_AMBIENT`.
    Ambient = 0,
    /// `Graphic3d_TOR_DIFFUSE`.
    Diffuse = 1,
    /// `Graphic3d_TOR_SPECULAR`.
    Specular = 2,
    /// `Graphic3d_TOR_EMISSION`.
    Emission = 3,
}
/// `Graphic3d_TypeOfReflection_NB`. (`Graphic3d_TypeOfReflection_NB`)
pub const GRAPHIC3D_TYPE_OF_REFLECTION_NB: i32 = 4;

/// Definition of the color shading model.
///
/// Native enumeration `Graphic3d_TypeOfShadingModel` (`Graphic3d_TypeOfShadingModel.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfShadingModel {
    /// `Graphic3d_TypeOfShadingModel_DEFAULT`.
    Default = -1,
    /// `Graphic3d_TypeOfShadingModel_Unlit`.
    Unlit = 0,
    /// `Graphic3d_TypeOfShadingModel_PhongFacet`.
    PhongFacet = 1,
    /// `Graphic3d_TypeOfShadingModel_Gouraud`.
    Gouraud = 2,
    /// `Graphic3d_TypeOfShadingModel_Phong`.
    Phong = 3,
    /// `Graphic3d_TypeOfShadingModel_Pbr`.
    Pbr = 4,
    /// `Graphic3d_TypeOfShadingModel_PbrFacet`.
    PbrFacet = 5,
}
impl Graphic3dTypeOfShadingModel {
    /// Obsolete alias `Graphic3d_TOSM_DEFAULT` of [`Self::Default`].
    pub const TOSM_DEFAULT: Self = Self::Default;
    /// Obsolete alias `Graphic3d_TOSM_UNLIT` of [`Self::Unlit`].
    pub const TOSM_UNLIT: Self = Self::Unlit;
    /// Obsolete alias `Graphic3d_TOSM_FACET` of [`Self::PhongFacet`].
    pub const TOSM_FACET: Self = Self::PhongFacet;
    /// Obsolete alias `Graphic3d_TOSM_VERTEX` of [`Self::Gouraud`].
    pub const TOSM_VERTEX: Self = Self::Gouraud;
    /// Obsolete alias `Graphic3d_TOSM_FRAGMENT` of [`Self::Phong`].
    pub const TOSM_FRAGMENT: Self = Self::Phong;
    /// Obsolete alias `Graphic3d_TOSM_PBR` of [`Self::Pbr`].
    pub const TOSM_PBR: Self = Self::Pbr;
    /// Obsolete alias `Graphic3d_TOSM_PBR_FACET` of [`Self::PbrFacet`].
    pub const TOSM_PBR_FACET: Self = Self::PbrFacet;
    /// Obsolete alias `Graphic3d_TOSM_NONE` of [`Self::Unlit`].
    pub const TOSM_NONE: Self = Self::Unlit;
    /// Obsolete alias `V3d_COLOR` of [`Self::Unlit`].
    pub const V3D_COLOR: Self = Self::Unlit;
    /// Obsolete alias `V3d_FLAT` of [`Self::PhongFacet`].
    pub const V3D_FLAT: Self = Self::PhongFacet;
    /// Obsolete alias `V3d_GOURAUD` of [`Self::Gouraud`].
    pub const V3D_GOURAUD: Self = Self::Gouraud;
    /// Obsolete alias `V3d_PHONG` of [`Self::Phong`].
    pub const V3D_PHONG: Self = Self::Phong;
}
/// `Graphic3d_TypeOfShadingModel_NB`. (`Graphic3d_TypeOfShadingModel_NB`)
pub const GRAPHIC3D_TYPE_OF_SHADING_MODEL_NB: i32 = 6;

/// Definition of all the type of light source.
///
/// Native enumeration `Graphic3d_TypeOfLightSource` (`Graphic3d_TypeOfLightSource.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfLightSource {
    /// ambient light.
    Ambient = 0,
    /// directional light.
    Directional = 1,
    /// positional light.
    Positional = 2,
    /// spot light.
    Spot = 3,
}
impl Graphic3dTypeOfLightSource {
    /// Obsolete alias `Graphic3d_TOLS_AMBIENT` of [`Self::Ambient`].
    pub const TOLS_AMBIENT: Self = Self::Ambient;
    /// Obsolete alias `Graphic3d_TOLS_DIRECTIONAL` of [`Self::Directional`].
    pub const TOLS_DIRECTIONAL: Self = Self::Directional;
    /// Obsolete alias `Graphic3d_TOLS_POSITIONAL` of [`Self::Positional`].
    pub const TOLS_POSITIONAL: Self = Self::Positional;
    /// Obsolete alias `Graphic3d_TOLS_SPOT` of [`Self::Spot`].
    pub const TOLS_SPOT: Self = Self::Spot;
    /// Obsolete alias `V3d_AMBIENT` of [`Self::Ambient`].
    pub const V3D_AMBIENT: Self = Self::Ambient;
    /// Obsolete alias `V3d_DIRECTIONAL` of [`Self::Directional`].
    pub const V3D_DIRECTIONAL: Self = Self::Directional;
    /// Obsolete alias `V3d_POSITIONAL` of [`Self::Positional`].
    pub const V3D_POSITIONAL: Self = Self::Positional;
    /// Obsolete alias `V3d_SPOT` of [`Self::Spot`].
    pub const V3D_SPOT: Self = Self::Spot;
}
/// `Graphic3d_TypeOfLightSource_NB`. (`Graphic3d_TypeOfLightSource_NB`)
pub const GRAPHIC3D_TYPE_OF_LIGHT_SOURCE_NB: i32 = 4;

/// Transform Persistence Mode defining whether to lock in object position, rotation and / or
/// zooming relative to camera position.
///
/// Native enumeration `Graphic3d_TransModeFlags` (`Graphic3d_TransModeFlags.hxx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Graphic3dTransModeFlags(pub i32);
impl Graphic3dTransModeFlags {
    /// no persistence attributes (normal 3D object).
    pub const NONE: Self = Self(0);
    /// object does not resize.
    pub const ZOOM_PERS: Self = Self(2);
    /// object does not rotate;.
    pub const ROTATE_PERS: Self = Self(8);
    /// object behaves like trihedron - it is fixed at the corner of view and does not resizing (but rotating).
    pub const TRIEDRON_PERS: Self = Self(32);
    /// object is defined in 2D screen coordinates (pixels) and does not resize, pan and rotate.
    pub const TMF_2D: Self = Self(64);
    /// object is in front of the camera.
    pub const CAMERA_PERS: Self = Self(128);
    /// object is forced to be rendered with orthographic projection.
    pub const ORTHO_PERS: Self = Self(256);
    /// object does not resize with axial scale.
    pub const AXIAL_SCALE_PERS: Self = Self(512);
    /// object doesn't resize and rotate.
    pub const ZOOM_ROTATE_PERS: Self = Self(10);
    /// object does not visually resize with either object or axial scale.
    pub const AXIAL_ZOOM_PERS: Self = Self(514);
}

/// Defines the horizontal position of the text
/// relative to its anchor.
///
/// Native enumeration `Graphic3d_HorizontalTextAlignment` (`Graphic3d_HorizontalTextAlignment.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dHorizontalTextAlignment {
    /// `Graphic3d_HTA_LEFT`.
    Left = 0,
    /// `Graphic3d_HTA_CENTER`.
    Center = 1,
    /// `Graphic3d_HTA_RIGHT`.
    Right = 2,
}

/// Defines the vertical position of the text
/// relative to its anchor.
///
/// Native enumeration `Graphic3d_VerticalTextAlignment` (`Graphic3d_VerticalTextAlignment.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dVerticalTextAlignment {
    /// The anchor lies on the last line of the text.
    Bottom = 0,
    /// The anchor lies on the center of the text (not bounding box).
    Center = 1,
    /// The anchor lies on the line preceding to the first line of the text.
    Top = 2,
    /// The anchor lies on the first line of the text.
    Topfirstline = 3,
}

/// Direction in which text is displayed.
///
/// Native enumeration `Graphic3d_TextPath` (`Graphic3d_TextPath.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTextPath {
    /// `Graphic3d_TP_UP`.
    Up = 0,
    /// `Graphic3d_TP_DOWN`.
    Down = 1,
    /// `Graphic3d_TP_LEFT`.
    Left = 2,
    /// `Graphic3d_TP_RIGHT`.
    Right = 3,
}

/// The type of primitive array in a group in a structure.
///
/// Native enumeration `Graphic3d_TypeOfPrimitiveArray` (`Graphic3d_TypeOfPrimitiveArray.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfPrimitiveArray {
    /// undefined primitive type.
    Undefined = 0,
    /// individual points.
    Points = 1,
    /// segments array - each 2 vertexes define 1 segment.
    Segments = 2,
    /// line strip - each new vertex in array defines segment with previous one.
    Polylines = 3,
    /// triangle array - each 3 vertexes define 1 triangle.
    Triangles = 4,
    /// triangle strip - each new vertex in array defines triangle with 2 previous vertexes.
    Trianglestrips = 5,
    /// triangle fan - each new vertex in array define triangle with the previous vertex and the very first vertex (fan center).
    Trianglefans = 6,
    /// ADVANCED - same as Graphic3d_TOPA_SEGMENTS, but each pair of vertexes defining 1 segment.
    LinesAdjacency = 7,
    /// ADVANCED - same as Graphic3d_TOPA_POLYLINES, but each sequence of vertexes defining 1 polyline.
    LineStripAdjacency = 8,
    /// ADVANCED - same as Graphic3d_TOPA_TRIANGLES, but each vertex defining of triangle.
    TrianglesAdjacency = 9,
    /// ADVANCED - same as Graphic3d_TOPA_TRIANGLESTRIPS, but with extra adjacent vertexes.
    TriangleStripAdjacency = 10,
    /// DEPRECATED - triangle array should be used instead;.
    Quadrangles = 11,
    /// DEPRECATED - triangle array should be used instead;.
    Quadranglestrips = 12,
    /// DEPRECATED - triangle array should be used instead;.
    Polygons = 13,
}

impl std::ops::BitOr for Graphic3dTransModeFlags {
    type Output = Self;
    /// `operator|` of two transform-persistence modes (on their unsigned values).
    fn bitor(self, other: Self) -> Self {
        Self(((self.0 as u32) | (other.0 as u32)) as i32)
    }
}
