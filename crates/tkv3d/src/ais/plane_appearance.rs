//! Native copy-on-write creates default aspects, rather than copying inherited styles.
use super::AisPlane;
use crate::{prs_mgr::*, prs3d::*, select_mgr::*};
use rustycad_tkernel::precision::confusion;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
impl AisPlane {
    pub(super) fn init_drawer_attributes(&self) -> Result<(), Failure> {
        let shading = Handle::new(Prs3dShadingAspect::new()?);
        let model = AspectTypeOfFacingModel::BothSide;
        shading.set_material(
            &Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified),
            model,
        );
        shading.set_color(Color::named(NamedColor::GRAY40), model)?;
        self.attributes().set_shading_aspect(shading.clone());
        let mut material = shading.aspect().front_material();
        material.set_transparency(0.8_f32)?;
        shading.aspect().set_front_material(material.clone());
        shading.aspect().set_back_material(material);
        Ok(())
    }
    fn writable_plane_aspects(
        &self,
    ) -> Result<(Handle<Prs3dPlaneAspect>, Handle<Prs3dDatumAspect>), Failure> {
        let drawer = self.attributes();
        let mut pa = drawer.plane_aspect();
        let mut da = drawer.datum_aspect();
        if drawer.has_link() && drawer.link().plane_aspect() == pa {
            pa = Handle::new(Prs3dPlaneAspect::new()?);
            drawer.set_plane_aspect(pa.clone());
        }
        if drawer.has_link() && drawer.link().datum_aspect() == da {
            da = Handle::new(Prs3dDatumAspect::new()?);
            drawer.set_datum_aspect(da.clone());
        }
        Ok((pa, da))
    }
    fn inherited_plane_aspects(
        &self,
    ) -> Result<(Handle<Prs3dPlaneAspect>, Handle<Prs3dDatumAspect>), Failure> {
        let drawer = self.attributes();
        if drawer.has_link() {
            return Ok((drawer.link().plane_aspect(), drawer.link().datum_aspect()));
        }
        Ok((
            Handle::new(Prs3dPlaneAspect::new()?),
            Handle::new(Prs3dDatumAspect::new()?),
        ))
    }
    fn refresh_plane_size(&self) -> Result<(), Failure> {
        self.set_to_update_all();
        self.update_presentations(false)?;
        self.update_selection(-1);
        Ok(())
    }
    /// Set equal X/Y lengths and request presentation and selection updates; `plane.set_size(10.)?`.
    pub fn set_size(&self, length: f64) -> Result<(), Failure> {
        self.set_size_xy(length, length)
    }
    /// Set rectangular lengths, with Z datum length equal to X; `plane.set_size_xy(4.,6.)?`.
    pub fn set_size_xy(&self, x: f64, y: f64) -> Result<(), Failure> {
        let (pa, da) = self.writable_plane_aspects()?;
        pa.set_plane_length(x, y);
        da.set_axis_length(x, y, x);
        self.fields.write().has_own_size = true;
        self.refresh_plane_size()
    }
    /// Inspect explicit size ownership; `plane.has_own_size()`.
    pub fn has_own_size(&self) -> bool {
        self.fields.read().has_own_size
    }
    /// Restore inherited lengths or release uncolored aspects; `plane.unset_size()?`.
    pub fn unset_size(&self) -> Result<(), Failure> {
        if !self.has_own_size() {
            return Ok(());
        }
        if !self.has_color() {
            self.release_plane_aspects();
        } else {
            self.restore_plane_lengths()?;
        }
        self.fields.write().has_own_size = false;
        self.refresh_plane_size()
    }
    fn release_plane_aspects(&self) {
        self.attributes().set_plane_aspect(Handle::default());
        self.attributes().set_datum_aspect(Handle::default());
    }
    fn restore_plane_lengths(&self) -> Result<(), Failure> {
        let (pa, da) = self.inherited_plane_aspects()?;
        self.attributes()
            .plane_aspect()
            .set_plane_length(pa.plane_x_length(), pa.plane_y_length());
        self.attributes().datum_aspect().set_axis_length(
            da.axis_length(Prs3dDatumParts::XAxis),
            da.axis_length(Prs3dDatumParts::YAxis),
            da.axis_length(Prs3dDatumParts::ZAxis),
        );
        Ok(())
    }
    /// Assign lengths and report equality within Confusion; `plane.size(&mut x,&mut y)`.
    pub fn size(&self, x: &mut f64, y: &mut f64) -> bool {
        let aspect = self.attributes().plane_aspect();
        *x = aspect.plane_x_length();
        *y = aspect.plane_y_length();
        (*x - *y).abs() <= confusion()
    }
    pub(super) fn set_plane_color(&self, color: Color) -> Result<(), Failure> {
        let (pa, da) = self.writable_plane_aspects()?;
        apply_plane_color(&pa, &da, color);
        self.attributes()
            .shading_aspect()
            .set_color(color, AspectTypeOfFacingModel::BothSide)?;
        self.base.presentable.fields.write().has_own_color = true;
        self.attributes().set_color(color);
        Ok(())
    }
    pub(super) fn unset_plane_color(&self) -> Result<(), Failure> {
        if !self.has_color() {
            return Ok(());
        }
        if !self.has_own_size() {
            self.release_plane_aspects();
        } else {
            let (pa, _) = self.inherited_plane_aspects()?;
            apply_plane_color(
                &self.attributes().plane_aspect(),
                &self.attributes().datum_aspect(),
                pa.edges_aspect().aspect().interior_color(),
            );
        }
        // Native UnsetColor leaves shading and drawer colors untouched.
        self.base.presentable.fields.write().has_own_color = false;
        Ok(())
    }
}
fn apply_plane_color(pa: &Handle<Prs3dPlaneAspect>, da: &Handle<Prs3dDatumAspect>, color: Color) {
    pa.edges_aspect().set_color(color);
    for part in [
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
    ] {
        da.line_aspect(part).set_color(color);
    }
}
