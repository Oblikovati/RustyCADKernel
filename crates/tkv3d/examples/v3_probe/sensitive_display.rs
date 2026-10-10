//! Exact native geometry buffers, graphic callbacks and retained display structures.
use super::common::{Reader, attempt, end, flag, int, real};
#[allow(dead_code)]
#[path = "../../tests/support/sensitive_display_driver.rs"]
mod display_driver;
#[allow(dead_code)]
#[path = "../../tests/support/sensitive_display_scene.rs"]
mod display_scene;
use display_driver::{DisplayTrace, SensitiveDisplayDriver};
use display_scene::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{select_mgr::*, v3d::V3dViewer};
fn display_trace(trace: &DisplayTrace) {
    let trace = trace.read();
    int(trace.events.len() as i32);
    for value in &trace.events {
        real(*value);
    }
}
fn display_structure_state(prs: &Handle<Graphic3dStructure>) {
    int(prs.groups().len() as i32);
    int(prs.display_priority() as i32);
    flag(prs.is_displayed());
    let persistence = prs.transform_persistence();
    int(if persistence.is_null() {
        0
    } else {
        persistence.mode().0
    });
}
pub fn sensitive_prs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let [kind, transform, flags, repeats, fail]: [i32; 5] = std::array::from_fn(|_| r.integer());
    let (driver, trace) = SensitiveDisplayDriver::new_handle(fail);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let selection = display_selection(kind, flags).unwrap();
    if flags & 8 != 0 {
        trace.write().append_selection = selection.clone();
    }
    let persistence = display_probe_persistence(flags);
    for _ in 0..repeats {
        attempt(|| {
            SelectMgr::compute_sensitive_prs(
                &prs.cast(),
                &selection,
                &display_location(transform),
                &persistence,
            )
        });
    }
    display_trace(&trace);
    display_structure_state(&prs);
    end();
}
use super::boxed_selectable;
#[path = "../../tests/support/sensitive_display_view.rs"]
mod display_view;
pub fn sensitive_display_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let transform = r.integer();
    let flags = r.integer();
    let mode = r.integer();
    let repeats = r.integer();
    let fail = r.integer();
    let fixture = display_view::SensitiveDisplayView::new(kind, transform, flags, fail).unwrap();
    for i in 0..repeats {
        attempt(|| fixture.run(if mode == 3 { i % 3 } else { mode }, transform));
    }
    display_trace(&fixture.trace);
    let fields = fixture.selector.viewer_selector_base().fields.read();
    int(fields.debug_structures.length());
    for prs in fields.debug_structures.iter() {
        display_structure_state(prs);
    }
    end();
}

fn display_probe_persistence(flags: i32) -> Handle<dyn Graphic3dTransformPersSource> {
    if flags & 2 == 0 {
        return Handle::default();
    }
    Handle::new(
        Graphic3dTransformPers::with_point(
            Graphic3dTransModeFlags::ZOOM_PERS,
            &Point3::new(1., 2., 3.),
        )
        .unwrap(),
    )
    .cast()
}
