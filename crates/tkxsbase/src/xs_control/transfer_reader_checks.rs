// Rust port of DataExchange/TKXSBase/XSControl/XSControl_TransferReader (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the check lists.

//! The check lists of [`XSControlTransferReader`]: the checks of the final results, the
//! checked entities, the last transfer and the shapes of the results.
use super::XSControlTransferReader;
use super::transfer_reader_results::{listed, live_model};
use crate::interface::{InterfaceCheckIterator, InterfaceCheckStatus, InterfaceModelEntities};
use crate::transfer_brep::TransferBRep;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};

impl XSControlTransferReader {
    /// `CheckList(theEnt, theLevel = 0)`: the checks of the final results: of all of
    /// them for the model, of the items of a list, of the last transfer of `ent`
    /// (negative level), or of `ent`, named by kind.
    pub fn check_list(
        &self,
        ent: &Handle<StandardTransient>,
        level: i32,
    ) -> Result<InterfaceCheckIterator, Failure> {
        let mut chl = InterfaceCheckIterator::new();
        let model = self.model();
        let Some(m) = model.get().filter(|_| ent.is_some()) else {
            return Ok(chl);
        };
        let is_model = ent.object_address() == model.object_address();
        if is_model {
            for i in 1..=m.nb_entities() {
                if let Some(rec) = self.result_from_number(i)?.get() {
                    chl.merge(&rec.check_list(false, 2)?);
                }
            }
        } else if let Some(list) = listed(ent) {
            for item in list {
                if let Some(rec) = self.final_result(&item).get() {
                    chl.merge(&rec.check_list(false, level)?);
                }
            }
        } else if level < 0 {
            let tp = self.transient_process();
            let Some(process) = tp.get() else {
                return Ok(chl);
            };
            chl.add(&process.check(ent), m.number(ent));
        } else {
            let rec = self.final_result(ent);
            let Some(live) = rec.get() else {
                return Ok(chl);
            };
            chl = live.check_list(false, level)?; // manque level ...
        }
        chl.set_name(match level {
            _ if is_model => "XSControl : CheckList complete Model",
            l if l < 0 => "XSControl : CheckList Last",
            0 => "XSControl : CheckList Final Main",
            1 => "XSControl : CheckList Final Main+Subs",
            _ => "XSControl : CheckList Final Complete",
        });
        Ok(chl)
    }
    /// `HasChecks(theEnt, FailsOnly)`: the final result of `ent` has fails (or warnings
    /// when not `FailsOnly`).
    pub fn has_checks(
        &self,
        ent: &Handle<StandardTransient>,
        fails_only: bool,
    ) -> Result<bool, Failure> {
        let resu = self.final_result(ent);
        let Some(live) = resu.get() else {
            return Ok(false);
        };
        let stat = live.compute_check_status(false)? as i32;
        Ok(match stat {
            0 => false,
            s if s > 1 => true,
            _ => !fails_only,
        })
    }
    /// `CheckedList(theEnt, WithCheck = Interface_CheckAny, theResult = true)`: the
    /// starts whose check complies with `withcheck` in the final results of the model, of
    /// the items of a list, or of `ent`. Native quirk kept: for the model and a list the
    /// lists of the results are appended (the native `Append` of a sequence handle).
    pub fn checked_list(
        &self,
        ent: &Handle<StandardTransient>,
        withcheck: InterfaceCheckStatus,
        result: bool,
    ) -> Result<Vec<Handle<StandardTransient>>, Failure> {
        let mut res = Vec::new();
        if ent.is_null() {
            return Ok(res);
        }
        let model = self.model();
        if ent.object_address() == model.object_address() {
            for i in 1..=live_model(&model, "CheckedList")?.nb_entities() {
                if let Some(rec) = self.result_from_number(i)?.get() {
                    res.extend(rec.checked_list(withcheck, result)?);
                }
            }
        } else if let Some(list) = listed(ent) {
            for item in list {
                if let Some(rec) = self.final_result(&item).get() {
                    res.extend(rec.checked_list(withcheck, result)?);
                }
            }
        } else if let Some(rec) = self.final_result(ent).get() {
            res = rec.checked_list(withcheck, result)?;
        }
        Ok(res)
    }
    /// `LastCheckList`: the check list of the process (empty without one).
    pub fn last_check_list(&self) -> InterfaceCheckIterator {
        self.transient_process()
            .get()
            .map_or_else(InterfaceCheckIterator::new, |p| p.check_list(false))
    }
    /// `LastTransferList(theRoots)`: the roots (or mapped items) of the process with a
    /// result.
    pub fn last_transfer_list(
        &self,
        roots: bool,
    ) -> Result<Vec<Handle<StandardTransient>>, Failure> {
        let (tp, model) = (self.transient_process(), self.model());
        let mut li = Vec::new();
        let Some(process) = tp.get() else {
            return Ok(li);
        };
        let nb = if roots {
            process.nb_roots()
        } else {
            process.nb_mapped()
        };
        for j in 1..=nb {
            let i = if roots {
                live_model(&model, "LastTransferList")?.number(&process.root(j)?)
            } else {
                j
            };
            let bnd = process.map_item(i)?;
            if bnd.get().is_some_and(|b| b.has_result()) {
                li.push(process.mapped(i)?);
            }
        }
        Ok(li)
    }
    /// `ShapeResultList(theRec)`: the shapes of the last transfer (computed once from the
    /// roots of the process), or (`rec`) the shapes of the final results.
    pub fn shape_result_list(&self, rec: bool) -> Result<Vec<Shape>, Failure> {
        if !rec {
            if self.state.read().shape_result.is_none() {
                let shapes = TransferBRep::shapes(&self.transient_process(), true)?;
                self.state.write().shape_result = Some(shapes.unwrap_or_default());
            }
            return Ok(self.state.read().shape_result.clone().unwrap_or_default());
        }
        let model = self.model();
        let Some(m) = model.get() else {
            let mut st = self.state.write();
            return Ok(st.shape_result.get_or_insert_with(Vec::new).clone());
        };
        let mut shapes = Vec::new();
        for i in 1..=m.nb_entities() {
            let sh = self.shape_result(&m.value(i)?)?;
            if !sh.is_null() {
                shapes.push(sh);
            }
        }
        self.state.write().shape_result = Some(shapes.clone());
        Ok(shapes)
    }
}
