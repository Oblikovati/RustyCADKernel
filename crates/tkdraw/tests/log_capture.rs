//! The command log captures the process's standard streams at descriptor level. This
//! file holds a single test: the capture redirects descriptor 1 of the whole process, so
//! another test (or the harness reporting one) printing meanwhile would end up in the
//! log; every log scenario therefore runs inside this one test.
use rustycad_tkdraw::draw::{DrawInterpretor, ProcessConsole, USER_COMMANDS};
use std::io::Write;
use std::rc::Rc;

#[test]
fn the_log_captures_the_process_streams() {
    let di = DrawInterpretor::new(Rc::new(ProcessConsole));
    di.init();
    let say = |di: &DrawInterpretor, a: &[&str]| {
        di.append(a[1])?;
        // Written to the descriptor, past the test harness's output capture.
        std::io::stdout()
            .write_all(b"printed by the command\n")
            .unwrap();
        Ok(0)
    };
    di.add("say", "", Box::new(say), USER_COMMANDS).unwrap();
    di.set_do_log(true);
    di.eval("say logged").unwrap();
    di.set_do_log(false);
    assert_eq!(
        di.get_log(),
        "say logged \nprinted by the command\nlogged\n"
    );
    log_lifecycle(&di);
}

/// AddLog, ResetLog and the descriptor of an existing log.
fn log_lifecycle(di: &DrawInterpretor) {
    assert!(!di.get_do_log());
    di.add_log("start\n");
    assert!(di.get_log().ends_with("logged\nstart\n"));
    di.reset_log();
    assert_eq!(di.get_log(), "");
    di.set_do_log(true);
    assert!(di.get_do_log());
    di.set_do_log(false);
    assert!(di.log_file_descriptor().is_some_and(|fd| fd > 2));
}
