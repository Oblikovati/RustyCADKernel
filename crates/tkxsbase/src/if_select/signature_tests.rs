//! Unit tests of the signatures, the signature lists and counters, and the selection by
//! signature, over the fake chain (d shares c, c shares a and b).
use super::*;
use crate::interface::test_norm::{FakeEntity, fake_chain};
use crate::interface::{
    InterfaceCheck, InterfaceCheckIterator, InterfaceGraph, InterfaceInterfaceModel,
    InterfaceSignType,
};
use crate::moni_tool::MoniToolSignText;
use rustycad_tkernel::{
    standard::{Handle, RuntimeTyped, StandardTransient},
    tcollection::AsciiString,
};

fn chain() -> (
    Handle<dyn InterfaceInterfaceModel>,
    InterfaceGraph,
    Vec<Handle<StandardTransient>>,
) {
    let (model, ents) = fake_chain();
    let graph = InterfaceGraph::new(&model, true).unwrap();
    (model, graph, ents)
}

fn text(s: &str) -> AsciiString {
    AsciiString::new(s)
}

#[test]
fn signature_base_members() {
    let sign = IFSelectSignType::new(false);
    assert_eq!(
        (sign.name(), sign.is_int_case()),
        ("Dynamic Type".to_owned(), None)
    );
    sign.set_int_case(true, 2, false, 9);
    assert_eq!(sign.is_int_case(), Some((true, 2, false, 0)));
    assert!(sign.case_list().is_none());
    sign.add_case("x");
    assert_eq!(sign.case_list().unwrap().len(), 1);
    assert!(IFSelectSignatureBase::match_value("Fake_Entity", &text("Entity"), false).unwrap());
    assert!(!IFSelectSignatureBase::match_value("Fake_Entity", &text("Entity"), true).unwrap());
    assert!(IFSelectSignatureBase::match_value("abc", &text(""), false).is_err());
    assert_eq!(
        (
            IFSelectSignatureBase::int_value(7),
            IFSelectSignatureBase::int_value(-12)
        ),
        ("7".to_owned(), "-12".to_owned())
    );
}

#[test]
fn signatures_by_type_category_validity() {
    let (model, _, ents) = chain();
    let dynamic = IFSelectSignType::new(false);
    assert_eq!(dynamic.value(&ents[0], &model), "Fake_Entity");
    assert_eq!(dynamic.value(&Handle::default(), &model), "");
    let type_entity: Handle<StandardTransient> = Handle::new(FakeEntity::default()).cast();
    assert_eq!(
        IFSelectSignType::new(true).value(&type_entity, &model),
        "Entity"
    );
    assert_eq!(
        dynamic.text(&ents[0], &model.cast()).as_bytes(),
        b"Fake_Entity"
    );
    let ancestor = IFSelectSignAncestor::new(false);
    assert!(
        ancestor
            .matches(&ents[0], &model, &text("Standard_Transient"), true)
            .unwrap()
    );
    assert!(
        !ancestor
            .matches(&ents[0], &model, &text("Geom_Line"), true)
            .unwrap()
    );
    assert!(
        !ancestor
            .matches(&Handle::default(), &model, &text("x"), true)
            .unwrap()
    );
    assert_eq!(ancestor.value(&ents[0], &model), "Fake_Entity");
    let category = IFSelectSignCategory::new();
    assert!(category.case_list().is_some_and(|c| !c.is_empty()));
    assert_eq!(category.value(&Handle::default(), &model), "");
    assert!(!category.value(&ents[0], &model).is_empty());
    let validity = IFSelectSignValidity::new();
    assert_eq!(
        (
            validity.value(&ents[0], &model),
            validity.case_list().unwrap().len()
        ),
        ("OK".to_owned(), 7)
    );
    assert_eq!(
        IFSelectSignValidity::c_val(&ents[0], &Handle::default()),
        ""
    );
}

#[test]
fn multiple_signatures() {
    let (model, _, ents) = chain();
    let multi = IFSelectSignMultiple::new("Multi");
    multi.add(&Handle::new(IFSelectSignType::new(true)).cast(), 8, false);
    multi.add(&Handle::new(IFSelectSignValidity::new()).cast(), 0, false);
    multi.add(&Handle::default(), 3, false);
    assert_eq!(multi.value(&ents[0], &model), "Entity     OK   ");
    assert!(multi.matches(&ents[0], &model, &text("OK"), false).unwrap());
    assert!(!multi.matches(&ents[0], &model, &text("OK"), true).unwrap());
    let maxi = IFSelectSignMultiple::new("Maxi");
    maxi.add(&Handle::new(IFSelectSignType::new(false)).cast(), 4, true);
    assert_eq!(
        maxi.value(&ents[0], &model),
        "Fake_Entit._Entity   ",
        "native defect: no truncation"
    );
}

#[test]
fn signature_lists() {
    let (model, _, ents) = chain();
    let list = IFSelectSignatureList::new(true);
    list.add(&ents[0], "A");
    list.add(&ents[1], "A");
    list.add(&ents[2], "12");
    list.add(&ents[3], "");
    assert_eq!(
        (list.nb_times("A"), list.nb_times("B"), list.nb_nulls()),
        (2, 0, 1)
    );
    assert_eq!(list.entities("A").unwrap().len(), 2);
    assert!(list.entities("B").unwrap().is_empty());
    assert_eq!(list.list("").len(), 2);
    assert_eq!(list.list("1").len(), 1);
    let mut out = Vec::new();
    list.print_count(&mut out).unwrap();
    let count = String::from_utf8(out).unwrap();
    assert!(
        count.contains("     2\tA")
            && count.contains("... Nul : 1")
            && count.contains("Nb Total:3  for 2 items")
    );
    let mut out = Vec::new();
    list.print_list(&mut out, &model, IFSelectPrintCount::EntitiesByItem)
        .unwrap();
    let printed = String::from_utf8(out).unwrap();
    assert!(
        printed.contains("... : A\n  - Nb: 2 :   1:") && printed.contains("  ??"),
        "native quirk: entity 2 prints ??"
    );
    let mut out = Vec::new();
    list.print_sum(&mut out).unwrap();
    let sum = String::from_utf8(out).unwrap();
    assert!(sum.contains("Nb Integer Items : 1") && sum.contains("Average Value    : 12 0/10"));
    let mut out = Vec::new();
    list.print_list(&mut out, &model, IFSelectPrintCount::ItemsByEntity)
        .unwrap();
    assert!(out.is_empty());
    list.set_mode_sign_only(true);
    list.add(&ents[0], "Last");
    assert!(
        list.mode_sign_only()
            && list.last_value().as_bytes() == b"Last"
            && list.nb_times("Last") == 0
    );
    list.set_name("Named");
    assert_eq!(list.name(), "Named");
    let copy = IFSelectSignatureList::new(true);
    copy.init(
        "Copy",
        &IFSelectSignatureCounts::default(),
        &IFSelectSignatureEntities::default(),
        3,
    );
    assert!(!copy.has_entities() && copy.nb_nulls() == 3 && copy.entities("A").is_none());
    let mut out = Vec::new();
    copy.print_list(&mut out, &model, IFSelectPrintCount::ListByItem)
        .unwrap();
    assert!(
        String::from_utf8(out)
            .unwrap()
            .starts_with(" SignatureList Copy : PrintList, list not available")
    );
    list.clear();
    assert_eq!(list.nb_times("A"), 0);
    list.set_list(false);
    assert!(!list.has_entities());
}

#[test]
fn sign_counters() {
    let (model, g, ents) = chain();
    let sign: Handle<dyn IFSelectSignature> = Handle::new(IFSelectSignType::new(true)).cast();
    let counter = IFSelectSignCounter::with_signature(&sign, true, true).unwrap();
    assert_eq!(counter.name(), "Class Type");
    assert!(
        counter.add_entity(&ents[0], &model).unwrap()
            && !counter.add_entity(&ents[0], &model).unwrap()
    );
    counter.add_model(&model).unwrap();
    assert_eq!(counter.nb_times("Entity"), 4, "the map stops the duplicate");
    counter.set_map(false);
    counter.add_list(Some(&[ents[0].clone()]), &model).unwrap();
    assert_eq!(counter.nb_times("Entity"), 5);
    assert_eq!(
        counter
            .sign(&ents[0], &model)
            .get()
            .map(|s| String::from_utf8_lossy(s.string().as_bytes()).into_owned()),
        Some("Entity".to_owned())
    );
    assert!(counter.sign(&Handle::default(), &model).is_null());
    assert_eq!(
        counter.computed_sign(&ents[1], &g).unwrap().as_bytes(),
        b"Entity"
    );
    let roots: Handle<dyn IFSelectSelection> = Handle::new(IFSelectSelectModelRoots::new()).cast();
    assert!(!counter.compute_selected(&g, false).unwrap());
    counter.set_selection(&roots);
    assert_eq!(counter.sel_mode(), 2);
    assert!(counter.compute_selected(&g, false).unwrap());
    assert_eq!(counter.nb_times("Entity"), 6);
    assert!(counter.compute_selected(&g, false).unwrap());
    assert_eq!(
        counter.nb_times("Entity"),
        6,
        "same sizes: not computed again"
    );
    counter.add_from_selection(&roots, &g).unwrap();
    assert_eq!(counter.nb_times("Entity"), 7);
    counter.set_sel_mode(0);
    assert!(counter.selection().is_null() && counter.signature() == sign);
    assert!(IFSelectSignCounter::with_signature(&Handle::default(), true, false).is_err());
}

#[test]
fn graph_and_check_counters() {
    let (model, g, ents) = chain();
    let graph_counter = IFSelectGraphCounter::new(true, false);
    graph_counter.add_with_graph(Some(&ents), &g).unwrap();
    assert_eq!(
        graph_counter.nb_nulls(),
        4,
        "no deduction: no signature, null values"
    );
    let shared: Handle<dyn IFSelectSelectDeduct> = Handle::new(IFSelectSelectShared::new()).cast();
    graph_counter.set_applied(&shared);
    assert!(graph_counter.applied() == shared);
    graph_counter.add_with_graph(Some(&ents), &g).unwrap();
    assert_eq!(
        (
            graph_counter.nb_times("0"),
            graph_counter.nb_times("1"),
            graph_counter.nb_times("2")
        ),
        (2, 1, 1)
    );
    let checks = IFSelectCheckCounter::new(false);
    assert_eq!(checks.name(), "Check");
    let mut list = InterfaceCheckIterator::with_name("load");
    let check = Handle::new(InterfaceCheck::new());
    check.add_fail_text("bad", "");
    check.add_warning_text("careful", "");
    list.add(&check, 1);
    let lost = Handle::new(InterfaceCheck::new());
    lost.add_fail_text("orphan", "");
    list.add(&lost, 0);
    checks.analyse(&list, &model, false, false).unwrap();
    assert_eq!(checks.name(), "Check load");
    // without signature, the type text is the type name of the model (its GTool)
    let tyname =
        crate::interface::InterfaceModelEntities::type_name(model.get().unwrap(), &ents[0], true);
    assert_eq!(
        (
            checks.nb_times(&format!("F:{tyname}: bad")),
            checks.nb_times(&format!("W:{tyname}: careful")),
            checks.nb_times("F: orphan")
        ),
        (1, 1, 1)
    );
    checks.set_signature(&Handle::new(IFSelectSignType::new(true)).cast());
    assert!(checks.signature().is_some());
    checks.clear();
    checks.analyse(&list, &model, false, true).unwrap();
    assert_eq!(
        (
            checks.nb_times("F:Entity: bad"),
            checks.nb_times("W:Entity: careful")
        ),
        (1, 0)
    );
}

#[test]
fn selections_by_signature() {
    let (_, g, ents) = chain();
    let all: Handle<dyn IFSelectSelection> = Handle::new(IFSelectSelectModelEntities::new()).cast();
    let sign: Handle<dyn IFSelectSignature> = Handle::new(IFSelectSignType::new(false)).cast();
    let exact = IFSelectSelectSignature::with_signature(&sign, "Fake_Entity", true);
    exact.set_input(&all);
    assert!(exact.is_exact() && exact.signature() == sign && exact.counter().is_null());
    assert_eq!(exact.root_result(&g).unwrap().nb_entities(), 4);
    let containing = IFSelectSelectSignature::with_signature(&sign, "Entity", false);
    containing.set_input(&all);
    assert_eq!(
        (
            containing.root_result(&g).unwrap().nb_entities(),
            containing.label().as_bytes()
        ),
        (4, &b"Picked: Dynamic Type containing Entity"[..])
    );
    let listed = IFSelectSelectSignature::with_signature(&sign, "Other|=Fake_Entity", false);
    listed.set_input(&all);
    assert_eq!(listed.root_result(&g).unwrap().nb_entities(), 4);
    assert_eq!(
        listed.extract_label().as_bytes(),
        b"Dynamic Type matching list Other|=Fake_Entity"
    );
    let negated = IFSelectSelectSignature::with_signature(&sign, "!Fake", false);
    negated.set_input(&all);
    assert_eq!(negated.root_result(&g).unwrap().nb_entities(), 0);
    let broken = IFSelectSelectSignature::with_signature(&sign, "A|", false);
    broken.set_input(&all);
    assert!(
        broken.root_result(&g).is_err(),
        "native quirk: a trailing separator counts an item"
    );
    let graph_counter = IFSelectGraphCounter::new(false, false);
    graph_counter.set_applied(&Handle::new(IFSelectSelectShared::new()).cast());
    let counter: Handle<dyn IFSelectSignCounterOps> = Handle::new(graph_counter).cast();
    let by_count = IFSelectSelectSignature::with_counter(&counter, ">=1", false);
    by_count.set_input(&all);
    assert_eq!(
        by_count.root_result(&g).unwrap().content(),
        vec![ents[2].clone(), ents[3].clone()]
    );
    assert_eq!(FakeEntity::static_type().name(), "Fake_Entity");
}
