// Rust port of DataExchange/TKXSBase/XSAlgo/XSAlgo_ShapeProcessor.cxx (addMessages,
// MergeShapeTransferInfo) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The merge of the shape modifications of a processing into the results of the
//! transfer processes.
use super::XSAlgoShapeProcessor;
use crate::transfer::{
    TransferBinder, TransferFinderProcess, TransferTransientListBinder, TransferTransientProcess,
};
use crate::transfer_brep::{TransferBRep, TransferBRepShapeBinder};
use rustycad_tkbrep::{
    brep_tools::{BRepToolsReShape, ReShape},
    topexp::Explorer,
    topods::{Shape, ShapeIterator},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendMsgRegistrator, shape_process::ShapeProcessShapeMap,
};

/// `addMessages` (private): the messages recorded on `shape` as warnings of `binder`.
fn add_messages(
    messages: &Handle<ShapeExtendMsgRegistrator>,
    shape: &Shape,
    binder: &Handle<dyn TransferBinder>,
) {
    let Some(registrator) = messages.get() else {
        return;
    };
    let texts: Vec<(String, String)> = registrator.map_shape(|map| {
        map.seek(shape)
            .map(|list| {
                list.iter()
                    .map(|m| {
                        (
                            c_text(&m.value().to_utf8()),
                            c_text(&m.original().to_utf8()),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    });
    for (value, original) in texts {
        binder.add_warning(&value, &original);
    }
}

/// The messages of the shape binders of `tp` from item `start` (the container's merge
/// without modified shapes).
pub(super) fn add_binder_messages(
    tp: &Handle<TransferTransientProcess>,
    start: i32,
    messages: &Handle<ShapeExtendMsgRegistrator>,
) -> Result<(), Failure> {
    let process = tp.get().ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::null_object(
            "XSAlgo_AlgoContainer::MergeTransferInfo: null process",
        )
    })?;
    for i in start.max(1)..=process.nb_mapped() {
        let binder = process.map_item(i)?;
        let original = binder
            .cast::<TransferBRepShapeBinder>()
            .get()
            .map(|b| b.result())
            .unwrap_or_default();
        if !original.is_null() {
            add_messages(messages, &original, &binder);
        }
    }
    Ok(())
}

/// A message text as `TCollection_AsciiString(ext).ToCString()`: up to the first NUL.
fn c_text(text: &str) -> String {
    text.split('\0').next().unwrap_or_default().to_owned()
}

/// The registrator has messages on shapes.
fn has_shape_messages(messages: &Handle<ShapeExtendMsgRegistrator>) -> bool {
    messages
        .get()
        .is_some_and(|r| r.map_shape(|map| !map.is_empty()))
}

/// The modified shape of `original` in the binder sense: the map image, the image of
/// the unlocated shape, or the shape with its modified edges replaced.
fn modified_shape(
    original: &Shape,
    modified: &ShapeProcessShapeMap,
) -> Result<Option<Shape>, Failure> {
    if let Some(found) = modified.seek(original) {
        return Ok(Some(found.clone()));
    }
    if !original.location().is_identity() {
        let unlocated = original.located(&Location::default(), false)?;
        return Ok(modified.seek(&unlocated).cloned());
    }
    let reshaper = BRepToolsReShape::new();
    let mut has_modified_edges = false;
    let mut explorer = Explorer::new(original, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        if let Some(edge) = modified.seek(explorer.current()) {
            has_modified_edges = true;
            reshaper.replace(explorer.current(), edge)?;
        }
        explorer.next()?;
    }
    if !has_modified_edges {
        return Ok(None);
    }
    Ok(Some(reshaper.apply(original, ShapeEnum::Shape)?))
}

impl XSAlgoShapeProcessor {
    /// `MergeShapeTransferInfo(theTransientProcess, theModifiedShapesMap,
    /// theFirstTPItemIndex, theMessages)`: the shape binders of the process from
    /// `first_tp_item_index` get the modified shapes, and the messages as warnings.
    pub fn merge_shape_transfer_info(
        transient_process: &Handle<TransferTransientProcess>,
        modified: &ShapeProcessShapeMap,
        first_tp_item_index: i32,
        messages: &Handle<ShapeExtendMsgRegistrator>,
    ) -> Result<(), Failure> {
        if modified.is_empty() {
            return Ok(());
        }
        let process = transient_process.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "XSAlgo_ShapeProcessor::MergeShapeTransferInfo: null process",
            )
        })?;
        let to_print = has_shape_messages(messages);
        for i in first_tp_item_index.max(1)..=process.nb_mapped() {
            let binder = process.map_item(i)?;
            let shape_binder = binder.cast::<TransferBRepShapeBinder>();
            let Some(live) = shape_binder.get() else {
                continue;
            };
            let original = live.result();
            if original.is_null() {
                continue;
            }
            if let Some(result) = modified_shape(&original, modified)? {
                live.set_result(&result)?;
            }
            if to_print {
                add_messages(messages, &original, &binder);
            }
        }
        Ok(())
    }
    /// `MergeShapeTransferInfo(theFinderProcess, theModifiedShapesMap, theMessages)`: binds
    /// each original shape to the binder of its result (a list of the transient results of
    /// the sub-shapes when a shape was split), with the messages as warnings.
    pub fn merge_shape_transfer_info_finder(
        finder_process: &Handle<TransferFinderProcess>,
        modified: &ShapeProcessShapeMap,
        messages: &Handle<ShapeExtendMsgRegistrator>,
    ) -> Result<(), Failure> {
        if modified.is_empty() {
            return Ok(());
        }
        let process = finder_process.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "XSAlgo_ShapeProcessor::MergeShapeTransferInfo: null process",
            )
        })?;
        let to_print = has_shape_messages(messages);
        let pairs: Vec<(Shape, Shape)> = modified
            .items()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (original, result) in pairs {
            let result_binder = result_binder(finder_process, &original, &result)?;
            let original_mapper = TransferBRep::shape_mapper(finder_process, &original)?.cast();
            let original_binder = process.find(&original_mapper);
            match original_binder.get() {
                None => process.bind(&original_mapper, &result_binder)?,
                Some(live) => live.add_result(&result_binder),
            }
            if to_print {
                add_messages(messages, &original, &result_binder);
            }
        }
        Ok(())
    }
}

/// The binder of the result shape: the mapped one, else a new shape binder (of the only
/// transferred sub-shape, or with a list of the transient results of the sub-shapes,
/// when the original shape was split).
fn result_binder(
    finder_process: &Handle<TransferFinderProcess>,
    original: &Shape,
    result: &Shape,
) -> Result<Handle<dyn TransferBinder>, Failure> {
    let process = finder_process.get().ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::null_object(
            "XSAlgo_ShapeProcessor: null finder process",
        )
    })?;
    let mapper = TransferBRep::shape_mapper(finder_process, result)?.cast();
    let found = process.find(&mapper);
    if found.is_some() {
        return Ok(found);
    }
    let binder: Handle<dyn TransferBinder> =
        Handle::new(TransferBRepShapeBinder::with_result(result)).cast();
    if original.shape_type()? <= result.shape_type()? {
        return Ok(binder);
    }
    let list = TransferTransientListBinder::new();
    let mut sub_shape = Shape::default();
    let mut it = ShapeIterator::new(result, true, true);
    while it.more() {
        let current = it.value()?.clone();
        let sub_mapper = TransferBRep::shape_mapper(finder_process, &current)?;
        let transient = process.find_transient(&sub_mapper.cast());
        if transient.is_some() {
            list.add_transient_result(&transient);
            sub_shape = current;
        }
        it.next();
    }
    match list.nb_transients() {
        1 => Ok(Handle::new(TransferBRepShapeBinder::with_result(&sub_shape)).cast()),
        0 => Ok(binder),
        _ => {
            binder.add_result(&Handle::new(list).cast());
            Ok(binder)
        }
    }
}
