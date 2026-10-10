//! Axis appearance at OCCT 3d097a0 preserves native mutation order and cached line handles.
use super::{AisAxis, axis_geometry::AXIS_PARTS};
use crate::{prs_mgr::*, prs3d::Prs3dLineAspect};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsMembers, Graphic3dPresentationAttributesSource,
};
impl AisAxis {
    /// Install the line aspect and propagate its color to all datum axes; `axis.set_display_aspect(&line)?`.
    pub fn set_display_aspect(&self, aspect: &Handle<Prs3dLineAspect>) -> Result<(), Failure> {
        self.attributes().set_line_aspect(aspect.clone());
        self.geometry.write().line = self.attributes().line_aspect();
        self.set_axis_color(aspect.aspect().interior_color());
        Ok(())
    }
    pub(super) fn set_axis_color(&self, color: Color) {
        self.base.presentable.fields.write().has_own_color = true;
        let drawer = self.attributes();
        drawer.set_color(color);
        drawer.line_aspect().set_color(color);
        for part in AXIS_PARTS {
            drawer.datum_aspect().line_aspect(part).set_color(color);
        }
        self.synchronize_aspects();
    }
    pub(super) fn unset_axis_color(&self) {
        let drawer = self.attributes();
        drawer
            .line_aspect()
            .set_color(Color::named(NamedColor::RED));
        self.base.presentable.fields.write().has_own_color = false;
        for part in AXIS_PARTS {
            drawer
                .datum_aspect()
                .line_aspect(part)
                .set_color(Color::named(NamedColor::TURQUOISE));
        }
        self.synchronize_aspects();
    }
    pub(super) fn set_axis_width(&self, width: f64) -> Result<(), Failure> {
        if width < 0. {
            return Ok(());
        }
        // Native 3d097a0 resets first, then still calls SetWidth(0), which can throw.
        if width == 0. {
            self.unset_axis_width()?;
        }
        self.apply_axis_width(width)
    }
    pub(super) fn unset_axis_width(&self) -> Result<(), Failure> {
        self.base.presentable.fields.write().own_width = 0.;
        self.apply_axis_width(1.)
    }
    fn apply_axis_width(&self, width: f64) -> Result<(), Failure> {
        let drawer = self.attributes();
        drawer.line_aspect().set_width(width)?;
        for part in AXIS_PARTS {
            drawer.datum_aspect().line_aspect(part).set_width(width)?;
        }
        self.synchronize_aspects();
        Ok(())
    }
}
