// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_ModelCopier (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_ModelCopier`: the split of a model by a share-out into file models, kept
//! for a later sending or sent at once, with the record of the sent files and of the
//! entities left out of every file (the remaining data).
//!
//! The copy and send operations are in `model_copier_send`.
use super::{IFSelectAppliedModifiers, IFSelectShareOut};
use crate::interface::{InterfaceCopyTool, InterfaceGraph, InterfaceInterfaceModel};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::{AsciiString, HAsciiString},
};

/// The fields of `IFSelect_ModelCopier`.
#[derive(Default)]
pub(super) struct CopierState {
    pub(super) thefilemodels: Vec<Handle<dyn InterfaceInterfaceModel>>,
    pub(super) thefilenames: Vec<AsciiString>,
    pub(super) theapplieds: Vec<Handle<IFSelectAppliedModifiers>>,
    pub(super) theshareout: Handle<IFSelectShareOut>,
    /// The `0..=Upper` counts of the copies of each entity (`None` for a null array).
    pub(super) theremain: Option<Vec<i32>>,
    pub(super) thesentfiles: Option<Vec<Handle<HAsciiString>>>,
}

/// `IFSelect_ModelCopier`.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectModelCopier;
/// let copier = IFSelectModelCopier::new();
/// assert_eq!(copier.nb_files(), 0);
/// ```
#[derive(Default)]
pub struct IFSelectModelCopier {
    transient: StandardTransient,
    pub(super) state: SharedState<CopierState>,
}
impl_transient!(
    IFSelectModelCopier,
    "IFSelect_ModelCopier",
    StandardTransient,
    transient
);

/// The 0-based index of the 1-based `num` in a sequence of `len` items.
fn seq_index(num: i32, len: usize) -> Option<usize> {
    usize::try_from(num - 1).ok().filter(|i| *i < len)
}

/// `Standard_OutOfRange` for the file `num` out of 1..`len`.
fn file_out_of_range(what: &str, num: i32, len: usize) -> Failure {
    exceptions::out_of_range(&format!(
        "IFSelect_ModelCopier::{what}: file {num}; expected [1, {len}]"
    ))
}

impl IFSelectModelCopier {
    /// `IFSelect_ModelCopier()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetShareOut(sho)`.
    pub fn set_share_out(&self, sho: &Handle<IFSelectShareOut>) {
        self.state.write().theshareout = sho.clone();
    }
    /// `ClearResult`: the files and the remaining data are dropped.
    pub fn clear_result(&self) {
        let mut st = self.state.write();
        st.thefilemodels.clear();
        st.thefilenames.clear();
        st.theapplieds.clear();
        st.theremain = None;
    }
    /// True when `filename` is not empty and already names a file.
    fn name_taken(st: &CopierState, filename: &AsciiString) -> bool {
        !filename.is_empty() && st.thefilenames.iter().any(|n| n == filename)
    }
    /// `AddFile(filename, content)`: false when the name is already used (an empty name
    /// is never a duplicate).
    pub fn add_file(
        &self,
        filename: &AsciiString,
        content: &Handle<dyn InterfaceInterfaceModel>,
    ) -> bool {
        let mut st = self.state.write();
        if Self::name_taken(&st, filename) {
            return false;
        }
        st.thefilenames.push(filename.clone());
        st.thefilemodels.push(content.clone());
        st.theapplieds.push(Handle::default());
        true
    }
    /// `NameFile(num, filename)`: false out of range or for a name already used.
    pub fn name_file(&self, num: i32, filename: &AsciiString) -> bool {
        let mut st = self.state.write();
        let Some(i) = seq_index(num, st.thefilenames.len()) else {
            return false;
        };
        if Self::name_taken(&st, filename) {
            return false;
        }
        st.thefilenames[i] = filename.clone();
        true
    }
    /// `ClearFile(num)`: the name is emptied (the file is not sent); false out of range.
    pub fn clear_file(&self, num: i32) -> bool {
        let mut st = self.state.write();
        let Some(i) = seq_index(num, st.thefilenames.len()) else {
            return false;
        };
        st.thefilenames[i] = AsciiString::default();
        true
    }
    /// `SetAppliedModifiers(num, applied)`: false out of range.
    pub fn set_applied_modifiers(
        &self,
        num: i32,
        applied: &Handle<IFSelectAppliedModifiers>,
    ) -> bool {
        let mut st = self.state.write();
        let Some(i) = seq_index(num, st.theapplieds.len()) else {
            return false;
        };
        st.theapplieds[i] = applied.clone();
        true
    }
    /// `ClearAppliedModifiers(num)`: false out of range.
    pub fn clear_applied_modifiers(&self, num: i32) -> bool {
        self.set_applied_modifiers(num, &Handle::default())
    }
    /// `NbFiles`.
    pub fn nb_files(&self) -> i32 {
        self.state.read().thefilemodels.len() as i32
    }
    /// `FileName(num)`: `Standard_OutOfRange` out of 1..NbFiles.
    pub fn file_name(&self, num: i32) -> Result<AsciiString, Failure> {
        let st = self.state.read();
        let len = st.thefilenames.len();
        seq_index(num, len)
            .map(|i| st.thefilenames[i].clone())
            .ok_or_else(|| file_out_of_range("FileName", num, len))
    }
    /// `FileModel(num)`: `Standard_OutOfRange` out of 1..NbFiles.
    pub fn file_model(&self, num: i32) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        let st = self.state.read();
        let len = st.thefilemodels.len();
        seq_index(num, len)
            .map(|i| st.thefilemodels[i].clone())
            .ok_or_else(|| file_out_of_range("FileModel", num, len))
    }
    /// `AppliedModifiers(num)`: `Standard_OutOfRange` out of 1..NbFiles.
    pub fn applied_modifiers(&self, num: i32) -> Result<Handle<IFSelectAppliedModifiers>, Failure> {
        let st = self.state.read();
        let len = st.theapplieds.len();
        seq_index(num, len)
            .map(|i| st.theapplieds[i].clone())
            .ok_or_else(|| file_out_of_range("AppliedModifiers", num, len))
    }
    /// `SetRemaining(CG)`: adds the copy counts to the statuses of `CG` (the negative
    /// statuses are kept) and resets the counts. False when the size of `CG` is not the
    /// upper bound of the counts; without counts, true only for an empty graph. Native
    /// quirk kept: the counts of `CopiedRemaining` have the bound `size + 1`, so they
    /// never match.
    pub fn set_remaining(&self, cg: &mut InterfaceGraph) -> Result<bool, Failure> {
        let nb = cg.size();
        let mut st = self.state.write();
        let Some(remain) = st.theremain.as_mut() else {
            return Ok(nb == 0);
        };
        if nb != remain.len() as i32 - 1 {
            return Ok(false);
        }
        for i in 1..=nb {
            let status = cg.status(i)?;
            if status >= 0 {
                cg.set_status(i, status + remain[i as usize])?;
            }
        }
        remain.fill(0);
        Ok(true)
    }
    /// `BeginSentFiles(sho, record)`: starts (or stops) the record of the sent files;
    /// the result of `sho` is cleared, its last run kept.
    pub fn begin_sent_files(&self, sho: &Handle<IFSelectShareOut>, record: bool) {
        self.state.write().thesentfiles = record.then(Vec::new);
        let Some(live) = sho.get() else {
            return;
        };
        let lastrun = live.last_run();
        live.clear_result(true);
        live.set_last_run(lastrun); // we are only interested in the numbers
    }
    /// `AddSentFile(filename)`: recorded when the record is on.
    pub fn add_sent_file(&self, filename: &str) {
        if let Some(files) = self.state.write().thesentfiles.as_mut() {
            files.push(Handle::new(HAsciiString::new(filename)));
        }
    }
    /// `SentFiles`: `None` when the record is off.
    pub fn sent_files(&self) -> Option<Vec<Handle<HAsciiString>>> {
        self.state.read().thesentfiles.clone()
    }
    /// Updates the copy count of each entity copied since the last clear of `tc`, within
    /// the counts (`CopiedModel` adds one, `CopiedRemaining` sets one).
    pub(super) fn update_copies(
        &self,
        tc: &InterfaceCopyTool,
        update: fn(&mut i32),
    ) -> Result<(), Failure> {
        let (mut ent1, mut ent2) = (Handle::default(), Handle::default());
        let mut ic = tc.last_copied_after(0, &mut ent1, &mut ent2)?;
        while ic > 0 {
            if let Some(slot) = self
                .state
                .write()
                .theremain
                .as_mut()
                .and_then(|r| r.get_mut(ic as usize))
            {
                update(slot);
            }
            ic = tc.last_copied_after(ic, &mut ent1, &mut ent2)?;
        }
        Ok(())
    }
}
