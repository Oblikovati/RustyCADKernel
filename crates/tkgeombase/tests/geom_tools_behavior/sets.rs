//! The three set classes: indexing, null handles, the table text, appending reads, a
//! wrong tag and progress interruption.
use crate::fakes::{
    BreakingIndicator, RecordingConsole, circle, line, output_stream, text_stream, unread,
    written_text,
};
use rustycad_tkernel::message::{ProgressIndicator, ProgressRange};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::geom_tools::{Curve2dSet, CurveSet, SurfaceSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

#[test]
fn curve_set_indexes_by_identity_and_refuses_null_handles() {
    let (first, second) = (line(0.0), line(0.0));
    let mut set = CurveSet::new();
    assert_eq!(
        (set.add(&first), set.add(&second), set.add(&first)),
        (1, 2, 1)
    );
    assert_eq!(set.index(&second), 2);
    assert_eq!(set.add(&Handle::default()), 0);
    assert_eq!(set.index(&Handle::default()), 0);
    assert!(set.curve(0).is_null() && set.curve(3).is_null());
    set.clear();
    assert!(set.curve(1).is_null());
}

#[test]
fn curve2d_and_surface_sets_store_a_null_handle_like_any_element() {
    let mut curves2d = Curve2dSet::new();
    let null2d: Handle<dyn Geom2dCurve> = Handle::default();
    assert_eq!(
        (
            curves2d.index(&null2d),
            curves2d.add(&null2d),
            curves2d.add(&null2d)
        ),
        (0, 1, 1)
    );
    assert_eq!(curves2d.index(&null2d), 1);
    let mut surfaces = SurfaceSet::new();
    let null: Handle<dyn GeomSurface> = Handle::default();
    assert_eq!(surfaces.add(&null), 1);
    assert_eq!(surfaces.index(&null), 1);
    let error = surfaces.dump(&mut output_stream()).unwrap_err();
    assert_eq!(
        error.exception_type(),
        "Standard_NullObject",
        "the native Dump dereferences it"
    );
}

#[test]
fn dump_numbers_each_element_and_write_tags_the_table() {
    let mut set = CurveSet::new();
    set.add(&line(1.0));
    let mut stream = output_stream();
    set.dump(&mut stream).unwrap();
    let dump = written_text(stream);
    assert!(
        dump.starts_with("\n -------\nDump of 1 Curves \n -------\n\n   1 : Line\n"),
        "{dump}"
    );
    let mut stream = output_stream();
    set.write(&mut stream, &ProgressRange::default()).unwrap();
    assert_eq!(written_text(stream), "Curves 1\n1 1 0 0 0 0 1 \n");
}

#[test]
fn read_appends_to_the_set_instead_of_clearing_it() {
    let mut set = CurveSet::new();
    set.add(&circle(1.0));
    set.read(
        &mut text_stream("Curves 1\n1 0 0 0 0 0 1 \n"),
        &ProgressRange::default(),
    );
    assert!(set.curve(1).is_instance_name("Geom_Circle"));
    assert!(set.curve(2).is_instance_name("Geom_Line"));
}

#[test]
fn a_failed_record_is_stored_once_as_a_null_curve() {
    let _guard = crate::fakes::handler_lock();
    let mut set = CurveSet::new();
    set.read(
        &mut text_stream("Curves 3\n42\n42\n1 0 0 0 0 0 1 \n"),
        &ProgressRange::default(),
    );
    assert!(set.curve(1).is_null());
    assert!(
        set.curve(2).is_instance_name("Geom_Line"),
        "the second null maps to index 1"
    );
}

#[test]
fn a_wrong_tag_reports_on_the_console_and_reads_nothing() {
    let console = Arc::new(RecordingConsole::default());
    let mut set = SurfaceSet::with_console(console.clone());
    let mut stream = text_stream("Curves 1\n1 0 0 0 0 0 1 1 0 0 0 1 0\n");
    set.read(&mut stream, &ProgressRange::default());
    assert_eq!(console.text(), "Not a surface table\n");
    assert!(set.surface(1).is_null());
    assert!(unread(&mut stream).starts_with(" 1\n1 0"));
}

#[test]
fn a_breaking_progress_indicator_stops_the_write_after_its_limit() {
    let mut set = CurveSet::new();
    for x in 0..3 {
        set.add(&line(f64::from(x)));
    }
    let calls = Arc::new(AtomicI32::new(0));
    let indicator = ProgressIndicator::new(Box::new(BreakingIndicator {
        calls: calls.clone(),
        limit: 1,
    }));
    let mut stream = output_stream();
    set.write(&mut stream, &indicator.start()).unwrap();
    assert_eq!(written_text(stream), "Curves 3\n1 0 0 0 0 0 1 \n");
    assert!(
        calls.load(Ordering::SeqCst) >= 2,
        "More() asked the indicator each step"
    );
}
