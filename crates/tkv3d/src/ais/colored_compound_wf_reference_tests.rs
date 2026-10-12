//! Actual StdPrs WF calls replay native scoped Value counts, checked failure and repair without replacing geometry.
use super::{colored_compound_reference_graphics::*, *};
use crate::{prs3d::Prs3dDrawer, std_prs::StdPrsWfShape};
use rustycad_tkbrep::{brep::BRepBuilder, topods::Compound};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::Graphic3dStructureMembers;
use std::{
    fmt::Write,
    sync::{Arc, atomic::Ordering},
};
#[test]
fn actual_wf_callbacks_preserve_uncounted_compound_reference_through_failure_and_repair() {
    let mut rows = String::new();
    for fail in [false, true] {
        rows.push_str(&compound_wf_reference_rows(fail));
    }
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-compound-wf-reference.txt")
    );
}
fn compound_wf_reference_rows(fail: bool) -> String {
    let drawer = compound_wf_reference_drawer();
    let key = Handle::new(AisColoredDrawer::new(&drawer));
    let input = colored_shape_expansion_inputs::ColoredExpansionInputs::new();
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    BRepBuilder::default()
        .add(&mut compound, &input.faces[0])
        .unwrap();
    let map = Arc::new(AisColoredDrawerCompounds::default());
    map.add(&key, &compound).unwrap();
    let trace = ColoredCompoundGraphicsTrace::new(&map, fail);
    let prs = compound_reference_presentation(&trace);
    record_compound_wf_before(&trace, fail, compound.tshape().ref_count());
    project_compound_wireframe(&map, &prs, &drawer, &trace);
    record_compound_wf_finish(&map, &compound, &prs, &trace);
    std::mem::take(&mut *trace.rows.lock().unwrap())
}
fn record_compound_wf_before(trace: &ColoredCompoundGraphicsTrace, fail: bool, count: i32) {
    let failed = i32::from(fail);
    writeln!(trace.rows.lock().unwrap(), "wf-before {failed} {count}").unwrap();
}
fn compound_wf_reference_drawer() -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.set_auto_triangulation(false);
    drawer.set_iso_on_plane(false);
    drawer
}
fn project_compound_wireframe(
    map: &AisColoredDrawerCompounds,
    prs: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource>,
    drawer: &Handle<Prs3dDrawer>,
    trace: &ColoredCompoundGraphicsTrace,
) {
    let alias = map.node_ref(1).unwrap();
    let result = StdPrsWfShape::add_reference(prs, &alias, drawer);
    if let Err(failure) = result {
        assert_eq!(failure.exception_type(), "Standard_ProgramError");
        let count = alias.compound_ref_count().unwrap();
        writeln!(trace.rows.lock().unwrap(), "wf-failed {count}").unwrap();
        trace.fail.store(false, Ordering::Relaxed);
        alias
            .with_shape(|shape| StdPrsWfShape::add(prs, shape, drawer))
            .unwrap();
    }
}
fn record_compound_wf_finish(
    map: &AisColoredDrawerCompounds,
    compound: &Compound,
    prs: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource>,
    trace: &ColoredCompoundGraphicsTrace,
) {
    let count = compound.tshape().ref_count();
    writeln!(
        trace.rows.lock().unwrap(),
        "wf-after {} {count}",
        prs.groups().len()
    )
    .unwrap();
    map.clear(false);
    let count = compound.tshape().ref_count();
    writeln!(trace.rows.lock().unwrap(), "wf-cleared {count}").unwrap();
}
