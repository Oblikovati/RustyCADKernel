//! A real group override replaces the actual shading field before forwarding its original argument.
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::standard::{Handle, HandleReference, SharedState, WeakHandle};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{
    ais::*, prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dShadingAspect, v3d::V3dViewer,
};
use std::{fmt::Write, sync::Arc};
struct ColoredAspectFieldReplacementProbe {
    shading: WeakHandle<Prs3dShadingAspect>,
    previous: WeakHandle<Graphic3dAspectFillArea3d>,
    incoming: WeakHandle<Graphic3dAspectFillArea3d>,
    rows: Arc<SharedState<String>>,
}
impl display_driver::SensitiveAspectArgumentCallback for ColoredAspectFieldReplacementProbe {
    fn aspect_received(&self, argument: &dyn HandleReference<dyn Graphic3dAspectsSource>) {
        let shading = self.shading.observe();
        let reference = shading.aspect_ref();
        let same = i32::from(argument.reference_address() == reference.reference_address());
        self.record_before(argument, same);
        shading.set_aspect(self.incoming.observe().clone());
        self.record_after(argument, same);
    }
}
impl ColoredAspectFieldReplacementProbe {
    fn record_before(&self, argument: &dyn HandleReference<dyn Graphic3dAspectsSource>, same: i32) {
        let actual = argument.observe().ref_count();
        let previous = self.previous.observe().ref_count();
        let incoming = self.incoming.observe().ref_count();
        writeln!(
            &mut *self.rows.write(),
            "aspect-argument-before {same} {actual} {previous} {incoming}"
        )
        .unwrap();
    }
    fn record_after(&self, argument: &dyn HandleReference<dyn Graphic3dAspectsSource>, same: i32) {
        let equal = i32::from(argument.object_address() == self.incoming.object_address());
        let actual = argument.observe().ref_count();
        let previous = self.previous.observe().ref_count();
        let incoming = self.incoming.observe().ref_count();
        writeln!(
            &mut *self.rows.write(),
            "aspect-argument-after {same} {equal} {actual} {previous} {incoming}"
        )
        .unwrap();
    }
}
#[test]
fn native_colored_group_receives_live_shading_aspect_argument_and_retains_replacement() {
    let shape = super::colored_shape_presentation_slot::presentation_swap_shape();
    let object = AisColoredShape::new(&shape);
    object.attributes().setup_own_defaults().unwrap();
    object.attributes().set_auto_triangulation(false);
    let shading = object.attributes().shading_aspect();
    let previous = shading.aspect();
    let incoming = Handle::new(Graphic3dAspectFillArea3d::default());
    let rows = Arc::new(SharedState::new(String::new()));
    render_aspect_argument_scene(&object, &shading, &previous, &incoming, &rows);
    let native = include_str!("fixtures/colored-shape-compute/native-aspect-argument.txt");
    assert_eq!(*rows.read(), native);
}
fn render_aspect_argument_scene(
    object: &AisColoredShape,
    shading: &Handle<Prs3dShadingAspect>,
    previous: &Handle<Graphic3dAspectFillArea3d>,
    incoming: &Handle<Graphic3dAspectFillArea3d>,
    rows: &Arc<SharedState<String>>,
) {
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(-1);
    arm_aspect_argument_probe(&trace, shading, previous, incoming, rows);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    ais_colored_shape_compute(object, &prs.cast(), 1).unwrap();
    record_aspect_argument_finish(&prs, previous, incoming, rows);
}

fn arm_aspect_argument_probe(
    trace: &display_driver::DisplayTrace,
    shading: &Handle<Prs3dShadingAspect>,
    previous: &Handle<Graphic3dAspectFillArea3d>,
    incoming: &Handle<Graphic3dAspectFillArea3d>,
    rows: &Arc<SharedState<String>>,
) {
    trace.write().aspect_argument_callback = Some(Box::new(ColoredAspectFieldReplacementProbe {
        shading: shading.downgrade(),
        previous: previous.downgrade(),
        incoming: incoming.downgrade(),
        rows: rows.clone(),
    }));
}

fn record_aspect_argument_finish(
    prs: &Graphic3dStructure,
    previous: &Handle<Graphic3dAspectFillArea3d>,
    incoming: &Handle<Graphic3dAspectFillArea3d>,
    rows: &SharedState<String>,
) {
    let (groups, old, next) = (
        prs.groups().len(),
        previous.ref_count(),
        incoming.ref_count(),
    );
    writeln!(
        &mut *rows.write(),
        "aspect-argument-finished {groups} {old} {next}"
    )
    .unwrap();
}
