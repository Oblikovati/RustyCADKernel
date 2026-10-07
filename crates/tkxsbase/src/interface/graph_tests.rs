//! Unit tests of `Interface_Graph`, `Interface_HGraph`, `Interface_ShareTool`,
//! `Interface_CopyTool` and `Interface_CheckIterator` on the fake norm.
use super::test_norm::{FakeEntity, fake_chain, fake_entity, fake_model};
use super::*;
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardTransient};

#[test]
fn sharings_and_roots() {
    let (model, ents) = fake_chain();
    let graph = InterfaceGraph::new(&model, true).unwrap();
    assert_eq!(graph.size(), 4);
    assert_eq!(graph.shareds(&ents[2]).unwrap().nb_entities(), 2);
    let sharings = graph.get_sharings(&ents[0]).unwrap().unwrap();
    assert!(sharings == vec![ents[2].clone()]);
    assert!(
        graph
            .get_sharings(&fake_entity("x", &[]))
            .unwrap()
            .is_none()
    );
    let roots = graph.root_entities().unwrap().content();
    assert!(roots == vec![ents[3].clone()]);
    let typed = graph
        .typed_sharings(&ents[0], FakeEntity::static_type())
        .unwrap();
    assert_eq!(typed.nb_entities(), 1);
    assert!(!graph.has_share_errors(&ents[0]).unwrap());
    assert!(graph.name(&ents[0]).unwrap().is_null());
    assert_eq!(
        graph
            .name(&fake_entity("out", &[]))
            .unwrap()
            .string()
            .as_bytes(),
        b"out"
    );
    assert_eq!(graph.sharing_table().unwrap().len(), 4);
}

#[test]
fn share_errors_for_entities_out_of_the_model() {
    let model = fake_model();
    let outside = fake_entity("outside", &[]);
    model.add_entity(&fake_entity("a", &[outside])).unwrap();
    let graph = InterfaceGraph::new(&model, true).unwrap();
    assert!(graph.has_share_errors(&graph.entity(1).unwrap()).unwrap());
    let without = InterfaceGraph::new(&model, false).unwrap();
    assert!(!without.mode_stat() && !without.has_share_errors(&graph.entity(1).unwrap()).unwrap());
}

#[test]
fn statuses_and_presence() {
    let (model, ents) = fake_chain();
    let mut graph = InterfaceGraph::new(&model, true).unwrap();
    assert!(!graph.is_present(1).unwrap());
    graph.get_from_entity(&ents[2], true, 7).unwrap();
    assert!(
        graph.is_present(1).unwrap()
            && graph.is_present(3).unwrap()
            && !graph.is_present(4).unwrap()
    );
    assert_eq!(graph.status(1).unwrap(), 7);
    graph
        .get_from_entity_overlap(&ents[0], false, 2, 10, true)
        .unwrap();
    assert_eq!(graph.status(1).unwrap(), 17);
    graph.change_status(7, 8).unwrap();
    assert_eq!(graph.status(3).unwrap(), 8);
    graph.remove_status(8).unwrap();
    assert!(!graph.is_present(3).unwrap());
    let mut other = InterfaceGraph::from_graph(&graph, true).unwrap();
    assert_eq!(other.nb_statuses(), 4);
    other.reset_status().unwrap();
    other.get_from_graph(&graph).unwrap();
    assert!(other.is_present(1).unwrap());
    other.get_from_model().unwrap();
    assert!(other.is_present(4).unwrap());
    let mut iter = InterfaceEntityIterator::new();
    iter.add_item(&ents[3]);
    graph.get_from_iter(&iter, 3).unwrap();
    assert_eq!(graph.status(4).unwrap(), 3);
    graph.get_from_iter_overlap(&iter, 3, 1, false).unwrap();
    graph.get_from_graph_status(&other, 0).unwrap();
    let (foreign, _) = fake_chain();
    let stranger = InterfaceGraph::new(&foreign, true).unwrap();
    assert_eq!(
        graph
            .get_from_graph(&stranger)
            .err()
            .unwrap()
            .exception_type(),
        "Standard_DomainError"
    );
    graph.remove_item(4).unwrap();
    graph.reset().unwrap();
    assert_eq!(graph.status(1).unwrap(), 0);
    graph.assign(&other).unwrap();
    assert!(graph.is_present(2).unwrap());
}

#[test]
fn share_tool_queries() {
    let (model, ents) = fake_chain();
    let tool = InterfaceShareTool::new(&model).unwrap();
    assert!(tool.is_shared(&ents[2]).unwrap() && !tool.is_shared(&ents[0]).unwrap());
    assert_eq!(
        tool.nb_typed_sharings(&ents[0], FakeEntity::static_type())
            .unwrap(),
        1
    );
    assert!(
        tool.typed_sharing(&ents[0], FakeEntity::static_type())
            .unwrap()
            == ents[2]
    );
    let none = tool
        .typed_sharing(&ents[3], FakeEntity::static_type())
        .err()
        .unwrap();
    assert_eq!(none.exception_type(), "Interface_InterfaceError");
    let all = tool.all(&ents[3], true).unwrap().content();
    assert!(
        all == vec![
            ents[3].clone(),
            ents[2].clone(),
            ents[0].clone(),
            ents[1].clone()
        ]
    );
    let whole = tool.all(&model.cast(), true).unwrap().content();
    assert_eq!(whole.len(), 4);
    assert_eq!(tool.sharings(&ents[1]).unwrap().nb_entities(), 1);
    assert_eq!(tool.shareds(&ents[3]).unwrap().nb_entities(), 1);
    let mut out = Vec::new();
    tool.print(&tool.root_entities().unwrap(), &mut out)
        .unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        " Nb.Entities : 1 :  n0/id:4:#4\n"
    );
    let graph = InterfaceGraph::new(&model, true).unwrap();
    assert!(InterfaceShareTool::from_graph(&graph).unwrap().model() == model);
}

#[test]
fn copy_tool_copies_with_references() {
    let (model, ents) = fake_chain();
    let mut tool = InterfaceCopyTool::with_protocol(&model, &model.protocol()).unwrap();
    let copy = tool.transferred(&ents[2]).unwrap();
    let copied = copy.cast::<FakeEntity>();
    assert_eq!(copied.name.read().as_str(), "copy of c");
    assert_eq!(copied.shared.read().len(), 2);
    // The shared entities are copied before the level is raised, so they are roots too
    // (as natively).
    assert_eq!(tool.root_result(false).unwrap().nb_entities(), 3);
    assert_eq!(tool.complete_result(false).unwrap().nb_entities(), 3);
    let mut res = Handle::default();
    assert!(tool.search(&ents[0], &mut res).unwrap());
    let (mut ent, mut found) = (Handle::default(), Handle::default());
    assert_eq!(tool.last_copied_after(0, &mut ent, &mut found).unwrap(), 1);
    let target = fake_model();
    tool.fill_model(&target).unwrap();
    assert_eq!(target.nb_entities(), 3);
    let bound = fake_entity("bound", &[]);
    tool.bind(&ents[3], &bound).unwrap();
    assert_eq!(
        tool.bind(&ents[3], &bound).err().unwrap().exception_type(),
        "Interface_InterfaceError"
    );
    tool.clear().unwrap();
    assert!(!tool.search(&ents[0], &mut res).unwrap());
    let text: Handle<StandardTransient> =
        Handle::new(rustycad_tkernel::tcollection::HAsciiString::new("t")).cast();
    let mut out = Handle::default();
    assert!(tool.copy(&text, &mut out, false, false).unwrap());
    tool.transfer_entity(&ents[3]).unwrap();
    assert!(tool.control().is_some());
    InterfaceProtocolBase::clear_active();
    assert_eq!(
        InterfaceCopyTool::new(&model)
            .err()
            .unwrap()
            .exception_type(),
        "Interface_InterfaceError"
    );
}

#[test]
fn check_iterator_lists_and_prints() {
    let (model, ents) = fake_chain();
    let mut list = InterfaceCheckIterator::with_name("checks");
    list.set_model(&model);
    let first = Handle::new(InterfaceCheck::with_entity(&ents[1]));
    first.add_fail_text("bad", "");
    list.add(&first, 0);
    let again = Handle::new(InterfaceCheck::with_entity(&ents[1]));
    again.add_warning_text("w", "");
    list.add(&again, 0);
    assert_eq!(list.items().len(), 1);
    assert_eq!(first.nb_warnings(), 1);
    assert_eq!(list.status(), InterfaceCheckStatus::CheckFail);
    assert!(
        list.complies(InterfaceCheckStatus::CheckFail)
            && !list.complies(InterfaceCheckStatus::CheckOK)
    );
    assert_eq!(
        list.extract(InterfaceCheckStatus::CheckFail).items().len(),
        1
    );
    assert_eq!(
        list.extract_message("w", 0, InterfaceCheckStatus::CheckWarning)
            .unwrap()
            .items()
            .len(),
        1
    );
    assert!(list.check_entity(&ents[1]) == first);
    assert_eq!(list.check(9).nb_fails(), 0);
    assert_eq!(list.checkeds(false, true).unwrap().unwrap().len(), 1);
    let mut out = Vec::new();
    list.print(&mut out, false, 0).unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(
        text.starts_with(" **  checks  **\nCheck:1 -- Entity (n0:id) 2:#2   Type:Entity\nbad\nw\n"),
        "{text}"
    );
    list.start();
    assert!(list.more() && list.number().unwrap() == 2);
    list.next();
    assert!(!list.more() && list.value().is_err());
    assert!(
        list.remove("w", 0, InterfaceCheckStatus::CheckWarning)
            .unwrap()
    );
    let created = list.c_check(5);
    assert!(list.c_check(5) == created);
    let by_entity = list.c_check_entity(&fake_entity("free", &[]));
    assert!(by_entity.is_some());
    assert!(!list.is_empty(false) && !list.is_empty(true));
    let mut merged = InterfaceCheckIterator::new();
    merged.merge(&list);
    assert!(merged.model() == model);
    list.clear();
    assert!(list.is_empty(false));
}
