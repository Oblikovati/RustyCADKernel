//! Diagnostic, frame-statistics and limit enumerations of Graphic3d.

/// Diagnostic info categories bit flags.
///
/// Native enumeration `Graphic3d_DiagnosticInfo` (`Graphic3d_DiagnosticInfo.hxx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Graphic3dDiagnosticInfo(pub i32);
impl Graphic3dDiagnosticInfo {
    /// device / vendor / version information.
    pub const DEVICE: Self = Self(1);
    /// framebuffer information.
    pub const FRAME_BUFFER: Self = Self(2);
    /// hardware limits.
    pub const LIMITS: Self = Self(4);
    /// memory counters.
    pub const MEMORY: Self = Self(8);
    /// native platform API information (e.g. WGL / GLX / EGL).
    pub const NATIVE_PLATFORM: Self = Self(16);
    /// vendor extension list (usually very long).
    pub const EXTENSIONS: Self = Self(32);
    /// minimal information.
    pub const SHORT: Self = Self(7);
    /// basic information, without extension list.
    pub const BASIC: Self = Self(31);
    /// complete information, including extension list.
    pub const COMPLETE: Self = Self(63);
}

/// Stats counter.
///
/// Native enumeration `Graphic3d_FrameStatsCounter` (`Graphic3d_FrameStatsCounter.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dFrameStatsCounter {
    /// number of ZLayers.
    NbLayers = 0,
    /// number of defined OpenGl_Structure.
    NbStructs = 1,
    /// estimated GPU memory used for geometry.
    EstimatedBytesGeom = 2,
    /// estimated GPU memory used for FBOs.
    EstimatedBytesFbos = 3,
    /// estimated GPU memory used for textures.
    EstimatedBytesTextures = 4,
    /// number of not culled ZLayers.
    NbLayersNotCulled = 5,
    /// number of not culled OpenGl_Structure.
    NbStructsNotCulled = 6,
    /// number of not culled OpenGl_Group.
    NbGroupsNotCulled = 7,
    /// number of not culled OpenGl_Element.
    NbElemsNotCulled = 8,
    /// number of not culled OpenGl_PrimitiveArray drawing triangles.
    NbElemsFillNotCulled = 9,
    /// number of not culled OpenGl_PrimitiveArray drawing lines.
    NbElemsLineNotCulled = 10,
    /// number of not culled OpenGl_PrimitiveArray drawing points.
    NbElemsPointNotCulled = 11,
    /// number of not culled OpenGl_Text.
    NbElemsTextNotCulled = 12,
    /// number of not culled (as structure) triangles.
    NbTrianglesNotCulled = 13,
    /// number of not culled (as structure) line segments.
    NbLinesNotCulled = 14,
    /// number of not culled (as structure) points.
    NbPointsNotCulled = 15,
    /// number of ZLayers in immediate layer.
    NbLayersImmediate = 16,
    /// number of OpenGl_Structure in immediate layer.
    NbStructsImmediate = 17,
    /// number of OpenGl_Group in immediate layer.
    NbGroupsImmediate = 18,
    /// number of OpenGl_Element in immediate layer.
    NbElemsImmediate = 19,
    /// number of OpenGl_PrimitiveArray drawing triangles in immediate layer.
    NbElemsFillImmediate = 20,
    /// number of OpenGl_PrimitiveArray drawing lines in immediate layer.
    NbElemsLineImmediate = 21,
    /// number of OpenGl_PrimitiveArray drawing points in immediate layer.
    NbElemsPointImmediate = 22,
    /// number of OpenGl_Text in immediate layer.
    NbElemsTextImmediate = 23,
    /// number of triangles in immediate layer.
    NbTrianglesImmediate = 24,
    /// number of line segments in immediate layer.
    NbLinesImmediate = 25,
    /// number of points in immediate layer.
    NbPointsImmediate = 26,
}
/// `Graphic3d_FrameStatsCounter_NB`. (`Graphic3d_FrameStatsCounter_NB`)
pub const GRAPHIC3D_FRAME_STATS_COUNTER_NB: i32 = 27;
/// `Graphic3d_FrameStatsCounter_SCENE_LOWER`. (`Graphic3d_FrameStatsCounter_SCENE_LOWER`)
pub const GRAPHIC3D_FRAME_STATS_COUNTER_SCENE_LOWER: i32 = 0;
/// `Graphic3d_FrameStatsCounter_SCENE_UPPER`. (`Graphic3d_FrameStatsCounter_SCENE_UPPER`)
pub const GRAPHIC3D_FRAME_STATS_COUNTER_SCENE_UPPER: i32 = 4;
/// `Graphic3d_FrameStatsCounter_RENDERED_LOWER`. (`Graphic3d_FrameStatsCounter_RENDERED_LOWER`)
pub const GRAPHIC3D_FRAME_STATS_COUNTER_RENDERED_LOWER: i32 = 5;
/// `Graphic3d_FrameStatsCounter_RENDERED_UPPER`. (`Graphic3d_FrameStatsCounter_RENDERED_UPPER`)
pub const GRAPHIC3D_FRAME_STATS_COUNTER_RENDERED_UPPER: i32 = 15;
/// `Graphic3d_FrameStatsCounter_IMMEDIATE_LOWER`. (`Graphic3d_FrameStatsCounter_IMMEDIATE_LOWER`)
pub const GRAPHIC3D_FRAME_STATS_COUNTER_IMMEDIATE_LOWER: i32 = 16;
/// `Graphic3d_FrameStatsCounter_IMMEDIATE_UPPER`. (`Graphic3d_FrameStatsCounter_IMMEDIATE_UPPER`)
pub const GRAPHIC3D_FRAME_STATS_COUNTER_IMMEDIATE_UPPER: i32 = 26;

/// Timers for collecting frame performance statistics.
///
/// Native enumeration `Graphic3d_FrameStatsTimer` (`Graphic3d_FrameStatsTimer.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dFrameStatsTimer {
    /// `Graphic3d_FrameStatsTimer_ElapsedFrame`.
    ElapsedFrame = 0,
    /// `Graphic3d_FrameStatsTimer_CpuFrame`.
    CpuFrame = 1,
    /// `Graphic3d_FrameStatsTimer_CpuCulling`.
    CpuCulling = 2,
    /// `Graphic3d_FrameStatsTimer_CpuPicking`.
    CpuPicking = 3,
    /// `Graphic3d_FrameStatsTimer_CpuDynamics`.
    CpuDynamics = 4,
}
/// `Graphic3d_FrameStatsTimer_NB`. (`Graphic3d_FrameStatsTimer_NB`)
pub const GRAPHIC3D_FRAME_STATS_TIMER_NB: i32 = 5;

/// Type of graphic resource limit.
///
/// Native enumeration `Graphic3d_TypeOfLimit` (`Graphic3d_TypeOfLimit.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Graphic3dTypeOfLimit {
    /// maximum number of active light sources.
    MaxNbLights = 0,
    /// maximum number of active clipping planes.
    MaxNbClipPlanes = 1,
    /// maximum number of views.
    MaxNbViews = 2,
    /// maximum size of texture.
    MaxTextureSize = 3,
    /// maximum width  for image dump.
    MaxViewDumpSizeX = 4,
    /// maximum height for image dump.
    MaxViewDumpSizeY = 5,
    /// maximum number of combined texture units for multitexturing.
    MaxCombinedTextureUnits = 6,
    /// maximum number of MSAA samples.
    MaxMsaa = 7,
    /// indicates whether PBR metallic-roughness shading model is supported.
    HasPBR = 8,
    /// indicates whether ray tracing is supported.
    HasRayTracing = 9,
    /// indicates whether ray tracing textures are supported.
    HasRayTracingTextures = 10,
    /// indicates whether adaptive screen sampling is supported.
    HasRayTracingAdaptiveSampling = 11,
    /// indicates whether optimized adaptive screen sampling is supported (hardware supports atomic float operations).
    HasRayTracingAdaptiveSamplingAtomic = 12,
    /// indicates whether sRGB rendering is supported.
    HasSRGB = 13,
    /// indicates whether necessary GL extensions for Weighted, Blended OIT available (without MSAA).
    HasBlendedOit = 14,
    /// indicates whether necessary GL extensions for Weighted, Blended OIT available (with MSAA).
    HasBlendedOitMsaa = 15,
    /// indicates whether Flat shading (Graphic3d_TypeOfShadingModel_PhongFacet) is supported.
    HasFlatShading = 16,
    /// indicates whether advanced mesh edges presentation is supported.
    HasMeshEdges = 17,
    /// indicates whether workaround for Intel driver problem with empty FBO for images with big width is applied.
    IsWorkaroundFBO = 18,
    /// number of elements in this enumeration.
    Nb = 19,
}
