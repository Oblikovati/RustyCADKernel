//! Unit tests of the work session: checks, queries, dumps, evaluations, parameters, the
//! virtual `ClearData` and the guarded calls, over the fake chain (a, b, c, d: d shares
//! c, c shares a and b).
use super::session_tests::{loaded, loaded_with};
use super::*;
use crate::if_select::{
    IFSelectDispGlobal, IFSelectDispatch, IFSelectEditForm, IFSelectPrintCount,
    IFSelectSelectModelEntities, IFSelectSelectModelRoots, IFSelectSelection,
    IFSelectSignatureList,
};
use crate::interface::{InterfaceCheckIterator, InterfaceParamType, InterfaceTypedValue};
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};

/// A derived session counting the calls of its `ClearData` override.
struct CountingSession {
    session: IFSelectWorkSession,
    clears: SharedState<Vec<i32>>,
}
rustycad_tkernel::impl_transient!(CountingSession, "Test_CountingSession", IFSelectWorkSession, session, interfaces: [dyn IFSelectWorkSessionOps]);

impl IFSelectWorkSessionOps for CountingSession {
    fn work_session(&self) -> &IFSelectWorkSession {
        &self.session
    }
    fn clear_data(&self, mode: i32) -> Result<(), rustycad_tkernel::standard::Failure> {
        self.clears.write().push(mode);
        self.session.clear_data_base(mode)
    }
}

fn dumped(f: impl FnOnce(&mut Vec<u8>)) -> String {
    let mut out = Vec::new();
    f(&mut out);
    String::from_utf8(out).unwrap()
}

#[test]
fn session_clear_data_is_virtual() {
    let derived = Handle::new(CountingSession {
        session: IFSelectWorkSession::new(),
        clears: SharedState::new(Vec::new()),
    });
    derived.session.bind_self(&derived.clone().cast());
    let (model, _) = crate::interface::test_norm::fake_chain();
    derived.session.set_model(&model, false).unwrap();
    assert_eq!(*derived.clears.read(), vec![3, 0]);
    derived.session.clear_data_virtual(1).unwrap();
    assert_eq!(*derived.clears.read(), vec![3, 0, 1, 2, 4]);
    assert!(!derived.session.has_model());
}

#[test]
fn session_clear_pointed_data() {
    let (session, ents, _) = loaded();
    let pointed = session
        .new_select_pointed(Some(&[ents[0].clone()]), "pt")
        .unwrap();
    let list = Handle::new(IFSelectSignatureList::new(false));
    session.add_item(&list.cast(), true).unwrap();
    session.clear_data_base(4).unwrap();
    assert!(
        session.max_ident() == 0
            && pointed
                .cast::<crate::if_select::IFSelectSelectPointed>()
                .nb_items()
                == 0
    );
    session.clear_data_base(2).unwrap();
    assert!(!session.is_loaded() && session.compute_graph(false).unwrap());
    session.clear_data_base(1).unwrap();
    assert!(!session.has_model() && session.loaded_file().is_empty());
}

#[test]
fn session_checks_and_queries() {
    let (session, ents, _) = loaded_with(true);
    assert!(session.compute_check(true).unwrap());
    let checks = session.model_check_list(true).unwrap();
    assert!(checks.is_empty(false) && checks.name().as_bytes() == b"Model Complete Check List");
    assert_eq!(
        session.model_check_list(false).unwrap().name().as_bytes(),
        b"Model Syntactic Check List"
    );
    let one = session.check_one(&ents[0], true).unwrap();
    assert_eq!(one.name().as_bytes(), b"Data Check (One Entity)");
    assert!(
        session
            .check_one(&Handle::default(), false)
            .unwrap()
            .is_empty(false)
    );
    let mut chl = InterfaceCheckIterator::new();
    chl.c_check(2).add_fail_text("bad", "");
    chl.c_check(3).add_warning_text("odd", "");
    session.query_check_list(&chl).unwrap();
    let statuses: Vec<i32> = ents.iter().map(|e| session.query_check_status(e)).collect();
    assert_eq!(statuses, vec![0, 2, 1, 0], "native: nothing propagated");
    let mut global = InterfaceCheckIterator::new();
    global.c_check(0).add_fail_text("global", "");
    assert!(
        session.query_check_list(&global).is_err(),
        "native: a global check is out of the status string"
    );
    assert_eq!(session.query_parent(&ents[3], &ents[0]).unwrap(), 2);
    assert_eq!(
        (
            session.query_parent(&ents[0], &ents[0]).unwrap(),
            session.query_parent(&ents[0], &ents[3]).unwrap()
        ),
        (0, -1)
    );
    let empty = IFSelectWorkSession::new();
    assert!(empty.model_check_list(true).unwrap().check(0).has_failed());
    assert_eq!(empty.query_check_status(&ents[0]), -1);
}

#[test]
fn session_dumps() {
    let (session, ents, _) = loaded_with(true);
    let disp: Handle<dyn IFSelectDispatch> = Handle::new(IFSelectDispGlobal::new()).cast();
    session.add_named_item("glob", &disp.cast(), true).unwrap();
    let roots: Handle<dyn IFSelectSelection> = Handle::new(IFSelectSelectModelRoots::new()).cast();
    session
        .add_named_item("roots", &roots.cast(), true)
        .unwrap();
    session.dump_share().unwrap();
    session.list_items("");
    session.list_items("Selection");
    session.list_final_modifiers(true).unwrap();
    session.dump_selection(&roots).unwrap();
    session.dump_selection(&Handle::default()).unwrap();
    let model0 = dumped(|o| session.dump_model(0, o).unwrap());
    assert!(
        model0.contains("No name for Loaded File")
            && model0.contains("Model : 4 Entities, of which 1 Root(s)"),
        "{model0}"
    );
    let model1 = dumped(|o| session.dump_model(1, o).unwrap());
    assert!(
        model1.contains("Root Entities")
            && model1.contains("#ROOT#")
            && model1.contains("There are 4 Entities"),
        "{model1}"
    );
    let model2 = dumped(|o| session.dump_model(2, o).unwrap());
    assert!(model2.contains(" List of 4 Entities :"), "{model2}");
    assert!(dumped(|o| session.dump_model(5, o).unwrap()).contains("There are 4 Entities"));
    session.trace_dump_model(-1).unwrap();
    let entity = dumped(|o| session.dump_entity(&ents[0], 1, o).unwrap());
    assert_eq!(
        entity,
        "        ********  Dumping Entity n0 1 level:1  ********\nlevel 1"
    );
    assert!(
        dumped(|o| session.dump_entity(&Handle::default(), 1, o).unwrap())
            .contains("not in the Model")
    );
    session.trace_dump_entity(&ents[0], 0).unwrap();
    let status = dumped(|o| session.print_entity_status(&ents[2], o).unwrap());
    assert!(
        status.contains("Name:c") && status.contains("Type(CDL):Fake_Entity"),
        "{status}"
    );
    assert!(
        status.contains("Super-entities:1 : (n0/id):")
            && status.contains("Sub-entities:2 , i.e. (n0/id):"),
        "{status}"
    );
}

#[test]
fn session_listings_and_evaluations() {
    let (session, ents, _) = loaded_with(true);
    let all = session
        .eval_selection(&Handle::new(IFSelectSelectModelEntities::new()).cast())
        .unwrap();
    let numbers = dumped(|o| session.list_entities(&all, 2, o).unwrap());
    assert_eq!(numbers, " List of 4 Entities :\n(1,2,3,4)\n");
    let marked = dumped(|o| session.list_entities(&all, 0, o).unwrap());
    assert!(
        marked.contains("  Keys : R Root") && marked.contains("(R)"),
        "{marked}"
    );
    let lines = dumped(|o| session.list_entities(&all, -1, o).unwrap());
    assert!(
        !lines.contains("List of") && lines.contains("Type:Entity"),
        "{lines}"
    );
    let mut chl = InterfaceCheckIterator::new();
    chl.c_check(1).add_fail_text("bad", "");
    let by_entity = dumped(|o| {
        session
            .print_check_list(o, &chl, false, IFSelectPrintCount::ItemsByEntity)
            .unwrap()
    });
    let counted = dumped(|o| {
        session
            .print_check_list(o, &chl, false, IFSelectPrintCount::CountByItem)
            .unwrap()
    });
    assert!(
        by_entity.contains("bad") && counted.contains(" Count\tCheck"),
        "{by_entity}\n{counted}"
    );
    session
        .print_signature_list(
            &mut Vec::new(),
            &Handle::default(),
            IFSelectPrintCount::CountByItem,
        )
        .unwrap();
    let roots: Handle<dyn IFSelectSelection> = Handle::new(IFSelectSelectModelRoots::new()).cast();
    session.add_item(&roots.cast(), true).unwrap();
    session.evaluate_selection(&roots).unwrap();
    let disp: Handle<dyn IFSelectDispatch> = Handle::new(IFSelectDispGlobal::new()).cast();
    disp.set_final_selection(&roots);
    session.add_item(&disp.cast(), true).unwrap();
    for mode in 0..=3 {
        session.evaluate_dispatch(&disp, mode).unwrap();
        session.evaluate_complete(mode).unwrap();
    }
    let _ = ents;
}

#[test]
fn session_params_and_statics() {
    let (session, _, _) = loaded();
    let count: Handle<StandardTransient> = Handle::new(
        InterfaceTypedValue::new("count", InterfaceParamType::ParamInteger, "1").unwrap(),
    )
    .cast();
    let name: Handle<StandardTransient> =
        Handle::new(InterfaceTypedValue::new("name", InterfaceParamType::ParamText, "x").unwrap())
            .cast();
    session
        .set_params(&[count, Handle::default(), name], &[1, 6, 2])
        .unwrap();
    assert!(
        session.named_item("xst-params-edit").is_some()
            && session.named_item("xst-params-all").is_some()
    );
    let general = session
        .named_item("xst-params-general")
        .cast::<IFSelectEditForm>();
    assert_eq!(
        general.number_from_rank(1),
        0,
        "native quirk: the 0-based rank"
    );
    assert!(
        session.named_item("xst-params-write").is_some()
            && session.named_item("xst-params-read").is_some()
    );
    assert!(
        session.named_item("xst-params-load").is_null()
            && session.named_item("xst-params-send").is_null()
    );
    for use_ in [1, 4, 5, 6, 9, -4] {
        session.trace_statics(use_, 0).unwrap();
    }
}

#[test]
fn session_guarded_calls() {
    let (session, _, _) = loaded();
    assert!(session.error_handle());
    let null_sel: Handle<dyn IFSelectSelection> = Handle::default();
    assert_eq!(
        session.eval_selection(&null_sel).unwrap().nb_entities(),
        0,
        "the error is reported"
    );
    session.set_error_handle(false);
    assert!(
        session.eval_selection(&null_sel).is_err(),
        "the error is raised"
    );
    session.set_error_handle(true);
    assert!(session.eval_selection(&null_sel).is_ok());
}
