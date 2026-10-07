//! `Graphic3d_RenderingParams::DumpJson`, in the native field order (no class name).
use super::{
    Graphic3dAspectsSource, Graphic3dRayTracingParams, Graphic3dRenderingParams,
    Graphic3dTransformPersSource,
};
use crate::json_dump::{field_dumped, field_integer, field_real, field_unsigned};
use rustycad_tkernel::standard::dump::{DumpStream, dump_vector};
use std::io;

/// A numerical field of the dump: an integer (enumerations and booleans) or a real.
enum Number {
    Int(i32),
    Real(f64),
}

fn write_numbers(stream: &mut DumpStream, fields: &[(&str, Number)]) -> io::Result<()> {
    for (name, value) in fields {
        match value {
            Number::Int(v) => field_integer(stream, name, *v)?,
            Number::Real(v) => field_real(stream, name, *v)?,
        }
    }
    Ok(())
}

fn real(value: f32) -> Number {
    Number::Real(f64::from(value))
}
fn flag(value: bool) -> Number {
    Number::Int(i32::from(value))
}

impl Graphic3dRenderingParams {
    /// `DumpJson`; `params.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        write_numbers(stream, &self.general_numbers())?;
        write_numbers(stream, &ray_tracing_numbers(&self.ray_tracing))?;
        self.dump_stereo(stream, depth)?;
        self.dump_stats(stream, depth)?;
        field_unsigned(stream, "Resolution", u64::from(self.resolution))
    }
    fn general_numbers(&self) -> Vec<(&'static str, Number)> {
        use Number::Int;
        vec![
            ("Method", Int(self.method as i32)),
            ("ShadingModel", Int(self.shading_model as i32)),
            ("TransparencyMethod", Int(self.transparency_method as i32)),
            ("LineFeather", real(self.line_feather)),
            ("PbrEnvPow2Size", Int(self.pbr_env_pow2_size)),
            (
                "PbrEnvSpecMapNbLevels",
                Int(self.pbr_env_spec_map_nb_levels),
            ),
            (
                "PbrEnvBakingDiffNbSamples",
                Int(self.pbr_env_baking_diff_nb_samples),
            ),
            (
                "PbrEnvBakingSpecNbSamples",
                Int(self.pbr_env_baking_spec_nb_samples),
            ),
            (
                "PbrEnvBakingProbability",
                real(self.pbr_env_baking_probability),
            ),
            ("OitDepthFactor", real(self.oit_depth_factor)),
            ("NbMsaaSamples", Int(self.nb_msaa_samples)),
            ("RenderResolutionScale", real(self.render_resolution_scale)),
            ("ToEnableDepthPrepass", flag(self.to_enable_depth_prepass)),
            (
                "ToEnableAlphaToCoverage",
                flag(self.to_enable_alpha_to_coverage),
            ),
        ]
    }
    fn dump_stereo(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let s = &self.stereo;
        write_numbers(
            stream,
            &[
                ("StereoMode", Number::Int(s.stereo_mode as i32)),
                ("AnaglyphFilter", Number::Int(s.anaglyph_filter as i32)),
            ],
        )?;
        let (left, right) = (s.anaglyph_left, s.anaglyph_right);
        field_dumped(stream, depth, "&AnaglyphLeft", &|o, _| left.dump_json(o))?;
        field_dumped(stream, depth, "&AnaglyphRight", &|o, _| right.dump_json(o))?;
        field_integer(stream, "ToReverseStereo", i32::from(s.to_reverse_stereo))
    }
    fn dump_stats(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let s = &self.stats;
        for (name, position) in [
            ("StatsPosition.get()", &s.stats_position),
            ("ChartPosition.get()", &s.chart_position),
        ] {
            if let Some(position) = position.get() {
                field_dumped(stream, depth, name, &|o, d| position.dump_json(o, d))?;
            }
        }
        let size = s.chart_size;
        field_dumped(stream, depth, "&ChartSize", &|o, _| {
            dump_vector(o, "Vec2", &[f64::from(size.x), f64::from(size.y)])
        })?;
        if let Some(aspect) = s.stats_text_aspect.get() {
            field_dumped(stream, depth, "StatsTextAspect.get()", &|o, d| {
                aspect.dump_json(o, d)
            })?;
        }
        write_numbers(
            stream,
            &[
                ("StatsUpdateInterval", real(s.stats_update_interval)),
                ("StatsTextHeight", Number::Int(s.stats_text_height)),
                ("StatsNbFrames", Number::Int(s.stats_nb_frames)),
                ("StatsMaxChartTime", real(s.stats_max_chart_time)),
                ("CollectedStats", Number::Int(s.collected_stats.0)),
                ("ToShowStats", flag(s.to_show_stats)),
            ],
        )
    }
}

fn ray_tracing_numbers(r: &Graphic3dRayTracingParams) -> Vec<(&'static str, Number)> {
    use Number::Int;
    vec![
        (
            "IsGlobalIlluminationEnabled",
            flag(r.is_global_illumination_enabled),
        ),
        ("SamplesPerPixel", Int(r.samples_per_pixel)),
        ("RaytracingDepth", Int(r.raytracing_depth)),
        ("IsShadowEnabled", flag(r.is_shadow_enabled)),
        ("IsReflectionEnabled", flag(r.is_reflection_enabled)),
        ("IsAntialiasingEnabled", flag(r.is_antialiasing_enabled)),
        (
            "IsTransparentShadowEnabled",
            flag(r.is_transparent_shadow_enabled),
        ),
        (
            "UseEnvironmentMapBackground",
            flag(r.use_environment_map_background),
        ),
        (
            "ToIgnoreNormalMapInRayTracing",
            flag(r.to_ignore_normal_map_in_ray_tracing),
        ),
        (
            "CoherentPathTracingMode",
            flag(r.coherent_path_tracing_mode),
        ),
        ("AdaptiveScreenSampling", flag(r.adaptive_screen_sampling)),
        (
            "AdaptiveScreenSamplingAtomic",
            flag(r.adaptive_screen_sampling_atomic),
        ),
        ("ShowSamplingTiles", flag(r.show_sampling_tiles)),
        ("TwoSidedBsdfModels", flag(r.two_sided_bsdf_models)),
        ("RadianceClampingValue", real(r.radiance_clamping_value)),
        (
            "RebuildRayTracingShaders",
            flag(r.rebuild_ray_tracing_shaders),
        ),
        ("RayTracingTileSize", Int(r.ray_tracing_tile_size)),
        ("NbRayTracingTiles", Int(r.nb_ray_tracing_tiles)),
        ("CameraApertureRadius", real(r.camera_aperture_radius)),
        ("CameraFocalPlaneDist", real(r.camera_focal_plane_dist)),
        ("FrustumCullingState", Int(r.frustum_culling_state as i32)),
        ("ToneMappingMethod", Int(r.tone_mapping_method as i32)),
        ("Exposure", real(r.exposure)),
        ("WhitePoint", real(r.white_point)),
    ]
}
