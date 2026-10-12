//! Native custom setters defer flags until drawer mutation succeeds and ignore null topology.
use super::{AisColoredShapeSource, shape_appearance::set_shape_drawer_color};
use super::{
    shape_transparency::set_shape_drawer_transparency, shape_width::set_shape_drawer_width,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{quantity::Color, standard::Failure};
/// Native nonvirtual custom appearance members; `object.set_custom_width(&edge,2.0)?`.
pub trait AisColoredShapeCustomAppearance: AisColoredShapeSource {
    /// Override all native color aspects without invalidating an existing entry; `object.set_custom_color(&edge,color)?`.
    fn set_custom_color(&self, shape: &Shape, color: Color) -> Result<(), Failure> {
        if shape.is_null() {
            return Ok(());
        }
        let drawer = self.custom_aspects(shape)?;
        set_shape_drawer_color(self, &drawer.cast(), color)?;
        drawer.set_own_color(color);
        Ok(())
    }
    /// Override shading before publishing ownership; `object.set_custom_transparency(&face,0.4)?`.
    fn set_custom_transparency(&self, shape: &Shape, value: f64) -> Result<(), Failure> {
        if shape.is_null() {
            return Ok(());
        }
        let drawer = self.custom_aspects(shape)?;
        set_shape_drawer_transparency(self, &drawer.cast(), value)?;
        drawer.set_own_transparency(value);
        Ok(())
    }
    /// Override six line categories before publishing ownership; `object.set_custom_width(&wire,3.0)?`.
    fn set_custom_width(&self, shape: &Shape, value: f64) -> Result<(), Failure> {
        if shape.is_null() {
            return Ok(());
        }
        let drawer = self.custom_aspects(shape)?;
        set_shape_drawer_width(&drawer.cast(), value)?;
        drawer.set_own_width(value);
        Ok(())
    }
}
impl<S: AisColoredShapeSource + ?Sized> AisColoredShapeCustomAppearance for S {}
