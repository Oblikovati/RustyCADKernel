//! Unit tests of the transfer processes, their actors and their iterators.
use super::*;
use crate::interface::InterfaceHGraph;
use crate::interface::test_norm::{FakeEntity, fake_chain, fake_entity};
use rustycad_tkernel::{
    impl_transient,
    message::{Msg, ProgressRange},
    standard::{Failure, Handle, RuntimeTyped, StandardTransient},
    tcollection::HAsciiString,
};

/// What [`FakeActor`] does with an object.
#[derive(Clone, Copy, Default, PartialEq)]
enum FakeMode {
    /// A text result named after the entity.
    #[default]
    Text,
    /// A failure.
    Fail,
    /// Transfers the same object again (a dead loop), then a text result.
    Loop,
    /// Transfers the shared entities first, then a text result.
    Nested,
}

/// A named actor of transients.
#[derive(Default)]
struct FakeActor {
    base: TransferActorOfProcessForTransient,
    mode: FakeMode,
}
impl_transient!(FakeActor, "Fake_Actor", TransferActorOfProcessForTransient, base, interfaces: [dyn TransferActorOps<TransferTransientKind>]);

fn fake_actor(mode: FakeMode) -> Handle<dyn TransferActorOps<TransferTransientKind>> {
    Handle::new(FakeActor {
        mode,
        ..FakeActor::default()
    })
    .cast()
}

fn text_result(start: &Handle<StandardTransient>) -> Handle<dyn TransferBinder> {
    let name = start
        .cast::<FakeEntity>()
        .get()
        .map(|e| e.name.read().clone())
        .unwrap_or_default();
    let binder = TransferSimpleBinderOfTransient::new();
    let _ = binder.set_result(&Handle::new(HAsciiString::new(&name)).cast());
    Handle::new(binder).cast()
}

impl TransferActorOps<TransferTransientKind> for FakeActor {
    fn actor(&self) -> &TransferActor<TransferTransientKind> {
        self.base.actor()
    }
    fn recognize(&self, start: &Handle<StandardTransient>) -> bool {
        start.cast::<FakeEntity>().is_some()
    }
    fn transferring(
        &self,
        start: &Handle<StandardTransient>,
        tp: &Handle<dyn TransferProcessOps<TransferTransientKind>>,
        progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        match self.mode {
            FakeMode::Text => {}
            FakeMode::Fail => return Err(Failure::new("boom")),
            FakeMode::Loop => {
                tp.process().transferring(start, progress)?;
            }
            FakeMode::Nested => {
                let shared = start
                    .cast::<FakeEntity>()
                    .get()
                    .map(|e| e.shared.read().clone())
                    .unwrap_or_default();
                for ent in shared {
                    tp.process().transferring(&ent, progress)?;
                }
            }
        }
        Ok(text_result(start))
    }
}

fn result_text(
    tp: &TransferProcess<TransferTransientKind>,
    start: &Handle<StandardTransient>,
) -> String {
    let res = tp.find_transient(start);
    res.cast::<HAsciiString>()
        .get()
        .map(|t| String::from_utf8_lossy(t.string().as_bytes()).into_owned())
        .unwrap_or_default()
}

#[test]
fn binding_finding_and_roots() {
    let tp = TransferProcessForTransient::new(10);
    let (a, b) = (fake_entity("a", &[]), fake_entity("b", &[]));
    assert!(tp.find(&a).is_null() && !tp.is_bound(&a));
    assert_eq!(
        tp.is_already_used(&a).unwrap_err().exception_type(),
        "Transfer_TransferFailure"
    );
    tp.bind_transient(&a, &Handle::new(HAsciiString::new("ra")).cast())
        .unwrap();
    assert!(tp.is_bound(&a) && !tp.is_already_used(&a).unwrap());
    tp.add_warning_text(&b, "careful", "");
    assert!(!tp.is_bound(&b) && tp.check(&b).has_warnings());
    tp.bind_transient(&b, &Handle::new(HAsciiString::new("rb")).cast())
        .unwrap();
    assert!(
        tp.check(&b).has_warnings(),
        "the void binder gives its check"
    );
    assert_eq!((tp.nb_mapped(), tp.map_index(&b)), (2, 2));
    assert!(tp.mapped(2).unwrap() == b && tp.mapped(3).is_err() && tp.map_item(0).is_err());
    tp.set_root(&b);
    assert_eq!(
        (tp.nb_roots(), tp.root_index(&b), tp.root_index(&a)),
        (1, 1, 0)
    );
    assert!(tp.root(1).unwrap() == b && tp.root(2).is_err() && tp.root_item(1).unwrap().is_some());
    assert!(tp.unbind(&b) && tp.nb_roots() == 0);
    assert!(tp.is_bound(&b), "native quirk: Unbind keeps the binder");
    assert!(!tp.unbind(&fake_entity("c", &[])));
    tp.clean();
    tp.resize(100);
    tp.clear();
    assert_eq!(tp.nb_mapped(), 0);
}

#[test]
fn multiple_results_and_used_binders() {
    let tp = TransferProcessForTransient::new(10);
    let a = fake_entity("a", &[]);
    assert_eq!(
        tp.add_multiple(&a, &Handle::default())
            .unwrap_err()
            .message(),
        "TransferProcess : AddMultiple, nothing bound"
    );
    tp.bind_multiple(&a).unwrap();
    tp.bind_multiple(&a).unwrap();
    tp.add_multiple(&a, &Handle::new(StandardTransient::default()))
        .unwrap();
    assert!(
        tp.find(&a)
            .cast::<TransferMultipleBinder>()
            .get()
            .is_some_and(|m| m.nb_results() == 1)
    );
    let b = fake_entity("b", &[]);
    tp.bind_transient(&b, &Handle::new(StandardTransient::default()))
        .unwrap();
    assert!(tp.bind_multiple(&b).is_err());
    assert!(tp.add_multiple(&b, &Handle::default()).is_err());
    tp.find(&b).set_already_used();
    let err = tp
        .bind(&b, &Handle::new(TransferVoidBinder::new()).cast())
        .unwrap_err();
    assert_eq!(err.message(), "TransferProcess : Bind, already Bound");
    let mut val = Handle::default();
    assert!(
        tp.find_typed_transient(&b, StandardTransient::static_type(), &mut val) && val.is_some()
    );
    let mut typed: Handle<HAsciiString> = Handle::default();
    assert!(
        !tp.find_typed_transient_as(&b, StandardTransient::static_type(), &mut typed)
            && typed.is_null()
    );
    let mut any: Handle<StandardTransient> = Handle::default();
    assert!(
        tp.get_typed_transient_as(&tp.find(&b), StandardTransient::static_type(), &mut any)
            && any.is_some()
    );
    assert!(tp.find_else_bind(&fake_entity("c", &[])).is_some() && tp.nb_mapped() == 3);
}

#[test]
fn transfer_through_actors_with_roots() {
    let tp = TransferProcessForTransient::new(10);
    tp.set_root_management(true);
    let last = fake_actor(FakeMode::Text);
    last.set_last(true);
    tp.set_actor(&last);
    tp.set_actor(&fake_actor(FakeMode::Nested));
    assert!(tp.actor().next() == last);
    let (_, ents) = fake_chain();
    let progress = ProgressRange::default();
    assert!(tp.recognize(&ents[3]) && !tp.recognize(&Handle::new(StandardTransient::default())));
    assert!(tp.transfer(&ents[3], &progress).unwrap());
    assert_eq!(result_text(&tp, &ents[3]), "d");
    assert_eq!(result_text(&tp, &ents[0]), "a");
    // every nested transfer ends at a level its root level reaches: all are roots
    assert_eq!((tp.nb_mapped(), tp.nb_roots()), (4, 4));
    assert!(
        tp.root(1).unwrap() == ents[0] && tp.root(4).unwrap() == ents[3] && tp.nesting_level() == 0
    );
    // a second transfer marks the result used
    tp.transferring(&ents[3], &progress).unwrap();
    assert_eq!(tp.find(&ents[3]).status(), TransferStatusResult::StatusUsed);
    assert_eq!(tp.complete_result(true).number(), 4);
    assert_eq!(tp.root_result(false).number(), 4);
    assert_eq!(tp.abnormal_result().number(), 0);
    assert!(
        !tp.transfer(&Handle::new(StandardTransient::default()), &progress)
            .unwrap()
    );
}

#[test]
fn failures_are_caught_or_returned() {
    let tp = TransferProcessForTransient::new(10);
    tp.set_actor(&fake_actor(FakeMode::Fail));
    let a = fake_entity("a", &[]);
    let progress = ProgressRange::default();
    assert!(tp.transferring(&a, &progress).unwrap().is_null());
    let check = tp.check(&a);
    assert!(check.has_failed() && tp.nesting_level() == 0);
    let list = tp.check_list(true);
    assert_eq!(list.items().len(), 1);
    assert_eq!(list.items()[0].0, 1, "no CheckNum: the map index");
    tp.set_error_handle(false);
    let b = fake_entity("b", &[]);
    assert_eq!(
        tp.transferring(&b, &progress).unwrap_err().message(),
        "boom"
    );
    assert!(!tp.error_handle() && tp.nesting_level() == 1);
    tp.reset_nesting_level();
    // a binder in error raises a failure
    tp.find_else_bind(&b)
        .set_status_exec(TransferStatusExec::StatusError);
    let err = tp.transferring(&b, &progress).unwrap_err();
    assert_eq!(err.message(), "TransferProcess : Transfer in Error Status");
    assert_eq!(
        tp.abnormal_result().number(),
        2,
        "the fail of a sets its status in error"
    );
}

#[test]
fn dead_loop_is_detected() {
    let tp = TransferProcessForTransient::new(10);
    tp.set_actor(&fake_actor(FakeMode::Loop));
    let a = fake_entity("a", &[]);
    let progress = ProgressRange::default();
    // a loop is seen only on a bound object (a running binder): bind a void one first
    tp.find_else_bind(&a);
    let binder = tp.transferring(&a, &progress).unwrap();
    assert!(binder.is_some() && result_text(&tp, &a) == "a");
    assert_eq!(
        binder.status_exec(),
        TransferStatusExec::StatusLoop,
        "merged from the looping void binder"
    );
    assert_eq!(tp.abnormal_result().number(), 1);
    let list = tp.check_list(false);
    assert!(
        list.items()[0].1.has_failed(),
        "abnormal status: a fail is added"
    );
    assert!(!tp.is_looping(1) && tp.is_looping(2));
}

#[test]
fn messages_and_traces() {
    let tp = TransferProcessForTransient::with_messenger(None, 10);
    tp.set_trace_level(4);
    assert_eq!(tp.trace_level(), 4);
    let a = fake_entity("a", &[]);
    tp.add_fail_text(&a, "bad", "orig");
    tp.add_error(&a, "worse", "");
    tp.send_warning(&a, &Msg::new());
    tp.send_fail(&a, &Msg::new());
    tp.send_msg(&a, &Msg::new());
    tp.add_warning_msg(&a, &Msg::new());
    assert!(tp.check(&a).has_failed());
    tp.mend(&a, "CA").unwrap();
    assert!(!tp.check(&a).has_failed());
    tp.set_messenger(None);
    let mut text = Vec::new();
    tp.print_trace(&a, &mut text).unwrap();
    assert_eq!(String::from_utf8_lossy(&text), " Type:Fake_Entity");
    assert_eq!(tp.check_num(&a), 0);
    assert!(
        tp.result_one(&a, 0, true).number() == 0
            && tp.check_list_one(&a, 1, false).items().is_empty()
    );
    assert!(
        tp.is_check_list_empty(&a, 0, true)
            && !tp.is_check_list_empty(&fake_entity("z", &[]), 0, true)
    );
    tp.remove_result(&a, 0, true);
}

#[test]
fn transient_process_model_graph_and_context() {
    let tp = TransferTransientProcess::new(10);
    let (model, ents) = fake_chain();
    tp.set_graph(&Handle::new(InterfaceHGraph::new(&model, true).unwrap()));
    assert!(tp.has_graph() && tp.model().is_some() && tp.graph().unwrap().get().size() == 4);
    assert_eq!(
        tp.typed_sharings(&ents[0], FakeEntity::static_type())
            .unwrap()
            .nb_entities(),
        1
    );
    assert_eq!(tp.check_num(&ents[2]), 3);
    assert!(tp.is_data_loaded(&ents[0]) && !tp.is_data_fail(&ents[0]));
    let mut text = Vec::new();
    tp.print_trace(&ents[0], &mut text).unwrap();
    assert!(String::from_utf8_lossy(&text).starts_with("Entity "));
    tp.set_context("ctx", &ents[1]);
    let mut ctx = Handle::default();
    assert!(tp.get_context("ctx", Some(FakeEntity::static_type()), &mut ctx) && ctx == ents[1]);
    assert!(!tp.get_context("ctx", Some(HAsciiString::static_type()), &mut ctx) && ctx.is_null());
    assert!(!tp.get_context("none", None, &mut ctx));
    assert_eq!(tp.context().len(), 1);
    tp.roots_for_transfer().write().push(ents[3].clone());
    assert_eq!(tp.roots_for_transfer().read().len(), 1);
    tp.set_graph(&Handle::default());
    assert!(tp.model().is_null() && tp.graph().is_err());
    tp.set_model(&model);
    tp.add_fail_text(&ents[1], "bad", "");
    assert_eq!(
        tp.check_list(true).items()[0].0,
        2,
        "CheckNum: the model number, not the map index"
    );
    let mut stats = Vec::new();
    tp.print_stats(1, &mut stats).unwrap();
    let stats = String::from_utf8_lossy(&stats);
    assert!(
        stats.contains("Nb Entities         : 4") && stats.contains("Errors on :    1 Entities")
    );
}

#[test]
fn finder_process_mappers() {
    let fp = TransferFinderProcess::new(10);
    let obj = Handle::new(StandardTransient::default());
    let mapper = fp.transient_mapper(&obj);
    let finder: Handle<dyn TransferFinder> = mapper.cast();
    fp.bind_transient(&finder, &Handle::new(StandardTransient::default()))
        .unwrap();
    let again = fp.transient_mapper(&obj);
    assert!(again == mapper && fp.nb_mapped() == 1);
    mapper.set_integer_attribute("n", 1);
    assert_eq!(
        (
            fp.next_mapped_with_attribute("n", 0),
            fp.next_mapped_with_attribute("n", 1)
        ),
        (1, 0)
    );
    let mut text = Vec::new();
    fp.print_trace(&finder, &mut text).unwrap();
    assert_eq!(String::from_utf8_lossy(&text), " Type:Standard_Transient");
    fp.set_model(&Handle::default());
    assert!(fp.model().is_null());
    fp.add_fail_text(&finder, "bad", "");
    let mut stats = Vec::new();
    fp.print_stats(1, &mut stats).unwrap();
    assert!(String::from_utf8_lossy(&stats).contains("Errors on :   1 Entities"));
    let mut short = Vec::new();
    fp.print_stats(0, &mut short).unwrap();
    assert_eq!(short.len(), 70);
}

#[test]
fn process_actors_wrap_transient_results() {
    let tp = TransferTransientProcess::new(10);
    let actor = TransferActorOfTransientProcess::new();
    let ops: Handle<dyn TransferProcessOps<TransferTransientKind>> = tp.cast();
    let a = fake_entity("a", &[]);
    let progress = ProgressRange::default();
    assert!(actor.transferring(&a, &ops, &progress).unwrap().is_null());
    assert!(actor.recognize(&a) && actor.transient_result(&Handle::default()).is_null());
    actor.shape_processing().set_flags(Default::default());
    assert!(
        actor.shape_processing().flags().1
            && actor.shape_processing().shape_fix_parameters().is_empty()
    );
    let fp = TransferFinderProcess::new(10);
    let factor = TransferActorOfFinderProcess::new();
    let fops: Handle<dyn TransferProcessOps<TransferFinderKind>> = fp.cast();
    let finder: Handle<dyn TransferFinder> = fp.transient_mapper(&a).cast();
    assert!(
        factor
            .transferring(&finder, &fops, &progress)
            .unwrap()
            .is_null()
    );
    assert_eq!(*factor.mode_trans(), 0);
}

#[test]
fn iterator_selections() {
    let mut iter = TransferIteratorOfProcess::<TransferTransientKind>::new(true);
    let (a, b, c) = (
        fake_entity("a", &[]),
        fake_entity("b", &[]),
        fake_entity("c", &[]),
    );
    assert!(iter.add(&text_result(&a)).is_err());
    iter.add_with_start(&text_result(&a), &a);
    iter.add_with_start(&Handle::new(TransferVoidBinder::new()).cast(), &b);
    iter.add_with_start(&Handle::new(TransferMultipleBinder::new()).cast(), &c);
    assert_eq!(iter.number(), 3);
    iter.start();
    assert!(
        iter.has_result().unwrap()
            && iter.has_unique_result().unwrap()
            && iter.has_transient_result().unwrap()
    );
    assert!(
        iter.transient_result().unwrap().is_some()
            && iter.status().unwrap() == TransferStatusExec::StatusDone
    );
    assert!(
        !iter.has_fails().unwrap()
            && !iter.has_warnings().unwrap()
            && iter.check().unwrap().is_some()
    );
    iter.select_result(HAsciiString::static_type(), true)
        .unwrap();
    assert_eq!(iter.number(), 1);
    iter.select_item(2, true);
    iter.select_binder(TransferVoidBinder::static_type(), false)
        .unwrap();
    assert_eq!(iter.number(), 1);
    iter.select_unique(false).unwrap();
    assert_eq!(iter.number(), 0);
    iter.filter(Some(std::slice::from_ref(&b)), true);
    iter.start();
    assert!(!iter.more(), "the maximum stays lowered");
    let mut fresh = TransferIteratorOfProcess::<TransferTransientKind>::new(true);
    for ent in [&a, &b, &c] {
        fresh.add_with_start(&text_result(ent), ent);
    }
    fresh.filter(Some(std::slice::from_ref(&b)), true);
    fresh.start();
    assert!(fresh.more() && fresh.starting().unwrap() == b && fresh.number() == 1);
    fresh.next();
    assert!(!fresh.more() && fresh.value().is_err());
    let plain = TransferIteratorOfProcess::<TransferTransientKind>::new(false);
    assert!(!plain.has_starting() && plain.starting().is_err());
}
