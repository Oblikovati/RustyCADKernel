// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_ListEditor (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_ListEditor`: the edition of a list of texts (the value of an editor which
//! is a list), each item checked against the definition of the value, with the status
//! of each item (original, modified, added).
use crate::interface::{InterfaceInterfaceModel, InterfaceParamType, InterfaceTypedValue};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::HAsciiString,
};

/// A list of texts: `None` for a null sequence.
pub type IFSelectTextList = Option<Vec<Handle<HAsciiString>>>;

/// The status of an edited item: 0 original, 1 modified, 2 added.
const ORIGINAL: i32 = 0;
const MODIFIED: i32 = 1;
const ADDED: i32 = 2;

/// The fields of `IFSelect_ListEditor`.
#[derive(Default)]
struct ListEditorState {
    themax: i32,
    thedef: Handle<InterfaceTypedValue>,
    thetouc: i32,
    theorig: IFSelectTextList,
    theedit: IFSelectTextList,
    thestat: Option<Vec<i32>>,
    themodl: Handle<dyn InterfaceInterfaceModel>,
}

impl ListEditorState {
    /// The edited values with their statuses, `None` before any edition.
    fn lists_mut(&mut self) -> Option<(&mut Vec<Handle<HAsciiString>>, &mut Vec<i32>)> {
        match (&mut self.theedit, &mut self.thestat) {
            (Some(edit), Some(stat)) => Some((edit, stat)),
            _ => None,
        }
    }
}

/// `IFSelect_ListEditor`. Its virtual members have no override natively; they are
/// inherent methods.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectListEditor;
/// let editor = IFSelectListEditor::new();
/// assert!(!editor.is_touched() && editor.nb_values(true) == 0);
/// ```
#[derive(Default)]
pub struct IFSelectListEditor {
    transient: StandardTransient,
    state: SharedState<ListEditorState>,
}
impl_transient!(
    IFSelectListEditor,
    "IFSelect_ListEditor",
    StandardTransient,
    transient
);

/// `CheckValue(val, modl, thedef)`: true without value, model or definition; else the
/// value satisfies the definition and, for an entity identifier, names an entity of the
/// model.
fn check_value(
    val: &Handle<HAsciiString>,
    modl: &Handle<dyn InterfaceInterfaceModel>,
    thedef: &Handle<InterfaceTypedValue>,
) -> bool {
    let (Some(text), Some(model), Some(def)) = (val.get(), modl.get(), thedef.get()) else {
        return true;
    };
    if !def.satisfies(val) {
        return false;
    }
    if def.param_type() == InterfaceParamType::ParamIdent {
        let label = String::from_utf8_lossy(text.string().as_bytes()).into_owned();
        return model.next_number_for_label(&label, 0, true) > 0;
    }
    true
}

/// The value `num` (1-based) of a list, null out of range.
fn item(list: &IFSelectTextList, num: i32) -> Handle<HAsciiString> {
    list.as_ref()
        .and_then(|l| usize::try_from(num - 1).ok().and_then(|i| l.get(i)))
        .cloned()
        .unwrap_or_default()
}

impl IFSelectListEditor {
    /// `IFSelect_ListEditor()`: no definition, no limit.
    pub fn new() -> Self {
        Self::default()
    }
    /// `IFSelect_ListEditor(def, max = 0)`: the items checked against `def`, at most
    /// `max` of them (0: no limit).
    pub fn with_definition(def: &Handle<InterfaceTypedValue>, max: i32) -> Self {
        let state = ListEditorState {
            themax: max,
            thedef: def.clone(),
            ..ListEditorState::default()
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `LoadModel(model)`: the model which checks the entity identifiers.
    pub fn load_model(&self, model: &Handle<dyn InterfaceInterfaceModel>) {
        self.state.write().themodl = model.clone();
    }
    /// `LoadValues(vals)`: the original values; the edition restarts from them.
    pub fn load_values(&self, vals: Option<&[Handle<HAsciiString>]>) {
        self.state.write().theorig = vals.map(<[_]>::to_vec);
        self.clear_edit();
    }
    /// `SetTouched`.
    pub fn set_touched(&self) {
        self.state.write().thetouc = 1;
    }
    /// `ClearEdit`: the edited values become the original ones. Native quirk kept:
    /// without original values the touched status is kept.
    pub fn clear_edit(&self) {
        let mut st = self.state.write();
        st.theedit = Some(Vec::new());
        st.thestat = Some(Vec::new());
        let Some(orig) = st.theorig.clone() else {
            return;
        };
        st.thestat = Some(vec![ORIGINAL; orig.len()]);
        st.theedit = Some(orig);
        st.thetouc = 0;
    }
    /// `LoadEdited(list)`: the whole edited list, each item modified; false for a null
    /// list, an item which does not satisfy, or more items than the maximum. Native
    /// quirk kept: the maximum 0 (no limit) refuses any non empty list.
    pub fn load_edited(&self, list: Option<&[Handle<HAsciiString>]>) -> bool {
        let Some(list) = list else {
            return false;
        };
        let mut st = self.state.write();
        if list.len() as i32 > st.themax {
            return false;
        }
        if st.thedef.is_some() && !list.iter().all(|v| check_value(v, &st.themodl, &st.thedef)) {
            return false;
        }
        st.theedit = Some(list.to_vec());
        st.thestat = Some(vec![MODIFIED; list.len()]);
        st.thetouc = 1;
        true
    }
    /// `SetValue(num, val)`: the item `num` modified; false out of range or when `val`
    /// does not satisfy.
    pub fn set_value(&self, num: i32, val: &Handle<HAsciiString>) -> bool {
        let mut st = self.state.write();
        let len = st.theedit.as_ref().map_or(0, Vec::len);
        let Some(i) = usize::try_from(num - 1).ok().filter(|i| *i < len) else {
            return false;
        };
        if !check_value(val, &st.themodl, &st.thedef) {
            return false;
        }
        if let Some((edit, stat)) = st.lists_mut() {
            edit[i] = val.clone();
            stat[i] = MODIFIED;
        }
        st.thetouc = 1;
        true
    }
    /// `AddValue(val, atnum = 0)`: an item added at the end (0) or before `atnum`; false
    /// beyond the maximum or when `val` does not satisfy; `Standard_OutOfRange` for
    /// `atnum` beyond the end.
    pub fn add_value(&self, val: &Handle<HAsciiString>, atnum: i32) -> Result<bool, Failure> {
        let mut st = self.state.write();
        let Some(len) = st.theedit.as_ref().map(Vec::len) else {
            return Ok(false);
        };
        if st.themax > 0 && len as i32 >= st.themax {
            return Ok(false);
        }
        if !check_value(val, &st.themodl, &st.thedef) {
            return Ok(false);
        }
        let at = if atnum > 0 { atnum as usize - 1 } else { len };
        if at > len {
            return Err(exceptions::out_of_range(&format!(
                "IFSelect_ListEditor::AddValue: {atnum}; expected [1, {}]",
                len + 1
            )));
        }
        if let Some((edit, stat)) = st.lists_mut() {
            edit.insert(at, val.clone());
            stat.insert(at, ADDED);
        }
        st.thetouc = 2;
        Ok(true)
    }
    /// `Remove(num = 0, howmany = 1)`: the items `num` to `howmany` (the native
    /// sequence removes a range, not a count); `num = 0` stands for `nb - howmany`.
    /// False for a negative start or `num + howmany` beyond the count. Native defects
    /// kept as errors: `Standard_OutOfRange` when `num > howmany`, and a never ending
    /// recursion (a `Standard_ProgramError` here) for `num = 0` and `howmany = nb`.
    pub fn remove(&self, num: i32, howmany: i32) -> Result<bool, Failure> {
        let Some(nb) = self.state.read().theedit.as_ref().map(|l| l.len() as i32) else {
            return Ok(false);
        };
        if num < 0 {
            return Ok(false);
        }
        if num == 0 {
            if nb == howmany {
                return Err(exceptions::program_error(&format!(
                    "IFSelect_ListEditor::Remove(0, {howmany}) on {nb} values: recursion without end; expected howmany != {nb}"
                )));
            }
            return self.remove(nb - howmany, howmany);
        }
        if num + howmany > nb {
            return Ok(false);
        }
        if num > howmany {
            return Err(exceptions::out_of_range(&format!(
                "IFSelect_ListEditor::Remove: range {num}..{howmany}; expected from <= to"
            )));
        }
        let mut st = self.state.write();
        let range = num as usize - 1..howmany as usize;
        if let Some((edit, stat)) = st.lists_mut() {
            edit.drain(range.clone());
            stat.drain(range);
        }
        st.thetouc = 3;
        Ok(true)
    }
    /// `OriginalValues`.
    pub fn original_values(&self) -> IFSelectTextList {
        self.state.read().theorig.clone()
    }
    /// `EditedValues`.
    pub fn edited_values(&self) -> IFSelectTextList {
        self.state.read().theedit.clone()
    }
    /// `NbValues(edited = true)`.
    pub fn nb_values(&self, edited: bool) -> i32 {
        let st = self.state.read();
        let list = if edited { &st.theedit } else { &st.theorig };
        list.as_ref().map_or(0, |l| l.len() as i32)
    }
    /// `Value(num, edited = true)`: null out of range.
    pub fn value(&self, num: i32, edited: bool) -> Handle<HAsciiString> {
        let st = self.state.read();
        item(if edited { &st.theedit } else { &st.theorig }, num)
    }
    /// The status of the item `num`, `None` out of range.
    fn status(&self, num: i32) -> Option<i32> {
        let st = self.state.read();
        st.thestat
            .as_ref()
            .and_then(|s| usize::try_from(num - 1).ok().and_then(|i| s.get(i)))
            .copied()
    }
    /// `IsChanged(num)`: modified or added.
    pub fn is_changed(&self, num: i32) -> bool {
        self.status(num).is_some_and(|s| s != ORIGINAL)
    }
    /// `IsModified(num)`.
    pub fn is_modified(&self, num: i32) -> bool {
        self.status(num) == Some(MODIFIED)
    }
    /// `IsAdded(num)`.
    pub fn is_added(&self, num: i32) -> bool {
        self.status(num) == Some(ADDED)
    }
    /// `IsTouched`.
    pub fn is_touched(&self) -> bool {
        self.state.read().thetouc != 0
    }
}
