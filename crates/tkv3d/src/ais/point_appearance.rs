//! Preserve OCCT point override precedence and mutation order, including failed scales.
use super::AisPoint;
use crate::{prs_mgr::*, prs3d::Prs3dPointAspect};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};
impl AisPoint {
    fn inherited_point_values(&self) -> (Color, AspectTypeOfMarker, f64) {
        let drawer = self.attributes();
        if !drawer.has_link() {
            return (
                Color::named(NamedColor::YELLOW),
                AspectTypeOfMarker::Plus,
                1.,
            );
        }
        let aspect = drawer.link().point_aspect().aspect();
        (
            aspect.interior_color(),
            aspect.marker_kind(),
            f64::from(aspect.marker_scale()),
        )
    }
    fn effective_point_values(&self) -> (Color, AspectTypeOfMarker, f64) {
        let (mut color, mut kind, mut scale) = self.inherited_point_values();
        if self.has_color() {
            color = self.attributes().color();
        }
        if self.has_width() {
            scale = f64::from(self.base.presentable.fields.read().own_width);
        }
        if self.has_marker() {
            kind = self.marker.read().kind;
        }
        (color, kind, scale)
    }
    pub(super) fn update_point_values(&self) -> Result<(), Failure> {
        if !self.has_color() && !self.has_width() && !self.has_marker() {
            self.replace_point_aspect(Handle::default());
            return Ok(());
        }
        let (color, kind, scale) = self.effective_point_values();
        if self.attributes().has_own_point_aspect() {
            let aspect = self.attributes().point_aspect();
            aspect.set_color(color);
            aspect.set_type_of_marker(kind);
            aspect.set_scale(scale)?;
            self.synchronize_aspects();
            return Ok(());
        }
        self.replace_point_aspect(Handle::new(Prs3dPointAspect::new(kind, color, scale)?));
        Ok(())
    }
    fn replace_point_aspect(&self, aspect: Handle<Prs3dPointAspect>) {
        let drawer = self.attributes();
        if !drawer.has_link() {
            drawer.set_point_aspect(aspect);
            return;
        }
        let old = drawer
            .point_aspect()
            .aspect()
            .cast::<dyn Graphic3dAspectsSource>();
        let new = if aspect.is_null() {
            drawer.link().point_aspect().aspect().cast()
        } else {
            aspect.aspect().cast()
        };
        if old == new {
            return;
        }
        drawer.set_point_aspect(aspect);
        self.replace_aspects(&Graphic3dAspectsReplaceMap::from([(
            old.object_address() as usize,
            (old, new),
        )]));
    }
    pub(super) fn set_point_color(&self, color: Color) -> Result<(), Failure> {
        self.base.presentable.fields.write().has_own_color = true;
        self.attributes().set_color(color);
        self.update_point_values()
    }
    pub(super) fn unset_point_color(&self) -> Result<(), Failure> {
        self.base.presentable.fields.write().has_own_color = false;
        self.update_point_values()
    }
}
