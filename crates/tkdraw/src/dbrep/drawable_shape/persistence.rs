// Rust translation of DBRep_DrawableShape::Save and ::Restore
// (Draw/TKDraw/DBRep/DBRep_DrawableShape.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `bsave`/`brestore` format of a drawable shape: a `BRepTools_ShapeSet` and the
//! shape record (the existing BRep text persistence of TKBRep).
use super::{DBRepDrawableShape, HlrSettings, ShapeColors};
use crate::dbrep::params::current;
use crate::draw::{DrawIStream, DrawOStream, DrawSession, DrawableHandle, active_progress_bar};
use rustycad_tkbrep::brep_tools::BRepToolsShapeSet;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeSet;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use std::rc::Rc;

impl DBRepDrawableShape {
    /// `Save`: the shape set of the shape, then the shape record, unless the user
    /// stopped the progress indicator of the `bsave` command meanwhile.
    pub(super) fn save_shape(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        let mut set = BRepToolsShapeSet::new(true, false);
        set.add(&self.shape)?;
        let progress = active_progress_bar();
        stream.write_through(|io| {
            let range = progress
                .as_ref()
                .map_or_else(ProgressRange::default, |p| p.start());
            set.write(io, &range)?;
            if progress.as_ref().is_none_or(|p| !p.user_break()) {
                set.write_shape(&self.shape, io)?;
            }
            Ok(())
        })
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DBRep_DrawableShape", Self::restore);
    }

    /// `Restore`: reads the shape set and the shape record and makes a drawable with
    /// the standard colors and the settings of `DBRep::Parameters()`; nothing when the
    /// user stopped the progress indicator. A read failure is returned, as the C++
    /// exception leaves `brestore`.
    pub fn restore(
        session: &DrawSession,
        stream: &mut DrawIStream,
    ) -> Result<Option<DrawableHandle>, Failure> {
        let params = current(session);
        let progress = session.progress_bar();
        let shape = stream.read_through(|io| -> Result<Option<Shape>, Failure> {
            let mut set = BRepToolsShapeSet::new(true, false);
            let range = progress
                .as_ref()
                .map_or_else(ProgressRange::default, |p| p.start());
            set.read(io, &range)?;
            if progress.as_ref().is_some_and(|p| p.user_break()) {
                return Ok(None);
            }
            Ok(Some(set.read_shape(io)?))
        })?;
        let Some(shape) = shape else {
            return Ok(None);
        };
        let drawable = Self::new(
            shape,
            ShapeColors::standard(),
            params.size,
            params.nb_isos,
            params.discretization,
        );
        drawable.set_display_triangulation(params.disp_triangles);
        drawable.set_display_polygons(params.display_polygons);
        drawable.display_hlr(HlrSettings {
            with_hlr: params.with_hlr,
            with_rg1: params.with_rg1,
            with_rgn: params.with_rgn,
            with_hid: params.with_hid,
            angle: params.hlr_angle,
        });
        Ok(Some(Rc::new(drawable)))
    }
}
