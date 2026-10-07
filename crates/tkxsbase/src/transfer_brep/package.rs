// Rust port of DataExchange/TKXSBase/TransferBRep/TransferBRep (.hxx, .cxx): the shape
// results at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TransferBRep`: the shape results of the transient processes and the shapes mapped in
//! the finder processes.
use super::{
    TransferBRepBinderOfShape, TransferBRepShapeBinder, TransferBRepShapeListBinder,
    TransferBRepShapeMapper,
};
use crate::transfer::{
    TransferBinder, TransferFinder, TransferFinderProcess, TransferSimpleBinderOfTransient,
    TransferTransientProcess,
};
use rustycad_tkbrep::topods::{HShape, Shape};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions};
use rustycad_tkg3d::top_abs::Orientation;

/// `TransferBRep`: the package functions.
pub struct TransferBRep;

/// `ShapeAppend` (static): the shapes of a chain of binders.
fn shape_append(
    binder: &Handle<dyn TransferBinder>,
    shapes: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let mut current = binder.clone();
    while let Some(live) = current.get() {
        if let Some(shbind) = current.cast::<TransferBRepBinderOfShape>().get() {
            if shbind.has_result() {
                shapes.push(shbind.result());
            }
        } else if let Some(slbind) = current.cast::<TransferBRepShapeListBinder>().get() {
            for i in 1..=slbind.nb_shapes()? {
                shapes.push(slbind.shape(i)?);
            }
        } else if let Some(trbind) = current.cast::<TransferSimpleBinderOfTransient>().get() {
            shapes.extend(trbind.result().cast::<HShape>().get().map(HShape::shape));
        }
        let next = live.next_result();
        current = next;
    }
    Ok(())
}

/// The finder of a shape: a new shape mapper.
fn mapper_of(shape: &Shape) -> Handle<dyn TransferFinder> {
    Handle::new(TransferBRepShapeMapper::new(shape)).cast()
}

/// The live finder process (a native null dereference without one).
fn live_finder_process(
    fp: &Handle<TransferFinderProcess>,
) -> Result<&TransferFinderProcess, Failure> {
    fp.get().ok_or_else(|| {
        exceptions::null_object("TransferBRep: null finder process; expected a finder process")
    })
}

impl TransferBRep {
    /// `ShapeResult(binder)`: the first shape of the chain (a shape binder, or a simple
    /// binder of `TopoDS_HShape`), else null.
    pub fn shape_result(binder: &Handle<dyn TransferBinder>) -> Shape {
        let mut bnd = binder.clone();
        while let Some(live) = bnd.get() {
            if let Some(shb) = bnd.cast::<TransferBRepBinderOfShape>().get() {
                return shb.result();
            }
            if let Some(hsp) = bnd
                .cast::<TransferSimpleBinderOfTransient>()
                .get()
                .map(|b| b.result().cast::<HShape>())
                .filter(|h| h.is_some())
            {
                return hsp.get().map(HShape::shape).unwrap_or_default();
            }
            let next = live.next_result();
            bnd = next;
        }
        Shape::default()
    }
    /// `ShapeResult(TP, ent)`: the shape result of `ent` in `TP` (or of `ent` itself as a
    /// binder or a `TopoDS_HShape`); `Standard_NullObject` for a null process.
    pub fn shape_result_of(
        tp: &Handle<TransferTransientProcess>,
        ent: &Handle<StandardTransient>,
    ) -> Result<Shape, Failure> {
        let process = tp.get().ok_or_else(|| {
            exceptions::null_object(
                "TransferBRep::ShapeResult: null transient process; expected one",
            )
        })?;
        let mut binder = process.find(ent);
        if binder.is_null() {
            binder = ent.cast();
        }
        if binder.is_some() {
            return Ok(Self::shape_result(&binder));
        }
        Ok(ent
            .cast::<HShape>()
            .get()
            .map(HShape::shape)
            .unwrap_or_default())
    }
    /// `SetShapeResult(TP, ent, result)`: binds a shape binder (nothing for a null
    /// argument); `Transfer_TransferFailure` from `Bind`.
    pub fn set_shape_result(
        tp: &Handle<TransferTransientProcess>,
        ent: &Handle<StandardTransient>,
        result: &Shape,
    ) -> Result<(), Failure> {
        let Some(process) = tp.get().filter(|_| !result.is_null() && ent.is_some()) else {
            return Ok(());
        };
        process.bind(
            ent,
            &Handle::new(TransferBRepShapeBinder::with_result(result)).cast(),
        )
    }
    /// `Shapes(TP, rootsonly = true)`: the shapes of the root (or all) results; `None`
    /// for a null process.
    pub fn shapes(
        tp: &Handle<TransferTransientProcess>,
        rootsonly: bool,
    ) -> Result<Option<Vec<Shape>>, Failure> {
        let Some(process) = tp.get() else {
            return Ok(None);
        };
        let mut shapes = Vec::new();
        let mut list = if rootsonly {
            process.root_result(false)
        } else {
            process.complete_result(false)
        };
        list.start();
        while list.more() {
            shape_append(&list.value()?, &mut shapes)?;
            list.next();
        }
        Ok(Some(shapes))
    }
    /// `Shapes(TP, list)`: the shapes of the results of the entities of `list`; `None`
    /// when both are null. Native null dereferences when only one is null:
    /// `Standard_NullObject`.
    pub fn shapes_of(
        tp: &Handle<TransferTransientProcess>,
        list: Option<&[Handle<StandardTransient>]>,
    ) -> Result<Option<Vec<Shape>>, Failure> {
        if tp.is_null() && list.is_none() {
            return Ok(None);
        }
        let list = list.ok_or_else(|| {
            exceptions::null_object("TransferBRep::Shapes: null list; expected a list")
        })?;
        let mut shapes = Vec::new();
        for ent in list {
            let process = tp.get().ok_or_else(|| {
                exceptions::null_object("TransferBRep::Shapes: null process; expected one")
            })?;
            shape_append(&process.find(ent), &mut shapes)?;
        }
        Ok(Some(shapes))
    }
    /// `ShapeState(FP, shape)`: `EXTERNAL` when not mapped, else `FORWARD` or `REVERSED`
    /// by the orientation of the mapped shape.
    pub fn shape_state(
        fp: &Handle<TransferFinderProcess>,
        shape: &Shape,
    ) -> Result<Orientation, Failure> {
        let Some(process) = fp.get().filter(|_| !shape.is_null()) else {
            return Ok(Orientation::External);
        };
        let index = process.map_index(&mapper_of(shape));
        if index == 0 {
            return Ok(Orientation::External);
        }
        let mapped = process.mapped(index)?;
        let Some(sm) = mapped
            .cast::<TransferBRepShapeMapper>()
            .get()
            .map(|m| m.value().clone())
        else {
            return Ok(Orientation::External);
        };
        Ok(if sm.orientation() != shape.orientation() {
            Orientation::Reversed
        } else {
            Orientation::Forward
        })
    }
    /// `ResultFromShape(FP, shape)`: the binder of `shape` (null when not mapped).
    pub fn result_from_shape(
        fp: &Handle<TransferFinderProcess>,
        shape: &Shape,
    ) -> Handle<dyn TransferBinder> {
        match fp.get().filter(|_| !shape.is_null()) {
            Some(process) => process.find(&mapper_of(shape)),
            None => Handle::default(),
        }
    }
    /// `TransientFromShape(FP, shape)`: the transient result of `shape`.
    pub fn transient_from_shape(
        fp: &Handle<TransferFinderProcess>,
        shape: &Shape,
    ) -> Handle<StandardTransient> {
        match fp.get().filter(|_| !shape.is_null()) {
            Some(process) => process.find_transient(&mapper_of(shape)),
            None => Handle::default(),
        }
    }
    /// `SetTransientFromShape(FP, shape, result)`: binds `result` to `shape`.
    pub fn set_transient_from_shape(
        fp: &Handle<TransferFinderProcess>,
        shape: &Shape,
        result: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        match fp.get().filter(|_| !shape.is_null()) {
            Some(process) => process.bind_transient(&mapper_of(shape), result),
            None => Ok(()),
        }
    }
    /// `ShapeMapper(FP, shape)`: the mapped mapper of `shape`, else a new one;
    /// `Standard_NullObject` for a null process.
    pub fn shape_mapper(
        fp: &Handle<TransferFinderProcess>,
        shape: &Shape,
    ) -> Result<Handle<TransferBRepShapeMapper>, Failure> {
        let process = live_finder_process(fp)?;
        let mapper = Handle::new(TransferBRepShapeMapper::new(shape));
        let index = process.map_index(&mapper.cast());
        if index == 0 {
            return Ok(mapper);
        }
        Ok(process.mapped(index)?.cast())
    }
}
