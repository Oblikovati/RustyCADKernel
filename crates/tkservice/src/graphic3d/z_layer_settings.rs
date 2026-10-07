//! `Graphic3d_ZLayerSettings`: the rendering settings of one Z layer (origin, culling, depth
//! and polygon offset options, lights).
use super::{Graphic3dLightSet, Graphic3dPolygonOffset};
use crate::aspect::AspectPolygonOffsetMode;
use crate::json_dump::{class_begin, field_dumped, field_integer, field_real, field_string};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::{Transform3, Vector3, Xyz, resolution};
use rustycad_tkmath::toploc::Datum3;
use std::io;

/// The settings of a Z layer; `Graphic3dZLayerSettings::default().is_raytracable()`.
#[derive(Clone)]
pub struct Graphic3dZLayerSettings {
    name: AsciiString,
    lights: Handle<Graphic3dLightSet>,
    origin_trsf: Handle<Datum3>,
    origin: Xyz,
    culling_distance: f64,
    culling_size: f64,
    polygon_offset: Graphic3dPolygonOffset,
    is_immediate: bool,
    to_raytrace: bool,
    use_environment_texture: bool,
    to_enable_depth_test: bool,
    to_enable_depth_write: bool,
    to_clear_depth: bool,
    to_render_in_depth_prepass: bool,
}

impl Default for Graphic3dZLayerSettings {
    /// Infinite culling, not immediate, ray-traced, depth test/write/clear/pre-pass on.
    fn default() -> Self {
        Self {
            name: AsciiString::default(),
            lights: Handle::default(),
            origin_trsf: Handle::default(),
            origin: Xyz::new(0.0, 0.0, 0.0),
            culling_distance: precision::infinite(),
            culling_size: precision::infinite(),
            polygon_offset: Graphic3dPolygonOffset::default(),
            is_immediate: false,
            to_raytrace: true,
            use_environment_texture: true,
            to_enable_depth_test: true,
            to_enable_depth_write: true,
            to_clear_depth: true,
            to_render_in_depth_prepass: true,
        }
    }
}

/// `!Precision::IsInfinite(value) && value > 0`.
fn is_finite_positive(value: f64) -> bool {
    !precision::is_infinite(value) && value > 0.0
}

impl Graphic3dZLayerSettings {
    /// The user name; `settings.name()`.
    pub fn name(&self) -> &AsciiString {
        &self.name
    }
    /// Sets the user name; `settings.set_name(&"overlay".into())`.
    pub fn set_name(&mut self, name: &AsciiString) {
        self.name = name.clone();
    }
    /// The layer lights (null for the view lights); `settings.lights()`.
    pub fn lights(&self) -> &Handle<Graphic3dLightSet> {
        &self.lights
    }
    /// Sets the layer lights; `settings.set_lights(lights)`.
    pub fn set_lights(&mut self, lights: Handle<Graphic3dLightSet>) {
        self.lights = lights;
    }
    /// The origin of the layer objects; `settings.origin()`.
    pub fn origin(&self) -> Xyz {
        self.origin
    }
    /// The translation to the origin (null for the zero origin); `settings.origin_transformation()`.
    pub fn origin_transformation(&self) -> &Handle<Datum3> {
        &self.origin_trsf
    }
    /// Sets the origin and its translation (none within `gp::Resolution()` of zero).
    /// Example: `settings.set_origin(Xyz::new(1.0e6, 0.0, 0.0))`.
    pub fn set_origin(&mut self, origin: Xyz) {
        self.origin = origin;
        self.origin_trsf = Handle::default();
        if !origin.is_equal(&Xyz::new(0.0, 0.0, 0.0), resolution()) {
            let mut transform = Transform3::default();
            transform.set_translation(Vector3::from(origin));
            self.origin_trsf = Handle::new(Datum3::new(transform));
        }
    }
    /// Whether a finite positive culling distance is set; `settings.has_culling_distance()`.
    pub fn has_culling_distance(&self) -> bool {
        is_finite_positive(self.culling_distance)
    }
    /// The culling distance; `settings.culling_distance()`.
    pub fn culling_distance(&self) -> f64 {
        self.culling_distance
    }
    /// Sets the culling distance; `settings.set_culling_distance(100.0)`.
    pub fn set_culling_distance(&mut self, distance: f64) {
        self.culling_distance = distance;
    }
    /// Whether a finite positive culling size is set; `settings.has_culling_size()`.
    pub fn has_culling_size(&self) -> bool {
        is_finite_positive(self.culling_size)
    }
    /// The culling size; `settings.culling_size()`.
    pub fn culling_size(&self) -> f64 {
        self.culling_size
    }
    /// Sets the culling size; `settings.set_culling_size(2.0)`.
    pub fn set_culling_size(&mut self, size: f64) {
        self.culling_size = size;
    }
    /// The polygon offset; `settings.polygon_offset()`.
    pub fn polygon_offset(&self) -> Graphic3dPolygonOffset {
        self.polygon_offset
    }
    /// Sets the polygon offset; `settings.set_polygon_offset(offset)`.
    pub fn set_polygon_offset(&mut self, offset: Graphic3dPolygonOffset) {
        self.polygon_offset = offset;
    }
    /// The polygon offset for editing; `settings.change_polygon_offset().units = 2.0`.
    pub fn change_polygon_offset(&mut self) -> &mut Graphic3dPolygonOffset {
        &mut self.polygon_offset
    }
    /// Fill offset (1, 1); `settings.set_depth_offset_positive()`.
    pub fn set_depth_offset_positive(&mut self) {
        self.set_fill_offset(1.0);
    }
    /// Fill offset (1, -1); `settings.set_depth_offset_negative()`.
    pub fn set_depth_offset_negative(&mut self) {
        self.set_fill_offset(-1.0);
    }
    fn set_fill_offset(&mut self, units: f32) {
        self.polygon_offset = Graphic3dPolygonOffset {
            mode: AspectPolygonOffsetMode::FILL,
            factor: 1.0,
            units,
        };
    }
}

/// Generates the boolean option accessors of the settings.
macro_rules! layer_flags {
    ($($(#[$doc:meta])* $field:ident: $get:ident, $set:ident;)+) => {
        impl Graphic3dZLayerSettings {
            $(
                $(#[$doc])*
                pub fn $get(&self) -> bool {
                    self.$field
                }
                /// Sets the option above.
                pub fn $set(&mut self, value: bool) {
                    self.$field = value;
                }
            )+
        }
    };
}
layer_flags! {
    /// Whether the layer is drawn after the normal layers; `settings.is_immediate()`.
    is_immediate: is_immediate, set_immediate;
    /// Whether the ray-tracing engine renders the layer; `settings.is_raytracable()`.
    to_raytrace: is_raytracable, set_raytracable;
    /// Whether environment texture mapping is allowed; `settings.use_environment_texture()`.
    use_environment_texture: use_environment_texture, set_environment_texture;
    /// Whether the depth test is on; `settings.to_enable_depth_test()`.
    to_enable_depth_test: to_enable_depth_test, set_enable_depth_test;
    /// Whether depth values are written; `settings.to_enable_depth_write()`.
    to_enable_depth_write: to_enable_depth_write, set_enable_depth_write;
    /// Whether the depth is cleared before the layer; `settings.to_clear_depth()`.
    to_clear_depth: to_clear_depth, set_clear_depth;
    /// Whether the layer is in the depth pre-pass; `settings.to_render_in_depth_prepass()`.
    to_render_in_depth_prepass: to_render_in_depth_prepass, set_render_in_depth_prepass;
}

impl Graphic3dZLayerSettings {
    /// `DumpJson`; `settings.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_ZLayerSettings")?;
        field_string(stream, "myName", self.name.as_bytes())?;
        if let Some(datum) = self.origin_trsf.get() {
            field_dumped(stream, depth, "myOriginTrsf.get()", &|o, d| {
                datum.dump_json(o, d)
            })?;
        }
        let origin = self.origin;
        field_dumped(stream, depth, "&myOrigin", &|o, _| origin.dump_json(o))?;
        field_real(stream, "myCullingDistance", self.culling_distance)?;
        field_real(stream, "myCullingSize", self.culling_size)?;
        let offset = self.polygon_offset;
        field_dumped(stream, depth, "&myPolygonOffset", &|o, d| {
            offset.dump_json(o, d)
        })?;
        let flags = [
            ("myIsImmediate", self.is_immediate),
            ("myToRaytrace", self.to_raytrace),
            ("myUseEnvironmentTexture", self.use_environment_texture),
            ("myToEnableDepthTest", self.to_enable_depth_test),
            ("myToEnableDepthWrite", self.to_enable_depth_write),
            ("myToClearDepth", self.to_clear_depth),
            ("myToRenderInDepthPrepass", self.to_render_in_depth_prepass),
        ];
        for (name, flag) in flags {
            field_integer(stream, name, i32::from(flag))?;
        }
        Ok(())
    }
}
