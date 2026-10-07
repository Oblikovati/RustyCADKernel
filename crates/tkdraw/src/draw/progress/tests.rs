use super::*;
use crate::draw::RecordingConsole;
use rustycad_tkernel::tcollection::AsciiString;
use std::rc::Rc;

fn interpretor() -> DrawInterpretor {
    let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
    di.init();
    di
}

#[test]
fn named_scopes_are_listed_with_value_and_maximum() {
    let di = interpretor();
    let progress = DrawProgressIndicator::new(&di, 1.0);
    progress.set_tcl_mode(true);
    {
        let range = progress.start();
        let scope = ProgressScope::new(&range, &AsciiString::new("Step"), 4.0, false);
        drop(scope.next(1.0));
    }
    let result = di.result().unwrap();
    assert!(result.starts_with("Progress: 0%\n"), "{result}");
    assert!(result.contains("Progress: 25% Step: 1 / 4\n"), "{result}");
}

#[test]
fn small_steps_are_not_shown_unless_forced() {
    let di = interpretor();
    let progress = DrawProgressIndicator::new(&di, 50.0);
    progress.set_tcl_mode(true);
    {
        let range = progress.start();
        let scope = ProgressScope::unnamed(&range, 10.0, false);
        drop(scope.next(1.0));
    }
    // The start shows 0%; 10% is below the 50% threshold; closing reaches 100%, which
    // is always shown.
    assert_eq!(di.result().unwrap(), "Progress: 0%\nProgress: 100%\n");
}

#[test]
fn the_stop_request_breaks_this_indicator_only() {
    let di = interpretor();
    di.eval("proc XProgress {args} {}").unwrap();
    let progress = DrawProgressIndicator::new(&di, 1.0);
    let other = DrawProgressIndicator::new(&di, 1.0);
    set_stop_indicator(progress.id());
    assert!(progress.user_break());
    assert!(!other.user_break());
    set_stop_indicator(0);
    progress.reset();
    assert!(!progress.user_break());
}

#[test]
fn modes_default_from_the_process_settings() {
    let di = interpretor();
    let progress = DrawProgressIndicator::new(&di, 1.0);
    assert!(!progress.graph_mode());
    progress.set_console_mode(true);
    assert!(progress.console_mode());
    assert!(!progress.tcl_mode());
}

#[test]
fn show_presents_the_current_position() {
    let di = interpretor();
    let progress = DrawProgressIndicator::new(&di, 1.0);
    progress.set_tcl_mode(true);
    progress.show(progress.indicator().root_scope(), true);
    assert_eq!(di.result().unwrap(), "Progress: 0%\n");
}
