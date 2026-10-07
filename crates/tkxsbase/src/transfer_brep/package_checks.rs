// Rust port of DataExchange/TKXSBase/TransferBRep/TransferBRep (.hxx, .cxx): the result
// counts and the check lists at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The result counts (`TransferResultInfo`, `PrintResultInfo`) and the check lists of
//! shapes (`ResultCheckList`, `Checked`, `CheckedShapes`, `CheckObject`) of
//! [`TransferBRep`].
use super::{
    TransferBRep, TransferBRepBinderOfShape, TransferBRepShapeMapper,
    TransferBRepTransferResultInfo,
};
use crate::interface::{
    InterfaceCheck, InterfaceCheckIterator, InterfaceInterfaceModel, InterfaceModelEntities,
};
use crate::transfer::{
    TransferBinder, TransferFinder, TransferFinderProcess, TransferTransientProcess,
};
use rustycad_tkbrep::topods::{HShape, Shape};
use rustycad_tkernel::{
    message::{Gravity, Msg, PrinterHandle},
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::AsciiString,
};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `FillInfo` (static): counts `binder` in `info` by result, warnings and fails.
fn fill_info(
    binder: &Handle<dyn TransferBinder>,
    check: &InterfaceCheck,
    info: &TransferBRepTransferResultInfo,
) {
    let (warn, fail) = (check.has_warnings(), check.has_failed());
    let kind = match (warn, fail) {
        (true, true) => 3,
        (true, false) => 1,
        (false, true) => 2,
        (false, false) => 0,
    };
    let offset = if binder.has_result() { 0 } else { 4 };
    info.bump(offset + kind);
}

/// The shape a check entity stands for: a `TopoDS_HShape`, a binder of shape or a shape
/// mapper (in that order, the last found wins as natively), else null.
fn shape_of(ent: &Handle<StandardTransient>) -> Shape {
    let mut shape = Shape::default();
    if let Some(hs) = ent.cast::<HShape>().get() {
        shape = hs.shape();
    }
    if let Some(sb) = ent.cast::<TransferBRepBinderOfShape>().get() {
        shape = sb.result();
    }
    if let Some(sm) = ent.cast::<TransferBRepShapeMapper>().get() {
        shape = sm.value().clone();
    }
    shape
}

/// The checks of `chl` with messages and an entity.
fn checks_with_messages(
    chl: &InterfaceCheckIterator,
) -> Vec<(Handle<InterfaceCheck>, Handle<StandardTransient>)> {
    chl.items()
        .into_iter()
        .filter(|(_, ach)| ach.nb_fails() + ach.nb_warnings() != 0)
        .map(|(_, ach)| {
            let ent = ach.entity();
            (ach, ent)
        })
        .collect()
}

impl TransferBRep {
    /// `TransferResultInfo(TP, EntityTypes, InfoSeq)`: the result counts of the mapped
    /// entities per kind of the sample entities of `entity_types`; `Standard_NullObject`
    /// for a null mapped entity or sample (native null dereferences).
    pub fn transfer_result_info(
        tp: &Handle<TransferTransientProcess>,
        entity_types: Option<&[Handle<StandardTransient>]>,
    ) -> Result<Vec<Handle<TransferBRepTransferResultInfo>>, Failure> {
        let (Some(process), Some(types)) = (tp.get(), entity_types) else {
            return Ok(Vec::new());
        };
        let infos: Vec<_> = types
            .iter()
            .map(|_| Handle::new(TransferBRepTransferResultInfo::new()))
            .collect();
        for i in 1..=process.nb_mapped() {
            let entity = process.mapped(i)?;
            let binder = process.find(&entity);
            if binder.is_null() {
                continue;
            }
            let check = binder.check();
            for (sample, info) in types.iter().zip(&infos) {
                let kind = sample.dynamic_type().ok_or_else(|| exceptions::null_object("TransferBRep::TransferResultInfo: null entity type sample; expected an entity"))?;
                if entity
                    .dynamic_type()
                    .ok_or_else(|| {
                        exceptions::null_object(
                            "TransferBRep::TransferResultInfo: null mapped entity",
                        )
                    })?
                    .is_subtype(kind)
                {
                    fill_info(&binder, &check, info);
                }
            }
        }
        Ok(infos)
    }
    /// `TransferResultInfo(FP, ShapeTypes, InfoSeq)`: the result counts of the mapped
    /// shapes per shape type (`TopAbs_SHAPE` takes all); `Standard_NullObject` for a
    /// mapped finder that is not a shape mapper (a native null dereference).
    pub fn transfer_result_info_shapes(
        fp: &Handle<TransferFinderProcess>,
        shape_types: Option<&[i32]>,
    ) -> Result<Vec<Handle<TransferBRepTransferResultInfo>>, Failure> {
        let (Some(process), Some(types)) = (fp.get(), shape_types) else {
            return Ok(Vec::new());
        };
        let infos: Vec<_> = types
            .iter()
            .map(|_| Handle::new(TransferBRepTransferResultInfo::new()))
            .collect();
        for i in 1..=process.nb_mapped() {
            let finder = process.mapped(i)?;
            let mapper = finder.cast::<TransferBRepShapeMapper>();
            let live = mapper.get().ok_or_else(|| {
                exceptions::null_object(
                    "TransferBRep::TransferResultInfo: a mapped finder is not a shape mapper",
                )
            })?;
            let binder = process.find(&mapper.cast::<dyn TransferFinder>());
            if binder.is_null() {
                continue;
            }
            let check = binder.check();
            let shape_type = live.value().shape_type()?;
            for (current, info) in types.iter().zip(&infos) {
                if *current == shape_type as i32 || *current == ShapeEnum::Shape as i32 {
                    fill_info(&binder, &check, info);
                }
            }
        }
        Ok(infos)
    }
    /// `PrintResultInfo(Printer, Header, ResultInfo, printEmpty = true)`: the header,
    /// then one message per count (the empty ones only with `print_empty`) and the totals,
    /// through the messages "Result.Print.MSG30" to "MSG48".
    pub fn print_result_info(
        printer: &PrinterHandle,
        header: &Msg,
        result_info: &TransferBRepTransferResultInfo,
        print_empty: bool,
    ) -> Result<(), Failure> {
        let [r, rw, rf, rwf, nr, nrw, nrf, nrwf] = result_info.snapshot();
        let mut local = header.clone();
        printer.send_extended(local.get()?, Gravity::Info);
        let lines = [
            ("MSG30", r, true),
            ("MSG32", rw, print_empty || rw > 0),
            ("MSG34", rf, print_empty || rf > 0),
            ("MSG36", rwf, print_empty || rwf > 0),
            ("MSG38", r + rw + rf + rwf, true),
            ("MSG40", nr, print_empty || nr > 0),
            ("MSG42", nrw, print_empty || nrw > 0),
            ("MSG44", nrf, print_empty || nrf > 0),
            ("MSG46", nrwf, print_empty || nrwf > 0),
            ("MSG48", nr + nrw + nrf + nrwf, true),
        ];
        for (key, value, shown) in lines {
            if shown {
                let mut msg = Msg::from_key(&AsciiString::new(&format!("Result.Print.{key}")));
                msg.arg_integer(value)?;
                printer.send_extended(msg.get()?, Gravity::Info);
            }
        }
        Ok(())
    }
    /// `ResultCheckList(chl, FP, model)`: the checks with messages, each moved to the
    /// transient result of its finder (numbered in `model`); empty for a null argument.
    pub fn result_check_list(
        chl: &InterfaceCheckIterator,
        fp: &Handle<TransferFinderProcess>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> InterfaceCheckIterator {
        let mut nchl = InterfaceCheckIterator::new();
        let (Some(process), Some(live_model)) = (fp.get(), model.get()) else {
            return nchl;
        };
        nchl.set_model(model);
        for (ach, entity) in checks_with_messages(chl) {
            let starting = entity.cast::<dyn TransferFinder>();
            let ent = if starting.is_some() {
                process.find_transient(&starting)
            } else {
                Handle::default()
            };
            let mut num = 0;
            if ent.is_some() {
                ach.set_entity(&ent);
                num = live_model.number(&ent);
            }
            nchl.add(&ach, num);
        }
        nchl
    }
    /// `Checked(chl, alsoshapes = false)`: the entities of the checks with messages
    /// (without the shape entities unless `alsoshapes`).
    pub fn checked(
        chl: &InterfaceCheckIterator,
        alsoshapes: bool,
    ) -> Vec<Handle<StandardTransient>> {
        checks_with_messages(chl)
            .into_iter()
            .map(|(_, ent)| ent)
            .filter(|ent| ent.is_some())
            .filter(|ent| {
                alsoshapes
                    || (ent.cast::<TransferBRepBinderOfShape>().is_null()
                        && ent.cast::<HShape>().is_null()
                        && ent.cast::<TransferBRepShapeMapper>().is_null())
            })
            .collect()
    }
    /// `CheckedShapes(chl)`: the shapes of the checks with messages (one per kind of
    /// shape entity found, as natively).
    pub fn checked_shapes(chl: &InterfaceCheckIterator) -> Vec<Shape> {
        let mut shapes = Vec::new();
        for (_, ent) in checks_with_messages(chl) {
            if let Some(hs) = ent.cast::<HShape>().get() {
                shapes.push(hs.shape());
            }
            if let Some(sb) = ent.cast::<TransferBRepBinderOfShape>().get() {
                shapes.push(sb.result());
            }
            if let Some(sm) = ent.cast::<TransferBRepShapeMapper>().get() {
                shapes.push(sm.value().clone());
            }
        }
        shapes
    }
    /// `CheckObject(chl, obj)`: the checks with messages on `obj`, or on the same shape
    /// (same TShape, location and orientation) when `obj` stands for a shape.
    pub fn check_object(
        chl: &InterfaceCheckIterator,
        obj: &Handle<StandardTransient>,
    ) -> InterfaceCheckIterator {
        let shape = shape_of(obj);
        let mut nchl = InterfaceCheckIterator::new();
        for (ach, ent) in checks_with_messages(chl) {
            if ent.is_null() {
                continue;
            }
            let matches = if shape.is_null() {
                ent == *obj
            } else {
                shape_of(&ent) == shape
            };
            if matches {
                ach.set_entity(&ent);
                nchl.add(&ach, 0);
            }
        }
        nchl
    }
}
