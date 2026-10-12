//! Native UV mapping flags and stored coordinates: repeat remains a state flag, not sampler policy.
use super::textured_shape_source::AisTexturedShapeSource;
use rustycad_tkmath::gp::Point2;
/// Native UV property operations; `object.set_texture_repeat(false,2.,3.)` still stores the factors.
pub trait AisTexturedShapeMappingMembers: AisTexturedShapeSource {
    /// Read native repeat state; `object.texture_repeat()`.
    fn texture_repeat(&self) -> bool {
        let base = self.textured_shape_base();
        base.fields.read().repeat
    }
    /// Read native custom-origin state; `object.texture_origin()`.
    fn texture_origin(&self) -> bool {
        let base = self.textured_shape_base();
        base.fields.read().custom_origin
    }
    /// Read native scale state; `object.texture_scale()`.
    fn texture_scale(&self) -> bool {
        let base = self.textured_shape_base();
        base.fields.read().scale
    }
    /// Inspect the stored coordinate even when disabled; `object.u_repeat()`.
    fn u_repeat(&self) -> f64 {
        let base = self.textured_shape_base();
        base.shape.fields.read().uv_repeat.x()
    }
    /// Inspect the stored coordinate even when disabled; `object.v_repeat()`.
    fn v_repeat(&self) -> f64 {
        let base = self.textured_shape_base();
        base.shape.fields.read().uv_repeat.y()
    }
    /// Inspect the stored coordinate even when disabled; `object.texture_u_origin()`.
    fn texture_u_origin(&self) -> f64 {
        let base = self.textured_shape_base();
        base.shape.fields.read().uv_origin.x()
    }
    /// Inspect the stored coordinate even when disabled; `object.texture_v_origin()`.
    fn texture_v_origin(&self) -> f64 {
        let base = self.textured_shape_base();
        base.shape.fields.read().uv_origin.y()
    }
    /// Inspect the stored coordinate even when disabled; `object.texture_scale_u()`.
    fn texture_scale_u(&self) -> f64 {
        let base = self.textured_shape_base();
        base.shape.fields.read().uv_scale.x()
    }
    /// Inspect the stored coordinate even when disabled; `object.texture_scale_v()`.
    fn texture_scale_v(&self) -> f64 {
        let base = self.textured_shape_base();
        base.shape.fields.read().uv_scale.y()
    }
    /// Store repeat flag and factors without changing sampler parameters; `object.set_texture_repeat(false,2.,3.)`.
    fn set_texture_repeat(&self, enabled: bool, u: f64, v: f64) {
        let base = self.textured_shape_base();
        base.fields.write().repeat = enabled;
        base.shape.fields.write().uv_repeat = Point2::new(u, v);
    }
    /// Store origin flag and coordinates without recomputing; `object.set_texture_origin(false,2.,3.)`.
    fn set_texture_origin(&self, enabled: bool, u: f64, v: f64) {
        let base = self.textured_shape_base();
        base.fields.write().custom_origin = enabled;
        base.shape.fields.write().uv_origin = Point2::new(u, v);
    }
    /// Store scale flag and factors without validating or recomputing; `object.set_texture_scale(false,2.,3.)`.
    fn set_texture_scale(&self, enabled: bool, u: f64, v: f64) {
        let base = self.textured_shape_base();
        base.fields.write().scale = enabled;
        base.shape.fields.write().uv_scale = Point2::new(u, v);
    }
}
impl<T: AisTexturedShapeSource + ?Sized> AisTexturedShapeMappingMembers for T {}
