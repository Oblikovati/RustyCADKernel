// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_ShareOut (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_ShareOut`: the dispatches of a share-out, the model and file modifiers, and
//! the naming of the output files (prefix, root names, default root, extension).
use super::{IFSelectDispatch, IFSelectGeneralModifier, IFSelectModifier};
use rustycad_tkernel::{
    impl_transient,
    standard::{
        Failure, Handle, SharedState, StandardTransient,
        c_string::{FormatArg, sprintf_into},
        exceptions,
    },
    tcollection::{AsciiString, HAsciiString},
};

/// The fields of `IFSelect_ShareOut`.
struct ShareOutState {
    thedisps: Vec<Handle<dyn IFSelectDispatch>>,
    themodelmodifiers: Vec<Handle<dyn IFSelectGeneralModifier>>,
    thefilemodifiers: Vec<Handle<dyn IFSelectGeneralModifier>>,
    thepref: Handle<HAsciiString>,
    thedefrt: Handle<HAsciiString>,
    theext: Handle<HAsciiString>,
    thenbdefs: i32,
    thelastrun: i32,
}

/// `IFSelect_ShareOut`.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectShareOut;
/// let share = IFSelectShareOut::new();
/// assert_eq!(share.file_name(1, 1, 0).unwrap().as_bytes(), b"Default_1");
/// ```
pub struct IFSelectShareOut {
    transient: StandardTransient,
    state: SharedState<ShareOutState>,
}
impl_transient!(
    IFSelectShareOut,
    "IFSelect_ShareOut",
    StandardTransient,
    transient
);

impl Default for IFSelectShareOut {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether a modifier is a model modifier (`IFSelect_Modifier`); `Standard_NullObject`
/// for null (a native null dereference).
fn for_model(modifier: &Handle<dyn IFSelectGeneralModifier>) -> Result<bool, Failure> {
    if modifier.is_null() {
        return Err(exceptions::null_object(
            "IFSelect_ShareOut: null modifier; expected a modifier",
        ));
    }
    Ok(modifier.cast::<dyn IFSelectModifier>().is_some())
}

/// The item `num` (from 1) of a list; `Standard_OutOfRange` out of it.
fn item<T: Clone>(list: &[T], num: i32, what: &str) -> Result<T, Failure> {
    usize::try_from(num - 1)
        .ok()
        .and_then(|i| list.get(i))
        .cloned()
        .ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "IFSelect_ShareOut::{what}: {num}; expected [1, {}]",
                list.len()
            ))
        })
}

/// Inserts before `atnum` (1-based) or appends out of range.
fn insert_at<T>(list: &mut Vec<T>, atnum: i32, value: T) {
    match usize::try_from(atnum - 1).ok().filter(|i| *i < list.len()) {
        Some(index) => list.insert(index, value),
        None => list.push(value),
    }
}

/// The numeric suffix of a file name: "_n" or "_00n" (as many digits as the packet
/// count), none for a number below 1 with no more packets.
fn suffix(num: i32, npac: i32) -> Result<Vec<u8>, Failure> {
    let mut nbch = 0;
    if npac >= num {
        let mut nbpa: i64 = 1;
        while nbpa <= i64::from(npac) {
            nbpa *= 10;
            nbch += 1;
        }
    }
    let format = if nbch > 1 {
        format!("_%{nbch}.{nbch}d")
    } else if npac >= num || num >= 1 {
        String::from("_%d")
    } else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    sprintf_into(&mut out, format.as_bytes(), &[FormatArg::Int(num)])
        .map_err(|e| Failure::new(&format!("IFSelect_ShareOut::FileName: {e}")))?;
    out.truncate(out.iter().position(|b| *b == 0).unwrap_or(out.len()));
    Ok(out)
}

impl IFSelectShareOut {
    /// `IFSelect_ShareOut()`: default root name "Default".
    pub fn new() -> Self {
        let state = ShareOutState {
            thedisps: Vec::new(),
            themodelmodifiers: Vec::new(),
            thefilemodifiers: Vec::new(),
            thepref: Handle::default(),
            thedefrt: Handle::new(HAsciiString::new("Default")),
            theext: Handle::default(),
            thenbdefs: 0,
            thelastrun: 0,
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `Clear(onlydisp)`: the dispatches (and the modifiers unless `onlydisp`).
    pub fn clear(&self, onlydisp: bool) {
        self.state.write().thedisps.clear();
        self.clear_result(!onlydisp);
        if !onlydisp {
            let mut st = self.state.write();
            st.themodelmodifiers.clear();
            st.thefilemodifiers.clear();
        }
    }
    /// `ClearResult(alsoname)`: no run (and no default name counted with `alsoname`).
    pub fn clear_result(&self, alsoname: bool) {
        let mut st = self.state.write();
        st.thelastrun = 0;
        if alsoname {
            st.thenbdefs = 0;
        }
    }
    /// `RemoveItem(item)`: removes a modifier or a dispatch.
    pub fn remove_item(&self, item: &Handle<StandardTransient>) -> Result<bool, Failure> {
        let modifier: Handle<dyn IFSelectGeneralModifier> = item.cast();
        if modifier.is_some() {
            let formodel = for_model(&modifier)?;
            return self.remove_modifier(formodel, self.modifier_rank(&modifier)?);
        }
        let disp: Handle<dyn IFSelectDispatch> = item.cast();
        if disp.is_some() {
            return Ok(self.remove_dispatch(self.dispatch_rank(&disp)));
        }
        Ok(false)
    }
    /// `LastRun`.
    pub fn last_run(&self) -> i32 {
        self.state.read().thelastrun
    }
    /// `SetLastRun(last)`.
    pub fn set_last_run(&self, last: i32) {
        self.state.write().thelastrun = last;
    }
    /// `NbDispatches`.
    pub fn nb_dispatches(&self) -> i32 {
        self.state.read().thedisps.len() as i32
    }
    /// `DispatchRank(disp)`: 0 when absent.
    pub fn dispatch_rank(&self, disp: &Handle<dyn IFSelectDispatch>) -> i32 {
        if disp.is_null() {
            return 0;
        }
        self.state
            .read()
            .thedisps
            .iter()
            .rposition(|d| d == disp)
            .map_or(0, |p| p as i32 + 1)
    }
    /// `Dispatch(num)`: `Standard_OutOfRange` out of 1..NbDispatches.
    pub fn dispatch(&self, num: i32) -> Result<Handle<dyn IFSelectDispatch>, Failure> {
        item(&self.state.read().thedisps, num, "Dispatch")
    }
    /// `AddDispatch(disp)`: nothing for null.
    pub fn add_dispatch(&self, disp: &Handle<dyn IFSelectDispatch>) {
        if disp.is_some() {
            self.state.write().thedisps.push(disp.clone());
        }
    }
    /// `RemoveDispatch(rank)`: false for a dispatch already run or out of range.
    pub fn remove_dispatch(&self, rank: i32) -> bool {
        let mut st = self.state.write();
        if rank <= st.thelastrun || rank > st.thedisps.len() as i32 {
            return false;
        }
        st.thedisps.remove(rank as usize - 1);
        true
    }
    /// `AddModifier(modifier, atnum)`: added once, for all the dispatches.
    pub fn add_modifier(
        &self,
        modifier: &Handle<dyn IFSelectGeneralModifier>,
        atnum: i32,
    ) -> Result<(), Failure> {
        self.add_modifier_once(modifier, atnum)?;
        modifier.set_dispatch(&Handle::default());
        Ok(())
    }
    /// `AddModifier(modifier, dispnum, atnum)`: added once, for the dispatch `dispnum`.
    pub fn add_modifier_for(
        &self,
        modifier: &Handle<dyn IFSelectGeneralModifier>,
        dispnum: i32,
        atnum: i32,
    ) -> Result<(), Failure> {
        self.add_modifier_once(modifier, atnum)?;
        modifier.set_dispatch(&self.dispatch(dispnum)?);
        Ok(())
    }
    /// Adds `modifier` at `atnum` when not yet present.
    fn add_modifier_once(
        &self,
        modifier: &Handle<dyn IFSelectGeneralModifier>,
        atnum: i32,
    ) -> Result<(), Failure> {
        let formodel = for_model(modifier)?;
        if self.modifier_rank(modifier)? == 0 {
            self.add_modif(modifier, formodel, atnum);
        }
        Ok(())
    }
    /// `AddModif(modifier, formodel, atnum = 0)`: inserts before `atnum` or appends.
    pub fn add_modif(
        &self,
        modifier: &Handle<dyn IFSelectGeneralModifier>,
        formodel: bool,
        atnum: i32,
    ) {
        let mut st = self.state.write();
        let list = if formodel {
            &mut st.themodelmodifiers
        } else {
            &mut st.thefilemodifiers
        };
        insert_at(list, atnum, modifier.clone());
    }
    /// `NbModifiers(formodel)`.
    pub fn nb_modifiers(&self, formodel: bool) -> i32 {
        let st = self.state.read();
        (if formodel {
            &st.themodelmodifiers
        } else {
            &st.thefilemodifiers
        })
        .len() as i32
    }
    /// `GeneralModifier(formodel, num)`: `Standard_OutOfRange` out of range.
    pub fn general_modifier(
        &self,
        formodel: bool,
        num: i32,
    ) -> Result<Handle<dyn IFSelectGeneralModifier>, Failure> {
        let st = self.state.read();
        item(
            if formodel {
                &st.themodelmodifiers
            } else {
                &st.thefilemodifiers
            },
            num,
            "GeneralModifier",
        )
    }
    /// `ModelModifier(num)`: the model modifier `num` as a modifier (null when not one).
    pub fn model_modifier(&self, num: i32) -> Result<Handle<dyn IFSelectModifier>, Failure> {
        Ok(item(&self.state.read().themodelmodifiers, num, "ModelModifier")?.cast())
    }
    /// `ModifierRank(modifier)`: its rank in its list (0 when absent).
    pub fn modifier_rank(
        &self,
        modifier: &Handle<dyn IFSelectGeneralModifier>,
    ) -> Result<i32, Failure> {
        let formodel = for_model(modifier)?;
        let st = self.state.read();
        let list = if formodel {
            &st.themodelmodifiers
        } else {
            &st.thefilemodifiers
        };
        Ok(list
            .iter()
            .rposition(|m| m == modifier)
            .map_or(0, |p| p as i32 + 1))
    }
    /// `RemoveModifier(formodel, num)`: false out of range.
    pub fn remove_modifier(&self, formodel: bool, num: i32) -> Result<bool, Failure> {
        let mut st = self.state.write();
        let list = if formodel {
            &mut st.themodelmodifiers
        } else {
            &mut st.thefilemodifiers
        };
        if num <= 0 || num > list.len() as i32 {
            return Ok(false);
        }
        list.remove(num as usize - 1);
        Ok(true)
    }
    /// `ChangeModifierRank(formodel, before, after)`: moves a modifier.
    pub fn change_modifier_rank(&self, formodel: bool, before: i32, after: i32) -> bool {
        if before <= 0 || after <= 0 {
            return false;
        }
        if before == after {
            return true;
        }
        let mut st = self.state.write();
        let list = if formodel {
            &mut st.themodelmodifiers
        } else {
            &mut st.thefilemodifiers
        };
        let nb = list.len() as i32;
        if before > nb || after > nb {
            return false;
        }
        let moved = list.remove(before as usize - 1);
        if after == nb {
            list.push(moved);
        } else {
            list.insert(after as usize - 1, moved);
        }
        true
    }
    /// `SetRootName(num, name)`: false out of range or for a name already used.
    pub fn set_root_name(&self, num: i32, name: &Handle<HAsciiString>) -> Result<bool, Failure> {
        if num < 1 || num > self.nb_dispatches() || self.root_number(name)? != 0 {
            return Ok(false);
        }
        self.dispatch(num)?.set_root_name(name);
        Ok(true)
    }
    /// `HasRootName(num)`.
    pub fn has_root_name(&self, num: i32) -> bool {
        self.dispatch(num).is_ok_and(|d| d.has_root_name())
    }
    /// `RootName(num)`: null out of range.
    pub fn root_name(&self, num: i32) -> Handle<HAsciiString> {
        self.dispatch(num)
            .map(|d| d.root_name())
            .unwrap_or_default()
    }
    /// `RootNumber(name)`: -1 for the default root name, the dispatch of the name, or 0.
    pub fn root_number(&self, name: &Handle<HAsciiString>) -> Result<i32, Failure> {
        if name.is_null() {
            return Ok(0);
        }
        let (defrt, disps) = {
            let st = self.state.read();
            (st.thedefrt.clone(), st.thedisps.clone())
        };
        if let Some(live) = defrt.get()
            && live.is_same_string(name)?
        {
            return Ok(-1);
        }
        for (i, disp) in disps.iter().enumerate() {
            let root = disp.root_name();
            if let Some(live) = root.get()
                && live.is_same_string(name)?
            {
                return Ok(i as i32 + 1);
            }
        }
        Ok(0)
    }
    /// `SetPrefix(pref)`: the default names are counted again.
    pub fn set_prefix(&self, pref: &Handle<HAsciiString>) {
        let mut st = self.state.write();
        st.thepref = pref.clone();
        st.thenbdefs = 0;
    }
    /// `SetDefaultRootName(defrt)`: false for a name used by a dispatch.
    pub fn set_default_root_name(&self, defrt: &Handle<HAsciiString>) -> Result<bool, Failure> {
        if self.root_number(defrt)? != 0 {
            return Ok(false);
        }
        let current = self.state.read().thedefrt.clone();
        let same = match current.get() {
            Some(live) => live.is_same_string(defrt)?,
            None => false,
        };
        let mut st = self.state.write();
        if !same {
            st.thenbdefs = 0;
        }
        st.thedefrt = defrt.clone();
        Ok(true)
    }
    /// `SetExtension(ext)`: the default names are counted again.
    pub fn set_extension(&self, ext: &Handle<HAsciiString>) {
        let mut st = self.state.write();
        st.theext = ext.clone();
        st.thenbdefs = 0;
    }
    /// A text field, an empty string when null.
    fn or_empty(value: &Handle<HAsciiString>) -> Handle<HAsciiString> {
        if value.is_null() {
            Handle::new(HAsciiString::new(""))
        } else {
            value.clone()
        }
    }
    /// `Prefix`: empty when not set.
    pub fn prefix(&self) -> Handle<HAsciiString> {
        Self::or_empty(&self.state.read().thepref)
    }
    /// `DefaultRootName`: empty when not set.
    pub fn default_root_name(&self) -> Handle<HAsciiString> {
        Self::or_empty(&self.state.read().thedefrt)
    }
    /// `Extension`: empty when not set.
    pub fn extension(&self) -> Handle<HAsciiString> {
        Self::or_empty(&self.state.read().theext)
    }
    /// `FileName(dnum, pnum, nbpack = 0)`: prefix, root name (else the default root,
    /// numbered by the count of default names), the packet suffix and the extension.
    pub fn file_name(&self, dnum: i32, pnum: i32, nbpack: i32) -> Result<AsciiString, Failure> {
        let mut rot = self.root_name(dnum);
        let (mut num, mut npac, mut sufnum) = (pnum, nbpack, nbpack > 1 || pnum > 1);
        let mut st = self.state.write();
        if rot.is_null() {
            rot = st.thedefrt.clone();
            st.thenbdefs += 1;
            (num, npac, sufnum) = (st.thenbdefs, 0, true);
        }
        let mut res = Vec::new();
        for part in [&st.thepref, &rot] {
            if let Some(live) = part.get() {
                res.extend_from_slice(live.string().as_bytes());
            }
        }
        if sufnum {
            res.extend(suffix(num, npac)?);
        }
        if let Some(live) = st.theext.get() {
            res.extend_from_slice(live.string().as_bytes());
        }
        Ok(AsciiString::from_raw(&res))
    }
}
