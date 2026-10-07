//! Unit tests of the share-out, its packets and its result over the fake chain (d shares
//! c, c shares a and b).
use super::*;
use crate::interface::InterfaceGraph;
use crate::interface::test_norm::{fake_chain, fake_entity};
use rustycad_tkernel::{standard::Handle, tcollection::HAsciiString};

fn text(s: &str) -> Handle<HAsciiString> {
    Handle::new(HAsciiString::new(s))
}

fn global() -> Handle<dyn IFSelectDispatch> {
    let disp = IFSelectDispGlobal::new();
    disp.set_final_selection(&Handle::new(IFSelectSelectModelEntities::new()).cast());
    Handle::new(disp).cast()
}

#[test]
fn share_out_dispatches_and_modifiers() {
    let share = IFSelectShareOut::new();
    let (d1, d2) = (global(), global());
    share.add_dispatch(&d1);
    share.add_dispatch(&d2);
    share.add_dispatch(&Handle::default());
    assert_eq!((share.nb_dispatches(), share.dispatch_rank(&d2)), (2, 2));
    assert!(share.dispatch(3).is_err());
    let reorder: Handle<dyn IFSelectGeneralModifier> =
        Handle::new(IFSelectModifReorder::new(true)).cast();
    share.add_modifier_for(&reorder, 2, 0).unwrap();
    assert!(
        reorder.dispatch() == d2 && share.nb_modifiers(true) == 1 && share.nb_modifiers(false) == 0
    );
    share.add_modifier(&reorder, 0).unwrap();
    assert!(reorder.dispatch().is_null() && share.modifier_rank(&reorder).unwrap() == 1);
    share.add_modif(
        &Handle::new(IFSelectModifReorder::new(false)).cast(),
        true,
        1,
    );
    assert!(
        share.general_modifier(true, 2).unwrap() == reorder
            && share.model_modifier(1).unwrap().is_some()
    );
    assert!(
        share.change_modifier_rank(true, 1, 2)
            && share.general_modifier(true, 1).unwrap() == reorder
    );
    assert!(share.change_modifier_rank(true, 2, 1) && !share.change_modifier_rank(true, 0, 1));
    assert!(share.remove_item(&reorder.cast()).unwrap() && share.nb_modifiers(true) == 1);
    assert!(
        !share.remove_modifier(false, 1).unwrap()
            && share.add_modifier(&Handle::default(), 0).is_err()
    );
    share.set_last_run(1);
    assert!(!share.remove_dispatch(1) && share.last_run() == 1);
    share.clear_result(true);
    assert!(share.remove_item(&d1.cast()).unwrap() && share.nb_dispatches() == 1);
    assert!(!share.remove_item(&fake_entity("x", &[])).unwrap());
    share.clear(true);
    assert_eq!((share.nb_dispatches(), share.nb_modifiers(true)), (0, 1));
    share.clear(false);
    assert_eq!(share.nb_modifiers(true), 0);
}

#[test]
fn share_out_file_names() {
    let share = IFSelectShareOut::new();
    let disp = global();
    share.add_dispatch(&disp);
    assert!(!share.has_root_name(1) && share.root_name(1).is_null());
    assert!(share.set_root_name(1, &text("part")).unwrap() && share.has_root_name(1));
    assert!(
        !share.set_root_name(1, &text("Default")).unwrap()
            && !share.set_root_name(2, &text("x")).unwrap()
    );
    assert_eq!(
        (
            share.root_number(&text("part")).unwrap(),
            share.root_number(&text("Default")).unwrap()
        ),
        (1, -1)
    );
    share.set_prefix(&text("out/"));
    share.set_extension(&text(".stp"));
    assert_eq!(
        share.file_name(1, 1, 0).unwrap().as_bytes(),
        b"out/part.stp"
    );
    assert_eq!(
        share.file_name(1, 2, 12).unwrap().as_bytes(),
        b"out/part_02.stp"
    );
    assert_eq!(
        share.file_name(1, 3, 0).unwrap().as_bytes(),
        b"out/part_3.stp"
    );
    assert_eq!(
        share.file_name(2, 1, 0).unwrap().as_bytes(),
        b"out/Default_1.stp"
    );
    assert_eq!(
        share.file_name(2, 1, 0).unwrap().as_bytes(),
        b"out/Default_2.stp"
    );
    assert!(
        share.set_default_root_name(&text("Other")).unwrap()
            && !share.set_default_root_name(&text("part")).unwrap()
    );
    assert_eq!(
        share.file_name(2, 1, 0).unwrap().as_bytes(),
        b"out/Other_1.stp"
    );
    assert_eq!(share.default_root_name().string().as_bytes(), b"Other");
    assert_eq!(
        (
            share.prefix().string().as_bytes(),
            share.extension().string().as_bytes()
        ),
        (&b"out/"[..], &b".stp"[..])
    );
    let empty = IFSelectShareOut::new();
    empty.set_prefix(&Handle::default());
    assert!(empty.prefix().string().is_empty() && empty.extension().string().is_empty());
}

#[test]
fn packet_lists() {
    let (model, ents) = fake_chain();
    let packets = IFSelectPacketList::new(&model).unwrap();
    assert_eq!(packets.name().as_bytes(), b"Packets");
    assert!(packets.add(&ents[0]).is_err(), "no packet yet");
    packets.add_packet().unwrap();
    packets
        .add_list(Some(&[ents[0].clone(), ents[0].clone(), ents[1].clone()]))
        .unwrap();
    packets.add_packet().unwrap();
    packets.add(&ents[0]).unwrap();
    assert!(packets.add(&fake_entity("x", &[])).is_err());
    assert_eq!(
        (
            packets.nb_packets(),
            packets.nb_entities(1).unwrap(),
            packets.nb_entities(3).unwrap()
        ),
        (2, 2, 0)
    );
    assert_eq!(
        packets.entities(2).unwrap().content(),
        vec![ents[0].clone()]
    );
    assert_eq!(
        (
            packets.highest_duplication_count(),
            packets.nb_duplicated(1, false),
            packets.nb_duplicated(1, true)
        ),
        (2, 1, 2)
    );
    assert_eq!(packets.duplicated(0, false).unwrap().nb_entities(), 2);
    packets.set_name("P");
    assert!(packets.name().as_bytes() == b"P" && packets.model() == model);
    assert!(IFSelectPacketList::new(&Handle::default()).is_err());
}

#[test]
fn share_out_results() {
    let (model, _) = fake_chain();
    let g = InterfaceGraph::new(&model, true).unwrap();
    let share = Handle::new(IFSelectShareOut::new());
    share.add_dispatch(&global());
    let mut result = IFSelectShareOutResult::with_share_out(&share, &model).unwrap();
    assert!(result.share_out() == share && result.graph().size() == 4);
    assert_eq!(result.nb_packets().unwrap(), 1);
    assert_eq!(
        (result.dispatch_rank(), result.packets_in_dispatch()),
        (1, (1, 1))
    );
    assert!(
        result.packet_root().is_err(),
        "no current packet before More"
    );
    assert!(result.more().unwrap());
    assert_eq!(
        result.packet_root().unwrap().nb_entities(),
        0,
        "native defect: the reset parts record no entity"
    );
    result.reset();
    assert_eq!(result.file_name().unwrap().as_bytes(), b"Default_1");
    let packets = result.packets(true).unwrap();
    assert_eq!(packets.nb_packets(), 1);
    result.reset();
    result.next_dispatch().unwrap();
    assert_eq!(result.dispatch_rank(), 0);
    let mut by_dispatch = IFSelectShareOutResult::with_dispatch_graph(&global(), &g).unwrap();
    assert!(by_dispatch.dispatch().is_some() && by_dispatch.file_name().is_err());
    by_dispatch.evaluate().unwrap();
    assert!(by_dispatch.more().unwrap());
    by_dispatch.next_packet().unwrap();
    assert!(!by_dispatch.more().unwrap() && by_dispatch.packet_content().is_err());
    assert!(IFSelectShareOutResult::with_dispatch(&global(), &model).is_ok());
    assert!(IFSelectShareOutResult::with_share_out_graph(&share, &g).is_ok());
}
