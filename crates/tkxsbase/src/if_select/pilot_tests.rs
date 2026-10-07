//! Unit tests of the activators, the acts and the session pilot. The pilot commands
//! are process-wide (the first pilot is their activator), so the tests share one pilot
//! under a lock.
use super::*;
use crate::interface::test_norm::{fake_chain, fake_protocol};
use rustycad_tkernel::{
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use std::io::Cursor;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

/// A new session loaded with the fake chain, with the fake work library.
fn fresh_session() -> Handle<IFSelectWorkSession> {
    let session = Handle::new(IFSelectWorkSession::new());
    session.set_protocol(&fake_protocol());
    session.set_library(&super::test_work_library::fake_work_library());
    session.set_model(&fake_chain().0, true).unwrap();
    session
}

/// The shared pilot (the activator of the basic commands) with a new loaded session.
pub(super) fn shared_pilot() -> (MutexGuard<'static, ()>, Handle<IFSelectSessionPilot>) {
    static LOCK: Mutex<()> = Mutex::new(());
    static PILOT: OnceLock<Handle<IFSelectSessionPilot>> = OnceLock::new();
    let guard = LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let pilot = PILOT
        .get_or_init(|| {
            let pilot = IFSelectSessionPilot::new("").unwrap();
            IFSelectAct::add_func(
                "test.pilot.param",
                "records an integer parameter",
                record_param,
            )
            .unwrap();
            IFSelectAct::add_func("test.pilot.fail", "fails", |_| {
                Ok(IFSelectReturnStatus::RetFail)
            })
            .unwrap();
            pilot
        })
        .clone();
    pilot.set_session(&fresh_session());
    pilot.clear();
    pilot.set_record_mode(false);
    (guard, pilot)
}

/// An act function recording a new integer parameter.
fn record_param(pilot: &Handle<IFSelectSessionPilot>) -> Result<IFSelectReturnStatus, Failure> {
    Ok(pilot.record_item(&Handle::new(IFSelectIntParam::new()).cast()))
}

/// The count of entities of a counter (by signature, and the nulls).
fn counted(counter: &IFSelectSignCounter) -> i32 {
    let signs = counter.list("");
    let by_sign: i32 = signs
        .iter()
        .map(|s| counter.nb_times(&String::from_utf8_lossy(s.string().as_bytes())))
        .sum();
    by_sign + counter.nb_nulls()
}

fn line(s: &str) -> AsciiString {
    AsciiString::new(s)
}

#[test]
fn pilot_command_lines() {
    let (_guard, pilot) = shared_pilot();
    pilot.set_command_line(&line("xstep  foo bar ")).unwrap();
    assert_eq!(pilot.nb_words(), 3);
    assert_eq!(
        (pilot.arg(0).unwrap(), pilot.arg(2).unwrap()),
        (String::from("xstep"), String::from("bar"))
    );
    assert!(pilot.word(5).unwrap().is_empty() && pilot.word(-1).is_err());
    assert_eq!(
        pilot.command_line().as_bytes(),
        b"xstep  foo bar",
        "the last separator removed"
    );
    assert_eq!(pilot.command_part(1).as_bytes(), b"foo bar");
    assert!(pilot.command_part(3).is_empty() && pilot.command_part(0).len() == 14);
    assert!(pilot.remove_word(0).unwrap());
    assert_eq!(
        pilot.command_line().as_bytes(),
        b"oo bar",
        "native defect: the stale start of the word 1"
    );
    assert!(pilot.remove_word(1).unwrap() && pilot.command_line().as_bytes() == b"foo");
    assert!(!pilot.remove_word(5).unwrap());
    pilot.set_command_line(&line("a\u{e9}b")).unwrap();
    assert_eq!(
        pilot.nb_words(),
        2,
        "the bytes from 0x80 separate, as negative chars"
    );
    assert!(pilot.set_command_line(&line("")).is_err());
    let many = vec!["w"; 201].join(" ");
    assert!(
        pilot.set_command_line(&line(&many)).is_err(),
        "more than 200 words"
    );
}

#[test]
fn pilot_executes_commands() {
    let (_guard, pilot) = shared_pilot();
    assert_eq!(
        pilot.execute(&line("# comment")).unwrap(),
        IFSelectReturnStatus::RetVoid
    );
    assert_eq!(
        pilot.execute(&line("test.pilot.none")).unwrap(),
        IFSelectReturnStatus::RetError
    );
    assert_eq!(
        pilot.execute(&line("exit")).unwrap(),
        IFSelectReturnStatus::RetStop
    );
    pilot.set_record_mode(true);
    let before = pilot.session().max_ident();
    assert_eq!(
        pilot.execute(&line("test.pilot.param")).unwrap(),
        IFSelectReturnStatus::RetDone
    );
    assert_eq!(
        pilot.session().max_ident(),
        before + 1,
        "the recorded item joins the session"
    );
    assert_eq!(
        pilot.execute(&line("xstep test.pilot.param")).unwrap(),
        IFSelectReturnStatus::RetDone
    );
    assert_eq!(
        pilot.execute(&line("test.pilot.fail")).unwrap(),
        IFSelectReturnStatus::RetFail
    );
    assert_eq!(
        pilot.nb_commands(),
        3,
        "xstep records its inner command too, the failure is not recorded"
    );
    assert_eq!(pilot.command(1).unwrap().as_bytes(), b"test.pilot.param");
    assert_eq!(
        pilot.command(2).unwrap().as_bytes(),
        b"est.pilot.param",
        "native defect: RemoveWord(0) cuts at the start of the word 1"
    );
    assert!(pilot.command(4).is_err());
    assert_eq!(
        pilot.execute(&line("xnew mypar test.pilot.param")).unwrap(),
        IFSelectReturnStatus::RetDone
    );
    assert!(pilot.session().named_item("mypar").is_some());
    assert_eq!(
        pilot.execute(&line("xnew other test.pilot.fail")).unwrap(),
        IFSelectReturnStatus::RetFail
    );
    assert_eq!(
        pilot.execute(&line("xnew short")).unwrap(),
        IFSelectReturnStatus::RetVoid
    );
    assert_eq!(
        pilot.execute_alias(&line("test.pilot.fail")).unwrap(),
        IFSelectReturnStatus::RetFail
    );
    assert!(
        pilot.recorded_item().is_null()
            && pilot.record_item(&Handle::default()) == IFSelectReturnStatus::RetFail
    );
    assert_eq!(pilot.number("2").unwrap(), 2);
}

#[test]
fn pilot_help_and_control() {
    let (_guard, pilot) = shared_pilot();
    for command in [
        "?",
        "xhelp *",
        "xhelp test.pilot",
        "xhelp zzz",
        "xstep",
        "xcommand",
        "xcommand l",
        "xcommand r",
        "xcommand a b c",
        "xcommand q",
        "xcommand c",
    ] {
        assert_eq!(
            pilot.execute(&line(command)).unwrap(),
            IFSelectReturnStatus::RetVoid,
            "{command}"
        );
    }
    assert!(pilot.record_mode(), "toggled by xcommand r");
    assert_eq!(
        pilot.execute(&line("xcommand f")).unwrap(),
        IFSelectReturnStatus::RetError
    );
    assert_eq!(
        pilot.execute(&line("xsource")).unwrap(),
        IFSelectReturnStatus::RetError
    );
    assert_eq!(
        pilot.execute(&line("xsource /nonexistent/script")).unwrap(),
        IFSelectReturnStatus::RetFail
    );
    assert_eq!(pilot.help(4).as_bytes(), b"reads commands from a file");
    assert!(pilot.help(9).is_empty());
    assert_eq!(
        pilot.do_command(9, &pilot).unwrap(),
        IFSelectReturnStatus::RetError
    );
}

#[test]
fn pilot_scripts() {
    let (_guard, pilot) = shared_pilot();
    let mut script = Cursor::new(b"test.pilot.param\nexit\ntest.pilot.param\n".to_vec());
    assert_eq!(
        pilot.run_script(&mut script, Some("s")).unwrap(),
        IFSelectReturnStatus::RetStop
    );
    let mut failing = Cursor::new(b"test.pilot.fail\ntest.pilot.param\n".to_vec());
    assert_eq!(
        pilot.run_script(&mut failing, Some("s")).unwrap(),
        IFSelectReturnStatus::RetFail
    );
    let mut unterminated = Cursor::new(b"test.pilot.fail".to_vec());
    assert_eq!(
        pilot.run_script(&mut unterminated, Some("s")).unwrap(),
        IFSelectReturnStatus::RetVoid,
        "native: a last line without end of line is not read"
    );
    let path = std::env::temp_dir().join(format!("tkxsbase-pilot-{}.xs", std::process::id()));
    std::fs::write(&path, b"test.pilot.param\n").unwrap();
    let name = path.to_string_lossy().into_owned();
    assert_eq!(
        pilot.read_script(&name).unwrap(),
        IFSelectReturnStatus::RetVoid
    );
    pilot.set_record_mode(true);
    pilot.execute(&line("test.pilot.param")).unwrap();
    pilot.execute(&line(&format!("xcommand f {name}"))).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"test.pilot.param\n");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn pilot_counters_and_libraries() {
    let (_guard, pilot) = shared_pilot();
    let counter = Handle::new(IFSelectSignCounter::new(true, true));
    pilot.set_command_line(&line("count")).unwrap();
    assert_eq!(
        pilot
            .execute_counter(&counter, 1, IFSelectPrintCount::CountByItem)
            .unwrap(),
        IFSelectReturnStatus::RetVoid
    );
    assert_eq!(counted(&counter), 4);
    pilot.set_command_line(&line("count (1,2)")).unwrap();
    assert_eq!(
        pilot
            .execute_counter(&counter, 1, IFSelectPrintCount::CountByItem)
            .unwrap(),
        IFSelectReturnStatus::RetVoid
    );
    assert_eq!(counted(&counter), 2);
    assert_eq!(
        pilot
            .execute_counter(&Handle::default(), 1, IFSelectPrintCount::CountByItem)
            .unwrap(),
        IFSelectReturnStatus::RetError
    );
    pilot.set_library(&super::test_work_library::fake_work_library());
    assert!(pilot.library().unwrap().is_some());
    let lonely = IFSelectSessionPilot::new("p>").unwrap();
    assert!(lonely.library().is_err() && lonely.number("1").is_err());
}

#[test]
fn activators_and_acts() {
    let (_guard, _pilot) = shared_pilot();
    let act = IFSelectAct::new("test.act.none", "nothing", None);
    assert_eq!(act.group().as_bytes(), b"XSTEP");
    act.add(1, "test.act.none").unwrap();
    act.add(1, "test.act.none").unwrap();
    let (num, found) = IFSelectActivatorBase::select("test.act.none").unwrap();
    assert!(num == 1 && found.help(1).as_bytes() == b"nothing");
    assert_eq!(IFSelectActivatorBase::mode("test.act.none"), 0);
    IFSelectAct::set_group("TestGroup", "tg.txt");
    IFSelectAct::add_f_set("test.act.set", "set", |_| Ok(IFSelectReturnStatus::RetVoid)).unwrap();
    IFSelectAct::set_group("", "");
    let (_, set) = IFSelectActivatorBase::select("test.act.set").unwrap();
    assert!(set.group().as_bytes() == b"TestGroup" && set.file().as_bytes() == b"tg.txt");
    assert_eq!(IFSelectActivatorBase::mode("test.act.set"), 1);
    assert!(
        IFSelectActivatorBase::commands(1, "test.act")
            .iter()
            .any(|c| c.as_bytes() == b"test.act.set")
    );
    assert!(
        IFSelectActivatorBase::commands(-1, "test.act").is_empty(),
        "native quirk: the group must be the prefix"
    );
    assert!(IFSelectActivatorBase::commands(-1, "").len() >= 2);
    IFSelectActivatorBase::remove("test.act.none");
    assert!(
        IFSelectActivatorBase::select("test.act.none").is_none()
            && IFSelectActivatorBase::mode("test.act.none") == -1
    );
}
