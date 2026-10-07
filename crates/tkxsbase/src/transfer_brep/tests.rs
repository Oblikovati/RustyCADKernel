//! Unit tests of the TransferBRep package.
use super::*;
use crate::interface::test_norm::{fake_chain, fake_entity};
use crate::interface::{InterfaceCheck, InterfaceCheckIterator};
use crate::transfer::{
    TransferBinder, TransferFinder, TransferFinderProcess, TransferSimpleBinderOfTransient,
    TransferTransientProcess,
};
use rustycad_tkbrep::topods::{Builder, Compound, HShape, Shape, Wire};
use rustycad_tkernel::{
    message::{Gravity, Msg, Printer, PrinterBase, PrinterHandle, PrinterType},
    standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use std::sync::{Arc, Mutex};

fn compound() -> Shape {
    let mut c = Compound::default();
    Builder.make_compound(&mut c);
    c.into_shape()
}

fn wire() -> Shape {
    let mut w = Wire::default();
    Builder.make_wire(&mut w);
    w.into_shape()
}

/// A printer that records the texts it receives.
#[derive(Default)]
struct RecordingPrinter {
    base: PrinterBase,
    lines: Mutex<Vec<String>>,
}

impl Printer for RecordingPrinter {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        PrinterType::static_type()
    }
    fn send(&self, text: &AsciiString, _gravity: Gravity) {
        self.lines
            .lock()
            .unwrap()
            .push(String::from_utf8_lossy(text.as_bytes()).into_owned());
    }
}

#[test]
fn binders_of_shapes() {
    let binder = TransferBRepShapeBinder::with_result(&compound());
    assert!(binder.has_result() && binder.shape_type().unwrap() == ShapeEnum::Compound);
    assert!(binder.compound().is_ok() && binder.vertex().is_err() && binder.edge().is_err());
    assert!(binder.wire().is_err() && binder.face().is_err() && binder.shell().is_err());
    assert!(binder.solid().is_err() && binder.comp_solid().is_err());
    assert_eq!(binder.result_type_name(), "TopoDS_Compound");
    assert_eq!(binder.result_type().name(), "TopoDS_TShape");
    let plain = TransferBRepBinderOfShape::new();
    *plain.c_result().unwrap() = wire();
    assert!(plain.has_result() && plain.result_type_name() == "TopoDS_Wire");
    plain.set_already_used();
    assert!(plain.set_result(&wire()).is_err() && plain.c_result().is_err());
    let list = TransferBRepShapeListBinder::new();
    list.add_result(&compound()).unwrap();
    assert!(!list.is_multiple());
    list.add_result(&wire()).unwrap();
    assert!(list.is_multiple() && list.shape_type(2).unwrap() == ShapeEnum::Wire);
    list.set_result(1, &wire()).unwrap();
    assert!(list.wire(1).is_ok() && list.set_result(3, &wire()).is_err() && list.shape(0).is_err());
    assert!(
        list.vertex(1).is_err()
            && list.edge(1).is_err()
            && list.face(1).is_err()
            && list.shell(1).is_err()
    );
    assert!(list.solid(1).is_err() && list.comp_solid(1).is_err() && list.compound(1).is_err());
    assert_eq!(
        (
            list.result().unwrap().len(),
            list.result_type_name().as_str()
        ),
        (2, "list(TopoDS_Shape)")
    );
    assert_eq!(list.result_type().name(), "TransferBRep_ShapeListBinder");
    let null = TransferBRepShapeListBinder::with_list(None);
    assert!(null.nb_shapes().is_err() && null.add_result(&wire()).is_err() && !null.is_multiple());
}

#[test]
fn mappers_and_infos() {
    let shape = compound();
    let mapper = TransferBRepShapeMapper::new(&shape);
    let same: Handle<dyn TransferFinder> =
        Handle::new(TransferBRepShapeMapper::new(&shape.reversed())).cast();
    assert!(mapper.equates(&same) && !mapper.equates(&Handle::default()));
    let other: Handle<dyn TransferFinder> =
        Handle::new(TransferBRepShapeMapper::new(&compound())).cast();
    assert!(!mapper.equates(&other));
    assert_eq!(
        (
            mapper.value_type().name(),
            mapper.value_type_name().as_str()
        ),
        ("TopoDS_TShape", "TopoDS_Compound")
    );
    assert_eq!(TransferBRepShapeInfo::type_name(&wire()), "TopoDS_Wire");
    let info = TransferBRepTransferResultInfo::new();
    for (i, slot) in [info.result(), info.result_warning()]
        .into_iter()
        .enumerate()
    {
        drop(slot);
        let _ = i;
    }
    *info.result_fail() = 1;
    *info.result_warning_fail() = 2;
    *info.no_result() = 3;
    *info.no_result_warning() = 4;
    *info.no_result_fail() = 5;
    *info.no_result_warning_fail() = 6;
    assert_eq!(info.snapshot(), [0, 0, 1, 2, 3, 4, 5, 6]);
}

#[test]
fn shape_results_of_processes() {
    let tp = TransferTransientProcess::new(10);
    let (a, b, c) = (
        fake_entity("a", &[]),
        fake_entity("b", &[]),
        fake_entity("c", &[]),
    );
    TransferBRep::set_shape_result(&tp, &a, &compound()).unwrap();
    let simple = TransferSimpleBinderOfTransient::new();
    simple
        .set_result(&Handle::new(HShape::new(&wire())).cast())
        .unwrap();
    tp.bind(&b, &Handle::new(simple).cast()).unwrap();
    let list = TransferBRepShapeListBinder::new();
    list.add_result(&wire()).unwrap();
    tp.bind(&c, &Handle::new(list).cast()).unwrap();
    assert!(!TransferBRep::shape_result_of(&tp, &a).unwrap().is_null());
    assert!(!TransferBRep::shape_result_of(&tp, &b).unwrap().is_null());
    let hshape: Handle<StandardTransient> = Handle::new(HShape::new(&wire())).cast();
    assert!(
        !TransferBRep::shape_result_of(&tp, &hshape)
            .unwrap()
            .is_null()
    );
    assert!(TransferBRep::shape_result_of(&Handle::default(), &a).is_err());
    assert_eq!(TransferBRep::shapes(&tp, false).unwrap().unwrap().len(), 3);
    assert_eq!(TransferBRep::shapes(&tp, true).unwrap().unwrap().len(), 0);
    assert!(
        TransferBRep::shapes(&Handle::default(), true)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        TransferBRep::shapes_of(&tp, Some(&[a.clone(), c.clone()]))
            .unwrap()
            .unwrap()
            .len(),
        2
    );
    assert!(
        TransferBRep::shapes_of(&Handle::default(), None)
            .unwrap()
            .is_none()
    );
    assert!(TransferBRep::shapes_of(&tp, None).is_err());
    let infos = TransferBRep::transfer_result_info(&tp, Some(std::slice::from_ref(&a))).unwrap();
    assert_eq!(
        infos[0].snapshot(),
        [2, 0, 0, 0, 1, 0, 0, 0],
        "a list binder has no result status"
    );
}

#[test]
fn shapes_in_finder_processes() {
    let fp = TransferFinderProcess::new(10);
    let shape = compound();
    assert_eq!(
        TransferBRep::shape_state(&fp, &shape).unwrap(),
        Orientation::External
    );
    let result: Handle<StandardTransient> = Handle::new(StandardTransient::default());
    TransferBRep::set_transient_from_shape(&fp, &shape, &result).unwrap();
    assert!(TransferBRep::transient_from_shape(&fp, &shape) == result);
    assert!(TransferBRep::result_from_shape(&fp, &shape).is_some());
    assert_eq!(
        TransferBRep::shape_state(&fp, &shape).unwrap(),
        Orientation::Forward
    );
    assert_eq!(
        TransferBRep::shape_state(&fp, &shape.reversed()).unwrap(),
        Orientation::Reversed
    );
    let mapped = TransferBRep::shape_mapper(&fp, &shape).unwrap();
    assert!(mapped.cast::<dyn TransferFinder>() == fp.mapped(1).unwrap());
    assert!(TransferBRep::shape_mapper(&Handle::default(), &shape).is_err());
    let infos = TransferBRep::transfer_result_info_shapes(
        &fp,
        Some(&[
            ShapeEnum::Compound as i32,
            ShapeEnum::Shape as i32,
            ShapeEnum::Wire as i32,
        ]),
    )
    .unwrap();
    assert_eq!(
        infos.iter().map(|i| i.snapshot()[0]).collect::<Vec<_>>(),
        vec![1, 1, 0]
    );
    assert!(
        TransferBRep::transfer_result_info_shapes(&Handle::default(), None)
            .unwrap()
            .is_empty()
    );
    let printer = Arc::new(RecordingPrinter::default());
    let handle: PrinterHandle = printer.clone();
    TransferBRep::print_result_info(&handle, &Msg::new(), &infos[0], false).unwrap();
    assert_eq!(
        printer.lines.lock().unwrap().len(),
        4,
        "header, result and the two totals"
    );
    TransferBRep::print_result_info(&handle, &Msg::new(), &infos[0], true).unwrap();
    assert_eq!(printer.lines.lock().unwrap().len(), 4 + 11);
}

#[test]
fn shape_check_lists() {
    let (model, ents) = fake_chain();
    let fp = TransferFinderProcess::new(10);
    let shape = compound();
    let mapper: Handle<dyn TransferFinder> =
        Handle::new(TransferBRepShapeMapper::new(&shape)).cast();
    fp.bind_transient(&mapper, &ents[1]).unwrap();
    let mut chl = InterfaceCheckIterator::new();
    let on_mapper = Handle::new(InterfaceCheck::new());
    on_mapper.add_fail_text("bad", "");
    on_mapper.set_entity(&mapper.cast());
    chl.add(&on_mapper, 1);
    let on_entity = Handle::new(InterfaceCheck::new());
    on_entity.add_warning_text("w", "");
    on_entity.set_entity(&ents[0]);
    chl.add(&on_entity, 2);
    chl.add(&Handle::new(InterfaceCheck::new()), 3);
    assert_eq!(TransferBRep::checked(&chl, false).len(), 1);
    assert_eq!(TransferBRep::checked(&chl, true).len(), 2);
    assert_eq!(TransferBRep::checked_shapes(&chl).len(), 1);
    let same_shape: Handle<StandardTransient> = Handle::new(HShape::new(&shape)).cast();
    assert_eq!(
        TransferBRep::check_object(&chl, &same_shape).items().len(),
        1
    );
    assert_eq!(TransferBRep::check_object(&chl, &ents[0]).items().len(), 1);
    let moved = TransferBRep::result_check_list(&chl, &fp, &model);
    let items = moved.items();
    assert_eq!(items.len(), 2);
    assert!(
        items
            .iter()
            .any(|(num, check)| *num == 2 && check.entity() == ents[1])
    );
    assert!(
        TransferBRep::result_check_list(&chl, &Handle::default(), &model)
            .items()
            .is_empty()
    );
    assert_eq!(
        StandardTransient::static_type().name(),
        "Standard_Transient"
    );
}
