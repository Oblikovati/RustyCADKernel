//! Actual native NewGroup assignment changes both topology and location before later vertex collection.
use super::{colored_compound_reference_graphics::*, *};
use crate::{
    prs3d::Prs3dDrawer,
    std_prs::{StdPrsShadedShape, StdPrsVolume, StdPrsWfShape},
};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkservice::graphic3d::Graphic3dStructureMembers;
use std::{fmt::Write, sync::Arc};
#[test]
fn live_native_value_assignment_rereads_transformed_vertex_topology_after_graphics_callbacks() {
    let rows = replay_live_compound(false);
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-compound-wf-live.txt")
    );
}
#[test]
fn live_native_shaded_stages_reread_the_value_after_unmeshed_wireframe_callbacks() {
    let rows = replay_live_compound(true);
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-compound-shade-live.txt")
    );
}
fn replay_live_compound(shaded: bool) -> String {
    let fixture = compound_live_fixture();
    let map = Arc::new(AisColoredDrawerCompounds::default());
    let drawer = live_compound_drawer();
    map.add(
        &Handle::new(AisColoredDrawer::new(&drawer)),
        &fixture.original,
    )
    .unwrap();
    let trace = ColoredCompoundGraphicsTrace::with_replacement(&map, &fixture);
    let prs = compound_reference_presentation(&trace);
    render_live_compound(shaded, &prs, &map, &drawer);
    record_live_compound_finish(shaded, &trace, &prs, &fixture);
    trace.rows.lock().unwrap().clone()
}
fn record_live_compound_finish(
    shaded: bool,
    trace: &ColoredCompoundGraphicsTrace,
    prs: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource>,
    fixture: &ColoredCompoundLiveFixture,
) {
    let label = live_compound_finish_label(shaded);
    let counts = (
        fixture.original.tshape().ref_count(),
        fixture.incoming.tshape().ref_count(),
    );
    writeln!(
        trace.rows.lock().unwrap(),
        "{label} {} {} {}",
        prs.groups().len(),
        counts.0,
        counts.1
    )
    .unwrap();
}

fn live_compound_finish_label(shaded: bool) -> &'static str {
    if shaded {
        return "shade-live-after";
    }
    "wf-live-after"
}
fn render_live_compound(
    shaded: bool,
    prs: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource>,
    map: &AisColoredDrawerCompounds,
    drawer: &Handle<Prs3dDrawer>,
) {
    let reference = map.node_ref(1).unwrap();
    if shaded {
        StdPrsShadedShape::add_reference(
            prs,
            &reference,
            drawer,
            StdPrsVolume::Autodetection,
            &Handle::default(),
        )
        .unwrap();
        return;
    }
    StdPrsWfShape::add_reference(prs, &reference, drawer).unwrap();
}
fn live_compound_drawer() -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.set_auto_triangulation(false);
    drawer.set_iso_on_plane(false);
    drawer
}
fn compound_live_fixture() -> Arc<ColoredCompoundLiveFixture> {
    let input = colored_shape_expansion_inputs::ColoredExpansionInputs::new();
    let mut original = Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut original);
    builder.add(&mut original, &input.faces[0]).unwrap();
    Arc::new(ColoredCompoundLiveFixture {
        original,
        incoming: compound_replacement_vertex(),
    })
}
fn compound_replacement_vertex() -> Compound {
    let builder = BRepBuilder::default();
    let mut incoming = Compound::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(1., 2., 3.), 1e-7)
        .unwrap();
    builder.make_compound(&mut incoming);
    builder.add(&mut incoming, &vertex).unwrap();
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(10., 20., 30.));
    incoming
        .set_location(&Location::new(transform), false)
        .unwrap();
    incoming
}
