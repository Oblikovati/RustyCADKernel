//! The process-wide undefined-type handler: the base behaviour, installation, a custom
//! handler that stores another class, and the debug dump.
use crate::fakes::{
    HelixHandler, RecordingConsole, handler_lock, helix, line, output_stream, text_stream, unread,
    written_text,
};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomTrimmedCurve};
use rustycad_tkgeombase::geom_tools::{
    self, UndefinedTypeHandler, UndefinedTypeHandlerBase, geom_tools_dump,
};
use std::sync::Arc;

/// Runs `body` with `handler` installed, then restores the previous handler.
fn with_handler(handler: Handle<dyn UndefinedTypeHandler>, body: impl FnOnce()) {
    let _guard = handler_lock();
    let previous = geom_tools::get_undefined_type_handler();
    geom_tools::set_undefined_type_handler(&handler);
    body();
    geom_tools::set_undefined_type_handler(&previous);
}

fn recording_base(console: &Arc<RecordingConsole>) -> Handle<dyn UndefinedTypeHandler> {
    Handle::new(UndefinedTypeHandlerBase::with_console(console.clone())).into_cast()
}

#[test]
fn the_base_handler_dumps_to_the_stream_and_prints_compact_lines_to_the_console() {
    let console = Arc::new(RecordingConsole::default());
    with_handler(recording_base(&console), || {
        let curve = helix();
        let mut dump = output_stream();
        geom_tools::dump_curve(&curve, &mut dump).unwrap();
        assert_eq!(written_text(dump), "****** UNKNOWN CURVE TYPE ******\n");
        let mut compact = output_stream();
        geom_tools::write_curve(&curve, &mut compact).unwrap();
        assert_eq!(
            written_text(compact),
            "",
            "the compact form leaves the stream untouched"
        );
        assert_eq!(console.text(), "****** UNKNOWN CURVE TYPE ******\n");
    });
}

#[test]
fn a_null_handler_is_ignored() {
    let console = Arc::new(RecordingConsole::default());
    let installed = recording_base(&console);
    with_handler(installed.clone(), || {
        geom_tools::set_undefined_type_handler(&Handle::default());
        assert!(geom_tools::get_undefined_type_handler() == installed);
    });
}

#[test]
fn a_custom_handler_writes_and_reads_a_class_geomtools_does_not_know() {
    let console = Arc::new(RecordingConsole::default());
    let handler: Handle<dyn UndefinedTypeHandler> =
        Handle::new(HelixHandler::new(console.clone())).into_cast();
    with_handler(handler, || {
        let trimmed =
            Handle::new(GeomTrimmedCurve::new(&helix(), 0.0, 1.0, true, true).unwrap()).into_cast();
        let mut stream = IoStream::from_bytes(Vec::new());
        geom_tools::write_curve(&trimmed, &mut stream).unwrap();
        let text = written_text(stream);
        assert_eq!(text, "8 0 1\n42 1.5 0.25\n");
        let back = geom_tools::read_curve(&mut text_stream(&text));
        assert!(back.is_instance_name("Geom_TrimmedCurve"));
        assert_eq!(console.text(), "");
    });
}

#[test]
fn a_failing_handler_read_is_caught_as_a_null_curve() {
    let console = Arc::new(RecordingConsole::default());
    let handler: Handle<dyn UndefinedTypeHandler> =
        Handle::new(HelixHandler::new(console)).into_cast();
    with_handler(handler, || {
        let mut stream = text_stream("13 1 2");
        assert!(geom_tools::read_curve(&mut stream).is_null());
        assert_eq!(unread(&mut stream), " 1 2");
        let mut stream = text_stream("42 -1 1");
        assert!(
            geom_tools::read_curve(&mut stream).is_null(),
            "a negative helix radius raises"
        );
    });
}

#[test]
fn the_debug_dump_names_what_it_found() {
    let console = RecordingConsole::default();
    let null: Handle<StandardTransient> = Handle::default();
    assert_eq!(
        geom_tools_dump(&null, &console),
        "Error: Not a geometric object"
    );
    let point: Handle<StandardTransient> =
        Handle::new(GeomCartesianPoint::new(1.0, 2.0, 3.0)).into_cast();
    assert_eq!(
        geom_tools_dump(&point, &console),
        "Error: Not a geometric object"
    );
    assert_eq!(console.text(), "");
    let curve: Handle<StandardTransient> = line(0.0).cast();
    assert_eq!(
        geom_tools_dump(&curve, &console),
        "Found Geom_Curve, see dump in std::cout"
    );
    assert_eq!(
        console.text(),
        "\n\nLine\n  Origin :0, 0, 0 \n  Axis   :0, 0, 1 \n\n\n"
    );
}
