// Rust translation of DBRep_DrawableShape::DisplayHiddenLines
// (Draw/TKDraw/DBRep/DBRep_DrawableShape.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The hidden-line display of a drawable shape, cached per view.
use super::DBRepDrawableShape;
use super::plot::record_pick;
use crate::dbrep::hide_data::DBRepHideData;
use crate::draw::DrawDisplay;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkhlr::hlr_brep::poly_hlr_angle_and_deflection;
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
use rustycad_tkmesh::imesh_tools::MeshParameters;

impl DBRepDrawableShape {
    /// `DisplayHiddenLines`: meshes the shape (relative deflection from the HLR angle),
    /// then draws the hidden-line result of this view, computed again when the view
    /// rotated (or, in perspective, moved or changed focal) or the angle changed.
    pub fn display_hidden_lines(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        let id = display.view_id();
        let Some(view) = display.view_query() else {
            return Ok(());
        };
        let focal = if view.view_type == "PERS" {
            view.focal
        } else {
            -1.0
        };
        let settings = self.hlr.get();
        let (angle, deflection) = poly_hlr_angle_and_deflection(settings.angle);
        let parameters = MeshParameters {
            relative: true,
            deflection,
            angle,
            ..MeshParameters::default()
        };
        BRepMeshIncrementalMesh::with_parameters(
            &self.shape,
            &parameters,
            &ProgressRange::default(),
        )?;
        let flags = (settings.with_rg1, settings.with_rgn, settings.with_hid);
        let colors = (self.colors.connected, self.colors.isos);
        let mut cache = self.hidden.borrow_mut();
        if let Some(index) = cache.iter().position(|data| data.view_id() == id) {
            let recompute = !cache[index].is_same(&view.matrix, focal)
                || settings.angle != cache[index].angle();
            if !recompute {
                draw_cached(&mut cache[index], display, flags, colors);
                return Ok(());
            }
            cache.remove(index);
        }
        let mut data = DBRepHideData::default();
        data.set(id, (view.matrix, focal), &self.shape, settings.angle)?;
        draw_cached(&mut data, display, flags, colors);
        cache.push(data);
        Ok(())
    }
}

/// Draws one cached result and records its picked shape (parameters 0).
fn draw_cached(
    data: &mut DBRepHideData,
    display: &mut DrawDisplay<'_>,
    flags: (bool, bool, bool),
    colors: (crate::draw::DrawColor, crate::draw::DrawColor),
) {
    data.draw_on(display, flags, colors);
    if display.has_picked() {
        record_pick(data.last_pick(), 0.0, 0.0);
    }
}
