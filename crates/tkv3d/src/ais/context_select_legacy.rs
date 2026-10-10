//! Deprecated selection entry points preserve their selection scheme and redraw timing.
use super::*;
use crate::v3d::V3dView;
use rustycad_tkernel::{
    ncollection::{Array1, Vec2},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::Point2;
impl AisInteractiveContext {
    /// Replace selection with the detected owner and optionally redraw; `context.select(true)?`.
    pub fn select(&self, update: bool) -> Result<AisStatusOfPick, Failure> {
        let status = self.select_detected(AisSelectionScheme::Replace)?;
        Ok(self.redraw_selection_result(status, update))
    }
    /// Toggle the detected owner and optionally redraw; `context.shift_select(true)?`.
    pub fn shift_select(&self, update: bool) -> Result<AisStatusOfPick, Failure> {
        let status = self.select_detected(AisSelectionScheme::Xor)?;
        Ok(self.redraw_selection_result(status, update))
    }
    /// Replace selection within a rectangle and optionally redraw; `context.select_box(0,0,40,30,&view,true)?`.
    pub fn select_box(
        &self,
        xmin: i32,
        ymin: i32,
        xmax: i32,
        ymax: i32,
        view: &Handle<V3dView>,
        update: bool,
    ) -> Result<AisStatusOfPick, Failure> {
        let status = self.select_rectangle(
            Vec2::new(xmin, ymin),
            Vec2::new(xmax, ymax),
            view,
            AisSelectionScheme::Replace,
        )?;
        Ok(self.redraw_selection_result(status, update))
    }
    /// Toggle selection within a rectangle and optionally redraw; `context.shift_select_box(0,0,40,30,&view,true)?`.
    pub fn shift_select_box(
        &self,
        xmin: i32,
        ymin: i32,
        xmax: i32,
        ymax: i32,
        view: &Handle<V3dView>,
        update: bool,
    ) -> Result<AisStatusOfPick, Failure> {
        let status = self.select_rectangle(
            Vec2::new(xmin, ymin),
            Vec2::new(xmax, ymax),
            view,
            AisSelectionScheme::Xor,
        )?;
        Ok(self.redraw_selection_result(status, update))
    }
    /// Replace selection within a polygon and optionally redraw; `context.select_polyline(&points,&view,true)?`.
    pub fn select_polyline(
        &self,
        points: &Array1<'_, Point2>,
        view: &Handle<V3dView>,
        update: bool,
    ) -> Result<AisStatusOfPick, Failure> {
        let status = self.select_polygon(points, view, AisSelectionScheme::Replace)?;
        Ok(self.redraw_selection_result(status, update))
    }
    /// Toggle selection within a polygon and optionally redraw; `context.shift_select_polyline(&points,&view,true)?`.
    pub fn shift_select_polyline(
        &self,
        points: &Array1<'_, Point2>,
        view: &Handle<V3dView>,
        update: bool,
    ) -> Result<AisStatusOfPick, Failure> {
        let status = self.select_polygon(points, view, AisSelectionScheme::Xor)?;
        Ok(self.redraw_selection_result(status, update))
    }
    fn redraw_selection_result(&self, status: AisStatusOfPick, update: bool) -> AisStatusOfPick {
        if update {
            self.update_current_viewer();
        }
        status
    }
}
