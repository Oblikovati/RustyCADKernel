//! Unit tests of `XSControl_Functions`: the commands run by a pilot on a session of
//! the fake norm.
use super::test_norm::fake_controller;
use super::transfer_tests::reading_session;
use super::*;
use crate::if_select::IFSelectReturnStatus::{RetDone, RetError, RetFail, RetVoid};
use crate::if_select::{IFSelectReturnStatus, IFSelectSessionPilot};
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};

/// A pilot on a session of the fake norm (four roots, nothing transferred).
fn fake_pilot() -> Handle<IFSelectSessionPilot> {
    XSControlFunctions::init().unwrap();
    let pilot = IFSelectSessionPilot::new("xs>").unwrap();
    let (ws, _) = reading_session();
    pilot.set_session(&ws.cast());
    pilot
}

fn check(pilot: &Handle<IFSelectSessionPilot>, commands: &[(&str, IFSelectReturnStatus)]) {
    for (command, expected) in commands {
        let status = pilot.execute(&AsciiString::new(command)).unwrap();
        assert_eq!(status, *expected, "{command}");
    }
}

#[test]
fn norm_commands() {
    let pilot = fake_pilot();
    check(
        &pilot,
        &[
            ("xinit", RetVoid),
            ("xinit no-such-norm", RetFail),
            ("xnorm", RetVoid),
            ("xnorm no-such-norm", RetError),
            ("newmodel", RetDone),
        ],
    );
    fake_controller("FAKE-FUNCTIONS", "ffn")
        .auto_record()
        .unwrap();
    check(&pilot, &[("xnorm ffn", RetDone), ("xinit ffn", RetDone)]);
    let ws = XSControl::session(&pilot).unwrap();
    assert_eq!(ws.selected_norm(true).as_bytes(), b"ffn");
}

#[test]
fn reading_process_commands() {
    let pilot = fake_pilot();
    check(
        &pilot,
        &[
            ("tpstat", RetError),
            ("tpent 1", RetError),
            ("trbegin", RetDone),
            ("tread", RetDone),
        ],
    );
    check(
        &pilot,
        &[
            ("tpstat", RetVoid),
            ("tpstat g", RetVoid),
            ("tpstat *n", RetVoid),
            ("tpstat cx", RetVoid),
            ("tpstat x", RetError),
            ("tpstat n xst-model-all", RetVoid),
        ],
    );
    check(
        &pilot,
        &[
            ("tpent", RetError),
            ("tpent 9", RetError),
            ("tpent 1", RetVoid),
        ],
    );
    check(
        &pilot,
        &[
            ("tpitem", RetError),
            ("tpitem 1", RetVoid),
            ("tproot 1", RetVoid),
            ("twitem 1", RetVoid),
            ("twroot 1", RetVoid),
        ],
    );
    check(
        &pilot,
        &[
            ("tpclear", RetDone),
            ("twclear", RetDone),
            ("tpstat g", RetVoid),
        ],
    );
}

#[test]
fn transfer_reader_commands() {
    let pilot = fake_pilot();
    check(
        &pilot,
        &[
            ("trecord", RetError),
            ("trbegin init", RetDone),
            ("tread 1", RetDone),
            ("trecord", RetDone),
            ("trecord 1", RetDone),
            ("trecord 9", RetDone),
        ],
    );
    // As natively (a null main result dereferenced), an entity the process does not
    // know cannot be recorded.
    assert!(pilot.execute(&AsciiString::new("trecord 3")).is_err());
    check(
        &pilot,
        &[
            ("trstat", RetVoid),
            ("trstat 1", RetVoid),
            ("trstat 9", RetError),
            ("trstat 4", RetError),
        ],
    );
    check(&pilot, &[("trtp", RetVoid), ("tptr", RetDone)]);
    let ws = XSControl::session(&pilot).unwrap();
    assert!(ws.transfer_reader().transient_process().is_some());
}

#[test]
fn transfer_writer_commands() {
    let pilot = fake_pilot();
    let ws = XSControl::session(&pilot).unwrap();
    ws.norm_adaptor().set_mode_write(0, 1, true);
    check(
        &pilot,
        &[
            ("twmode", RetVoid),
            ("twmode 1", RetDone),
            ("twmode 7", RetDone),
            ("twstat", RetVoid),
        ],
    );
    assert_eq!(ws.transfer_writer().transfer_mode(), 7);
}

#[test]
fn select_transfer_records_a_selection() {
    let pilot = fake_pilot();
    let status = pilot.execute(&AsciiString::new("selecttransfer")).unwrap();
    assert_ne!(status, RetError);
    assert!(
        pilot
            .recorded_item()
            .cast::<XSControlSelectForTransfer>()
            .is_some()
    );
}

#[test]
fn commands_need_an_xscontrol_session() {
    XSControlFunctions::init().unwrap();
    let pilot = IFSelectSessionPilot::new("xs>").unwrap();
    pilot.set_session(&Handle::new(crate::if_select::IFSelectWorkSession::new()));
    assert!(pilot.execute(&AsciiString::new("newmodel")).is_err());
}
