//! `Graphic3d_RenderingParams`: the rendering parameters of a view (rasterization and
//! ray-tracing options, stereo output, on-screen statistics), a value class with public
//! fields like the native one.
use super::{
    Graphic3dAspectText3d, Graphic3dAspectsEdgeMembers, Graphic3dAspectsMembers,
    Graphic3dAspectsTextMembers, Graphic3dRenderTransparentMethod, Graphic3dRenderingMode,
    Graphic3dStereoMode, Graphic3dToneMappingMethod, Graphic3dTransModeFlags,
    Graphic3dTransformPers, Graphic3dTypeOfShadingModel,
};
use crate::aspect::{AspectTypeOfDisplayText, AspectTypeOfTriedronPosition};
use crate::font::{FontFontAspect, FontHinting};
use rustycad_tkernel::ncollection::{Mat4, Vec2, Vec4};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

/// `Graphic3d_RenderingParams::Anaglyph`: the anaglyph filters.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Graphic3dRenderingParamsAnaglyph {
    /// `Anaglyph_RedCyan_Simple`.
    RedCyanSimple = 0,
    /// `Anaglyph_RedCyan_Optimized`.
    RedCyanOptimized = 1,
    /// `Anaglyph_YellowBlue_Simple`.
    YellowBlueSimple = 2,
    /// `Anaglyph_YellowBlue_Optimized`.
    YellowBlueOptimized = 3,
    /// `Anaglyph_GreenMagenta_Simple`.
    GreenMagentaSimple = 4,
    /// `Anaglyph_UserDefined`: the `anaglyph_left`/`anaglyph_right` matrices.
    UserDefined = 5,
}

/// `Graphic3d_RenderingParams::PerfCounters`: the performance counter flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Graphic3dRenderingParamsPerfCounters(pub i32);
impl Graphic3dRenderingParamsPerfCounters {
    /// `PerfCounters_NONE`.
    pub const NONE: Self = Self(0x000);
    /// `PerfCounters_FrameRate`.
    pub const FRAME_RATE: Self = Self(0x001);
    /// `PerfCounters_CPU`.
    pub const CPU: Self = Self(0x002);
    /// `PerfCounters_Layers`.
    pub const LAYERS: Self = Self(0x004);
    /// `PerfCounters_Structures`.
    pub const STRUCTURES: Self = Self(0x008);
    /// `PerfCounters_Groups`.
    pub const GROUPS: Self = Self(0x010);
    /// `PerfCounters_GroupArrays`.
    pub const GROUP_ARRAYS: Self = Self(0x020);
    /// `PerfCounters_Triangles`.
    pub const TRIANGLES: Self = Self(0x040);
    /// `PerfCounters_Points`.
    pub const POINTS: Self = Self(0x080);
    /// `PerfCounters_Lines`.
    pub const LINES: Self = Self(0x100);
    /// `PerfCounters_EstimMem`.
    pub const ESTIM_MEM: Self = Self(0x200);
    /// `PerfCounters_FrameTime`.
    pub const FRAME_TIME: Self = Self(0x400);
    /// `PerfCounters_FrameTimeMax`.
    pub const FRAME_TIME_MAX: Self = Self(0x800);
    /// `PerfCounters_SkipImmediate`.
    pub const SKIP_IMMEDIATE: Self = Self(0x1000);
    /// `PerfCounters_Basic`.
    pub const BASIC: Self = Self(0x00F);
    /// `PerfCounters_Extended`.
    pub const EXTENDED: Self = Self(0x3FF);
    /// `PerfCounters_All`.
    pub const ALL: Self = Self(0xFFF);
}

/// `Graphic3d_RenderingParams::FrustumCulling`: the frustum culling states.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Graphic3dRenderingParamsFrustumCulling {
    /// `FrustumCulling_Off`.
    Off = 0,
    /// `FrustumCulling_On`.
    On = 1,
    /// `FrustumCulling_NoUpdate`: culling active, the culled list kept.
    NoUpdate = 2,
}

/// The rendering parameters; `Graphic3dRenderingParams::new()?.resolution == 72`.
#[derive(Clone)]
pub struct Graphic3dRenderingParams {
    /// `Method`.
    pub method: Graphic3dRenderingMode,
    /// `ShadingModel`.
    pub shading_model: Graphic3dTypeOfShadingModel,
    /// `TransparencyMethod`.
    pub transparency_method: Graphic3dRenderTransparentMethod,
    /// `Resolution` (PPI).
    pub resolution: u32,
    /// `FontHinting`.
    pub font_hinting: FontHinting,
    /// `LineFeather`.
    pub line_feather: f32,
    /// `PbrEnvPow2Size`.
    pub pbr_env_pow2_size: i32,
    /// `PbrEnvSpecMapNbLevels`.
    pub pbr_env_spec_map_nb_levels: i32,
    /// `PbrEnvBakingDiffNbSamples`.
    pub pbr_env_baking_diff_nb_samples: i32,
    /// `PbrEnvBakingSpecNbSamples`.
    pub pbr_env_baking_spec_nb_samples: i32,
    /// `PbrEnvBakingProbability`.
    pub pbr_env_baking_probability: f32,
    /// `OitDepthFactor`.
    pub oit_depth_factor: f32,
    /// `NbOitDepthPeelingLayers`.
    pub nb_oit_depth_peeling_layers: i32,
    /// `NbMsaaSamples`.
    pub nb_msaa_samples: i32,
    /// `RenderResolutionScale`.
    pub render_resolution_scale: f32,
    /// `ShadowMapResolution`.
    pub shadow_map_resolution: i32,
    /// `ShadowMapBias`.
    pub shadow_map_bias: f32,
    /// `ToEnableDepthPrepass`.
    pub to_enable_depth_prepass: bool,
    /// `ToEnableAlphaToCoverage`.
    pub to_enable_alpha_to_coverage: bool,
    /// The ray-tracing and path-tracing parameters.
    pub ray_tracing: Graphic3dRayTracingParams,
    /// The stereo and VR parameters.
    pub stereo: Graphic3dStereoParams,
    /// The on-screen statistics parameters.
    pub stats: Graphic3dStatsParams,
}

/// The ray-tracing and path-tracing fields of `Graphic3d_RenderingParams`.
#[derive(Clone, Debug, PartialEq)]
pub struct Graphic3dRayTracingParams {
    /// `IsGlobalIlluminationEnabled`.
    pub is_global_illumination_enabled: bool,
    /// `SamplesPerPixel`.
    pub samples_per_pixel: i32,
    /// `RaytracingDepth`.
    pub raytracing_depth: i32,
    /// `IsShadowEnabled`.
    pub is_shadow_enabled: bool,
    /// `IsReflectionEnabled`.
    pub is_reflection_enabled: bool,
    /// `IsAntialiasingEnabled`.
    pub is_antialiasing_enabled: bool,
    /// `IsTransparentShadowEnabled`.
    pub is_transparent_shadow_enabled: bool,
    /// `UseEnvironmentMapBackground`.
    pub use_environment_map_background: bool,
    /// `ToIgnoreNormalMapInRayTracing`.
    pub to_ignore_normal_map_in_ray_tracing: bool,
    /// `CoherentPathTracingMode`.
    pub coherent_path_tracing_mode: bool,
    /// `AdaptiveScreenSampling`.
    pub adaptive_screen_sampling: bool,
    /// `AdaptiveScreenSamplingAtomic`.
    pub adaptive_screen_sampling_atomic: bool,
    /// `ShowSamplingTiles`.
    pub show_sampling_tiles: bool,
    /// `TwoSidedBsdfModels`.
    pub two_sided_bsdf_models: bool,
    /// `RadianceClampingValue`.
    pub radiance_clamping_value: f32,
    /// `RebuildRayTracingShaders`.
    pub rebuild_ray_tracing_shaders: bool,
    /// `RayTracingTileSize`.
    pub ray_tracing_tile_size: i32,
    /// `NbRayTracingTiles`.
    pub nb_ray_tracing_tiles: i32,
    /// `CameraApertureRadius`.
    pub camera_aperture_radius: f32,
    /// `CameraFocalPlaneDist`.
    pub camera_focal_plane_dist: f32,
    /// `FrustumCullingState`.
    pub frustum_culling_state: Graphic3dRenderingParamsFrustumCulling,
    /// `ToneMappingMethod`.
    pub tone_mapping_method: Graphic3dToneMappingMethod,
    /// `Exposure`.
    pub exposure: f32,
    /// `WhitePoint`.
    pub white_point: f32,
}

impl Default for Graphic3dRayTracingParams {
    fn default() -> Self {
        Self {
            is_global_illumination_enabled: false,
            samples_per_pixel: 0,
            raytracing_depth: Graphic3dRenderingParams::THE_DEFAULT_DEPTH,
            is_shadow_enabled: true,
            is_reflection_enabled: false,
            is_antialiasing_enabled: false,
            is_transparent_shadow_enabled: false,
            use_environment_map_background: false,
            to_ignore_normal_map_in_ray_tracing: false,
            coherent_path_tracing_mode: false,
            adaptive_screen_sampling: false,
            adaptive_screen_sampling_atomic: false,
            show_sampling_tiles: false,
            two_sided_bsdf_models: false,
            radiance_clamping_value: 30.0,
            rebuild_ray_tracing_shaders: false,
            ray_tracing_tile_size: 32,
            nb_ray_tracing_tiles: 16 * 16,
            camera_aperture_radius: 0.0,
            camera_focal_plane_dist: 1.0,
            frustum_culling_state: Graphic3dRenderingParamsFrustumCulling::On,
            tone_mapping_method: Graphic3dToneMappingMethod::Disabled,
            exposure: 0.0,
            white_point: 1.0,
        }
    }
}

/// The stereo and VR fields of `Graphic3d_RenderingParams`.
#[derive(Clone, Debug, PartialEq)]
pub struct Graphic3dStereoParams {
    /// `StereoMode`.
    pub stereo_mode: Graphic3dStereoMode,
    /// `HmdFov2d`.
    pub hmd_fov_2d: f32,
    /// `AnaglyphFilter`.
    pub anaglyph_filter: Graphic3dRenderingParamsAnaglyph,
    /// `AnaglyphLeft`.
    pub anaglyph_left: Mat4<f32>,
    /// `AnaglyphRight`.
    pub anaglyph_right: Mat4<f32>,
    /// `ToReverseStereo`.
    pub to_reverse_stereo: bool,
    /// `ToSmoothInterlacing`.
    pub to_smooth_interlacing: bool,
    /// `ToMirrorComposer`.
    pub to_mirror_composer: bool,
}

/// The left and right default anaglyph matrices (red; green and blue).
fn anaglyph_matrices() -> (Mat4<f32>, Mat4<f32>) {
    let zero = Vec4::new(0.0, 0.0, 0.0, 0.0);
    let (mut left, mut right) = (Mat4::identity(), Mat4::identity());
    let left_rows = [Vec4::new(1.0, 0.0, 0.0, 0.0), zero, zero, zero];
    let right_rows = [
        zero,
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
        zero,
    ];
    for row in 0..4 {
        left.set_row(row, left_rows[row]);
        right.set_row(row, right_rows[row]);
    }
    (left, right)
}

impl Default for Graphic3dStereoParams {
    fn default() -> Self {
        let (anaglyph_left, anaglyph_right) = anaglyph_matrices();
        Self {
            stereo_mode: Graphic3dStereoMode::QuadBuffer,
            hmd_fov_2d: 30.0,
            anaglyph_filter: Graphic3dRenderingParamsAnaglyph::RedCyanOptimized,
            anaglyph_left,
            anaglyph_right,
            to_reverse_stereo: false,
            to_smooth_interlacing: true,
            to_mirror_composer: true,
        }
    }
}

/// The on-screen statistics fields of `Graphic3d_RenderingParams`.
#[derive(Clone)]
pub struct Graphic3dStatsParams {
    /// `StatsPosition`.
    pub stats_position: Handle<Graphic3dTransformPers>,
    /// `ChartPosition`.
    pub chart_position: Handle<Graphic3dTransformPers>,
    /// `ChartSize`.
    pub chart_size: Vec2<i32>,
    /// `StatsTextAspect`.
    pub stats_text_aspect: Handle<Graphic3dAspectText3d>,
    /// `StatsUpdateInterval`.
    pub stats_update_interval: f32,
    /// `StatsTextHeight`.
    pub stats_text_height: i32,
    /// `StatsNbFrames`.
    pub stats_nb_frames: i32,
    /// `StatsMaxChartTime`.
    pub stats_max_chart_time: f32,
    /// `CollectedStats`.
    pub collected_stats: Graphic3dRenderingParamsPerfCounters,
    /// `ToShowStats`.
    pub to_show_stats: bool,
}

/// The white mono-spaced shadowed statistics text aspect.
fn stats_text_aspect() -> Graphic3dAspectText3d {
    let aspect = Graphic3dAspectText3d::default();
    aspect.set_interior_color(Color::named(NamedColor::WHITE));
    aspect.set_edge_color(Color::named(NamedColor::BLACK));
    aspect.set_font(&AsciiString::from("Courier"));
    aspect.set_text_display_type(AspectTypeOfDisplayText::Shadow);
    aspect.set_text_zoomable(false);
    aspect.set_text_font_aspect(FontFontAspect::Regular);
    aspect
}

impl Graphic3dStatsParams {
    fn new() -> Result<Self, Failure> {
        let corner = |position| {
            Graphic3dTransformPers::with_corner(
                Graphic3dTransModeFlags::TMF_2D,
                position,
                Vec2::new(20, 20),
            )
            .map(Handle::new)
        };
        Ok(Self {
            stats_position: corner(AspectTypeOfTriedronPosition::LEFT_UPPER)?,
            chart_position: corner(AspectTypeOfTriedronPosition::RIGHT_UPPER)?,
            chart_size: Vec2::new(-1, -1),
            stats_text_aspect: Handle::new(stats_text_aspect()),
            stats_update_interval: 1.0,
            stats_text_height: 16,
            stats_nb_frames: 1,
            stats_max_chart_time: 0.1,
            collected_stats: Graphic3dRenderingParamsPerfCounters::BASIC,
            to_show_stats: false,
        })
    }
}

impl Graphic3dRenderingParams {
    /// `THE_DEFAULT_RESOLUTION`: the default pixel density (72 PPI).
    pub const THE_DEFAULT_RESOLUTION: u32 = 72;
    /// `THE_DEFAULT_DEPTH`: the default ray-tracing depth.
    pub const THE_DEFAULT_DEPTH: i32 = 3;

    /// The native defaults (the 2D statistics positions cannot raise; the `Result` carries the
    /// transform persistence constructor). Example: `Graphic3dRenderingParams::new()?`.
    pub fn new() -> Result<Self, Failure> {
        Ok(Self {
            method: Graphic3dRenderingMode::Rasterization,
            shading_model: Graphic3dTypeOfShadingModel::Phong,
            transparency_method: Graphic3dRenderTransparentMethod::BlendUnordered,
            resolution: Self::THE_DEFAULT_RESOLUTION,
            font_hinting: FontHinting::OFF,
            line_feather: 1.0,
            pbr_env_pow2_size: 9,
            pbr_env_spec_map_nb_levels: 6,
            pbr_env_baking_diff_nb_samples: 1024,
            pbr_env_baking_spec_nb_samples: 256,
            pbr_env_baking_probability: 0.99,
            oit_depth_factor: 0.0,
            nb_oit_depth_peeling_layers: 4,
            nb_msaa_samples: 0,
            render_resolution_scale: 1.0,
            shadow_map_resolution: 1024,
            shadow_map_bias: 0.005,
            to_enable_depth_prepass: false,
            to_enable_alpha_to_coverage: true,
            ray_tracing: Graphic3dRayTracingParams::default(),
            stereo: Graphic3dStereoParams::default(),
            stats: Graphic3dStatsParams::new()?,
        })
    }
    /// `ResolutionRatio`: the resolution over 72 PPI, in float; `params.resolution_ratio()`.
    pub fn resolution_ratio(&self) -> f32 {
        self.resolution as f32 / Self::THE_DEFAULT_RESOLUTION as f32
    }
}
