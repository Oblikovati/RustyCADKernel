//! Unit tests of the work session: selections, lists, share-out, sending and remaining
//! data, over the fake chain (a, b, c, d: d shares c, c shares a and b).
use super::session_tests::{loaded, loaded_with, text};
use super::*;
use crate::if_select::{
    IFSelectDispGlobal, IFSelectDispatch, IFSelectGeneralModifier, IFSelectModifReorder,
    IFSelectModifier, IFSelectRemainMode, IFSelectReturnStatus, IFSelectSelectModelEntities,
    IFSelectSelectModelRoots, IFSelectSelectRange, IFSelectSelectShared, IFSelectSelectUnion,
    IFSelectSelection, IFSelectSignType, IFSelectTransformStandard, IFSelectTransformer,
};
use crate::interface::InterfaceModelEntities;
use rustycad_tkernel::standard::{Handle, StandardTransient};

fn selection(
    session: &IFSelectWorkSession,
    name: &str,
    sel: Handle<dyn IFSelectSelection>,
) -> Handle<dyn IFSelectSelection> {
    session.add_named_item(name, &sel.cast(), true).unwrap();
    sel
}

/// A global dispatch on all the entities, active in the share-out.
fn global(session: &IFSelectWorkSession) -> Handle<dyn IFSelectDispatch> {
    let all = selection(
        session,
        "all",
        Handle::new(IFSelectSelectModelEntities::new()).cast(),
    );
    let disp: Handle<dyn IFSelectDispatch> = Handle::new(IFSelectDispGlobal::new()).cast();
    session.add_named_item("glob", &disp.cast(), true).unwrap();
    assert!(session.set_item_selection(&disp.cast(), &all));
    disp
}

#[test]
fn session_selections() {
    let (plain, _, _) = loaded();
    let shared = selection(
        &plain,
        "shared",
        Handle::new(IFSelectSelectShared::new()).cast(),
    );
    plain.set_input_selection(
        &shared,
        &selection(
            &plain,
            "roots",
            Handle::new(IFSelectSelectModelRoots::new()).cast(),
        ),
    );
    assert_eq!(
        plain.selection_result(&shared).unwrap().map(|l| l.len()),
        Some(0),
        "native: no statuses, nothing recorded"
    );
    let (session, ents, _) = loaded_with(true);
    let roots = selection(
        &session,
        "roots",
        Handle::new(IFSelectSelectModelRoots::new()).cast(),
    );
    assert_eq!(
        session.eval_selection(&roots).unwrap().content(),
        vec![ents[3].clone()]
    );
    assert_eq!(
        session.selection_result(&roots).unwrap().map(|l| l.len()),
        Some(1)
    );
    assert!(
        session
            .selection_result(&Handle::default())
            .unwrap()
            .is_none()
    );
    let shared = selection(
        &session,
        "shared",
        Handle::new(IFSelectSelectShared::new()).cast(),
    );
    assert!(session.set_input_selection(&shared, &roots) && session.nb_sources(&shared) == 1);
    assert!(session.source(&shared, 1).unwrap() == roots);
    assert_eq!(
        session.selection_result(&shared).unwrap().map(|l| l.len()),
        Some(1),
        "c"
    );
    let from_list = session
        .selection_result_from_list(&shared, Some(&[ents[2].clone()]))
        .unwrap();
    assert_eq!(from_list.map(|l| l.len()), Some(2), "a and b, shared by c");
    assert!(!session.is_reversed_select_extract(&roots));
    let range = selection(
        &session,
        "range",
        Handle::new(IFSelectSelectRange::new()).cast(),
    );
    assert!(session.toggle_select_extract(&range) && session.is_reversed_select_extract(&range));
    let union = selection(
        &session,
        "union",
        Handle::new(IFSelectSelectUnion::new()).cast(),
    );
    assert_eq!(
        (
            session.combine_add(&union, &roots, 0),
            session.combine_add(&union, &shared, 0)
        ),
        (1, 2)
    );
    assert!(session.nb_sources(&union) == 2 && session.source(&union, 2).unwrap() == shared);
    assert!(session.source(&union, 3).is_err());
    assert!(session.combine_remove(&union, &roots).unwrap() && session.nb_sources(&union) == 1);
    let pointed = session
        .new_select_pointed(Some(&[ents[0].clone()]), "pt")
        .unwrap();
    assert!(session.set_select_pointed(&pointed, Some(&[ents[1].clone()]), 1));
    assert_eq!(
        session.selection_result(&pointed).unwrap().map(|l| l.len()),
        Some(2)
    );
    assert!(
        session.set_select_pointed(&pointed, Some(&[ents[1].clone()]), -1)
            && !session.set_select_pointed(&roots, None, 0)
    );
    assert!(
        session.give_selection("roots") == roots && session.give_selection("unknown(x)").is_null()
    );
    assert!(session.item_selection(&roots.cast()).is_null());
}

#[test]
fn session_give_lists() {
    let (session, ents, _) = loaded_with(true);
    let seq = rustycad_tkernel::ncollection::HSequence::<Handle<StandardTransient>>::default();
    seq.append(ents[1].clone());
    let as_list = session.give_list(&Handle::new(seq).cast()).unwrap();
    assert_eq!(
        as_list,
        Some(vec![ents[1].clone()]),
        "a list object is the list"
    );
    selection(
        &session,
        "roots",
        Handle::new(IFSelectSelectModelRoots::new()).cast(),
    );
    selection(
        &session,
        "shared",
        Handle::new(IFSelectSelectShared::new()).cast(),
    );
    let sign: Handle<StandardTransient> = Handle::new(IFSelectSignType::new(false)).cast();
    session.add_named_item("type", &sign, true).unwrap();
    assert_eq!(
        session.give_list_texts("(1, 3,1)", "").unwrap(),
        Some(vec![ents[0].clone(), ents[2].clone()])
    );
    assert_eq!(
        session.give_list_texts(" 2", "").unwrap(),
        Some(vec![ents[1].clone()])
    );
    assert_eq!(
        session.give_list_texts("roots", "").unwrap(),
        Some(vec![ents[3].clone()])
    );
    assert_eq!(
        session
            .give_list_texts("shared", "(3)")
            .unwrap()
            .map(|l| l.len()),
        Some(2)
    );
    assert_eq!(
        session
            .give_list_texts("shared roots", "")
            .unwrap()
            .map(|l| l.len()),
        Some(1),
        "c, shared by the root"
    );
    assert_eq!(
        session
            .give_list_texts("type(Fake_Entity)", "")
            .unwrap()
            .map(|l| l.len()),
        Some(4)
    );
    assert!(session.give_list_texts("", "").unwrap().is_none());
    assert_eq!(
        session
            .give_list(&session.model().cast())
            .unwrap()
            .map(|l| l.len()),
        Some(4)
    );
    assert_eq!(
        session.give_list(&ents[1]).unwrap(),
        Some(vec![ents[1].clone()])
    );
    assert!(session.give_list(&Handle::default()).unwrap().is_none());
    assert_eq!(
        session
            .give_list_from_list("(4)", &Handle::default())
            .unwrap(),
        Some(vec![ents[3].clone()])
    );
    assert!(
        session
            .give_list_combined(None, Some(&[]), 0)
            .unwrap()
            .is_none()
    );
    assert!(
        session.give_list_combined(Some(&[]), Some(&[]), 1).is_err(),
        "native defect: no result list"
    );
}

#[test]
fn session_share_out_and_files() {
    let (session, _, _) = loaded_with(true);
    let disp = global(&session);
    assert_eq!(session.applied_dispatches().unwrap(), vec![2]);
    assert_eq!(session.dispatch_rank(&disp).unwrap(), 1);
    assert!(
        session.set_file_root(&disp, "part").unwrap()
            && text(&session.file_root(&disp).unwrap()) == "part"
    );
    session.set_file_prefix("out/").unwrap();
    session.set_file_extension(".stp").unwrap();
    assert!(
        session.set_default_file_root("def").unwrap()
            && text(&session.default_file_root().unwrap()) == "def"
    );
    assert_eq!(
        session.give_file_complete("model").unwrap().as_bytes(),
        b"out/model.stp"
    );
    assert_eq!(
        session.give_file_complete("m.igs").unwrap().as_bytes(),
        b"out/m.igs",
        "the extension mark is present, the prefix one not"
    );
    // OSD_Path uses native platform syntax; Windows does not parse this Unix path.
    let root_fixture = if cfg!(windows) {
        r"C:\tmp\dir\name.stp"
    } else {
        "/tmp/dir/name.stp"
    };
    assert_eq!(session.give_file_root(root_fixture).as_bytes(), b"name");
    session.evaluate_file().unwrap();
    assert!(session.nb_files() == 1 && session.file_name(1).unwrap().as_bytes() == b"out/part.stp");
    assert!(session.file_model(1).unwrap().is_some() && session.file_model(2).unwrap().is_null());
    assert!(session.last_run_check_list().is_empty(false));
    session.clear_file().unwrap();
    assert_eq!(session.nb_files(), 0);
    assert_eq!(
        session.eval_split().unwrap().map(|p| p.nb_packets()),
        Some(1),
        "the cleared result runs the dispatch again"
    );
    session.clear_share_out(true).unwrap();
    assert_eq!(session.share_out().nb_dispatches(), 0);
}

#[test]
fn session_modifiers_and_transformers() {
    let (session, ents, _) = loaded();
    let disp = global(&session);
    let reorder: Handle<dyn IFSelectGeneralModifier> =
        Handle::new(IFSelectModifReorder::new(false)).cast();
    session
        .add_named_item("reorder", &reorder.cast(), true)
        .unwrap();
    assert!(
        session
            .set_applied_modifier(&reorder, &session.share_out().cast())
            .unwrap()
    );
    assert_eq!(
        (
            session.nb_final_modifiers(true).unwrap(),
            session.modifier_rank(&reorder).unwrap()
        ),
        (1, 1)
    );
    assert_eq!(session.final_modifier_idents(true).unwrap(), vec![3]);
    assert!(
        session
            .uses_applied_modifier(&reorder)
            .unwrap()
            .object_address()
            == session.share_out().object_address()
    );
    assert!(
        session
            .set_applied_modifier(&reorder, &disp.cast())
            .unwrap()
    );
    assert!(session.uses_applied_modifier(&reorder).unwrap() == disp.cast::<StandardTransient>());
    assert!(
        session.reset_applied_modifier(&reorder).unwrap()
            && session.nb_final_modifiers(true).unwrap() == 0
    );
    let stf = session.new_transform_standard(false, "edit").unwrap();
    assert!(session.set_applied_modifier(&reorder, &stf.cast()).unwrap());
    assert_eq!(stf.cast::<IFSelectTransformStandard>().nb_modifiers(), 1);
    assert_eq!(
        session.run_transformer(&stf).unwrap(),
        4,
        "on the spot, the model kept"
    );
    let model = session.model();
    assert_eq!(
        session
            .run_modifier(&reorder.cast::<dyn IFSelectModifier>(), true)
            .unwrap(),
        3,
        "a copy"
    );
    assert!(session.model() != model && session.model().nb_entities() == 4);
    assert_eq!(
        session
            .run_transformer(&Handle::<dyn IFSelectTransformer>::default())
            .unwrap(),
        0
    );
    let roots = selection(
        &session,
        "roots",
        Handle::new(IFSelectSelectModelRoots::new()).cast(),
    );
    let before = session.model();
    assert!(session.set_model_content(&roots, true).unwrap());
    assert!(
        session.model() != before && session.nb_starting_entities() == 4,
        "d with what it shares"
    );
    assert!(
        session
            .set_remaining(IFSelectRemainMode::RemainUndo)
            .unwrap()
            && session.model() == before
    );
    assert!(session.set_model_content(&roots, false).unwrap());
    assert_eq!(session.nb_starting_entities(), 3, "c with a and b");
    session.clear_final_modifiers().unwrap();
    assert!(
        session.change_modifier_rank(true, 1, 1).unwrap()
            && !session.change_modifier_rank(true, 1, 2).unwrap()
    );
    let _ = ents;
}

#[test]
fn session_sending() {
    let (session, _, lib) = loaded_with(true);
    let disp = global(&session);
    session.set_file_root(&disp, "part").unwrap();
    session.begin_sent_files(true);
    assert!(session.send_split().unwrap());
    assert_eq!(*lib.written.read(), vec![String::from("part_1")]);
    assert_eq!(
        session.sent_files().map(|f| f.len()),
        Some(0),
        "native: the selected sendings are not recorded"
    );
    assert_eq!(
        session.send_all("all.stp", false).unwrap(),
        IFSelectReturnStatus::RetDone
    );
    assert_eq!(
        session.send_all("fail.stp", false).unwrap(),
        IFSelectReturnStatus::RetStop
    );
    let mut out = Vec::new();
    assert_eq!(
        session.send_all_stream(&mut out, "s", false).unwrap(),
        IFSelectReturnStatus::RetFail
    );
    let roots = selection(
        &session,
        "roots",
        Handle::new(IFSelectSelectModelRoots::new()).cast(),
    );
    assert_eq!(
        session.send_selected("sel.stp", &roots, false).unwrap(),
        IFSelectReturnStatus::RetDone
    );
    assert!(
        session.max_sending_count().unwrap() >= 1
            && session.sent_list(-1).unwrap().nb_entities() >= 1
    );
    assert!(
        session
            .set_remaining(IFSelectRemainMode::RemainDisplay)
            .unwrap()
    );
    assert_eq!(
        session.write_file("w.stp").unwrap(),
        IFSelectReturnStatus::RetDone
    );
    assert_eq!(
        session.write_file_selected("ws.stp", &roots).unwrap(),
        IFSelectReturnStatus::RetDone
    );
    assert_eq!(
        session
            .write_file_selected("ws.stp", &Handle::default())
            .unwrap(),
        IFSelectReturnStatus::RetVoid
    );
    assert_eq!(
        session.write_stream(&mut out, "").unwrap(),
        IFSelectReturnStatus::RetFail
    );
    assert!(
        session
            .set_remaining(IFSelectRemainMode::RemainDisplay)
            .unwrap()
    );
    assert!(
        session
            .set_remaining(IFSelectRemainMode::RemainForget)
            .unwrap()
    );
    assert_eq!(session.sent_list(-1).unwrap().nb_entities(), 0);
    assert!(
        session
            .set_remaining(IFSelectRemainMode::RemainCompute)
            .unwrap(),
        "nothing sent: all the entities remain, in a new model"
    );
    assert_eq!(session.nb_starting_entities(), 4);
    let empty = IFSelectWorkSession::new();
    assert_eq!(
        empty.send_all("x", false).unwrap(),
        IFSelectReturnStatus::RetVoid
    );
    assert!(!empty.send_split().unwrap());
}
