// Rust port of DataExchange/TKXSBase/Interface/Interface_CopyTool (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_CopyTool`: copies entities of a model through the general modules of its
//! norm, keeping the map of the copies, the reports and the roots of the copy.
use super::errors::interface_error;
use super::{
    InterfaceBitMap, InterfaceCopyControl, InterfaceCopyMap, InterfaceEntityIterator,
    InterfaceGeneralLib, InterfaceGeneralModule, InterfaceInterfaceModel, InterfaceModelEntities,
    InterfaceModelReports, InterfaceProtocol, InterfaceProtocolBase, InterfaceReportEntity,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, RuntimeTyped, StandardTransient, exceptions},
    tcollection::HAsciiString,
};

/// `Interface_CopyTool`. No OCCT class overrides its virtual members (`Clear`, `Copy`,
/// `NewVoid`, `Implied`), so they are inherent methods.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceCopyTool;
/// fn roots(tool: &InterfaceCopyTool) -> i32 { tool.root_result(false).map_or(0, |r| r.nb_entities()) }
/// ```
pub struct InterfaceCopyTool {
    thelib: InterfaceGeneralLib,
    themod: Handle<dyn InterfaceInterfaceModel>,
    themap: Handle<dyn InterfaceCopyControl>,
    therep: Handle<InterfaceCopyMap>,
    thelst: InterfaceBitMap,
    thelev: i32,
    therts: Vec<i32>,
    theimp: bool,
    theent: Handle<StandardTransient>,
    themdu: Handle<dyn InterfaceGeneralModule>,
    the_cn: i32,
}

impl InterfaceCopyTool {
    /// `Interface_CopyTool(amodel, lib)`.
    pub fn with_lib(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        lib: &InterfaceGeneralLib,
    ) -> Result<Self, Failure> {
        let nb = amodel.get().map_or(0, |m| m.nb_entities());
        let thelst = InterfaceBitMap::with_items(nb, 0)?;
        thelst.init(false, 0)?;
        Ok(Self {
            thelib: lib.clone(),
            themod: amodel.clone(),
            themap: Handle::new(InterfaceCopyMap::new(amodel)).cast(),
            therep: Handle::new(InterfaceCopyMap::new(amodel)),
            thelst,
            thelev: 0,
            therts: Vec::new(),
            theimp: false,
            theent: Handle::default(),
            themdu: Handle::default(),
            the_cn: 0,
        })
    }
    /// `Interface_CopyTool(amodel, protocol)`.
    pub fn with_protocol(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<Self, Failure> {
        Self::with_lib(amodel, &InterfaceGeneralLib::new(protocol))
    }
    /// `Interface_CopyTool(amodel)`: with the active protocol;
    /// `Interface_InterfaceError` when none is active.
    pub fn new(amodel: &Handle<dyn InterfaceInterfaceModel>) -> Result<Self, Failure> {
        let active = InterfaceProtocolBase::active();
        if active.is_null() {
            return Err(interface_error(
                "Interface CopyTool : Create with Active Protocol undefined",
            ));
        }
        Self::with_protocol(amodel, &active)
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.themod.clone()
    }
    /// `SetControl(othermap)`.
    pub fn set_control(&mut self, othermap: &Handle<dyn InterfaceCopyControl>) {
        self.themap = othermap.clone();
    }
    /// `Control`.
    pub fn control(&self) -> Handle<dyn InterfaceCopyControl> {
        self.themap.clone()
    }
    /// `Clear`: forgets the copies, the roots and the flags.
    pub fn clear(&mut self) -> Result<(), Failure> {
        if let Some(map) = self.themap.get() {
            map.clear();
        }
        self.therep.clear();
        self.thelev = 0;
        self.theimp = false;
        self.therts.clear();
        self.clear_last_flags()
    }
    /// `NewVoid(entfrom, entto)` (protected): an empty copy from the module (or a copy
    /// from `NewCopiedCase`); false without module.
    pub fn new_void(
        &mut self,
        entfrom: &Handle<StandardTransient>,
        entto: &mut Handle<StandardTransient>,
    ) -> Result<bool, Failure> {
        if *entfrom == self.theent {
            return Ok(self
                .themdu
                .get()
                .is_some_and(|m| m.new_void(self.the_cn, entto)));
        }
        self.theent = entfrom.clone();
        let selected = self.thelib.select(entfrom);
        let mut res = false;
        if let Some((module, cn)) = selected.clone() {
            (self.themdu, self.the_cn) = (module, cn);
            res = self.themdu.new_void(self.the_cn, entto);
        } else {
            (self.themdu, self.the_cn) = (Handle::default(), 0);
        }
        // Natively the module is dereferenced even when none was selected (a crash).
        if !res && let Some((module, cn)) = selected {
            res = module.new_copied_case(cn, entfrom, entto, self)?;
        }
        Ok(res)
    }
    /// `Copy(entfrom, entto, mapped, errstat)`: a new void entity, mapped when asked,
    /// filled by the module unless `errstat`; a string without module is duplicated.
    pub fn copy(
        &mut self,
        entfrom: &Handle<StandardTransient>,
        entto: &mut Handle<StandardTransient>,
        mapped: bool,
        errstat: bool,
    ) -> Result<bool, Failure> {
        let mut res = true;
        if *entfrom == self.theent {
            res = self.themdu.is_some();
        } else {
            self.theent = entfrom.clone();
            match self.thelib.select(entfrom) {
                Some((module, cn)) => (self.themdu, self.the_cn) = (module, cn),
                None => {
                    (self.themdu, self.the_cn) = (Handle::default(), 0);
                    res = false;
                }
            }
        }
        if !res {
            return Ok(copy_string(entfrom, entto));
        }
        res = self.new_void(entfrom, entto)?;
        if mapped {
            self.live_map()?.bind(entfrom, entto)?; // Map before continuing ...
        }
        if !errstat {
            let (module, cn) = (self.themdu.clone(), self.the_cn);
            module.copy_case(cn, entfrom, entto, self)?;
        }
        Ok(res)
    }
    /// `Implied(entfrom, entto)` (private): the module renews the implied references.
    fn implied(
        &self,
        entfrom: &Handle<StandardTransient>,
        entto: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        if let Some((module, cn)) = self.thelib.select(entfrom) {
            module.renew_implied_case(cn, entfrom, entto, self)?;
        }
        Ok(())
    }
    /// `Transferred(ent)`: the copy of `ent`, made when not yet done (with its report);
    /// `Interface_InterfaceError` for an entity out of the model beyond 100 levels.
    pub fn transferred(
        &mut self,
        ent: &Handle<StandardTransient>,
    ) -> Result<Handle<StandardTransient>, Failure> {
        let mut res = Handle::default();
        if ent.is_null() {
            return Ok(res); // Copy of a Null : very simple ...
        }
        let nument = self.live_model()?.number(ent);
        if nument == 0 && self.thelev > 100 {
            return Err(interface_error(
                "CopyTool : Transferred, Entity is not contained in Starting Model",
            ));
        }
        if !self.live_map()?.search(ent, &mut res)? {
            let redefined = self.live_model()?.is_redefined_content(nument);
            if !self.copy(ent, &mut res, nument != 0, redefined)? {
                return Ok(res);
            }
            self.thelev += 1;
            if nument != 0 {
                self.thelst.set_true(nument, 0)?;
            }
            self.copy_report(ent, nument, &res)?;
            self.thelev -= 1;
        }
        if self.thelev == 0 && nument > 0 {
            self.therts.push(nument);
        }
        Ok(res)
    }
    /// The report part of `Transferred`: the copy gets a report like the original's.
    fn copy_report(
        &mut self,
        ent: &Handle<StandardTransient>,
        nument: i32,
        res: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        let rep = if nument != 0 {
            self.live_model()?.report_entity(nument, false)
        } else {
            Handle::default()
        };
        let Some(report) = rep.get() else {
            return Ok(());
        };
        if report.is_unknown() {
            let copy = Handle::new(InterfaceReportEntity::unknown(res));
            return self.therep.bind(ent, &copy.cast());
        }
        let repto = Handle::new(InterfaceReportEntity::with_check(&report.check(), res));
        let contfrom = report.content();
        if contfrom.is_some() {
            let mut contto = Handle::default();
            if contfrom == *ent {
                contto = res.clone();
            } else {
                let contained = self.live_model()?.contains(&contfrom);
                self.copy(&contfrom, &mut contto, contained, false)?;
            }
            repto.set_content(&contto);
        }
        self.therep.bind(ent, &repto.cast())
    }
    /// `Bind(ent, res)`: records a copy made outside.
    pub fn bind(
        &mut self,
        ent: &Handle<StandardTransient>,
        res: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        let num = self.live_model()?.number(ent);
        self.live_map()?.bind(ent, res)?;
        self.thelst.set_true(num, 0)
    }
    /// `Search(ent, res)`.
    pub fn search(
        &self,
        ent: &Handle<StandardTransient>,
        res: &mut Handle<StandardTransient>,
    ) -> Result<bool, Failure> {
        self.live_map()?.search(ent, res)
    }
    /// `ClearLastFlags`.
    pub fn clear_last_flags(&mut self) -> Result<(), Failure> {
        self.thelst.init(false, 0)
    }
    /// `LastCopiedAfter(numfrom, ent, res)`: the first entity copied since the last
    /// `ClearLastFlags`, after `numfrom`; 0 when none.
    pub fn last_copied_after(
        &self,
        numfrom: i32,
        ent: &mut Handle<StandardTransient>,
        res: &mut Handle<StandardTransient>,
    ) -> Result<i32, Failure> {
        for num in numfrom + 1..=self.thelst.length() {
            if self.thelst.value(num, 0)? {
                *ent = self.live_model()?.value(num)?;
                if self.live_map()?.search(ent, res)? {
                    return Ok(num);
                }
            }
        }
        Ok(0)
    }
    /// `TransferEntity(ent)`: `Transferred` without the result.
    pub fn transfer_entity(&mut self, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        self.transferred(ent).map(|_| ())
    }
    /// `RenewImpliedRefs`: once, the implied references of the copies.
    pub fn renew_implied_refs(&mut self) -> Result<(), Failure> {
        if self.theimp {
            return Ok(()); // already done
        }
        self.theimp = true;
        for i in 1..=self.live_model()?.nb_entities() {
            let ent = self.live_model()?.value(i)?;
            let mut res = Handle::default();
            if !self.live_map()?.search(&ent, &mut res)? {
                continue; // entity not transferred
            }
            let mut rep = Handle::default();
            if !self.therep.search(&ent, &mut rep)? {
                self.implied(&ent, &res)?;
                continue;
            }
            let report = rep.cast::<InterfaceReportEntity>();
            if report.get().is_some_and(|r| !r.has_new_content()) {
                self.implied(&ent, &res)?;
            }
        }
        Ok(())
    }
    /// `FillModel(bmodel)`: `bmodel` gets the header of the model and the copies.
    pub fn fill_model(
        &mut self,
        bmodel: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<(), Failure> {
        bmodel.clear();
        bmodel.get_from_another(&self.themod)?;
        let list = self.complete_result(true)?;
        bmodel.get_from_transfer(&list)?;
        self.renew_implied_refs()
    }
    /// `CompleteResult(withreports = false)`: the copies in the order of the model.
    pub fn complete_result(&self, withreports: bool) -> Result<InterfaceEntityIterator, Failure> {
        let numbers: Vec<i32> = (1..=self.live_model()?.nb_entities()).collect();
        self.results(&numbers, withreports)
    }
    /// `RootResult(withreports = false)`: the copies of the roots of the transfer.
    pub fn root_result(&self, withreports: bool) -> Result<InterfaceEntityIterator, Failure> {
        self.results(&self.therts, withreports)
    }
    fn live_map(&self) -> Result<&dyn InterfaceCopyControl, Failure> {
        self.themap.get().ok_or_else(|| {
            exceptions::null_object("Interface_CopyTool: null copy control; expected a map")
        })
    }
    fn live_model(&self) -> Result<&dyn InterfaceInterfaceModel, Failure> {
        self.themod.get().ok_or_else(|| {
            exceptions::null_object("Interface_CopyTool: null model; expected a model")
        })
    }
    fn results(
        &self,
        numbers: &[i32],
        withreports: bool,
    ) -> Result<InterfaceEntityIterator, Failure> {
        let mut iter = InterfaceEntityIterator::new();
        for num in numbers {
            let ent = self.live_model()?.value(*num)?;
            let mut res = Handle::default();
            if !self.live_map()?.search(&ent, &mut res)? {
                continue;
            }
            let mut rep = Handle::default();
            if withreports && self.therep.search(&ent, &mut rep)? {
                res = rep;
            }
            iter.get_one_item(&res);
        }
        Ok(iter)
    }
}

/// The copy of `Copy` without module: a string is duplicated, anything else is not
/// copied.
fn copy_string(entfrom: &Handle<StandardTransient>, entto: &mut Handle<StandardTransient>) -> bool {
    let Some(kind) = entfrom.dynamic_type() else {
        return false;
    };
    if !std::ptr::eq(kind, HAsciiString::static_type()) {
        return false;
    }
    let text = entfrom.cast::<HAsciiString>();
    *entto = Handle::new(HAsciiString::copied(&text)).cast();
    true
}
