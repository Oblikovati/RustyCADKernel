//! Native aspect replacement keeps existing groups live and preserves width/color quirks.
use crate::{prs_mgr::*, prs3d::Prs3dLineAspect};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
pub(super) trait LineDatumAppearance: PrsMgrPresentableObject {
    fn replace_line_aspect(&self, aspect: Handle<Prs3dLineAspect>) {
        let drawer = self.attributes();
        if !drawer.has_link() {
            drawer.set_line_aspect(aspect);
            return;
        }
        // Copy the handle before replacing its wrapper (native 3d097a0 borrows freed storage).
        let old = drawer
            .line_aspect()
            .aspect()
            .cast::<dyn Graphic3dAspectsSource>();
        let new = self.replacement_graphic_aspect(&aspect);
        if old == new {
            return;
        }
        drawer.set_line_aspect(aspect);
        let map = Graphic3dAspectsReplaceMap::from([(old.object_address() as usize, (old, new))]);
        self.replace_aspects(&map);
    }
    fn replacement_graphic_aspect(
        &self,
        aspect: &Handle<Prs3dLineAspect>,
    ) -> Handle<dyn Graphic3dAspectsSource> {
        if aspect.is_null() {
            return self.attributes().link().line_aspect().aspect().cast();
        }
        aspect.aspect().cast()
    }
    fn install_line_aspect(&self, color: Color, width: f64) -> Result<(), Failure> {
        let aspect = Prs3dLineAspect::new(color, AspectTypeOfLine::Solid, width)?;
        self.replace_line_aspect(Handle::new(aspect));
        Ok(())
    }
    fn inherited_line_width(&self) -> f64 {
        let drawer = self.attributes();
        if drawer.has_link() {
            f64::from(drawer.link().line_aspect().aspect().width())
        } else {
            1.
        }
    }
    fn effective_line_color(&self) -> Color {
        let drawer = self.attributes();
        if self.has_color() {
            return drawer.color();
        }
        if drawer.has_link() {
            return drawer.link().line_aspect().aspect().interior_color();
        }
        Color::named(NamedColor::YELLOW)
    }
    fn set_line_color(&self, color: Color) -> Result<(), Failure> {
        self.presentable_base().fields.write().has_own_color = true;
        let drawer = self.attributes();
        drawer.set_color(color);
        let width = if self.has_width() {
            f64::from(self.presentable_base().fields.read().own_width)
        } else {
            self.inherited_line_width()
        };
        if !drawer.has_own_line_aspect() {
            self.install_line_aspect(color, width)?;
        } else {
            drawer.line_aspect().set_color(color);
            self.synchronize_aspects();
        }
        Ok(())
    }
    fn unset_line_color(&self) -> Result<(), Failure> {
        self.presentable_base().fields.write().has_own_color = false;
        if !self.has_width() {
            self.replace_line_aspect(Handle::default());
            return Ok(());
        }
        let color = self.effective_line_color();
        let drawer = self.attributes();
        drawer.line_aspect().set_color(color);
        drawer.set_color(color);
        self.synchronize_aspects();
        Ok(())
    }
    fn set_line_width(&self, width: f64) -> Result<(), Failure> {
        // Native assigns the narrowed own width before a failing aspect constructor/setter.
        self.presentable_base().fields.write().own_width = width as f32;
        let drawer = self.attributes();
        if !drawer.has_own_line_aspect() {
            self.install_line_aspect(self.effective_line_color(), width)?;
        } else {
            drawer.line_aspect().set_width(width)?;
            self.synchronize_aspects();
        }
        Ok(())
    }
    fn unset_line_width(&self, synchronize: bool) -> Result<(), Failure> {
        // OCCT 3d097a0 does not reset own_width in the no-color branch.
        if !self.has_color() {
            self.replace_line_aspect(Handle::default());
            return Ok(());
        }
        let width = self.inherited_line_width() as f32;
        self.attributes()
            .line_aspect()
            .set_width(f64::from(width))?;
        self.presentable_base().fields.write().own_width = width;
        // AIS_Circle omits this notification, unlike AIS_Line at OCCT 3d097a0.
        if synchronize {
            self.synchronize_aspects();
        }
        Ok(())
    }
}

impl<T: PrsMgrPresentableObject + ?Sized> LineDatumAppearance for T {}
