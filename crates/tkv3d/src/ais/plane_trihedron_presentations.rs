//! Plane axes deliberately share the X line aspect (native OCCT 3d097a0).
use super::AisPlaneTrihedron;
use crate::{dsg_prs::DsgPrsXYZAxisPresentation, prs_mgr::*, prs3d::Prs3dDatumParts};
use rustycad_tkernel::{
    quantity::Color,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::graphic3d::{
    Graphic3dPresentationAttributesSource, Graphic3dStructureMembers, Graphic3dStructureSource,
};
impl AisPlaneTrihedron {
    pub(super) fn plane_axis_geometry(&self) -> Result<[(Point3, Direction3, f64); 2], Failure> {
        // Ax2() normalizes indirect frames, unlike Pln().YAxis() used by YAxis().
        let frame = self.component().pln().position().ax2()?;
        let datum = self.attributes().datum_aspect();
        Ok([
            (
                frame.location(),
                frame.x_direction(),
                datum.axis_length(Prs3dDatumParts::XAxis),
            ),
            (
                frame.location(),
                frame.y_direction(),
                datum.axis_length(Prs3dDatumParts::YAxis),
            ),
        ])
    }
    fn compute_plane_axes(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let drawer = self.attributes();
        let labels = self.labels.read();
        for ((origin, direction, length), label) in
            self.plane_axis_geometry()?.into_iter().zip(labels.iter())
        {
            let last = Point3::from(origin.xyz() + direction.xyz() * length);
            DsgPrsXYZAxisPresentation::add_with_aspects(
                prs,
                &drawer.datum_aspect().line_aspect(Prs3dDatumParts::XAxis),
                &drawer.arrow_aspect(),
                &drawer.text_aspect(),
                &direction,
                length,
                label.as_bytes(),
                &origin,
                &last,
            )?;
        }
        prs.set_infinite_state(true);
        Ok(())
    }
}
impl PrsMgrPresentableObject for AisPlaneTrihedron {
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
        _: i32,
    ) -> Result<(), Failure> {
        self.compute_plane_axes(prs)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        self.base.presentable.fields.write().has_own_color = true;
        self.attributes().set_color(color);
        for part in [Prs3dDatumParts::XAxis, Prs3dDatumParts::YAxis] {
            self.attributes()
                .datum_aspect()
                .line_aspect(part)
                .set_color(color);
        }
        self.synchronize_aspects();
        Ok(())
    }
    crate::ais_presentable_overrides!();
}
