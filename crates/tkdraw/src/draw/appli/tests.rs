use super::*;
use crate::draw::RecordingConsole;

/// A batch session with the base commands, printing to a recording console.
fn session() -> (Rc<DrawSession>, Rc<RecordingConsole>) {
    let console = Rc::new(RecordingConsole::default());
    let session = DrawSession::new(console.clone(), DrawViewer::batch());
    register_base_commands(&session, |_| Ok(())).unwrap();
    session.interpretor().set_to_colorize(false);
    (session, console)
}

#[test]
fn a_command_prints_its_result_on_a_line() {
    let (session, console) = session();
    assert!(interprete(&session, "dset a 2"));
    assert!(interprete(&session, "dval a*3"));
    assert_eq!(console.take_output(), "2\n6\n");
}

#[test]
fn an_incomplete_command_is_not_evaluated_nor_continued() {
    let (session, console) = session();
    assert!(!interprete(&session, "set x {"));
    assert!(interprete(&session, "}"));
    assert_eq!(console.take_output(), "invalid command name \"}\"\n");
}

#[test]
fn errors_are_red_when_colors_are_on() {
    let (session, console) = session();
    session.interpretor().set_to_colorize(true);
    interprete(&session, "error oops");
    let output = console.take_output();
    if cfg!(windows) {
        assert_eq!(output, "oops\n");
    } else {
        assert!(
            output.starts_with("\u{1b}[") && output.contains("oops"),
            "{output:?}"
        );
    }
}

#[test]
fn the_spy_file_records_commands() {
    let (session, _console) = session();
    let path = std::env::temp_dir().join(format!("tkdraw_spy_{}.tcl", std::process::id()));
    let path_text = path.to_string_lossy().replace('\\', "/");
    interprete(&session, &format!("spy {path_text}"));
    interprete(&session, "set y 1");
    interprete(&session, "error bad");
    interprete(&session, "spy");
    let recorded = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(recorded, "set y 1\n# error bad\n");
}
