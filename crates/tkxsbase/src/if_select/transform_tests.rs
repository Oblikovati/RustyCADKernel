//! Unit tests of the work library defaults and of the standard transformer over the fake
//! chain (d shares c, c shares a and b).
use super::test_work_library::fake_work_library;
use super::*;
use crate::interface::test_norm::{fake_chain, fake_model, fake_protocol};
use crate::interface::{
    InterfaceCheckIterator, InterfaceCopyTool, InterfaceEntityIterator, InterfaceGraph,
    InterfaceModelEntities,
};
use rustycad_tkernel::standard::{Handle, StandardTransient};

#[test]
fn work_library_dump_levels_and_helps() {
    let wl = fake_work_library();
    let (model, ents) = fake_chain();
    assert_eq!(wl.dump_levels(), (0, -1));
    wl.set_dump_levels(2, 3);
    assert_eq!(wl.dump_levels(), (2, 3));
    wl.set_dump_help(1, "short");
    wl.set_dump_help(4, "ignored");
    assert_eq!(wl.dump_help(1).as_bytes(), b"short");
    assert!(wl.dump_help(4).is_empty() && wl.dump_help(-1).is_empty());
    let mut out = Vec::new();
    wl.dump_entity_default(&model, &fake_protocol(), &ents[0], &mut out)
        .unwrap();
    assert_eq!(out, b"level 2");
    wl.set_dump_levels(5, -1);
    out.clear();
    wl.dump_entity_default(&model, &fake_protocol(), &ents[0], &mut out)
        .unwrap();
    assert_eq!(out, b"level 0", "no levels: the default level is 0");
}

#[test]
fn work_library_stream_and_copy_defaults() {
    let wl = fake_work_library();
    let (model, ents) = fake_chain();
    let mut read = Handle::default();
    let mut input: &[u8] = b"";
    assert_eq!(
        wl.read_stream("s", &mut input, &mut read, &fake_protocol())
            .unwrap(),
        1
    );
    let mut ctx = IFSelectContextWrite::new(&model, &fake_protocol(), &Handle::default(), "f");
    assert!(!wl.write_stream(&mut ctx, &mut Vec::new()).unwrap());
    let newmod = fake_model();
    let mut tc = InterfaceCopyTool::with_protocol(&model, &fake_protocol()).unwrap();
    let mut list = InterfaceEntityIterator::new();
    list.add_item(&ents[2]);
    assert!(wl.copy_model(&model, &newmod, &list, &mut tc).unwrap());
    assert_eq!(newmod.nb_entities(), 3, "c with a and b");
}

#[test]
fn transform_standard_modifiers() {
    let trf = IFSelectTransformStandard::new();
    assert!(trf.copy_option() && trf.selection().is_null());
    let first: Handle<dyn IFSelectModifier> = Handle::new(IFSelectModifReorder::new(true)).cast();
    let second: Handle<dyn IFSelectModifier> = Handle::new(IFSelectModifReorder::new(false)).cast();
    assert!(trf.add_modifier(&first, 0) && !trf.add_modifier(&second, 2));
    assert_eq!(
        trf.label().as_bytes(),
        b"Standard Copy - Reorder, Roots last"
    );
    assert!(trf.add_modifier(&second, 1));
    assert_eq!(
        (trf.modifier_rank(&first), trf.modifier_rank(&second)),
        (2, 1)
    );
    assert_eq!(trf.label().as_bytes(), b"Standard Copy - 2 Modifiers");
    assert!(trf.modifier(3).is_err() && trf.modifier(1).unwrap() == second);
    assert!(trf.remove_modifier(&second) && !trf.remove_modifier_at(2));
    trf.set_copy_option(false);
    assert_eq!(
        trf.label().as_bytes(),
        b"On the spot Edition - Reorder, Roots last"
    );
    trf.set_selection(&Handle::new(IFSelectSelectModelRoots::new()).cast());
    assert!(trf.selection().is_some());
}

#[test]
fn transform_standard_copies() {
    let (model, ents) = fake_chain();
    let g = InterfaceGraph::new(&model, true).unwrap();
    let trf = IFSelectTransformStandard::new();
    let mut tc = InterfaceCopyTool::with_protocol(&model, &fake_protocol()).unwrap();
    let copy = trf.copy(&g, &mut tc).unwrap();
    assert!(copy != model && copy.nb_entities() == 4);
    trf.set_copy_option(false);
    let mut tc = InterfaceCopyTool::with_protocol(&model, &fake_protocol()).unwrap();
    assert!(trf.copy(&g, &mut tc).unwrap() == model);
    let mut image: Handle<StandardTransient> = Handle::default();
    assert!(tc.search(&ents[0], &mut image).unwrap() && image == ents[0]);
}

#[test]
fn transform_standard_perform() {
    let (model, ents) = fake_chain();
    let g = InterfaceGraph::new(&model, true).unwrap();
    let trf = IFSelectTransformStandard::new();
    let (mut checks, mut newmod) = (InterfaceCheckIterator::new(), Handle::default());
    assert!(
        trf.perform(&g, &fake_protocol(), &mut checks, &mut newmod)
            .unwrap()
    );
    assert!(newmod.is_some() && newmod != model);
    let mut image: Handle<StandardTransient> = Handle::default();
    assert!(trf.updated(&ents[0], &mut image) && image.is_some() && image != ents[0]);
    trf.set_copy_option(false);
    assert!(
        trf.perform(&g, &fake_protocol(), &mut checks, &mut newmod)
            .unwrap()
    );
    assert!(
        newmod.is_null(),
        "on the spot without a graph change: no new model"
    );
    trf.add_modifier(&Handle::new(IFSelectModifReorder::new(true)).cast(), 0);
    assert!(
        trf.perform(&g, &fake_protocol(), &mut checks, &mut newmod)
            .unwrap()
    );
    assert!(newmod == model, "a reorder may change the graph");
    trf.add_modifier(&Handle::default(), 0);
    assert!(
        trf.perform(&g, &fake_protocol(), &mut checks, &mut newmod)
            .is_err()
    );
}
