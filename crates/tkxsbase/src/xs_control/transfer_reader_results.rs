// Rust port of DataExchange/TKXSBase/XSControl/XSControl_TransferReader (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the queries on the results.

//! The queries of [`XSControlTransferReader`] on its final results: transient and
//! shape results, the entities of results.
use super::{XSControlTransferReader, XSControlUtils};
use crate::interface::{InterfaceInterfaceModel, InterfaceModelEntities, InterfaceStatic};
use crate::transfer::{
    TransferBinder, TransferResultFromModel, TransferResultFromTransient,
    TransferSimpleBinderOfTransient,
};
use crate::transfer_brep::TransferBRep;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle, StandardTransient, exceptions},
};
use rustycad_tkshhealing::shape_fix::ShapeFix;

/// A model, `Standard_NullObject` when null (a native null dereference).
pub(super) fn live_model<'m>(
    model: &'m Handle<dyn InterfaceInterfaceModel>,
    what: &str,
) -> Result<&'m dyn InterfaceInterfaceModel, Failure> {
    model.get().ok_or_else(|| {
        exceptions::null_object(&format!("XSControl_TransferReader::{what}: no model"))
    })
}

/// The items of a list object of transients, `None` when `ent` is not a list.
pub(super) fn listed(ent: &Handle<StandardTransient>) -> Option<Vec<Handle<StandardTransient>>> {
    ent.cast::<HSequence<Handle<StandardTransient>>>()
        .get()
        .map(|l| l.sequence(|s| s.iter().cloned().collect()))
}

impl XSControlTransferReader {
    /// `TransientResult(theEnt)`: the transient result of the final result of `ent`
    /// (null otherwise).
    pub fn transient_result(&self, ent: &Handle<StandardTransient>) -> Handle<StandardTransient> {
        let res = self.final_result(ent);
        let mres = res
            .get()
            .map(TransferResultFromModel::main_result)
            .unwrap_or_default();
        let binder = mres
            .get()
            .map(TransferResultFromTransient::binder)
            .unwrap_or_default();
        match binder.cast::<TransferSimpleBinderOfTransient>().get() {
            Some(bnd) if bnd.has_result() => bnd.result(),
            _ => Handle::default(),
        }
    }
    /// `ShapeResult(theEnt)`: the shape of the final result of `ent`, its regularity
    /// encoded when "read.encoderegularity.angle" is positive (null otherwise).
    pub fn shape_result(&self, ent: &Handle<StandardTransient>) -> Result<Shape, Failure> {
        let res = self.final_result(ent);
        let mres = res
            .get()
            .map(TransferResultFromModel::main_result)
            .unwrap_or_default();
        let Some(main) = mres.get() else {
            return Ok(Shape::default());
        };
        let sh = XSControlUtils::new().binder_shape(&main.binder().cast());
        let tolang = InterfaceStatic::r_val("read.encoderegularity.angle");
        if tolang <= 0.0 || sh.is_null() {
            return Ok(sh);
        }
        ShapeFix::encode_regularity(&sh, tolang)?;
        Ok(sh)
    }
    /// `ClearResult(theEnt, theMode)`: the mark of `ent` cleared (negative mode), else its
    /// final result stripped by mode; false when not marked.
    pub fn clear_result(&self, ent: &Handle<StandardTransient>, mode: i32) -> bool {
        let num = self.number(ent);
        let Some(mark) = self.result_at(num).filter(|_| num != 0) else {
            return false;
        };
        if mode < 0 {
            self.state.write().results.insert(num, Handle::default());
            return true;
        }
        match mark.cast::<TransferResultFromModel>().get() {
            Some(resu) => {
                resu.strip(mode);
                true
            }
            None => false,
        }
    }
    /// The entity of a binder (or of a transient result) among the roots (mode 0) or the
    /// mapped items (mode 1) of the process.
    fn entity_from_mapped(
        &self,
        res: &Handle<StandardTransient>,
        binder: &Handle<dyn TransferBinder>,
        mode: i32,
    ) -> Result<Handle<StandardTransient>, Failure> {
        let (tp, model) = (self.transient_process(), self.model());
        let Some(process) = tp.get() else {
            return Ok(Handle::default());
        };
        let nb = if mode == 0 {
            process.nb_roots()
        } else {
            process.nb_mapped()
        };
        for j in 1..=nb {
            let i = if mode == 0 {
                live_model(&model, "EntityFromResult")?.number(&process.root(j)?)
            } else {
                j
            };
            if i == 0 {
                continue;
            }
            let abinder = process.map_item(i)?;
            if abinder.is_null() {
                continue;
            }
            if binder.is_some() {
                if *binder == abinder {
                    return process.mapped(i);
                }
                continue;
            }
            if abinder
                .cast::<TransferSimpleBinderOfTransient>()
                .get()
                .is_some_and(|trb| trb.result() == *res)
            {
                return process.mapped(i);
            }
        }
        Ok(Handle::default())
    }
    /// `EntityFromResult(theRes, theMode = 0)`: the entity whose result is `res` (a shape
    /// as `EntityFromShapeResult`): among the roots (0) or the mapped items (1) of the
    /// process, or the starts of the final results (2 and more, the results of level
    /// `mode - 2`). Native quirk kept: from mode 2, the search stops at the first entity
    /// without final result.
    pub fn entity_from_result(
        &self,
        res: &Handle<StandardTransient>,
        mode: i32,
    ) -> Result<Handle<StandardTransient>, Failure> {
        let sh = XSControlUtils::new().binder_shape(res);
        if !sh.is_null() {
            return self.entity_from_shape_result(&sh, mode);
        }
        let binder = res.cast::<dyn TransferBinder>();
        if mode == 0 || mode == 1 {
            return self.entity_from_mapped(res, &binder, mode);
        }
        if mode < 2 {
            return Ok(Handle::default());
        }
        let model = self.model();
        for i in 1..=live_model(&model, "EntityFromResult")?.nb_entities() {
            let rec = self.result_from_number(i)?;
            let Some(live) = rec.get() else {
                return Ok(Handle::default());
            };
            for item in live.results(mode - 2)? {
                if let Some(rft) = item
                    .cast::<TransferResultFromTransient>()
                    .get()
                    .filter(|r| r.binder() == binder)
                {
                    return Ok(rft.start());
                }
            }
        }
        Ok(Handle::default())
    }
    /// `EntityFromShapeResult(theRes, theMode = 0)`: the entity whose shape result is
    /// `res` (else the same shape, else for mode -1 a partner) among the roots (0) or the
    /// mapped items (1, -1), or the starts of the final results (2 and more).
    pub fn entity_from_shape_result(
        &self,
        res: &Shape,
        mode: i32,
    ) -> Result<Handle<StandardTransient>, Failure> {
        if res.is_null() {
            return Ok(Handle::default());
        }
        if (-1..=1).contains(&mode) {
            return self.entity_from_mapped_shape(res, mode);
        }
        if mode < 2 {
            return Ok(Handle::default());
        }
        let model = self.model();
        let xu = XSControlUtils::new();
        for i in 1..=live_model(&model, "EntityFromShapeResult")?.nb_entities() {
            let rec = self.result_from_number(i)?;
            let Some(live) = rec.get() else {
                continue;
            };
            for item in live.results(mode - 2)? {
                let Some(rft) = item
                    .cast::<TransferResultFromTransient>()
                    .get()
                    .map(|r| (r.start(), r.binder()))
                else {
                    continue;
                };
                let sh = xu.binder_shape(&rft.1.cast());
                if !sh.is_null() && sh == *res {
                    return Ok(rft.0);
                }
            }
        }
        Ok(Handle::default())
    }
    /// The process branch of `EntityFromShapeResult`.
    fn entity_from_mapped_shape(
        &self,
        res: &Shape,
        mode: i32,
    ) -> Result<Handle<StandardTransient>, Failure> {
        let (tp, model) = (self.transient_process(), self.model());
        let Some(process) = tp.get() else {
            return Ok(Handle::default());
        };
        let (mut samesh, mut partner) = (Handle::default(), Handle::default());
        let nb = if mode == 0 {
            process.nb_roots()
        } else {
            process.nb_mapped()
        };
        for j in 1..=nb {
            let i = if mode == 0 {
                live_model(&model, "EntityFromShapeResult")?.number(&process.root(j)?)
            } else {
                j
            };
            if i == 0 {
                continue;
            }
            let ent = process.mapped(i)?;
            let sh = TransferBRep::shape_result_of(&tp, &ent)?;
            if sh.is_null() {
                continue;
            }
            if sh == *res {
                return Ok(ent);
            }
            if sh.is_same(res) {
                samesh = ent.clone();
            }
            if mode == -1 && sh.is_partner(res) {
                partner = ent;
            }
        }
        Ok(if samesh.is_some() { samesh } else { partner })
    }
    /// `EntitiesFromShapeList(theRes, theMode = 0)`: the entities whose shape result is in
    /// `res`. Native quirks kept: for modes 0 and 1 the search stops at the first entity
    /// found; from mode 2 the result list is read at the entity number (natively `i` for
    /// `ir`), `Standard_OutOfRange` beyond it.
    pub fn entities_from_shape_list(
        &self,
        res: Option<&[Shape]>,
        mode: i32,
    ) -> Result<Vec<Handle<StandardTransient>>, Failure> {
        let mut lt = Vec::new();
        let Some(shapes_list) = res.filter(|r| !r.is_empty()) else {
            return Ok(lt);
        };
        let mut shapes = MapOfShape::default();
        for sh in shapes_list {
            shapes.add(sh.clone());
        }
        let xu = XSControlUtils::new();
        let (tp, model) = (self.transient_process(), self.model());
        if (mode == 0 || mode == 1)
            && let Some(process) = tp.get()
        {
            let nb = if mode == 0 {
                process.nb_roots()
            } else {
                process.nb_mapped()
            };
            for j in 1..=nb {
                let i = if mode == 0 {
                    live_model(&model, "EntitiesFromShapeList")?.number(&process.root(j)?)
                } else {
                    j
                };
                if i == 0 {
                    continue;
                }
                let sh = xu.binder_shape(&process.map_item(i)?.cast());
                if !sh.is_null() && shapes.contains(&sh) {
                    lt.push(process.mapped(i)?);
                    break; // skl (for looking for entities in checkbrep)
                }
            }
        }
        if mode >= 2 {
            self.entities_from_results(&shapes, mode, &mut lt)?;
        }
        Ok(lt)
    }
    /// The final results branch of `EntitiesFromShapeList`.
    fn entities_from_results(
        &self,
        shapes: &MapOfShape,
        mode: i32,
        lt: &mut Vec<Handle<StandardTransient>>,
    ) -> Result<(), Failure> {
        let model = self.model();
        let xu = XSControlUtils::new();
        for i in 1..=live_model(&model, "EntitiesFromShapeList")?.nb_entities() {
            let rec = self.result_from_number(i)?;
            let Some(live) = rec.get() else {
                continue;
            };
            let list = live.results(mode - 2)?;
            for _ir in 1..=list.len() {
                let item = list.get(i as usize - 1).cloned().ok_or_else(|| {
                    exceptions::out_of_range(&format!(
                        "XSControl_TransferReader::EntitiesFromShapeList: item {i} of {} results",
                        list.len()
                    ))
                })?;
                let Some(rft) = item
                    .cast::<TransferResultFromTransient>()
                    .get()
                    .map(|r| (r.start(), r.binder()))
                else {
                    continue;
                };
                let sh = xu.binder_shape(&rft.1.cast());
                if !sh.is_null() && shapes.contains(&sh) {
                    lt.push(rft.0);
                }
            }
        }
        Ok(())
    }
}
