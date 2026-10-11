//! Named presentation-only subclass keeps selection explicitly unsupported, never silently empty.
use super::*;
use crate::{prs_mgr::*, select_mgr::*, v3d::V3dViewer};
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient, exceptions::program_error},
    tcollection::AsciiString,
};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use std::sync::atomic::{AtomicUsize, Ordering};
pub(super) struct CubePresentationProbe {
    transient: StandardTransient,
    pub display: AisViewCubePresentationBase,
    _context: Handle<AisInteractiveContext>,
    pub side_calls: AtomicUsize,
}
rustycad_tkernel::impl_transient!(CubePresentationProbe,"CubePresentationProbe",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn PrsMgrPresentableObject,dyn SelectMgrSelectableObject]);
impl CubePresentationProbe {
    pub fn new() -> Handle<Self> {
        let context =
            AisInteractiveContext::new(&V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap())
                .unwrap();
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            display: AisViewCubePresentationBase::new().unwrap(),
            _context: context.clone(),
            side_calls: AtomicUsize::new(0),
        });
        object.set_context(&context);
        prs_mgr_presentable_bind(&object.cast());
        object
    }
}
impl AisInteractiveObject for CubePresentationProbe {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.display.interactive
    }
}
impl AisViewCubePresentationSource for CubePresentationProbe {
    fn view_cube_presentation_base(&self) -> &AisViewCubePresentationBase {
        &self.display
    }
    fn create_box_side_triangles(
        &self,
        array: &Handle<Graphic3dArrayOfTriangles>,
        counts: &mut AisViewCubeTriangleCounts,
        orientation: crate::v3d::V3dTypeOfOrientation,
    ) -> Result<(), Failure> {
        self.side_calls.fetch_add(1, Ordering::Relaxed);
        ais_view_cube_create_box_side_triangles(self, array, counts, orientation)
    }
}
impl PrsMgrPresentableObject for CubePresentationProbe {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.display.interactive.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_view_cube_compute(self, prs, mode)
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for CubePresentationProbe {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.display.interactive.selectable
    }
    fn compute_selection(&self, _: &Handle<SelectMgrSelection>, mode: i32) -> Result<(), Failure> {
        Err(program_error(&format!(
            "CubePresentationProbe selection mode={mode}; expected application-provided selection implementation"
        )))
    }
}
pub(super) fn configured_cube(id: i32) -> Handle<CubePresentationProbe> {
    let cube = CubePresentationProbe::new();
    configure_cube_geometry(&cube, id);
    if id == 6 {
        let datum = cube.attributes().datum_aspect();
        datum.set_draw_datum_axes(crate::prs3d::Prs3dDatumAxes::XZAxes);
        datum.set_draw_labels(true);
        cube.display.settings.write().axes_labels = ["u", "v", "w"].map(AsciiString::from);
        cube.attributes().text_aspect().set_height(22.0);
    }
    cube.attributes().set_face_boundary_draw(id % 3 == 1);
    cube
}
fn configure_cube_geometry(cube: &CubePresentationProbe, id: i32) {
    let mut settings = cube.display.settings.write();
    match id {
        1 => settings.geometry.round_radius = 0.2,
        2 => configure_cube_compact(&mut settings),
        3 => settings.geometry.facet_extension = -3.0,
        4 => {
            settings.draw_edges = false;
            settings.draw_corners = false;
        }
        5 => settings.is_y_up = true,
        7 => configure_cube_minimum_parts(&mut settings),
        _ => {}
    }
}
fn configure_cube_compact(settings: &mut AisViewCubePresentationSettings) {
    let g = &mut settings.geometry;
    g.size = 43.0;
    g.round_radius = 0.5;
    g.facet_extension = 7.0;
    g.edge_gap = 4.0;
    g.edge_min_size = 3.0;
    g.corner_min_size = 4.0;
    g.axes_padding = 2.0;
    settings.draw_axes = false;
}
fn configure_cube_minimum_parts(settings: &mut AisViewCubePresentationSettings) {
    let g = &mut settings.geometry;
    g.round_radius = 0.1;
    g.facet_extension = 0.0;
    g.edge_min_size = 6.0;
    g.corner_min_size = 3.0;
    settings.draw_axes = false;
    settings.side_labels[4] = AsciiString::default();
}
