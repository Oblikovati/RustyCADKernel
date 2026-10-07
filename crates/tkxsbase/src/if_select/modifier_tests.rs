//! Unit tests of the modifiers and of the modification and writing contexts over the fake
//! chain (d shares c, c shares a and b).
use super::*;
use crate::interface::test_norm::{fake_chain, fake_entity, fake_model, fake_protocol};
use crate::interface::{
    InterfaceCheck, InterfaceCopyTool, InterfaceEntityIterator, InterfaceGraph, InterfaceHGraph,
    InterfaceModelEntities,
};
use rustycad_tkernel::standard::Handle;

#[test]
fn context_modif_without_copy() {
    let (model, ents) = fake_chain();
    let g = InterfaceGraph::new(&model, true).unwrap();
    let mut ctx = IFSelectContextModif::new(&g, "file").unwrap();
    assert!(
        ctx.is_for_all() && !ctx.is_for_none() && ctx.has_file_name() && ctx.control().is_null()
    );
    assert_eq!(
        (ctx.selected_count(), ctx.file_name().as_bytes()),
        (4, &b"file"[..])
    );
    assert!(ctx.original_model() == model && ctx.original_graph().size() == 4);
    let mut list = InterfaceEntityIterator::new();
    list.add_item(&ents[1]);
    ctx.select(&list).unwrap();
    assert!(!ctx.is_for_all() && ctx.is_selected(&ents[1]) && !ctx.is_selected(&ents[0]));
    assert_eq!(
        (
            ctx.selected_original().unwrap().nb_entities(),
            ctx.selected_result().unwrap().nb_entities()
        ),
        (1, 4)
    );
    assert!(ctx.is_transferred(&ents[0]).unwrap());
    ctx.start();
    assert!(
        ctx.more()
            && ctx.value_original().unwrap() == ents[1]
            && ctx.value_result().unwrap() == ents[1]
    );
    ctx.trace("note").unwrap();
    ctx.next();
    assert!(!ctx.more() && ctx.value_original().is_err());
    list.add_item(&fake_entity("x", &[]));
    assert!(
        ctx.select(&list).is_err(),
        "native defect: no copy map for an entity out of the graph"
    );
    ctx.set_protocol(&fake_protocol());
    assert!(ctx.protocol().is_some());
    ctx.trace_modifier(&Handle::new(IFSelectModifReorder::new(false)).cast());
    ctx.add_warning(&ents[0], "w", "");
    ctx.add_fail(&ents[0], "f", "");
    assert!(ctx.c_check(1).unwrap().entity() == ents[0]);
    assert!(
        ctx.c_check_entity(&fake_entity("y", &[]))
            .entity()
            .is_some()
    );
    let check = Handle::new(InterfaceCheck::new());
    check.add_fail_text("x", "");
    ctx.add_check(&check);
    ctx.add_check(&Handle::new(InterfaceCheck::new()));
    assert_eq!(ctx.check_list().items().len(), 3);
}

#[test]
fn context_modif_with_copy_and_reorder() {
    let (model, ents) = fake_chain();
    let g = InterfaceGraph::new(&model, true).unwrap();
    let mut tc = InterfaceCopyTool::with_protocol(&model, &fake_protocol()).unwrap();
    tc.transfer_entity(&ents[2]).unwrap();
    let mut ctx = IFSelectContextModif::with_copy(&g, &tc, "").unwrap();
    assert_eq!(ctx.selected_count(), 3, "c and the copied a and b");
    assert!(!ctx.is_transferred(&ents[3]).unwrap());
    assert_eq!(ctx.selected_result().unwrap().nb_entities(), 3);
    let target = fake_model();
    let modif = IFSelectModifReorder::new(true);
    assert!(modif.may_change_graph() && modif.label().as_bytes() == b"Reorder, Roots last");
    modif
        .perform(&mut ctx, &target, &fake_protocol(), &mut tc)
        .unwrap();
    assert_eq!(target.nb_entities(), 4);
    assert_eq!(
        IFSelectModifReorder::new(false).label().as_bytes(),
        b"Reorder, Roots first"
    );
    assert!(!modif.has_selection() && modif.dispatch().is_null());
    let disp: Handle<dyn IFSelectDispatch> = Handle::new(IFSelectDispGlobal::new()).cast();
    assert!(modif.applies(&disp));
    modif.set_dispatch(&disp);
    assert!(modif.applies(&disp) && !modif.applies(&Handle::new(IFSelectDispPerOne::new()).cast()));
    modif.set_selection(&Handle::new(IFSelectSelectModelRoots::new()).cast());
    assert!(modif.has_selection());
    modif.reset_selection();
    assert!(modif.selection().is_null());
}

#[test]
fn applied_modifiers_and_write_context() {
    let (model, ents) = fake_chain();
    let applied = Handle::new(IFSelectAppliedModifiers::new(1, 4).unwrap());
    let modif: Handle<dyn IFSelectGeneralModifier> =
        Handle::new(IFSelectModifReorder::new(true)).cast();
    assert!(
        applied.add_modif(&modif).unwrap()
            && applied.add_modif(&modif).unwrap()
            && !applied.add_modif(&modif).unwrap()
    );
    applied.add_num(3).unwrap();
    assert!(applied.item(3).unwrap().is_none());
    let (first, count) = applied.item(2).unwrap().unwrap();
    assert!(first == modif && count == 1 && !applied.is_for_all());
    assert_eq!(
        (applied.item_num(1).unwrap(), applied.item_list().unwrap()),
        (3, vec![3])
    );
    let mut ctx = IFSelectContextWrite::new(&model, &fake_protocol(), &applied, "out");
    assert!(ctx.model() == model && ctx.protocol().is_some() && ctx.applied_modifiers() == applied);
    assert_eq!(ctx.nb_modifiers(), 2);
    assert!(!ctx.set_modifier(3).unwrap() && ctx.set_modifier(2).unwrap());
    assert!(
        ctx.file_modifier() == modif
            && !ctx.is_for_none()
            && !ctx.is_for_all().unwrap()
            && ctx.nb_entities() == 1
    );
    ctx.start();
    assert!(ctx.more() && ctx.value().unwrap() == ents[2]);
    ctx.next();
    assert!(!ctx.more() && ctx.value().is_err());
    assert_eq!(ctx.graph().unwrap().size(), 4);
    ctx.add_warning(&ents[0], "w", "").unwrap();
    ctx.add_fail(&ents[1], "f", "").unwrap();
    assert!(ctx.c_check(1).unwrap().entity() == ents[0]);
    assert!(
        ctx.c_check_entity(&fake_entity("z", &[]))
            .unwrap()
            .entity()
            .is_some()
    );
    let check = Handle::new(InterfaceCheck::new());
    check.add_warning_text("x", "");
    ctx.add_check(&check).unwrap();
    assert_eq!(ctx.check_list().items().len(), 4);
    let hgraph = Handle::new(InterfaceHGraph::new(&model, true).unwrap());
    let from_graph =
        IFSelectContextWrite::with_graph(&hgraph, &fake_protocol(), &Handle::default(), "")
            .unwrap();
    assert!(from_graph.model() == model && from_graph.is_for_all().is_err());
    assert!(
        IFSelectContextWrite::with_graph(
            &Handle::default(),
            &fake_protocol(),
            &Handle::default(),
            ""
        )
        .is_err()
    );
}
