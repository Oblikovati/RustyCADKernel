// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_EditForm (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_EditForm`: the values of an editor (all of them, or an extraction by
//! numbers) loaded from an entity or a model, edited, then applied back or undone.
//!
//! The edition, printing and application members are in `edit_form_edit`.
use super::{IFSelectEditor, IFSelectListEditor, IFSelectTextList};
use crate::interface::InterfaceInterfaceModel;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle, exceptions},
    tcollection::{AsciiString, HAsciiString},
};
use std::sync::OnceLock;

/// A value of the form: natively a transient which is a text or a list of texts.
#[derive(Clone, Default)]
pub(super) enum FormItem {
    #[default]
    Null,
    Text(Handle<HAsciiString>),
    List(Vec<Handle<HAsciiString>>),
}

impl FormItem {
    /// A text item (null for a null text).
    pub(super) fn from_text(val: &Handle<HAsciiString>) -> Self {
        if val.is_null() {
            Self::Null
        } else {
            Self::Text(val.clone())
        }
    }
    /// A list item (null for a null list).
    pub(super) fn from_list(list: Option<&[Handle<HAsciiString>]>) -> Self {
        list.map_or(Self::Null, |l| Self::List(l.to_vec()))
    }
    /// The down cast to a text: null for a list.
    pub(super) fn text(&self) -> Handle<HAsciiString> {
        match self {
            Self::Text(t) => t.clone(),
            _ => Handle::default(),
        }
    }
    /// The down cast to a list: `None` for a text.
    pub(super) fn list(&self) -> IFSelectTextList {
        match self {
            Self::List(l) => Some(l.clone()),
            _ => None,
        }
    }
}

/// The fields of `IFSelect_EditForm`; the arrays keep the native `0..=Upper` indices.
pub(super) struct FormState {
    pub(super) thecomplete: bool,
    pub(super) theloaded: bool,
    pub(super) thekeepst: bool,
    pub(super) thelabel: AsciiString,
    pub(super) thenums: Vec<i32>,
    pub(super) theorigs: Vec<FormItem>,
    pub(super) themodifs: Vec<FormItem>,
    pub(super) thestatus: Vec<i32>,
    pub(super) theent: Handle<StandardTransient>,
    pub(super) themodel: Handle<dyn InterfaceInterfaceModel>,
    pub(super) thetouched: i32,
}

/// `IFSelect_EditForm`. Built as a handle (the editor receives it); its virtual members
/// have no override natively, they are inherent methods.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectEditForm;
/// use rustycad_tkernel::standard::Handle;
/// assert!(IFSelectEditForm::new(&Handle::default(), false, true, "").is_err());
/// ```
pub struct IFSelectEditForm {
    transient: StandardTransient,
    pub(super) theeditor: Handle<dyn IFSelectEditor>,
    pub(super) state: SharedState<FormState>,
    this: OnceLock<WeakHandle<IFSelectEditForm>>,
}
impl_transient!(
    IFSelectEditForm,
    "IFSelect_EditForm",
    StandardTransient,
    transient
);

/// The `Standard_OutOfRange` of an array access.
pub(super) fn slot_out_of_range(what: &str, index: i32, upper: usize) -> Failure {
    exceptions::out_of_range(&format!(
        "IFSelect_EditForm::{what}: {index}; expected [0, {upper}]"
    ))
}

/// The item `index` of an array, `Standard_OutOfRange` beyond it.
pub(super) fn slot<T: Clone>(what: &str, array: &[T], index: i32) -> Result<T, Failure> {
    usize::try_from(index)
        .ok()
        .and_then(|i| array.get(i))
        .cloned()
        .ok_or_else(|| slot_out_of_range(what, index, array.len().saturating_sub(1)))
}

impl IFSelectEditForm {
    /// The handle of a new form; `Standard_NullObject` for a null editor.
    fn build(
        editor: &Handle<dyn IFSelectEditor>,
        nums: Option<&[i32]>,
        readonly: bool,
        undoable: bool,
        label: &str,
    ) -> Result<Handle<Self>, Failure> {
        let live = editor.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_EditForm: null editor; expected an editor")
        })?;
        let nb = nums.map_or(live.nb_values(), |n| n.len() as i32).max(0) as usize;
        let mut thenums = vec![0; nums.map_or(2, |n| n.len() + 1)];
        if let Some(n) = nums {
            thenums[1..].copy_from_slice(n);
        }
        let state = FormState {
            thecomplete: nums.is_none(),
            theloaded: false,
            thekeepst: false,
            thelabel: AsciiString::new(label),
            thenums,
            theorigs: vec![FormItem::Null; if undoable { nb + 1 } else { 1 }],
            themodifs: vec![FormItem::Null; if readonly { 1 } else { nb + 1 }],
            thestatus: vec![0; if readonly { 1 } else { nb + 1 }],
            theent: Handle::default(),
            themodel: Handle::default(),
            thetouched: 0,
        };
        let form = Handle::new(Self {
            transient: StandardTransient::default(),
            theeditor: editor.clone(),
            state: SharedState::new(state),
            this: OnceLock::new(),
        });
        let _ = form.this.set(form.downgrade());
        Ok(form)
    }
    /// `IFSelect_EditForm(editor, readonly, undoable, label = "")`: a complete form.
    pub fn new(
        editor: &Handle<dyn IFSelectEditor>,
        readonly: bool,
        undoable: bool,
        label: &str,
    ) -> Result<Handle<Self>, Failure> {
        Self::build(editor, None, readonly, undoable, label)
    }
    /// `IFSelect_EditForm(editor, nums, readonly, undoable, label = "")`: the extraction
    /// of the values `nums` of the editor.
    pub fn with_numbers(
        editor: &Handle<dyn IFSelectEditor>,
        nums: &[i32],
        readonly: bool,
        undoable: bool,
        label: &str,
    ) -> Result<Handle<Self>, Failure> {
        Self::build(editor, Some(nums), readonly, undoable, label)
    }
    /// The handle `this` converts to natively.
    pub(super) fn this(&self) -> Handle<Self> {
        self.this.get().map(WeakHandle::upgrade).unwrap_or_default()
    }
    /// `EditKeepStatus`: the native reference read (see `set_edit_keep_status`).
    pub fn edit_keep_status(&self) -> bool {
        self.state.read().thekeepst
    }
    /// `EditKeepStatus() = keep`: the native reference written; true keeps the edit
    /// status after `Apply`.
    pub fn set_edit_keep_status(&self, keep: bool) {
        self.state.write().thekeepst = keep;
    }
    /// `Label`.
    pub fn label(&self) -> AsciiString {
        self.state.read().thelabel.clone()
    }
    /// `IsLoaded`.
    pub fn is_loaded(&self) -> bool {
        self.state.read().theloaded
    }
    /// `ClearData`: no entity, no model, not loaded.
    pub fn clear_data(&self) {
        let mut st = self.state.write();
        st.theent = Handle::default();
        st.themodel = Handle::default();
        st.theloaded = false;
    }
    /// `SetData(ent, model)`.
    pub fn set_data(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) {
        let mut st = self.state.write();
        st.theent = ent.clone();
        st.themodel = model.clone();
    }
    /// `SetEntity(ent)`.
    pub fn set_entity(&self, ent: &Handle<StandardTransient>) {
        self.state.write().theent = ent.clone();
    }
    /// `SetModel(model)`.
    pub fn set_model(&self, model: &Handle<dyn InterfaceInterfaceModel>) {
        self.state.write().themodel = model.clone();
    }
    /// `Entity`.
    pub fn entity(&self) -> Handle<StandardTransient> {
        self.state.read().theent.clone()
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.state.read().themodel.clone()
    }
    /// `Editor`.
    pub fn editor(&self) -> Handle<dyn IFSelectEditor> {
        self.theeditor.clone()
    }
    /// `IsComplete`.
    pub fn is_complete(&self) -> bool {
        self.state.read().thecomplete
    }
    /// `NbValues(editable)`: of the editor, or of the extraction.
    pub fn nb_values(&self, editable: bool) -> i32 {
        let st = self.state.read();
        if !editable || st.thecomplete {
            return self.theeditor.nb_values();
        }
        st.thenums.len() as i32 - 1
    }
    /// `NumberFromRank(rank)`: the editor number of a rank of the form (0 out of range).
    pub fn number_from_rank(&self, rank: i32) -> i32 {
        let st = self.state.read();
        if st.thecomplete {
            return rank;
        }
        if rank < 1 || rank >= st.thenums.len() as i32 {
            return 0;
        }
        st.thenums[rank as usize]
    }
    /// `RankFromNumber(number)`: the rank of an editor number in the form (0 if absent).
    pub fn rank_from_number(&self, number: i32) -> i32 {
        let st = self.state.read();
        if st.thecomplete {
            return number;
        }
        st.thenums
            .iter()
            .skip(1)
            .position(|n| *n == number)
            .map_or(0, |p| p as i32 + 1)
    }
    /// The rank in the extraction of the editor number `res`. Native defect kept: the
    /// search runs to `Length` (one beyond `Upper`), so a number absent from the
    /// extraction raises `Standard_OutOfRange`.
    fn extraction_rank(&self, what: &str, res: i32) -> Result<i32, Failure> {
        let st = self.state.read();
        let upper = st.thenums.len() - 1;
        match st.thenums.iter().skip(1).position(|n| *n == res) {
            Some(p) => Ok(p as i32 + 1),
            None => Err(slot_out_of_range(what, upper as i32 + 1, upper)),
        }
    }
    /// `NameNumber(name)`: the editor number of a name, negative when not extracted.
    /// Native defect kept: see the search to `Length` (an error, not a negative number).
    pub fn name_number(&self, name: &str) -> Result<i32, Failure> {
        let res = self.theeditor.name_number(name);
        if self.is_complete() || res == 0 {
            return Ok(res);
        }
        self.extraction_rank("NameNumber", res).map(|_| res)
    }
    /// `NameRank(name)`: the rank of a name in the form. Native defect kept: see the
    /// search to `Length`.
    pub fn name_rank(&self, name: &str) -> Result<i32, Failure> {
        let res = self.theeditor.name_number(name);
        if self.is_complete() || res == 0 {
            return Ok(res);
        }
        self.extraction_rank("NameRank", res)
    }
    /// `LoadDefault`: the original values from the editor, without data.
    pub fn load_default(&self) -> Result<(), Failure> {
        let nb = {
            let mut st = self.state.write();
            st.theloaded = true;
            st.thetouched = 0;
            st.theorigs.len() as i32 - 1
        };
        for i in 1..=nb {
            let num = self.number_from_rank(i);
            if num == 0 {
                continue;
            }
            let val = self.theeditor.string_value(&self.this(), num)?;
            self.state.write().theorigs[i as usize] = FormItem::from_text(&val);
        }
        Ok(())
    }
    /// The editor loads the form from `ent` and `model`; true when loaded.
    fn load_by_editor(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        self.state.write().thetouched = 0;
        self.theeditor.load(&self.this(), ent, model)
    }
    /// `LoadData(ent, model)`: loaded from `ent` and `model`, which are recorded.
    pub fn load_data(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        if !self.load_by_editor(ent, model)? {
            return Ok(false);
        }
        self.set_data(ent, model);
        self.state.write().theloaded = true;
        Ok(true)
    }
    /// `LoadEntity(ent)`: loaded from `ent` alone, which is recorded.
    pub fn load_entity(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        if !self.load_by_editor(ent, &Handle::default())? {
            return Ok(false);
        }
        self.set_entity(ent);
        self.state.write().theloaded = true;
        Ok(true)
    }
    /// `LoadModel(model)`: loaded from `model` alone; the entity becomes null.
    pub fn load_model(&self, model: &Handle<dyn InterfaceInterfaceModel>) -> Result<bool, Failure> {
        if !self.load_by_editor(&Handle::default(), model)? {
            return Ok(false);
        }
        self.set_data(&Handle::default(), model);
        self.state.write().theloaded = true;
        Ok(true)
    }
    /// `LoadData()`: loaded without entity nor model; nothing is recorded.
    pub fn load_data_default(&self) -> Result<bool, Failure> {
        if !self.load_by_editor(&Handle::default(), &Handle::default())? {
            return Ok(false);
        }
        self.state.write().theloaded = true;
        Ok(true)
    }
    /// The rank of `num` when within the original values (`None` else).
    fn original_rank(&self, num: i32) -> Option<usize> {
        let n = self.rank_from_number(num);
        let upper = self.state.read().theorigs.len() as i32 - 1;
        (n > 0 && n <= upper).then_some(n as usize)
    }
    /// `ListEditor(num)`: a list editor loaded with the list value `num` (null when not
    /// a list). Native quirk kept: the list status is read at the rank, not the number.
    pub fn list_editor(&self, num: i32) -> Result<Handle<IFSelectListEditor>, Failure> {
        let Some(n) = self.original_rank(num) else {
            return Ok(Handle::default());
        };
        if !self.theeditor.is_list(n as i32) {
            return Ok(Handle::default());
        }
        let led = self.theeditor.list_editor(num)?;
        let lis = self.theeditor.list_value(&self.this(), num)?;
        let live = led.get().ok_or_else(|| {
            exceptions::null_object(&format!(
                "IFSelect_EditForm::ListEditor: no list editor for {num}"
            ))
        })?;
        live.load_model(&self.model());
        live.load_values(lis.as_deref());
        Ok(led)
    }
    /// `LoadValue(num, val)`: the original value `num` (nothing out of range).
    pub fn load_value(&self, num: i32, val: &Handle<HAsciiString>) {
        if let Some(n) = self.original_rank(num) {
            self.state.write().theorigs[n] = FormItem::from_text(val);
        }
    }
    /// `LoadList(num, list)`: the original list `num` (nothing out of range).
    pub fn load_list(&self, num: i32, list: Option<&[Handle<HAsciiString>]>) {
        if let Some(n) = self.original_rank(num) {
            self.state.write().theorigs[n] = FormItem::from_list(list);
        }
    }
    /// The original item `num`, `None` when the form keeps no original (not undoable).
    fn original_item(&self, num: i32) -> Result<Option<FormItem>, Failure> {
        let n = self.rank_from_number(num);
        let st = self.state.read();
        if st.theorigs.len() == 1 {
            return Ok(None);
        }
        slot("OriginalValue", &st.theorigs, n).map(Some)
    }
    /// `OriginalValue(num)`: kept, else from the editor; `Standard_OutOfRange` beyond.
    pub fn original_value(&self, num: i32) -> Result<Handle<HAsciiString>, Failure> {
        match self.original_item(num)? {
            Some(item) => Ok(item.text()),
            None => self.theeditor.string_value(&self.this(), num),
        }
    }
    /// `OriginalList(num)`: kept, else from the editor; `Standard_OutOfRange` beyond.
    pub fn original_list(&self, num: i32) -> Result<IFSelectTextList, Failure> {
        match self.original_item(num)? {
            Some(item) => Ok(item.list()),
            None => self.theeditor.list_value(&self.this(), num),
        }
    }
    /// The modified item `num`, `None` when not modified or read only.
    fn modified_item(&self, num: i32) -> Result<Option<FormItem>, Failure> {
        if self.state.read().themodifs.len() == 1 || !self.is_modified(num)? {
            return Ok(None);
        }
        let n = self.rank_from_number(num);
        slot("EditedValue", &self.state.read().themodifs, n).map(Some)
    }
    /// `EditedValue(num)`: the modified value, else the original one.
    pub fn edited_value(&self, num: i32) -> Result<Handle<HAsciiString>, Failure> {
        match self.modified_item(num)? {
            Some(item) => Ok(item.text()),
            None => self.original_value(num),
        }
    }
    /// `EditedList(num)`: the modified list, else the original one.
    pub fn edited_list(&self, num: i32) -> Result<IFSelectTextList, Failure> {
        match self.modified_item(num)? {
            Some(item) => Ok(item.list()),
            None => self.original_list(num),
        }
    }
    /// The edit status of `num` (0 for a read only form); `Standard_OutOfRange` beyond.
    fn status_of(&self, num: i32) -> Result<i32, Failure> {
        let n = self.rank_from_number(num);
        let st = self.state.read();
        if st.thestatus.len() == 1 {
            return Ok(0);
        }
        slot("IsModified", &st.thestatus, n)
    }
    /// `IsModified(num)`: modified or touched.
    pub fn is_modified(&self, num: i32) -> Result<bool, Failure> {
        Ok(self.status_of(num)? != 0)
    }
    /// `IsTouched(num)`.
    pub fn is_touched(&self, num: i32) -> Result<bool, Failure> {
        Ok(self.status_of(num)? == 2)
    }
}
