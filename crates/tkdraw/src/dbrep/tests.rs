//! Tests of the DBRep drawables and commands; the drawing paths run on a recording
//! window system (the batch oracle scripts never draw).
use super::*;
use crate::draw::fake_window::FakeWindowSystem;
use crate::draw::{DrawColorKind, DrawViewer, RecordingConsole};
use rustycad_tkbrep::topods::Shape;
use std::rc::Rc;

/// A BRep file of the repository: a cylinder minus a box (planes and cylinders).
fn data_file() -> String {
    format!(
        "{}/../tktopalgo/tests/data/brep_gprop/occ49-cylinder-minus-box.brep",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// A batch session with the DBRep commands.
fn batch_session() -> Rc<DrawSession> {
    let session = DrawSession::new(Rc::new(RecordingConsole::default()), DrawViewer::batch());
    basic_commands(&session).unwrap();
    session
}

/// A session with a 3D view on a fake window, the DBRep commands and the shape `s`.
fn windowed_session() -> (Rc<DrawSession>, Rc<FakeWindowSystem>) {
    let system = FakeWindowSystem::new();
    let viewer = DrawViewer::with_window_system(system.clone());
    let session = DrawSession::new(Rc::new(RecordingConsole::default()), viewer);
    basic_commands(&session).unwrap();
    session
        .viewer()
        .make_view(1, "AXON", 0, 0, 400, 400)
        .unwrap();
    system.log.borrow_mut().clear();
    (session, system)
}

fn restore_shape(session: &DrawSession, name: &str) {
    let script = format!("brestore {{{}}} {name}", data_file());
    session.interpretor().eval(&script).unwrap();
}

fn segments(system: &FakeWindowSystem) -> usize {
    system
        .log
        .borrow()
        .iter()
        .filter(|line| line.contains("segment"))
        .count()
}

fn uses_color(system: &FakeWindowSystem, kind: DrawColorKind) -> bool {
    let suffix = format!("color {}", kind.index());
    system
        .log
        .borrow()
        .iter()
        .any(|line| line.ends_with(&suffix))
}

#[test]
fn a_null_shape_variable_reads_back_as_null() {
    let session = batch_session();
    session.set_shape("s", &Shape::default()).unwrap();
    assert!(session.shape("s").unwrap().is_null());
    assert!(
        session
            .get("s")
            .unwrap()
            .downcast_kind::<DBRepDrawableShape>()
            .is_some()
    );
}

#[test]
fn a_shape_draws_its_isos_edges_and_free_vertices() {
    let (session, system) = windowed_session();
    restore_shape(&session, "s");
    // Connected edges in yellow (the isos are blue, as the standard axes are).
    assert!(
        uses_color(&system, DrawColorKind::Jaune),
        "{:?}",
        system.log.borrow()
    );
    let wireframe = segments(&system);
    assert!(wireframe > 50, "{wireframe} segments");
    let drawable = session.get("s").unwrap();
    let shape = drawable.downcast_kind::<DBRepDrawableShape>().unwrap();
    shape.change_nb_isos(0, session.console().as_ref()).unwrap();
    system.log.borrow_mut().clear();
    session.viewer().repaint_all().unwrap();
    assert!(segments(&system) < wireframe);
}

#[test]
fn the_orientation_display_colors_edges_and_adds_arrows() {
    let (session, system) = windowed_session();
    restore_shape(&session, "s");
    let plain = segments(&system);
    session.interpretor().eval("vori s; repaint").unwrap();
    system.log.borrow_mut().clear();
    session.viewer().repaint_all().unwrap();
    assert!(uses_color(&system, DrawColorKind::Rouge));
    // Two more segments per edge: the arrow at its end.
    assert!(
        segments(&system) > plain,
        "{} <= {plain}",
        segments(&system)
    );
}

#[test]
fn the_hidden_line_mode_draws_the_projected_segments() {
    let (session, system) = windowed_session();
    restore_shape(&session, "s");
    session.interpretor().eval("hlr hlr s").unwrap();
    system.log.borrow_mut().clear();
    session.viewer().repaint_all().unwrap();
    let visible = segments(&system);
    assert!(visible > 0, "{:?}", system.log.borrow());
    session.interpretor().eval("hlr hid s").unwrap();
    system.log.borrow_mut().clear();
    session.viewer().repaint_all().unwrap();
    assert!(segments(&system) > visible);
}

#[test]
fn meshed_shapes_show_their_triangulations_and_polygons() {
    let (session, system) = windowed_session();
    restore_shape(&session, "s");
    session.interpretor().eval("hlr hlr s").unwrap();
    session.viewer().repaint_all().unwrap(); // meshes the shape
    session
        .interpretor()
        .eval("hlr nohlr s; triangles s; polygons s")
        .unwrap();
    system.log.borrow_mut().clear();
    session.viewer().repaint_all().unwrap();
    assert!(
        uses_color(&system, DrawColorKind::Rouge),
        "{:?}",
        system.log.borrow()
    );
}

#[test]
fn the_iso_builder_trims_the_isos_by_the_face_boundary() {
    let session = batch_session();
    restore_shape(&session, "s");
    session.interpretor().eval("explode s f").unwrap();
    let face = rustycad_tkbrep::topods::face(&session.shape("s_1").unwrap()).unwrap();
    let builder = DBRepIsoBuilder::new(&face, 100.0, 2).unwrap();
    assert!(builder.nb_domains() >= 2, "{}", builder.nb_domains());
    let mut record = DBRepFace::new(
        face,
        builder.nb_domains(),
        crate::draw::DrawColor::default(),
    );
    builder
        .load_isos(&mut record, &RecordingConsole::default())
        .unwrap();
    let first = record.iso(1).unwrap();
    assert!(first.first < first.last, "{first:?}");
}

#[test]
fn the_last_pick_starts_empty_and_the_orientation_colors_follow_the_legend() {
    assert!(last_pick().shape.is_null());
    let console = RecordingConsole::default();
    write_color_orientation(&console);
    assert_eq!(
        console.output(),
        "\nrouge  FORWARD\nbleu   REVERSED\nrose   EXTERNAL\norange INTERNAL\n"
    );
}
