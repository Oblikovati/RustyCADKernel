// Rust port of DataExchange/TKXSBase/Transfer/Transfer_ResultFromModel (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Transfer_ResultFromModel`: the result of the transfer of one entity of a model, kept
//! after the transfer process (its main result and the identity of the entity).
use super::{TransferResultFromTransient, TransferTransientProcess};
use crate::interface::{
    InterfaceCheckIterator, InterfaceCheckStatus, InterfaceInterfaceModel, InterfaceModelEntities,
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::IndexedMap,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::AsciiString,
};

/// The private fields of `Transfer_ResultFromModel`.
struct ModelResultState {
    themodel: Handle<dyn InterfaceInterfaceModel>,
    thename: AsciiString,
    themain: Handle<TransferResultFromTransient>,
    themlab: AsciiString,
    themnum: i32,
    themchk: InterfaceCheckStatus,
}

/// `Transfer_ResultFromModel`.
///
/// ```
/// use rustycad_tkxsbase::transfer::TransferResultFromModel;
/// let result = TransferResultFromModel::new();
/// result.set_file_name("part.stp");
/// assert_eq!(result.file_name().as_bytes(), b"part.stp");
/// assert!(!result.has_result() && result.main_number() == 0);
/// ```
pub struct TransferResultFromModel {
    transient: StandardTransient,
    state: SharedState<ModelResultState>,
}
impl_transient!(
    TransferResultFromModel,
    "Transfer_ResultFromModel",
    StandardTransient,
    transient
);

impl Default for TransferResultFromModel {
    fn default() -> Self {
        Self::new()
    }
}

/// The native null dereference of a missing main result.
fn no_main(member: &str) -> Failure {
    exceptions::null_object(&format!(
        "Transfer_ResultFromModel::{member}: no main result; expected Fill or SetMainResult first"
    ))
}

impl TransferResultFromModel {
    /// `Transfer_ResultFromModel()`: number 0, check status "any" (not computed).
    pub fn new() -> Self {
        let state = ModelResultState {
            themodel: Handle::default(),
            thename: AsciiString::default(),
            themain: Handle::default(),
            themlab: AsciiString::default(),
            themnum: 0,
            themchk: InterfaceCheckStatus::CheckAny,
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `SetModel(model)`.
    pub fn set_model(&self, model: &Handle<dyn InterfaceInterfaceModel>) {
        self.state.write().themodel = model.clone();
    }
    /// `SetFileName(filename)`.
    pub fn set_file_name(&self, filename: &str) {
        self.state.write().thename = AsciiString::new(filename);
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.state.read().themodel.clone()
    }
    /// `FileName`.
    pub fn file_name(&self) -> AsciiString {
        self.state.read().thename.clone()
    }
    /// Records the number and the label of `start` in the model.
    fn set_main_identity(&self, start: &Handle<StandardTransient>) {
        let model = self.model();
        let Some(live) = model.get() else { return };
        let num = live.number(start);
        let label = if num > 0 {
            live.string_label(start)
                .get()
                .map(|l| l.string().clone())
                .unwrap_or_default()
        } else {
            AsciiString::default()
        };
        let mut st = self.state.write();
        st.themnum = num;
        st.themlab = label;
    }
    /// `Fill(TP, ent)`: the main result of `ent` from its binder in `TP` (false without).
    pub fn fill(
        &self,
        tp: &Handle<TransferTransientProcess>,
        ent: &Handle<StandardTransient>,
    ) -> bool {
        let Some(process) = tp.get() else {
            return false;
        };
        if ent.is_null() {
            return false;
        }
        let binder = process.find(ent);
        if binder.is_null() {
            return false;
        }
        let main = Handle::new(TransferResultFromTransient::new());
        main.set_start(ent);
        main.set_binder(&binder);
        main.fill(tp);
        let tp_model = process.model();
        if tp_model.is_some() {
            self.state.write().themodel = tp_model;
        }
        self.state.write().themain = main;
        self.set_main_identity(ent);
        true
    }
    /// `Strip(mode)`: from mode 10, keeps only the check status and the main binder (the
    /// main result too beyond 10).
    pub fn strip(&self, mode: i32) {
        let main = self.main_result();
        let Some(live) = main.get() else { return };
        live.strip();
        if mode < 10 {
            return;
        }
        // the main result exists: the check list cannot fail
        let chk = self
            .compute_check_status(false)
            .unwrap_or(InterfaceCheckStatus::CheckAny);
        let mut st = self.state.write();
        st.themchk = chk;
        st.themodel = Handle::default();
        live.clear_subs();
        live.set_start(&Handle::default());
        if mode > 10 {
            st.themain = Handle::default();
        }
    }
    /// `FillBack(TP)`: gives the model to `TP`; `Standard_NullObject` without main result
    /// or `TP` (native null dereferences).
    pub fn fill_back(&self, tp: &Handle<TransferTransientProcess>) -> Result<(), Failure> {
        let model = self.model();
        if model.is_some() {
            tp.get().ok_or_else(|| exceptions::null_object("Transfer_ResultFromModel::FillBack: null process; expected a transient process"))?.set_model(&model);
        }
        self.main_result()
            .get()
            .ok_or_else(|| no_main("FillBack"))?
            .fill_back(tp);
        Ok(())
    }
    /// `HasResult`.
    pub fn has_result(&self) -> bool {
        self.main_result()
            .get()
            .is_some_and(TransferResultFromTransient::has_result)
    }
    /// `MainResult`.
    pub fn main_result(&self) -> Handle<TransferResultFromTransient> {
        self.state.read().themain.clone()
    }
    /// `SetMainResult(amain)`: resets the check status, records the identity of its start.
    pub fn set_main_result(&self, amain: &Handle<TransferResultFromTransient>) {
        {
            let mut st = self.state.write();
            st.themchk = InterfaceCheckStatus::CheckAny;
            st.themain = amain.clone();
        }
        if let Some(live) = amain.get() {
            self.set_main_identity(&live.start());
        }
    }
    /// `MainLabel`.
    pub fn main_label(&self) -> AsciiString {
        self.state.read().themlab.clone()
    }
    /// `MainNumber`.
    pub fn main_number(&self) -> i32 {
        self.state.read().themnum
    }
    /// `ResultFromKey(start)`: `Standard_NullObject` without main result.
    pub fn result_from_key(
        &self,
        start: &Handle<StandardTransient>,
    ) -> Result<Handle<TransferResultFromTransient>, Failure> {
        let main = self.main_result();
        if main.is_null() {
            return Err(no_main("ResultFromKey"));
        }
        Ok(TransferResultFromTransient::result_from_key(&main, start))
    }
    /// `Results(level)`: the main result (level 0 or less); the main result, its
    /// sub-results and the main result again (level 1, native quirk kept); all the results
    /// in depth (level 2 and more). `Standard_NullObject` without main result from level 1.
    pub fn results(&self, level: i32) -> Result<Vec<Handle<StandardTransient>>, Failure> {
        let main = self.main_result();
        if level <= 0 {
            return Ok(vec![main.cast()]);
        }
        let live = main.get().ok_or_else(|| no_main("Results"))?;
        if level == 1 {
            let mut list = vec![main.cast()];
            list.extend((1..=live.nb_sub_results()).map(|i| live.sub_result(i).cast()));
            list.push(main.cast());
            return Ok(list);
        }
        let buckets = self.model().get().map_or(1000, |m| m.nb_entities());
        let mut map = IndexedMap::new(buckets.max(1) as usize);
        map.add(main.cast());
        live.fill_map(&mut map);
        Ok(map.iter().cloned().collect())
    }
    /// `TransferredList(level = 2)`: the starts of the results with a result.
    pub fn transferred_list(&self, level: i32) -> Result<Vec<Handle<StandardTransient>>, Failure> {
        Ok(self
            .results(level)?
            .iter()
            .filter_map(|res| {
                res.cast::<TransferResultFromTransient>()
                    .get()
                    .filter(|r| r.has_result())
                    .map(TransferResultFromTransient::start)
            })
            .collect())
    }
    /// `CheckedList(check, result)`: the starts whose check complies with `check` (with a
    /// result when `result`).
    pub fn checked_list(
        &self,
        check: InterfaceCheckStatus,
        result: bool,
    ) -> Result<Vec<Handle<StandardTransient>>, Failure> {
        let mut list = Vec::new();
        for res in self.results(2)? {
            let unres = res.cast::<TransferResultFromTransient>();
            let Some(live) = unres.get() else { continue };
            if (!result || live.has_result()) && live.check().complies(check) {
                list.push(live.start());
            }
        }
        Ok(list)
    }
    /// `CheckList(erronly, level = 2)`: the binder checks of the results with messages
    /// (fails only for `erronly`), numbered in the model (0 without model).
    pub fn check_list(&self, erronly: bool, level: i32) -> Result<InterfaceCheckIterator, Failure> {
        let mut chl = InterfaceCheckIterator::new();
        let model = self.model();
        for res in self.results(level)? {
            let unres = res.cast::<TransferResultFromTransient>();
            let Some(live) = unres.get() else { continue };
            let stat = live.check_status();
            if stat == InterfaceCheckStatus::CheckOK
                || (stat == InterfaceCheckStatus::CheckWarning && erronly)
            {
                continue;
            }
            let bch = live.binder().check();
            bch.set_entity(&live.start());
            chl.add(&bch, model.get().map_or(0, |m| m.number(&live.start())));
        }
        Ok(chl)
    }
    /// `CheckStatus`: the recorded status, else the status of the whole check list.
    pub fn check_status(&self) -> Result<InterfaceCheckStatus, Failure> {
        let chk = self.state.read().themchk;
        if chk != InterfaceCheckStatus::CheckAny {
            return Ok(chk);
        }
        Ok(self.check_list(false, 2)?.status())
    }
    /// `ComputeCheckStatus(enforce)`: records `CheckStatus` when not recorded (or when
    /// `enforce`: native quirk kept, `CheckStatus` then returns the recorded one).
    pub fn compute_check_status(&self, enforce: bool) -> Result<InterfaceCheckStatus, Failure> {
        let chk = self.state.read().themchk;
        if chk != InterfaceCheckStatus::CheckAny && !enforce {
            return Ok(chk);
        }
        let computed = self.check_status()?;
        self.state.write().themchk = computed;
        Ok(computed)
    }
}
