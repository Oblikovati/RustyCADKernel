//! Native AIS_Plane presentation modes; public mode acceptance remains wireframe-only.
use super::AisPlane;
use crate::{dsg_prs::*, prs_mgr::*, std_prs::StdPrsPlane};
use rustycad_tkernel::{
    quantity::Color,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{geom::GeomSurface, geom_adaptor::GeomAdaptorSurface};
use rustycad_tkservice::graphic3d::*;
impl AisPlane {
    fn compute_plane_presentation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.compute_fields()?;
        prs.set_infinite_state(self.is_infinite());
        self.attributes()
            .plane_aspect()
            .edges_aspect()
            .set_width(if self.current_mode() { 3. } else { 1. })?;
        if mode != 0 && mode != 1 {
            return Ok(());
        }
        if self.is_xyz_plane() {
            return self.compute_xyz_presentation(prs, mode);
        }
        self.compute_frame()?;
        if mode == 1 {
            return self.compute_shaded_rectangle(prs);
        }
        let surface = GeomAdaptorSurface::with_surface(&Handle::new(self.centered_plane()).cast())?;
        StdPrsPlane::add(prs, &surface, &self.attributes())
    }
    fn compute_xyz_presentation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        let f = self.fields.read().clone();
        if mode == 0 {
            return DsgPrsXYZPlanePresentation::add(
                prs,
                &self.attributes(),
                &f.center,
                &f.pmin,
                &f.pmax,
            );
        }
        DsgPrsShadedPlanePresentation::add(prs, &self.attributes(), &f.center, &f.pmin, &f.pmax)
    }
    fn compute_shaded_rectangle(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let aspect = self.attributes().plane_aspect();
        let group = prs.current_group()?;
        group.set_primitives_aspect(&self.attributes().shading_aspect().aspect().cast());
        // OCCT reads lengths after the aspect callback and uses the untranslated component.
        let x = 0.5 * aspect.plane_x_length();
        let y = 0.5 * aspect.plane_y_length();
        let vertices = Handle::new(Graphic3dArrayOfQuadrangles::new(4, 0, 0)?);
        for (u, v) in [(-x, y), (x, y), (x, -y), (-x, -y)] {
            vertices.add_vertex_pnt(&self.component().d0(u, v)?)?;
        }
        group.add_primitive_array(&vertices.cast(), true);
        Ok(())
    }
}
impl PrsMgrPresentableObject for AisPlane {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.compute_plane_presentation(prs, mode)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        self.set_plane_color(color)
    }
    fn unset_color(&self) -> Result<(), Failure> {
        self.unset_plane_color()
    }
    crate::ais_presentable_overrides!();
}
