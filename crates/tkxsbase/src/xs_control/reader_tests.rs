//! Unit tests of `XSControl_Reader`.
use super::test_norm::fake_controller;
use super::transfer_tests::reading_session;
use super::*;
use crate::if_select::{IFSelectPrintCount, IFSelectReturnStatus};
use crate::interface::InterfaceModelEntities;
use rustycad_tkernel::{message::ProgressRange, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// A reader on the session of the fake norm, keeping its model of four roots.
fn fake_reader() -> XSControlReader {
    let (ws, _) = reading_session();
    XSControlReader::with_session(&ws, false).unwrap()
}

#[test]
fn norms_by_name() {
    let reader = XSControlReader::with_norm("no-such-norm").unwrap();
    assert!(reader.ws().is_some() && reader.ws().norm_adaptor().is_null());
    fake_controller("FAKE-READER", "frr").auto_record().unwrap();
    let mut reader = XSControlReader::with_norm("frr").unwrap();
    assert_eq!(reader.ws().selected_norm(true).as_bytes(), b"frr");
    assert!(reader.set_norm("FAKE-READER").unwrap());
    assert!(XSControlReader::with_session(&Default::default(), true).is_err());
}

#[test]
fn set_ws_with_scratch_takes_a_new_model() {
    let (ws, _) = reading_session();
    let reader = XSControlReader::with_session(&ws, true).unwrap();
    assert_eq!(reader.model().unwrap().nb_entities(), 0);
}

#[test]
fn roots_for_transfer_are_the_recognized_roots() {
    let mut reader = fake_reader();
    assert_eq!(reader.nb_roots_for_transfer().unwrap(), 4);
    assert!(reader.root_for_transfer(1).unwrap().is_some());
    assert!(reader.root_for_transfer(5).unwrap().is_null());
    assert!(reader.root_for_transfer(0).unwrap().is_null());
    assert_eq!(reader.give_list("", "").unwrap().map(|l| l.len()), Some(4));
    assert_eq!(
        reader
            .give_list("xst-model-all", "")
            .unwrap()
            .map(|l| l.len()),
        Some(4)
    );
}

#[test]
fn transfer_roots_collects_the_shapes() {
    let mut reader = fake_reader();
    assert_eq!(reader.transfer_roots(&ProgressRange::default()).unwrap(), 3);
    assert_eq!(reader.nb_shapes(), 3);
    assert!(
        reader.shape(3).unwrap().is_null(),
        "the transient result has no shape"
    );
    assert!(reader.shape(4).is_err() && reader.shape(0).is_err());
    let one = reader.one_shape();
    assert_eq!(one.shape_type().unwrap(), ShapeEnum::Compound);
    let stats = reader.get_stats_transfer(None).unwrap();
    assert_eq!(
        stats,
        XSControlTransferStats {
            nb_mapped: 4,
            nb_with_result: 3,
            nb_with_fail: 1
        }
    );
    assert!(
        !reader.shape_fix_parameters().is_empty(),
        "the missing parameters are set"
    );
    assert!(reader.shape_process_flags().1);
}

#[test]
fn transfer_one_entity_or_root() {
    let mut reader = fake_reader();
    assert!(
        reader
            .transfer_one_root(1, &ProgressRange::default())
            .unwrap()
    );
    assert!(
        !reader
            .transfer_one_root(9, &ProgressRange::default())
            .unwrap()
    );
    assert!(reader.transfer_one(2, &ProgressRange::default()).unwrap());
    assert!(
        !reader.transfer_one(4, &ProgressRange::default()).unwrap(),
        "the failing entity"
    );
    assert_eq!(reader.nb_shapes(), 2);
    reader.clear_shapes();
    assert!(reader.one_shape().is_null());
}

#[test]
fn transfer_list_counts_the_results() {
    let mut reader = fake_reader();
    let list = reader.give_list("", "").unwrap().unwrap();
    assert_eq!(
        reader
            .transfer_list(Some(&list[..2]), &ProgressRange::default())
            .unwrap(),
        2
    );
    assert_eq!(
        reader
            .transfer_list(None, &ProgressRange::default())
            .unwrap(),
        0
    );
    assert_eq!(reader.nb_shapes(), 2);
    let stats = reader.get_stats_transfer(Some(&list[..1])).unwrap();
    assert_eq!(stats.nb_mapped, 1);
}

#[test]
fn reports_of_the_load_and_the_transfer() {
    let mut reader = fake_reader();
    reader.transfer_roots(&ProgressRange::default()).unwrap();
    let mut out = Vec::new();
    reader
        .print_check_transfer_to(&mut out, true, IFSelectPrintCount::CountByItem)
        .unwrap();
    reader.print_stats_transfer_to(&mut out, 1, 0).unwrap();
    reader
        .print_check_load_to(&mut out, false, IFSelectPrintCount::ItemsByEntity)
        .unwrap();
    assert!(String::from_utf8_lossy(&out).contains("Statistics on Transfer (Read)"));
    reader.print_stats_transfer(1, 0).unwrap();
}

#[test]
fn read_file_by_the_library_of_the_norm() {
    let mut reader = fake_reader();
    assert_eq!(
        reader.read_file("model.fake").unwrap(),
        IFSelectReturnStatus::RetDone
    );
    assert_eq!(reader.model().unwrap().nb_entities(), 0);
    let mut stream: &[u8] = b"";
    assert!(reader.read_stream("stream.fake", &mut stream).is_ok());
}

/// A norm reader that keeps the first root only (its own `NbRootsForTransfer`).
struct FirstRootReader {
    base: XSControlReader,
}

impl XSControlReaderOps for FirstRootReader {
    fn xs_reader(&self) -> &XSControlReader {
        &self.base
    }
    fn xs_reader_mut(&mut self) -> &mut XSControlReader {
        &mut self.base
    }
    fn nb_roots_for_transfer(&mut self) -> Result<i32, Failure> {
        let base = &mut self.base;
        if !base.therootsta {
            base.therootsta = true;
            base.theroots = base
                .give_list("xst-model-roots", "")?
                .unwrap_or_default()
                .into_iter()
                .take(1)
                .collect();
        }
        Ok(base.theroots.len() as i32)
    }
}

#[test]
fn transfer_roots_calls_the_overridden_count() {
    let mut reader = FirstRootReader {
        base: fake_reader(),
    };
    assert_eq!(reader.transfer_roots(&ProgressRange::default()).unwrap(), 1);
    assert_eq!(reader.xs_reader().nb_shapes(), 1);
}
