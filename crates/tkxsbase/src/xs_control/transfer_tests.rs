//! Unit tests of `XSControl_TransferReader`, `XSControl_TransferWriter` and the
//! selections and signature of the transfer, driven by a session of the fake norm.
use super::test_norm::{fake_controller, shared_vertex, vertex_compound};
use super::*;
use crate::if_select::{
    IFSelectReturnStatus, IFSelectSelectDeduct, IFSelectSelectModelEntities, IFSelectSelection,
};
use crate::interface::test_norm::{fake_entity, fake_model};
use crate::interface::{
    InterfaceGraph, InterfaceInterfaceModel, InterfaceModelEntities, InterfaceSignType,
};
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, StandardTransient},
};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// A session of the fake norm on a model of four roots: two giving shapes ("s1",
/// "s2"), one giving itself ("p") and one failing ("fail").
pub(super) fn reading_session() -> (Handle<XSControlWorkSession>, Vec<Handle<StandardTransient>>) {
    let ws = XSControlWorkSession::new();
    ws.set_controller(&fake_controller("FAKE-READ", "frd"))
        .unwrap();
    let model = fake_model();
    let ents: Vec<_> = ["s1", "s2", "p", "fail"]
        .iter()
        .map(|n| fake_entity(n, &[]))
        .collect();
    for ent in &ents {
        model.add_entity(ent).unwrap();
    }
    ws.set_model(&model, true).unwrap();
    (ws, ents)
}

fn reader(ws: &XSControlWorkSession) -> Handle<XSControlTransferReader> {
    ws.transfer_reader()
}

#[test]
fn reader_takes_the_actor_and_graph_of_the_session() {
    let (ws, _) = reading_session();
    let tr = reader(&ws);
    assert!(tr.actor().is_some());
    assert!(tr.model() == ws.model());
    assert!(
        tr.transient_process().is_null(),
        "the process waits for InitTransferReader"
    );
    ws.init_transfer_reader(0).unwrap();
    assert!(tr.transient_process().is_some());
}

#[test]
fn transfer_roots_records_the_results_and_counts_the_shapes() {
    let (ws, ents) = reading_session();
    assert_eq!(
        ws.transfer_read_roots(&ProgressRange::default()).unwrap(),
        2
    );
    let tr = reader(&ws);
    assert!(tr.is_recorded(&ents[0]).unwrap() && tr.has_result(&ents[0]));
    assert!(!tr.shape_result(&ents[0]).unwrap().is_null());
    assert!(tr.transient_result(&ents[2]) == ents[2]);
    assert!(!tr.has_result(&ents[3]));
    assert_eq!(tr.shape_result_list(false).unwrap().len(), 2);
    assert!(
        tr.entity_from_shape_result(&tr.shape_result(&ents[1]).unwrap(), 0)
            .unwrap()
            == ents[1]
    );
}

#[test]
fn transfer_one_and_list_count_the_results() {
    let (ws, ents) = reading_session();
    let tr = reader(&ws);
    let none = ProgressRange::default();
    // Native quirk kept: TransferOne needs an actor already got (here by BeginTransfer).
    assert_eq!(tr.transfer_one(&ents[2], true, &none).unwrap(), 0);
    ws.init_transfer_reader(4).unwrap();
    assert_eq!(tr.transfer_one(&ents[2], true, &none).unwrap(), 1);
    assert!(tr.is_recorded(&ents[2]).unwrap());
    assert_eq!(
        tr.transfer_list(&ents, false, &ProgressRange::default())
            .unwrap(),
        3
    );
    assert!(!tr.is_recorded(&ents[0]).unwrap());
}

#[test]
fn check_list_reports_the_failed_root() {
    let (ws, ents) = reading_session();
    ws.transfer_read_roots(&ProgressRange::default()).unwrap();
    let tr = reader(&ws);
    let checks = tr.last_check_list();
    assert!(!checks.is_empty(true), "the failure of 'fail' is checked");
    let mut stats = Vec::new();
    tr.print_stats(&mut stats, 1, 0).unwrap();
    assert!(!stats.is_empty());
    assert!(!tr.is_marked(&fake_entity("other", &[])));
    let _ = ents;
}

#[test]
fn clear_forgets_the_results() {
    let (ws, ents) = reading_session();
    ws.transfer_read_roots(&ProgressRange::default()).unwrap();
    let tr = reader(&ws);
    tr.clear(-1);
    assert!(tr.transient_process().is_null() && !tr.has_result(&ents[0]));
    assert!(tr.model().is_null());
}

#[test]
fn transfer_status_signature_names_the_results() {
    let (ws, ents) = reading_session();
    ws.transfer_read_roots(&ProgressRange::default()).unwrap();
    let sign = XSControlSignTransferStatus::with_reader(&reader(&ws));
    let model = ws.model();
    assert!(sign.value(&ents[2], &model).contains("Fake_Entity"));
    assert!(sign.value(&ents[3], &model).starts_with("Fail"));
    assert!(sign.value(&Handle::default(), &model).is_empty());
}

#[test]
fn select_for_transfer_keeps_the_recognized_entities() {
    let (ws, ents) = reading_session();
    let model = ws.model();
    model
        .add_entity(&Handle::new(StandardTransient::default()))
        .unwrap();
    let sel = XSControlSelectForTransfer::with_reader(&reader(&ws));
    let input: Handle<dyn IFSelectSelection> =
        Handle::new(IFSelectSelectModelEntities::new()).cast();
    sel.set_input(&input);
    let g = InterfaceGraph::new(&model, true).unwrap();
    assert_eq!(sel.root_result(&g).unwrap().content().len(), ents.len());
}

#[test]
fn connected_shapes_share_a_vertex() {
    let (ws, ents) = reading_session();
    ws.transfer_read_roots(&ProgressRange::default()).unwrap();
    let tp = reader(&ws).transient_process();
    let near =
        XSControlConnectedShapes::adjacent_entities(&shared_vertex(), &tp, ShapeEnum::Compound)
            .unwrap();
    assert_eq!(near, vec![ents[0].clone(), ents[1].clone()]);
    assert!(
        XSControlConnectedShapes::adjacent_entities(&shared_vertex(), &tp, ShapeEnum::Face)
            .unwrap()
            .is_empty()
    );
    assert!(
        XSControlConnectedShapes::adjacent_entities(
            &shared_vertex(),
            &Handle::default(),
            ShapeEnum::Face
        )
        .is_err()
    );
}

#[test]
fn writer_writes_a_shape_into_the_model() {
    let ws = XSControlWorkSession::new();
    ws.set_controller(&fake_controller("FAKE-WRITE", "fwr"))
        .unwrap();
    let model = ws.new_model().unwrap();
    assert!(model.is_some() && model == ws.model());
    let status = ws
        .transfer_write_shape(&vertex_compound(), true, &ProgressRange::default())
        .unwrap();
    assert_eq!(status, IFSelectReturnStatus::RetDone);
    assert_eq!(model.nb_entities(), 1);
    assert!(ws.transfer_write_check_list().unwrap().is_empty(false));
    let tw = ws.transfer_writer();
    assert!(tw.recognize_shape(&vertex_compound()) && !tw.recognize_shape(&Default::default()));
    let mut out = Vec::new();
    assert!(ws.print_transfer_status(1, true, &mut out).unwrap());
    let text = String::from_utf8_lossy(&out);
    assert!(text.starts_with("Transfer Write item n0.1 of 1"), "{text}");
}

#[test]
fn writer_without_controller_or_model() {
    let tw = XSControlTransferWriter::new();
    let model: Handle<dyn InterfaceInterfaceModel> = fake_model();
    let shape = vertex_compound();
    let none = ProgressRange::default();
    assert_eq!(
        tw.transfer_write_shape(&model, &shape, &none).unwrap(),
        IFSelectReturnStatus::RetError
    );
    tw.set_controller(&fake_controller("FAKE-WRITE-2", "fw2"));
    assert_eq!(
        tw.transfer_write_shape(&Handle::default(), &shape, &none)
            .unwrap(),
        IFSelectReturnStatus::RetVoid
    );
    assert_eq!(
        tw.transfer_write_transient(&model, &fake_entity("t", &[]), &none)
            .unwrap(),
        IFSelectReturnStatus::RetDone
    );
    assert_eq!(model.nb_entities(), 1);
    assert!(tw.check_list().unwrap().is_empty(false));
}
