//! Unit tests of the model copier over the fake chain (a, b, c, d: d shares c, c shares
//! a and b) and the fake work library.
use super::model_copier_send::remaining_report;
use super::test_work_library::FakeWorkLibrary;
use super::*;
use crate::interface::test_norm::{fake_chain, fake_model, fake_protocol};
use crate::interface::{
    InterfaceCheckIterator, InterfaceCopyTool, InterfaceEntityIterator, InterfaceGraph,
    InterfaceModelEntities,
};
use rustycad_tkernel::{
    standard::Handle,
    tcollection::{AsciiString, HAsciiString},
};

fn name(s: &str) -> AsciiString {
    AsciiString::new(s)
}

fn library() -> (Handle<FakeWorkLibrary>, Handle<dyn IFSelectWorkLibrary>) {
    let fake = Handle::new(FakeWorkLibrary::default());
    let wl = fake.clone().cast();
    (fake, wl)
}

/// A share-out of one global dispatch on all the entities.
fn share_out() -> Handle<IFSelectShareOut> {
    let disp = IFSelectDispGlobal::new();
    disp.set_final_selection(&Handle::new(IFSelectSelectModelEntities::new()).cast());
    let share = Handle::new(IFSelectShareOut::new());
    share.add_dispatch(&Handle::new(disp).cast());
    share
}

fn fail_texts(checks: &InterfaceCheckIterator) -> Vec<Vec<u8>> {
    let check = checks.check(0);
    (1..=check.nb_fails())
        .map(|i| check.c_fail(i, true).unwrap().as_bytes().to_vec())
        .collect()
}

#[test]
fn model_copier_files() {
    let copier = IFSelectModelCopier::new();
    let model = fake_model();
    assert!(copier.add_file(&name("f1"), &model) && !copier.add_file(&name("f1"), &model));
    assert!(copier.add_file(&name(""), &model) && copier.add_file(&name(""), &model));
    assert_eq!(copier.nb_files(), 3, "an empty name is never a duplicate");
    assert!(!copier.name_file(2, &name("f1")) && copier.name_file(2, &name("f2")));
    assert!(!copier.name_file(4, &name("f4")) && copier.clear_file(1) && !copier.clear_file(0));
    assert!(copier.file_name(1).unwrap().is_empty() && copier.file_name(2).unwrap() == name("f2"));
    assert!(copier.file_name(4).is_err() && copier.file_model(1).unwrap() == model);
    let applied = Handle::new(IFSelectAppliedModifiers::new(1, 4).unwrap());
    assert!(
        copier.set_applied_modifiers(3, &applied) && !copier.set_applied_modifiers(4, &applied)
    );
    assert!(copier.applied_modifiers(3).unwrap() == applied && copier.clear_applied_modifiers(3));
    assert!(copier.applied_modifiers(3).unwrap().is_null() && copier.applied_modifiers(0).is_err());
    copier.clear_result();
    assert_eq!(copier.nb_files(), 0);
}

#[test]
fn model_copier_sent_files() {
    let copier = IFSelectModelCopier::new();
    copier.add_sent_file("lost");
    assert!(copier.sent_files().is_none());
    let share = share_out();
    share.set_last_run(1);
    share
        .set_root_name(1, &Handle::new(HAsciiString::new("part")))
        .unwrap();
    copier.begin_sent_files(&share, true);
    assert!(
        share.last_run() == 1 && share.has_root_name(1),
        "only the numbers are cleared"
    );
    copier.add_sent_file("one");
    let files = copier.sent_files().unwrap();
    assert!(files.len() == 1 && files[0].string().as_bytes() == b"one");
    copier.begin_sent_files(&Handle::default(), false);
    assert!(copier.sent_files().is_none());
}

#[test]
fn model_copier_copy_then_send() {
    let (model, _) = fake_chain();
    let (fake, wl) = library();
    let copier = IFSelectModelCopier::new();
    let share = share_out();
    let mut eval = IFSelectShareOutResult::with_share_out(&share, &model).unwrap();
    let checks = copier.copy(&mut eval, &wl, &fake_protocol()).unwrap();
    assert_eq!(
        checks.name().as_bytes(),
        b"X-STEP WorkSession : Split Copy (no Write)"
    );
    assert!(copier.nb_files() == 1 && copier.file_name(1).unwrap() == name("Default_1"));
    assert!(share.last_run() == 1 && copier.applied_modifiers(1).unwrap().is_null());
    copier.begin_sent_files(&Handle::default(), true);
    let checks = copier.send_copied(&wl, &fake_protocol()).unwrap();
    assert_eq!(
        checks.name().as_bytes(),
        b"X-STEP WorkSession : Split Send (Copy+Write)"
    );
    assert_eq!(*fake.written.read(), vec![String::from("Default_1")]);
    assert!(copier.nb_files() == 0 && copier.sent_files().unwrap().len() == 1);
    copier.add_file(&name("fail.stp"), &model);
    let checks = copier.send_copied(&wl, &fake_protocol()).unwrap();
    assert!(
        checks.name().is_empty(),
        "native quirk: no name on a failure"
    );
    assert_eq!(
        fail_texts(&checks),
        vec![b"Split Send (WriteFile) abandon on file n0.1".to_vec()]
    );
    assert_eq!(copier.nb_files(), 1, "kept after a failure");
}

#[test]
fn model_copier_send() {
    let (model, _) = fake_chain();
    let (fake, wl) = library();
    let copier = IFSelectModelCopier::new();
    let share = share_out();
    let mut eval = IFSelectShareOutResult::with_share_out(&share, &model).unwrap();
    let checks = copier.send(&mut eval, &wl, &fake_protocol()).unwrap();
    assert_eq!(
        checks.name().as_bytes(),
        b"X-STEP WorkSession : Split Send (only Write)"
    );
    assert!(fake.written.read().len() == 1 && share.last_run() == 1);
    share.set_last_run(0);
    share
        .set_root_name(1, &Handle::new(HAsciiString::new("fail")))
        .unwrap();
    let mut eval = IFSelectShareOutResult::with_share_out(&share, &model).unwrap();
    let checks = copier.send(&mut eval, &wl, &fake_protocol()).unwrap();
    assert!(
        !checks.is_empty(true) && share.last_run() == 0,
        "abandoned before the end"
    );
    let mut by_dispatch =
        IFSelectShareOutResult::with_dispatch(&share.dispatch(1).unwrap(), &model).unwrap();
    assert!(
        copier
            .send(&mut by_dispatch, &wl, &fake_protocol())
            .is_err(),
        "no share-out"
    );
}

#[test]
fn model_copier_send_all() {
    let (model, _) = fake_chain();
    let g = InterfaceGraph::new(&model, true).unwrap();
    let (fake, wl) = library();
    let copier = IFSelectModelCopier::new();
    copier.set_share_out(&share_out());
    let reorder: Handle<dyn IFSelectGeneralModifier> =
        Handle::new(IFSelectModifReorder::new(false)).cast();
    copier
        .state
        .read()
        .theshareout
        .add_modifier(&reorder, 0)
        .unwrap();
    let checks = copier
        .send_all("all.stp", &g, &wl, &fake_protocol())
        .unwrap();
    assert!(checks.is_empty(false) && checks.name().as_bytes() == b"X-STEP WorkSession : Send All");
    assert_eq!(*fake.written.read(), vec![String::from("all.stp")]);
    let mut out = Vec::new();
    let checks = copier
        .send_all_stream(&mut out, "s", &g, &wl, &fake_protocol())
        .unwrap();
    assert_eq!(
        fail_texts(&checks),
        vec![b"SendAll (WriteStream) has failed".to_vec()]
    );
    let checks = copier
        .send_all("x", &g, &Handle::default(), &fake_protocol())
        .unwrap();
    assert!(checks.is_empty(false), "nothing without work library");
}

#[test]
fn model_copier_send_selected_and_remaining() {
    let (model, ents) = fake_chain();
    let g = InterfaceGraph::new(&model, true).unwrap();
    let (fake, wl) = library();
    let copier = IFSelectModelCopier::new();
    let mut list = InterfaceEntityIterator::new();
    list.add_item(&ents[2]);
    let checks = copier
        .send_selected("sel", &g, &wl, &fake_protocol(), &list)
        .unwrap();
    assert!(checks.is_empty(false) && fake.written.read().len() == 1);
    let mut cg = InterfaceGraph::new(&model, true).unwrap();
    assert!(copier.set_remaining(&mut cg).unwrap());
    let statuses: Vec<i32> = (1..=4).map(|i| cg.status(i).unwrap()).collect();
    assert_eq!(statuses, vec![1, 1, 1, 0], "a, b and c were sent");
    cg.set_status(4, 0).unwrap();
    cg.set_status(1, 0).unwrap();
    let mut tc = InterfaceCopyTool::with_protocol(&model, &fake_protocol()).unwrap();
    let remaining = copier.copied_remaining(&cg, &wl, &mut tc).unwrap();
    assert_eq!(remaining.nb_entities(), 4, "a and d, d bringing c and b");
    assert!(
        !copier.set_remaining(&mut cg).unwrap(),
        "native quirk: bound size + 1"
    );
    let empty = IFSelectModelCopier::new();
    assert!(
        empty
            .set_remaining(&mut InterfaceGraph::new(&fake_model(), true).unwrap())
            .unwrap()
    );
}

#[test]
fn copied_remaining_reports() {
    assert_eq!(
        remaining_report(2, &[0, 1, -1, 1], 3),
        " Remaining Model : 2 Entities\n  -- Remaining data complete\n"
    );
    assert_eq!(
        remaining_report(1, &[0, 0, 1, 0, 0], 3),
        " Remaining Model : 1 Entities\n Refractory :  1 3  -- 2 Entities\n"
    );
}
