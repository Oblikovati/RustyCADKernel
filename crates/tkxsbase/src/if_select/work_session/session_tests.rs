//! Unit tests of the work session: model, entities, items and parameters, over the fake
//! chain (a, b, c, d: d shares c, c shares a and b) and the fake work library.
use super::give_list::parenthesized_ids;
use super::selections::split_selection_name;
use super::*;
use crate::if_select::test_work_library::FakeWorkLibrary;
use crate::if_select::{
    IFSelectDispGlobal, IFSelectDispatch, IFSelectIntParam, IFSelectModifReorder,
    IFSelectReturnStatus, IFSelectSelectModelEntities, IFSelectSelectModelRoots, IFSelectSelection,
    IFSelectSignType, IFSelectSignature, IFSelectTransformStandard,
};
use crate::interface::test_norm::{fake_chain, fake_protocol};
use crate::interface::{InterfaceParamType, InterfaceStatic};
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardTransient};
use rustycad_tkernel::tcollection::HAsciiString;

/// A session loaded with the fake chain and the fake work library.
pub(super) fn loaded() -> (
    IFSelectWorkSession,
    Vec<Handle<StandardTransient>>,
    Handle<FakeWorkLibrary>,
) {
    loaded_with(false)
}

/// As `loaded`, the graph with statuses (statistics mode) when `stats`: the selections
/// and dispatches which record entities on a graph copy need them.
pub(super) fn loaded_with(
    stats: bool,
) -> (
    IFSelectWorkSession,
    Vec<Handle<StandardTransient>>,
    Handle<FakeWorkLibrary>,
) {
    let session = IFSelectWorkSession::new();
    session.set_mode_stat(stats);
    let lib = Handle::new(FakeWorkLibrary::default());
    session.set_library(&lib.clone().cast());
    session.set_protocol(&fake_protocol());
    let (model, ents) = fake_chain();
    session.set_model(&model, true).unwrap();
    (session, ents, lib)
}

pub(super) fn text(h: &Handle<HAsciiString>) -> String {
    h.get()
        .map(|t| String::from_utf8_lossy(t.string().as_bytes()).into_owned())
        .unwrap_or_default()
}

#[test]
fn selection_names_and_identifiers() {
    let plain = split_selection_name("sel");
    assert!(plain.name == "sel" && plain.sign.is_none());
    let signed = split_selection_name("type(Line)");
    assert_eq!(
        (signed.name.as_str(), signed.sign.as_deref()),
        ("type", Some("Line"))
    );
    let nested = split_selection_name("t(a(b))");
    assert_eq!(
        (nested.name.as_str(), nested.sign.as_deref()),
        ("t(a", Some("b)")),
        "native: the last opening parenthesis"
    );
    assert!(split_selection_name("(x)").sign.is_none());
    assert_eq!(parenthesized_ids("(1, 2,a)"), vec!["1", "2", "a"]);
    assert_eq!(
        parenthesized_ids("(,) 3)"),
        vec!["3"],
        "an empty identifier does not stop"
    );
    assert!(parenthesized_ids("()").is_empty());
}

#[test]
fn session_settings_and_model() {
    let session = IFSelectWorkSession::new();
    assert!(!session.has_model() && !session.is_loaded() && !session.compute_graph(false).unwrap());
    assert!(session.share_out().is_some() && session.model_copier().is_some());
    session.set_mode_stat(true);
    assert!(session.get_mode_stat());
    assert_eq!(
        session.read_file("x").unwrap(),
        IFSelectReturnStatus::RetVoid,
        "no library"
    );
    assert!(session.set_model(&Handle::default(), true).is_err());
    let (session, ents, _) = loaded();
    assert!(session.is_loaded() && session.h_graph().unwrap().is_some());
    assert_eq!(
        (
            session.nb_starting_entities(),
            session.starting_number(&ents[2])
        ),
        (4, 3)
    );
    assert!(session.starting_entity(4) == ents[3] && session.starting_entity(5).is_null());
    assert_eq!(
        (
            session.number_from_label("2", 0),
            session.number_from_label("c", 0)
        ),
        (2, 3)
    );
    assert_eq!(session.number_from_label("zz", 0), 0);
    assert_eq!(text(&session.entity_label(&ents[0])), "a");
    assert!(
        text(&session.entity_name(&ents[0]).unwrap()) == "a"
            && session.category_number(&ents[0]) == 0
    );
    assert_eq!(session.validity_name(&ents[0]), "OK");
    assert_eq!(session.shareds(&ents[2]).unwrap().map(|l| l.len()), Some(2));
    assert_eq!(
        session.sharings(&ents[2]).unwrap().map(|l| l.len()),
        Some(1)
    );
    session.set_loaded_file("f.stp");
    assert_eq!(session.loaded_file().as_bytes(), b"f.stp");
    let sign: Handle<dyn IFSelectSignature> = Handle::new(IFSelectSignType::new(false)).cast();
    session.set_sign_type(&sign);
    assert!(session.sign_type() == sign && session.named_item("xst-sign-type").is_some());
    session.set_sign_type(&Handle::default());
    assert!(session.sign_type().is_null() && session.named_item("xst-sign-type").is_null());
}

#[test]
fn session_reads() {
    let (session, _, _) = loaded();
    assert_eq!(
        session.read_file("fail").unwrap(),
        IFSelectReturnStatus::RetError
    );
    assert_eq!(
        session.read_file("new.stp").unwrap(),
        IFSelectReturnStatus::RetDone
    );
    assert!(
        session.loaded_file().as_bytes() == b"new.stp" && !session.is_loaded(),
        "the fake read gives an empty model"
    );
    let mut input: &[u8] = b"";
    assert_eq!(
        session.read_stream("s", &mut input).unwrap(),
        IFSelectReturnStatus::RetFail,
        "streams not supported"
    );
}

#[test]
fn session_items_and_names() {
    let (session, _, _) = loaded();
    let roots: Handle<StandardTransient> = Handle::new(IFSelectSelectModelRoots::new()).cast();
    assert_eq!(session.add_named_item("roots", &roots, true).unwrap(), 1);
    assert_eq!(session.add_named_item("#x", &roots, true).unwrap(), 0);
    let all: Handle<StandardTransient> = Handle::new(IFSelectSelectModelEntities::new()).cast();
    assert_eq!(session.add_item(&all, true).unwrap(), 2);
    assert!(session.named_item("roots") == roots && session.named_item("#2") == all);
    assert_eq!(
        (
            session.name_ident("roots"),
            session.name_ident("#7"),
            session.name_ident("none")
        ),
        (1, 7, 0)
    );
    assert!(
        session.has_name(&roots)
            && !session.has_name(&all)
            && text(&session.name(&roots)) == "roots"
    );
    assert!(session.named_item_h(&Handle::new(HAsciiString::new("roots"))) == roots);
    assert!(session.remove_name("roots") && session.named_item("roots").is_null());
    assert!(
        session.has_name(&roots),
        "native quirk: the attribute keeps the name"
    );
    assert!(
        session.remove_item(&all).unwrap() && session.item(2).is_null() && session.max_ident() == 2
    );
    assert!(!session.remove_item(&all).unwrap() && session.item_ident(&all) == 0);
    assert_eq!(
        session.add_item(&all, false).unwrap(),
        2,
        "revived at its ident"
    );
    assert!(session.set_active(&Handle::default(), true).is_err());
    session.clear_items();
    assert_eq!(session.max_ident(), 0);
}

#[test]
fn session_item_labels_and_lists() {
    let (session, _, _) = loaded();
    let disp: Handle<dyn IFSelectDispatch> = Handle::new(IFSelectDispGlobal::new()).cast();
    session.add_named_item("glob", &disp.cast(), true).unwrap();
    assert_eq!(
        session.share_out().nb_dispatches(),
        1,
        "an active dispatch joins the share-out"
    );
    session
        .add_item(&Handle::new(IFSelectModifReorder::new(true)).cast(), true)
        .unwrap();
    session
        .add_item(&Handle::new(IFSelectTransformStandard::new()).cast(), true)
        .unwrap();
    session
        .add_item(&Handle::new(IFSelectSignType::new(false)).cast(), true)
        .unwrap();
    session
        .add_item(&Handle::new(IFSelectSelectModelRoots::new()).cast(), true)
        .unwrap();
    let labels: Vec<String> = (1..=5).map(|i| text(&session.item_label(i))).collect();
    assert_eq!(labels[0], "Dispatch:One File for All Input");
    assert!(
        labels[1].starts_with("ModelModifier:Reorder")
            && labels[2].starts_with("Transformer:Standard Copy")
    );
    assert!(
        labels[3].starts_with("Signature:") && labels[4].starts_with("Selection:"),
        "{labels:?}"
    );
    assert!(session.item_label(9).is_null());
    assert_eq!(
        session.item_idents(<dyn IFSelectSelection>::static_type()),
        vec![5]
    );
    assert_eq!(
        session
            .item_names(<dyn IFSelectDispatch>::static_type())
            .len(),
        1
    );
    assert_eq!(session.item_names_for_label("Selection").len(), 1);
    assert_eq!(
        (
            session.next_ident_for_label("Selection", 0, 1),
            session.next_ident_for_label("Roots", 0, 2)
        ),
        (5, 2),
        "the reorder label contains Roots"
    );
    assert_eq!(session.next_ident_for_label(&labels[0], 0, 0), 1);
    assert!(
        session.set_active(&disp.cast(), false).unwrap()
            && session.share_out().nb_dispatches() == 0
    );
    assert_eq!(session.dispatch(1), disp);
}

#[test]
fn session_parameters() {
    let (session, ents, _) = loaded();
    let intpar = session.new_int_param("n").unwrap();
    assert!(session.set_int_value(&intpar, 5) && session.int_value(&intpar) == 5);
    assert!(
        !session.set_int_value(&Handle::new(IFSelectIntParam::new()), 1)
            && session.int_value(&Handle::default()) == 0
    );
    assert_eq!(text(&session.item_label(1)), "Integer:5");
    assert!(session.int_param(1) == intpar && session.text_param(1).is_null());
    let textpar = session.new_text_param("t").unwrap();
    assert!(
        session.set_text_value(&textpar, "hello")
            && session.text_value(&textpar).as_bytes() == b"hello"
    );
    assert_eq!(text(&session.item_label(2)), "Text:hello");
    assert!(session.new_int_param("#bad").unwrap().is_null());
    InterfaceStatic::init("test", "test.ws.int", InterfaceParamType::ParamInteger, "4").unwrap();
    InterfaceStatic::init("test", "test.ws.text", InterfaceParamType::ParamText, "abc").unwrap();
    let p1 = session.new_param_from_static("test.ws.int", "pi").unwrap();
    assert_eq!(p1.cast::<IFSelectIntParam>().value(), 4);
    let p2 = session.new_param_from_static("test.ws.text", "").unwrap();
    assert_eq!(text(&p2.cast()), "abc");
    assert!(
        session
            .new_param_from_static("test.ws.none", "")
            .unwrap()
            .is_null()
    );
    let sign: Handle<dyn IFSelectSignature> = Handle::new(IFSelectSignType::new(false)).cast();
    session.add_item(&sign.cast(), true).unwrap();
    assert!(session.signature(session.item_ident(&sign.cast())) == sign);
    assert_eq!(session.sign_value(&sign, &ents[0]), "Fake_Entity");
    assert!(session.sign_value(&Handle::default(), &ents[0]).is_empty());
}
