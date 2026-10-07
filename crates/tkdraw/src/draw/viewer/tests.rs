use super::*;
use crate::draw::color::{DrawColor, DrawColorKind};
use crate::draw::display::{DrawDisplay, DrawingMode};
use crate::draw::drawables::{DrawGrid, DrawSegment2D, DrawSegment3D};
use crate::draw::fake_window::FakeWindowSystem;
use crate::draw::ostream::DrawOStream;
use crate::draw::window::{BUTTON_PRESS, DrawWindowEvent, MOTION_NOTIFY};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};

fn segment_3d() -> DrawableHandle {
    Rc::new(DrawSegment3D::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(10.0, 0.0, 0.0),
        DrawColor::new(DrawColorKind::Rouge),
    ))
}

#[test]
fn the_batch_viewer_ignores_everything() {
    let mut viewer = DrawViewer::batch();
    let drawable = segment_3d();
    viewer.display(&drawable).unwrap();
    assert!(!drawable.visible());
    assert!(viewer.drawables().is_empty());
    assert_eq!(viewer.zoom(0), 0.0);
    assert_eq!(viewer.focal(0), 1.0);
    assert!(viewer.select(true, &mut String::new()).is_none());
    assert_eq!(viewer.pick(0, 0, 0, 5, 0).unwrap().0, 0);
}

#[test]
fn a_displayed_segment_is_drawn_in_matching_views_only() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system.clone());
    viewer.make_view(1, "+X+Y", 0, 0, 200, 100).unwrap();
    viewer.make_view(2, "-2D-", 0, 0, 200, 100).unwrap();
    system.log.borrow_mut().clear();
    let drawable = segment_3d();
    viewer.display(&drawable).unwrap();
    let log = system.log.borrow().clone();
    // View 1 (3D) draws the segment from the window centre (100, 50); view 2 only gets
    // the display set-up calls, as upstream makes the display before the dimension test.
    assert_eq!(
        log,
        vec![
            "101 color 0",
            "101 mode 3",
            "101 color 1",
            "101 segment [[100, 50], [110, 50]]",
            "102 color 0",
            "102 mode 3"
        ]
    );
    assert!(drawable.visible());
    assert_eq!(drawable.bounds(), [0.0, 10.0, 0.0, 0.0]);
}

#[test]
fn an_unknown_view_type_removes_the_view() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system);
    viewer.make_view(3, "BAD!", 0, 0, 100, 100).unwrap();
    assert!(!viewer.has_view(3));
    viewer.make_view(3, "AXON", 0, 0, 100, 100).unwrap();
    assert_eq!(viewer.view_type(3), "AXON");
    assert!(viewer.is_3d(3));
}

#[test]
fn clear_2d_keeps_3d_objects() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system);
    viewer.make_view(1, "-2D-", 0, 0, 100, 100).unwrap();
    let flat: DrawableHandle = Rc::new(DrawSegment2D::new(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        DrawColor::default(),
    ));
    let solid = segment_3d();
    viewer.display(&flat).unwrap();
    viewer.display(&solid).unwrap();
    viewer.clear_2d();
    assert!(!flat.visible());
    assert!(solid.visible());
    viewer.clear();
    assert!(viewer.drawables().is_empty());
}

#[test]
fn zoom_and_fit_follow_the_cpp_formulas() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system);
    viewer.make_view(1, "+X+Y", 0, 0, 200, 100).unwrap();
    viewer.set_zoom(1, 2.0);
    assert_eq!(viewer.zoom(1), 2.0);
    assert_eq!(viewer.pan(1), Some((100, -50)));
    let drawable = segment_3d();
    viewer.display(&drawable).unwrap();
    viewer.fit_view(1, 10).unwrap();
    // Width 10 model units over 200 - 2*10 pixels: zoom 18; the height is degenerate.
    assert_eq!(viewer.zoom(1), 18.0);
    assert_eq!(viewer.pan(1), Some((10, -50)));
}

#[test]
fn pick_finds_the_segment_under_the_point() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system);
    viewer.make_view(1, "+X+Y", 0, 0, 200, 100).unwrap();
    let drawable = segment_3d();
    viewer.display(&drawable).unwrap();
    let (index, found) = viewer.pick(1, 5, 0, 2, 0).unwrap();
    assert_eq!(index, 1);
    assert!(Rc::ptr_eq(&found.unwrap(), &drawable));
    assert_eq!(viewer.pick(1, 50, 50, 2, 0).unwrap().0, 0);
}

#[test]
fn select_waits_for_a_click_and_returns_view_coordinates() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system.clone());
    viewer.make_view(4, "+X+Y", 0, 0, 200, 100).unwrap();
    let window = 101;
    system.events.borrow_mut().extend([
        DrawWindowEvent {
            kind: MOTION_NOTIFY,
            window,
            button: 0,
            x: 1,
            y: 1,
        },
        DrawWindowEvent {
            kind: BUTTON_PRESS,
            window,
            button: 3,
            x: 110,
            y: 40,
        },
    ]);
    let selection = viewer.select(true, &mut String::new()).unwrap();
    assert_eq!(
        selection,
        DrawSelection {
            view: 4,
            x: 10,
            y: 10,
            button: 3
        }
    );
}

#[test]
fn select_without_mapped_views_reports_it() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system);
    let mut errors = String::new();
    assert_eq!(
        viewer.select(true, &mut errors),
        Some(DrawSelection::default())
    );
    assert_eq!(errors, "No selection is possible with no open views\n");
}

#[test]
fn postscript_output_maps_view_to_paper() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system);
    viewer.make_view(1, "+X+Y", 0, 0, 200, 100).unwrap();
    let drawable = segment_3d();
    viewer.display(&drawable).unwrap();
    let mut output = DrawOStream::new();
    viewer
        .postscript_view(1, [0, 0, 100, 100], [0, 0, 200, 200], &mut output)
        .unwrap();
    let expected = "stroke\nnewpath\n1 setlinewidth\n0 setgray\nstroke\nnewpath\n1 setlinewidth\n0 setgray\nstroke\nnewpath\n0 0 m\n20 0 l\nstroke\n";
    assert_eq!(output.text(), expected);
}

#[test]
fn the_grid_draws_crosses_at_the_visible_nodes() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system.clone());
    viewer.make_view(1, "+X+Y", 0, 0, 20, 20).unwrap();
    let grid = Rc::new(DrawGrid::new());
    grid.set_steps(10.0, 10.0, 10.0);
    let handle: DrawableHandle = grid;
    system.log.borrow_mut().clear();
    viewer.display(&handle).unwrap();
    let segments = system
        .log
        .borrow()
        .iter()
        .filter(|line| line.contains("segment"))
        .count();
    // Frame [-10, 10] x [-10, 10] at zoom 1: nodes -1..1 on both axes, two strokes each.
    assert_eq!(segments, 18);
}

#[test]
fn exposing_a_view_repaints_it() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system.clone());
    viewer.make_view(1, "+X+Y", 0, 0, 200, 100).unwrap();
    viewer.display(&segment_3d()).unwrap();
    system.log.borrow_mut().clear();
    viewer.expose_view(1).unwrap();
    let log = system.log.borrow().clone();
    assert_eq!(log.first().map(String::as_str), Some("101 clear"));
    assert!(log.iter().any(|line| line.contains("segment")));
}

/// A drawable whose geometry evaluation fails, as a C++ `DrawOn` that throws.
struct FailingDrawable {
    base: crate::draw::Drawable3DBase,
}

impl crate::draw::Drawable3D for FailingDrawable {
    crate::draw::drawable::drawable_identity!("FailingDrawable");

    fn draw_on(&self, _display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        Err(Failure::with_message("Standard_DomainError", "no geometry"))
    }
}

#[test]
fn a_drawing_failure_reaches_the_caller_and_ends_the_pick_mode() {
    let system = FakeWindowSystem::new();
    let mut viewer = DrawViewer::with_window_system(system);
    viewer.make_view(1, "+X+Y", 0, 0, 200, 100).unwrap();
    let failing: DrawableHandle = Rc::new(FailingDrawable {
        base: crate::draw::Drawable3DBase::new(),
    });
    let failure = viewer.display(&failing).unwrap_err();
    assert!(failure.to_string().contains("no geometry"), "{failure}");
    assert!(failing.visible());
    assert!(viewer.repaint_all().is_err());
    // The failed display left the bounds at the origin, so the pick reaches the object.
    assert!(viewer.pick(1, 0, 0, 2, 0).is_err());
    assert_eq!(viewer.display_state().mode, DrawingMode::Draw);
    let mut output = DrawOStream::new();
    let paper = [0, 0, 200, 200];
    assert!(
        viewer
            .postscript_view(1, [0, 0, 100, 100], paper, &mut output)
            .is_err()
    );
    assert_eq!(viewer.display_state().mode, DrawingMode::Draw);
}
