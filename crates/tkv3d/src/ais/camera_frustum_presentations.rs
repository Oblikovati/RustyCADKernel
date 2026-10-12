//! Native presentation routing and in-place appearance synchronization.
use super::AisCameraFrustum;
use crate::prs_mgr::*;
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
impl AisCameraFrustum {
    fn update_frustum_color(&self, color: Color) -> Result<(), Failure> {
        self.attributes()
            .shading_aspect()
            .set_color(color, AspectTypeOfFacingModel::BothSide)?;
        self.attributes().line_aspect().set_color(color);
        self.synchronize_aspects();
        Ok(())
    }
    fn compute_frustum(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        prs.set_infinite_state(true);
        let fields = self.fields.read();
        if fields.triangles.is_null() {
            return Ok(());
        }
        if mode == 1 {
            let group = prs.new_group()?;
            group.set_group_primitives_aspect(&self.attributes().shading_aspect().aspect().cast());
            group.add_primitive_array(&fields.triangles.cast(), true);
        }
        if mode == 0 || mode == 1 {
            let group = prs.new_group()?;
            group.set_group_primitives_aspect(&self.attributes().line_aspect().aspect().cast());
            group.add_primitive_array(&fields.borders.cast(), true);
        }
        Ok(())
    }
}
impl PrsMgrPresentableObject for AisCameraFrustum {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0 || mode == 1
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.compute_frustum(&prs.observe(), mode)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        self.attributes().set_color(color);
        self.presentable_base().fields.write().has_own_color = true;
        self.update_frustum_color(color)
    }
    fn unset_color(&self) -> Result<(), Failure> {
        if !self.has_color() {
            return Ok(());
        }
        self.presentable_base().fields.write().has_own_color = false;
        self.update_frustum_color(Color::named(NamedColor::WHITE))
    }
    fn unset_transparency(&self) -> Result<(), Failure> {
        self.attributes()
            .shading_aspect()
            .set_transparency(0.0, AspectTypeOfFacingModel::BothSide)?;
        self.attributes().set_transparency(0.0);
        self.synchronize_aspects();
        Ok(())
    }
    crate::ais_presentable_overrides!();
}
