//! Unit tests of `XSControl_WorkSession`.
use super::test_norm::fake_controller;
use super::transfer_tests::reading_session;
use super::*;
use crate::if_select::IFSelectWorkSessionOps;
use crate::interface::InterfaceModelEntities;
use crate::interface::test_norm::{fake_entity, fake_model};
use crate::transfer::{TransferFinderProcess, TransferTransientProcess};
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, StandardTransient},
    tcollection::AsciiString,
};

#[test]
fn select_norm_takes_a_recorded_controller() {
    let ws = XSControlWorkSession::new();
    assert!(!ws.select_norm("no-such-norm").unwrap());
    let ctl = fake_controller("FAKE-NORM", "fnm");
    ctl.auto_record().unwrap();
    assert!(ws.select_norm("fnm").unwrap());
    assert!(ws.norm_adaptor() == ctl);
    assert_eq!(ws.selected_norm(false).as_bytes(), b"FAKE-NORM");
    assert_eq!(ws.selected_norm(true).as_bytes(), b"fnm");
    assert!(ws.select_norm("FAKE-NORM").unwrap());
}

#[test]
fn set_controller_customises_the_session() {
    let ws = XSControlWorkSession::new();
    assert!(ws.set_controller(&Handle::default()).is_err());
    ws.set_controller(&fake_controller("FAKE-CUSTOM", "fcu"))
        .unwrap();
    for name in [
        "xst-model-all",
        "xst-model-roots",
        "xst-transferrable-roots",
        "xst-transfer-status",
        "xst-static-params",
    ] {
        assert!(ws.named_item(name).is_some(), "{name} is added");
    }
    assert!(ws.work_library().is_some() && ws.protocol().is_some());
    assert!(ws.transfer_writer().controller() == ws.norm_adaptor());
}

#[test]
fn context_of_the_session_and_of_the_reader() {
    let ws = XSControlWorkSession::new();
    let mut ctx = XSControlContext::default();
    ctx.bind(AsciiString::new("unit"), fake_entity("mm", &[]));
    ws.set_all_context(&ctx).unwrap();
    assert_eq!(ws.context().extent(), 1);
    assert!(ws.transfer_reader().get_context("unit", None).is_some());
    ws.clear_context().unwrap();
    assert_eq!(ws.context().extent(), 0);
    assert!(ws.transfer_reader().get_context("unit", None).is_none());
}

#[test]
fn results_by_mode() {
    let (ws, ents) = reading_session();
    ws.transfer_read_roots(&ProgressRange::default()).unwrap();
    assert!(ws.result(&ents[2], 0).unwrap() == ents[2]);
    assert!(ws.result(&ents[2], 1).unwrap() == ents[2]);
    assert!(ws.result(&ents[2], 10).unwrap().is_some());
    assert!(ws.result(&ents[2], 20).unwrap().is_some());
    assert!(ws.result(&ents[3], 0).unwrap().is_null());
    assert!(
        ws.result(&ents[3], 11).unwrap().is_some(),
        "the failed binder of the process"
    );
}

#[test]
fn transfer_read_one_on_the_model_reads_the_roots() {
    let (ws, ents) = reading_session();
    let model: Handle<StandardTransient> = ws.model().cast();
    assert_eq!(
        ws.transfer_read_one(&model, &ProgressRange::default())
            .unwrap(),
        2
    );
    assert!(ws.transfer_reader().is_recorded(&ents[2]).unwrap());
    assert!(
        ws.transfer_read_one(&Handle::default(), &ProgressRange::default())
            .is_err()
    );
}

#[test]
fn map_reader_takes_the_graph_of_the_session() {
    let (ws, _) = reading_session();
    ws.init_transfer_reader(0).unwrap();
    assert!(!ws.set_map_reader(&Handle::default()).unwrap());
    // Native quirk kept: the graph of the session replaces the model of the process
    // before the models are compared, so a process of another model is accepted.
    let other = TransferTransientProcess::new(10);
    other.set_model(&fake_model());
    assert!(ws.set_map_reader(&other).unwrap());
    assert!(other.model() == ws.model());
    let own = TransferTransientProcess::new(10);
    assert!(ws.set_map_reader(&own).unwrap());
    assert!(ws.map_reader().unwrap() == own);
}

#[test]
fn map_writer_and_vars() {
    let ws = XSControlWorkSession::new();
    assert!(!ws.set_map_writer(&Handle::default()).unwrap());
    let fp = TransferFinderProcess::new(5);
    assert!(ws.set_map_writer(&fp).unwrap());
    assert!(ws.transfer_writer().finder_process() == fp);
    let vars: Handle<dyn XSControlVarsOps> = Handle::new(XSControlVars::new()).cast();
    ws.set_vars(&vars);
    assert!(ws.vars() == vars);
}

#[test]
fn init_transfer_reader_resets_the_reader_first() {
    let (ws, ents) = reading_session();
    ws.transfer_read_roots(&ProgressRange::default()).unwrap();
    let tr = ws.transfer_reader();
    let tp = tr.transient_process();
    tr.clear(1);
    assert!(!tr.is_recorded(&ents[0]).unwrap());
    // Native quirk kept: SetTransferReader gives the controller again, which clears the
    // reader, so the mode 3 records the roots of a new empty process.
    ws.init_transfer_reader(3).unwrap();
    assert!(!tr.is_recorded(&ents[0]).unwrap());
    assert!(tr.transient_process().is_some() && tr.transient_process() != tp);
    ws.init_transfer_reader(1).unwrap();
    assert!(tr.transient_process().is_null());
    ws.init_transfer_reader(2).unwrap();
    assert!(tr.transient_process().is_some());
}

#[test]
fn clear_data_of_the_transfers() {
    let (ws, ents) = reading_session();
    ws.transfer_read_roots(&ProgressRange::default()).unwrap();
    let tr = ws.transfer_reader();
    ws.clear_data(6).unwrap();
    assert!(!tr.is_recorded(&ents[0]).unwrap() && tr.transient_process().is_some());
    ws.clear_data(5).unwrap();
    assert!(tr.transient_process().is_null());
    assert!(tr.model() == ws.model(), "the reader takes the graph again");
}

#[test]
fn print_transfer_status_of_a_read_item() {
    let (ws, _) = reading_session();
    let mut out = Vec::new();
    assert!(
        !ws.print_transfer_status(1, false, &mut out).unwrap(),
        "no process yet"
    );
    ws.transfer_read_roots(&ProgressRange::default()).unwrap();
    assert!(!ws.print_transfer_status(0, false, &mut out).unwrap());
    assert!(!ws.print_transfer_status(99, false, &mut out).unwrap());
    assert!(ws.print_transfer_status(-1, false, &mut out).unwrap());
    let text = String::from_utf8_lossy(&out).into_owned();
    assert!(
        text.starts_with(
            "Transfer Read item n0.1 of 4  ** Transfer Root n0.1\n In Model, Entity 1:"
        ),
        "{text}"
    );
    out.clear();
    assert!(ws.print_transfer_status(4, false, &mut out).unwrap());
    let text = String::from_utf8_lossy(&out).into_owned();
    // Native quirk kept: the root index of an item is looked up for a null finder.
    assert!(text.starts_with("Transfer Read item n0.4 of 4\n"), "{text}");
    assert!(text.contains(" - Fails : 1 :"), "{text}");
}

#[test]
fn new_model_needs_a_controller() {
    let ws = XSControlWorkSession::new();
    assert!(ws.new_model().unwrap().is_null());
    ws.set_controller(&fake_controller("FAKE-NEW", "fnw"))
        .unwrap();
    let model = ws.new_model().unwrap();
    assert!(model.is_some() && model.nb_entities() == 0);
}
