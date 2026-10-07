//! Rendering, display, structure and shader enumerations of Graphic3d.

/// Defines how alpha value of base color / texture should be treated.
///
/// Native enumeration `Graphic3d_AlphaMode` (`Graphic3d_AlphaMode.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dAlphaMode {
    /// rendered output is fully opaque and alpha value is ignored.
    Opaque = 0,
    /// rendered output is either fully opaque or fully transparent depending on the alpha value and the alpha cutoff value.
    Mask = 1,
    /// rendered output is combined with the background.
    Blend = 2,
    /// performs in-place blending (without implicit reordering of opaque objects) with alpha-test.
    MaskBlend = 3,
    /// special value defined for backward compatibility - it is equal to Graphic3d_AlphaMode_Blend when Material transparency is not zero and Graphic3d_AlphaMode_Opaque otherwise;.
    BlendAuto = -1,
}

/// Define buffers available for dump
///
/// Native enumeration `Graphic3d_BufferType` (`Graphic3d_BufferType.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dBufferType {
    /// color buffer without alpha component.
    Rgb = 0,
    /// color buffer.
    Rgba = 1,
    /// depth buffer.
    Depth = 2,
    /// left view HDR color buffer for Ray-Tracing.
    RgbRayTraceHdrLeft = 3,
    /// color buffer, red channel.
    Red = 4,
    /// buffer with shadow map.
    ShadowMap = 5,
}

/// Enumeration of capping flags.
///
/// Native enumeration `Graphic3d_CappingFlags` (`Graphic3d_CappingFlags.hxx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Graphic3dCappingFlags(pub i32);
impl Graphic3dCappingFlags {
    /// no flags.
    pub const NONE: Self = Self(0);
    /// use object material.
    pub const OBJECT_MATERIAL: Self = Self(1);
    /// use object texture.
    pub const OBJECT_TEXTURE: Self = Self(2);
    /// use object GLSL program.
    pub const OBJECT_SHADER: Self = Self(8);
    /// use entire fill area aspect from object.
    pub const OBJECT_ASPECT: Self = Self(11);
}

/// Sides of cubemap in order of OpenGL rules
///
/// Native enumeration `Graphic3d_CubeMapSide` (`Graphic3d_CubeMapSide.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dCubeMapSide {
    /// X axis positive direction side.
    PosX = 0,
    /// X axis negative direction side.
    NegX = 1,
    /// Y axis positive direction side.
    PosY = 2,
    /// Y axis negative direction side.
    NegY = 3,
    /// Z axis positive direction side.
    PosZ = 4,
    /// Z axis negative direction side.
    NegZ = 5,
}

/// Structure priority - range (do not change this range!).
/// Values are between 0 and 10, with 5 used by default.
/// A structure of priority 10 is displayed the last and appears over the others (considering depth
/// test).
///
/// Native enumeration `Graphic3d_DisplayPriority` (`Graphic3d_DisplayPriority.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dDisplayPriority {
    /// `Graphic3d_DisplayPriority_INVALID`.
    Invalid = -1,
    /// `Graphic3d_DisplayPriority_Bottom`.
    Bottom = 0,
    /// `Graphic3d_DisplayPriority_AlmostBottom`.
    AlmostBottom = 1,
    /// `Graphic3d_DisplayPriority_Below2`.
    Below2 = 2,
    /// `Graphic3d_DisplayPriority_Below1`.
    Below1 = 3,
    /// `Graphic3d_DisplayPriority_Below`.
    Below = 4,
    /// `Graphic3d_DisplayPriority_Normal`.
    Normal = 5,
    /// `Graphic3d_DisplayPriority_Above`.
    Above = 6,
    /// `Graphic3d_DisplayPriority_Above1`.
    Above1 = 7,
    /// `Graphic3d_DisplayPriority_Above2`.
    Above2 = 8,
    /// `Graphic3d_DisplayPriority_Highlight`.
    Highlight = 9,
    /// `Graphic3d_DisplayPriority_Topmost`.
    Topmost = 10,
}
/// `Graphic3d_DisplayPriority_NB`. (`Graphic3d_DisplayPriority_NB`)
pub const GRAPHIC3D_DISPLAY_PRIORITY_NB: i32 = 11;

/// Identifies primitives aspects defined per group.
/// - ASPECT_LINE: aspect for line primitives;
/// - ASPECT_TEXT: aspect for text primitives;
/// - ASPECT_MARKER: aspect for marker primitives;
/// - ASPECT_FILL_AREA: aspect for face primitives.
///
/// Native enumeration `Graphic3d_GroupAspect` (`Graphic3d_GroupAspect.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dGroupAspect {
    /// `Graphic3d_ASPECT_LINE`.
    Line = 0,
    /// `Graphic3d_ASPECT_TEXT`.
    Text = 1,
    /// `Graphic3d_ASPECT_MARKER`.
    Marker = 2,
    /// `Graphic3d_ASPECT_FILL_AREA`.
    FillArea = 3,
}

/// Enumerates transparency rendering methods supported by rasterization mode.
///
/// Native enumeration `Graphic3d_RenderTransparentMethod` (`Graphic3d_RenderTransparentMethod.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dRenderTransparentMethod {
    /// Basic blend transparency with non-commuting blend operator without sorting.
    BlendUnordered = 0,
    /// Weighted Blended Order-Independent Transparency with depth weight factor.
    BlendOit = 1,
    /// Depth Peeling with specified number of depth layers.
    DepthPeelingOit = 2,
}

/// Describes rendering modes.
/// - RM_RASTERIZATION: enables OpenGL rasterization mode;
/// - RM_RAYTRACING: enables GPU ray-tracing mode.
///
/// Native enumeration `Graphic3d_RenderingMode` (`Graphic3d_RenderingMode.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dRenderingMode {
    /// `Graphic3d_RM_RASTERIZATION`.
    Rasterization = 0,
    /// `Graphic3d_RM_RAYTRACING`.
    Raytracing = 1,
}

/// This enumeration defines the list of stereoscopic output modes.
///
/// Native enumeration `Graphic3d_StereoMode` (`Graphic3d_StereoMode.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dStereoMode {
    /// OpenGL QuadBuffer.
    QuadBuffer = 0,
    /// Anaglyph glasses, the type should be specified in addition.
    Anaglyph = 1,
    /// Row-interlaced stereo.
    RowInterlaced = 2,
    /// Column-interlaced stereo.
    ColumnInterlaced = 3,
    /// chess-board stereo for DLP TVs.
    ChessBoard = 4,
    /// horizontal pair.
    SideBySide = 5,
    /// vertical   pair.
    OverUnder = 6,
    /// software PageFlip for shutter glasses, should NOT be used!.
    SoftPageFlip = 7,
    /// OpenVR (HMD).
    OpenVR = 8,
}
/// `Graphic3d_StereoMode_NB`. (`Graphic3d_StereoMode_NB`)
pub const GRAPHIC3D_STEREO_MODE_NB: i32 = 9;

/// Enumerates tone mapping methods.
///
/// Native enumeration `Graphic3d_ToneMappingMethod` (`Graphic3d_ToneMappingMethod.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dToneMappingMethod {
    /// Don't use tone mapping.
    Disabled = 0,
    /// Use filmic tone mapping.
    Filmic = 1,
}

/// Modes of display of back faces in the view.
///
/// Native enumeration `Graphic3d_TypeOfBackfacingModel` (`Graphic3d_TypeOfBackfacingModel.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfBackfacingModel {
    /// automatic back face culling enabled for opaque groups with closed flag.
    Auto = 0,
    /// no culling (double-sided shading).
    DoubleSided = 1,
    /// back  face culling.
    BackCulled = 2,
    /// front face culling.
    FrontCulled = 3,
}
impl Graphic3dTypeOfBackfacingModel {
    /// Obsolete alias `Graphic3d_TOBM_AUTOMATIC` of [`Self::Auto`].
    pub const TOBM_AUTOMATIC: Self = Self::Auto;
    /// Obsolete alias `Graphic3d_TOBM_FORCE` of [`Self::DoubleSided`].
    pub const TOBM_FORCE: Self = Self::DoubleSided;
    /// Obsolete alias `Graphic3d_TOBM_DISABLE` of [`Self::BackCulled`].
    pub const TOBM_DISABLE: Self = Self::BackCulled;
    /// Obsolete alias `V3d_TOBM_AUTOMATIC` of [`Self::Auto`].
    pub const V3D_TOBM_AUTOMATIC: Self = Self::Auto;
    /// Obsolete alias `V3d_TOBM_ALWAYS_DISPLAYED` of [`Self::DoubleSided`].
    pub const V3D_TOBM_ALWAYS_DISPLAYED: Self = Self::DoubleSided;
    /// Obsolete alias `V3d_TOBM_NEVER_DISPLAYED` of [`Self::BackCulled`].
    pub const V3D_TOBM_NEVER_DISPLAYED: Self = Self::BackCulled;
}

/// Describes type of view background.
///
/// Native enumeration `Graphic3d_TypeOfBackground` (`Graphic3d_TypeOfBackground.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfBackground {
    /// `Graphic3d_TOB_NONE`.
    None = -1,
    /// `Graphic3d_TOB_GRADIENT`.
    Gradient = 0,
    /// `Graphic3d_TOB_TEXTURE`.
    Texture = 1,
    /// `Graphic3d_TOB_CUBEMAP`.
    Cubemap = 2,
}
/// `Graphic3d_TypeOfBackground_NB`. (`Graphic3d_TypeOfBackground_NB`)
pub const GRAPHIC3D_TYPE_OF_BACKGROUND_NB: i32 = 3;

/// The answer of the method AcceptDisplay
/// AcceptDisplay means is it possible to display the
/// specified structure in the specified view ?
/// TOA_YES yes
/// TOA_NO  no
/// TOA_COMPUTE yes but we have to compute the representation
///
/// Native enumeration `Graphic3d_TypeOfAnswer` (`Graphic3d_TypeOfAnswer.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfAnswer {
    /// `Graphic3d_TOA_YES`.
    Yes = 0,
    /// `Graphic3d_TOA_NO`.
    No = 1,
    /// `Graphic3d_TOA_COMPUTE`.
    Compute = 2,
}

/// To manage the connections between the structures.
///
/// Native enumeration `Graphic3d_TypeOfConnection` (`Graphic3d_TypeOfConnection.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfConnection {
    /// `Graphic3d_TOC_ANCESTOR`.
    Ancestor = 0,
    /// `Graphic3d_TOC_DESCENDANT`.
    Descendant = 1,
}

/// Structural attribute indicating if it can be displayed
/// in wireframe, shadow mode, or both.
///
/// Native enumeration `Graphic3d_TypeOfStructure` (`Graphic3d_TypeOfStructure.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfStructure {
    /// `Graphic3d_TOS_WIREFRAME`.
    Wireframe = 0,
    /// `Graphic3d_TOS_SHADING`.
    Shading = 1,
    /// `Graphic3d_TOS_COMPUTED`.
    Computed = 2,
    /// `Graphic3d_TOS_ALL`.
    All = 3,
}

/// Modes of visualisation of objects in a view
///
/// TOV_WIREFRAME   wireframe visualisation
/// TOV_SHADING     shaded visualisation
///
/// Native enumeration `Graphic3d_TypeOfVisualization` (`Graphic3d_TypeOfVisualization.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfVisualization {
    /// `Graphic3d_TOV_WIREFRAME`.
    Wireframe = 0,
    /// `Graphic3d_TOV_SHADING`.
    Shading = 1,
}

/// identifier for invalid ZLayer. (`Graphic3d_ZLayerId_UNKNOWN`)
pub const GRAPHIC3D_ZLAYER_ID_UNKNOWN: i32 = -1;
/// default Z-layer for main presentations. (`Graphic3d_ZLayerId_Default`)
pub const GRAPHIC3D_ZLAYER_ID_DEFAULT: i32 = 0;
/// overlay for 3D presentations which inherits Depth from previous ZLayer. (`Graphic3d_ZLayerId_Top`)
pub const GRAPHIC3D_ZLAYER_ID_TOP: i32 = -2;
/// overlay for 3D presentations with independent Depth. (`Graphic3d_ZLayerId_Topmost`)
pub const GRAPHIC3D_ZLAYER_ID_TOPMOST: i32 = -3;
/// overlay for 2D presentations (On-Screen-Display). (`Graphic3d_ZLayerId_TopOSD`)
pub const GRAPHIC3D_ZLAYER_ID_TOP_OSD: i32 = -4;
/// underlay for 2D presentations (On-Screen-Display). (`Graphic3d_ZLayerId_BotOSD`)
pub const GRAPHIC3D_ZLAYER_ID_BOT_OSD: i32 = -5;

/// no flags. (`Graphic3d_ArrayFlags_None`)
pub const GRAPHIC3D_ARRAY_FLAGS_NONE: i32 = 0;
/// per-vertex normal attribute. (`Graphic3d_ArrayFlags_VertexNormal`)
pub const GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL: i32 = 1;
/// per-vertex color  attribute. (`Graphic3d_ArrayFlags_VertexColor`)
pub const GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR: i32 = 2;
/// per-vertex texel coordinates (UV) attribute. (`Graphic3d_ArrayFlags_VertexTexel`)
pub const GRAPHIC3D_ARRAY_FLAGS_VERTEX_TEXEL: i32 = 4;
/// `Graphic3d_ArrayFlags_BoundColor`. (`Graphic3d_ArrayFlags_BoundColor`)
pub const GRAPHIC3D_ARRAY_FLAGS_BOUND_COLOR: i32 = 16;
/// mutable array, which can be invalidated during lifetime without re-creation. (`Graphic3d_ArrayFlags_AttribsMutable`)
pub const GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_MUTABLE: i32 = 32;
/// non-interleaved vertex attributes packed into single array. (`Graphic3d_ArrayFlags_AttribsDeinterleaved`)
pub const GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_DEINTERLEAVED: i32 = 64;
/// mutable index array, which can be invalidated during lifetime without re-creation. (`Graphic3d_ArrayFlags_IndexesMutable`)
pub const GRAPHIC3D_ARRAY_FLAGS_INDEXES_MUTABLE: i32 = 128;

/// Standard GLSL program combination bits.
///
/// Native enumeration `Graphic3d_ShaderFlags` (`Graphic3d_ShaderFlags.hxx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Graphic3dShaderFlags(pub i32);
impl Graphic3dShaderFlags {
    /// per-vertex color.
    pub const VERT_COLOR: Self = Self(1);
    /// handle RGB texturing.
    pub const TEXTURE_RGB: Self = Self(2);
    /// handle environment map (obsolete, to be removed).
    pub const TEXTURE_ENV: Self = Self(4);
    /// extended texture set (with normal map).
    pub const TEXTURE_NORMAL: Self = Self(6);
    /// point marker without sprite.
    pub const POINT_SIMPLE: Self = Self(8);
    /// point sprite with RGB image.
    pub const POINT_SPRITE: Self = Self(16);
    /// point sprite with Alpha image.
    pub const POINT_SPRITE_A: Self = Self(24);
    /// stipple line.
    pub const STIPPLE_LINE: Self = Self(32);
    /// handle 1 clipping plane.
    pub const CLIP_PLANES1: Self = Self(64);
    /// handle 2 clipping planes.
    pub const CLIP_PLANES2: Self = Self(128);
    /// handle N clipping planes.
    pub const CLIP_PLANES_N: Self = Self(192);
    /// handle chains of clipping planes.
    pub const CLIP_CHAINS: Self = Self(256);
    /// draw mesh edges (wireframe).
    pub const MESH_EDGES: Self = Self(512);
    /// discard fragment by alpha test (defined by cutoff value).
    pub const ALPHA_TEST: Self = Self(1024);
    /// write coverage buffer for Blended Order-Independent Transparency.
    pub const WRITE_OIT: Self = Self(2048);
    /// handle Depth Peeling OIT.
    pub const OIT_DEPTH_PEELING: Self = Self(4096);
    /// apply per-vertex color only to front-facing fragments.
    pub const VERT_COLOR_FRONT_ONLY: Self = Self(8192);
    /// overall number of combinations.
    pub const NB: Self = Self(16384);
    /// `Graphic3d_ShaderFlags_IsPoint`.
    pub const IS_POINT: Self = Self(24);
    /// `Graphic3d_ShaderFlags_HasTextures`.
    pub const HAS_TEXTURES: Self = Self(6);
    /// `Graphic3d_ShaderFlags_NeedsGeomShader`.
    pub const NEEDS_GEOM_SHADER: Self = Self(512);
}

/// Type of the shader object.
///
/// Native enumeration `Graphic3d_TypeOfShaderObject` (`Graphic3d_TypeOfShaderObject.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfShaderObject {
    /// vertex shader object, mandatory.
    Vertex = 1,
    /// tessellation control shader object, optional.
    TessControl = 2,
    /// tessellation evaluation shader object, optional.
    TessEvaluation = 4,
    /// geometry shader object, optional.
    Geometry = 8,
    /// fragment shader object, mandatory.
    Fragment = 16,
    /// compute shader object, should be used as alternative to shader object types for rendering.
    Compute = 32,
}

/// A Z-layer identifier (`Graphic3d_ZLayerId`, an `int`).
pub type Graphic3dZLayerId = i32;
/// Primitive-array flags (`Graphic3d_ArrayFlags`, an `int` of `GRAPHIC3D_ARRAY_FLAGS_*` bits).
pub type Graphic3dArrayFlags = i32;
