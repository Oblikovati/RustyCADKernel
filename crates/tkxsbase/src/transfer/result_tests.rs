//! Unit tests of the transfer results and of `Transfer_TransferOutput`.
use super::*;
use crate::interface::test_norm::{fake_chain, fake_protocol};
use crate::interface::{InterfaceCheckStatus, InterfaceGraph, InterfaceModelEntities};
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    standard::{Failure, Handle, StandardTransient},
};

/// An actor of transient processes that gives each entity itself as result.
#[derive(Default)]
struct EchoActor {
    base: TransferActorOfTransientProcess,
}
impl_transient!(EchoActor, "Echo_Actor", TransferActorOfTransientProcess, base, interfaces: [dyn TransferActorOps<TransferTransientKind>, dyn TransferActorOfTransientProcessOps]);

impl TransferActorOps<TransferTransientKind> for EchoActor {
    fn actor(&self) -> &TransferActor<TransferTransientKind> {
        self.base.actor()
    }
    fn transferring(
        &self,
        start: &Handle<StandardTransient>,
        tp: &Handle<dyn TransferProcessOps<TransferTransientKind>>,
        progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        transferring_by_transient_process(self, start, tp, progress)
    }
}

impl TransferActorOfTransientProcessOps for EchoActor {
    fn shape_processing(&self) -> &TransferShapeProcessing {
        self.base.shape_processing()
    }
    fn transfer_transient(
        &self,
        start: &Handle<StandardTransient>,
        tp: &Handle<TransferTransientProcess>,
        _progress: &ProgressRange<'_>,
    ) -> Result<Handle<StandardTransient>, Failure> {
        assert!(tp.is_some(), "the actor receives the transient process");
        Ok(start.clone())
    }
}

fn sub(start: &Handle<StandardTransient>) -> Handle<TransferResultFromTransient> {
    let res = Handle::new(TransferResultFromTransient::new());
    res.set_start(start);
    res
}

#[test]
fn result_from_transient_tree() {
    let (_, ents) = fake_chain();
    let main = sub(&ents[3]);
    let child = sub(&ents[2]);
    let leaf = sub(&ents[0]);
    child.add_sub_result(&leaf);
    main.add_sub_result(&child);
    main.add_sub_result(&Handle::default());
    assert_eq!(
        (main.nb_sub_results(), main.sub_result(2).is_null()),
        (1, true)
    );
    assert!(TransferResultFromTransient::result_from_key(&main, &ents[0]) == leaf);
    assert!(TransferResultFromTransient::result_from_key(&main, &ents[1]).is_null());
    let mut map = rustycad_tkernel::ncollection::IndexedMap::new(10);
    main.fill_map(&mut map);
    assert_eq!(map.extent(), 2);
    assert!(main.check().is_some() && main.check_status() == InterfaceCheckStatus::CheckOK);
    let binder: Handle<dyn TransferBinder> = Handle::new(TransferVoidBinder::new()).cast();
    binder.add_warning("w", "");
    main.set_binder(&binder);
    assert!(main.binder() == binder && main.check_status() == InterfaceCheckStatus::CheckWarning);
    main.fill(&Handle::default());
    main.strip();
    main.fill_back(&Handle::default());
    main.clear_subs();
    assert_eq!(main.nb_sub_results(), 0);
}

#[test]
fn result_from_model_lists_and_status() {
    let (model, ents) = fake_chain();
    let tp = TransferTransientProcess::new(10);
    tp.set_model(&model);
    let result = TransferResultFromModel::new();
    assert!(
        !result.fill(&tp, &ents[3])
            && result.results(1).is_err()
            && result.results(0).unwrap()[0].is_null()
    );
    tp.bind_transient(&ents[3], &ents[3]).unwrap();
    tp.add_warning_text(&ents[3], "w", "");
    assert!(result.fill(&tp, &ents[3]) && result.has_result());
    assert_eq!((result.main_number(), result.model().is_some()), (4, true));
    let main = result.main_result();
    main.add_sub_result(&sub(&ents[2]));
    assert_eq!(
        result.results(1).unwrap().len(),
        3,
        "native quirk: the main result twice"
    );
    assert_eq!(result.results(2).unwrap().len(), 2);
    assert_eq!(result.transferred_list(2).unwrap(), vec![ents[3].clone()]);
    assert_eq!(
        result
            .checked_list(InterfaceCheckStatus::CheckWarning, true)
            .unwrap()
            .len(),
        1
    );
    let list = result.check_list(false, 2).unwrap();
    assert_eq!(list.items()[0].0, 4);
    assert_eq!(
        result.check_status().unwrap(),
        InterfaceCheckStatus::CheckWarning
    );
    assert!(result.result_from_key(&ents[2]).unwrap().is_some());
    result.fill_back(&tp).unwrap();
    result.strip(11);
    assert!(result.main_result().is_null() && result.model().is_null());
    assert_eq!(
        result.compute_check_status(true).unwrap(),
        InterfaceCheckStatus::CheckWarning
    );
    assert!(result.result_from_key(&ents[2]).is_err() && result.fill_back(&tp).is_err());
    result.set_model(&model);
    result.set_main_result(&sub(&ents[1]));
    assert_eq!(result.main_number(), 2);
    assert!(!result.main_label().as_bytes().is_empty());
}

#[test]
fn transfer_output_roots_and_status() {
    let (model, ents) = fake_chain();
    let actor = Handle::new(EchoActor::default());
    let output = TransferOutput::with_actor(&actor, &model).unwrap();
    let progress = ProgressRange::default();
    output
        .transfer_roots_protocol(&fake_protocol(), &progress)
        .unwrap();
    let tp = output.transient_process();
    assert_eq!((tp.nb_roots(), tp.root(1).unwrap() == ents[3]), (1, true));
    assert_eq!(output.list_for_status(true, true).unwrap().nb_entities(), 1);
    assert_eq!(
        output.list_for_status(false, false).unwrap().nb_entities(),
        0
    );
    output.transfer(&ents[0], &progress).unwrap();
    assert_eq!(tp.nb_mapped(), 2);
    let foreign = crate::interface::test_norm::fake_entity("x", &[]);
    assert_eq!(
        output
            .transfer(&foreign, &progress)
            .unwrap_err()
            .exception_type(),
        "Transfer_TransferFailure"
    );
    let graph = InterfaceGraph::new(&model, true).unwrap();
    output.transfer_roots_graph(&graph, &progress).unwrap();
    assert!(tp.model().is_some());
    let newmod = output
        .model_for_status(&fake_protocol(), true, false)
        .unwrap();
    assert_eq!(newmod.nb_entities(), 4, "d with its references, and a");
    assert!(TransferOutput::with_actor(&actor, &Handle::default()).is_err());
    let plain = TransferOutput::with_process(&tp, &Handle::default());
    assert!(
        plain
            .model_for_status(&fake_protocol(), true, true)
            .unwrap()
            .is_null()
    );
}
