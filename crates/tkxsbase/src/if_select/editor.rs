// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Editor (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_Editor`: the definition of a set of typed values (by number, with a short
//! and a complete name, an edit mode and a list status) that an edit form loads from an
//! entity or a model, edits and applies back.
use super::{IFSelectEditForm, IFSelectEditValue, IFSelectListEditor};
use crate::c_text::atoi;
use crate::interface::{InterfaceInterfaceModel, InterfaceMsg, InterfaceTypedValue};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{
        Failure, Handle, SharedState, StandardTransient, Transient, WeakHandle, exceptions,
    },
    tcollection::{AsciiString, HAsciiString},
};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::{self, Write};
use std::sync::OnceLock;

/// The fields of `IFSelect_Editor`; the arrays have the capacity given at construction.
struct EditorState {
    thenbval: i32,
    themaxsh: i32,
    themaxco: i32,
    themaxla: i32,
    thenames: HashMap<Vec<u8>, i32>,
    thevalues: Vec<Handle<InterfaceTypedValue>>,
    theshorts: Vec<AsciiString>,
    themodes: Vec<IFSelectEditValue>,
    thelists: Vec<i32>,
}

/// The embedded fields of an editor, with the handle that `this` converts to natively.
pub struct IFSelectEditorBase {
    state: SharedState<EditorState>,
    this: OnceLock<WeakHandle<dyn IFSelectEditor>>,
}

impl IFSelectEditorBase {
    /// `IFSelect_Editor(nbval)` (protected): `nbval` values, none a list.
    ///
    /// ```
    /// use rustycad_tkxsbase::if_select::IFSelectEditorBase;
    /// let _base = IFSelectEditorBase::new(3);
    /// ```
    pub fn new(nbval: i32) -> Self {
        let cap = nbval.max(0) as usize;
        let state = EditorState {
            thenbval: nbval,
            themaxsh: 0,
            themaxco: 0,
            themaxla: 0,
            thenames: HashMap::new(),
            thevalues: vec![Handle::default(); cap],
            theshorts: vec![AsciiString::default(); cap],
            themodes: vec![IFSelectEditValue::default(); cap],
            thelists: vec![-1; cap],
        };
        Self {
            state: SharedState::new(state),
            this: OnceLock::new(),
        }
    }
    /// Records the owning handle of the editor (natively `this` converts to it); the
    /// forms the editor creates need it.
    pub fn bind_self(&self, editor: &Handle<dyn IFSelectEditor>) {
        let _ = self.this.set(editor.downgrade());
    }
    /// The handle `this` converts to natively; `Standard_NullObject` before `bind_self`.
    pub fn this(&self) -> Result<Handle<dyn IFSelectEditor>, Failure> {
        self.this
            .get()
            .map(WeakHandle::upgrade)
            .filter(|h| h.is_some())
            .ok_or_else(|| {
                exceptions::null_object(
                    "IFSelect_Editor: editor not bound to its handle; expected bind_self",
                )
            })
    }
    /// The 0-based index of the value `num` within the editor values (1..NbValues).
    fn rank(&self, num: i32) -> Option<usize> {
        let nbval = self.state.read().thenbval;
        (1..=nbval).contains(&num).then(|| num as usize - 1)
    }
}

/// The capacity error of `SetNbValues`.
fn capacity_error(nbval: i32, cap: usize) -> Failure {
    exceptions::out_of_range(&format!(
        "IFSelect_Editor:SetNbValues: {nbval}; expected at most {cap}"
    ))
}

/// The label of an edit mode in `PrintDefs`.
fn edit_mode_label(edm: IFSelectEditValue) -> &'static str {
    match edm {
        IFSelectEditValue::Optional => "Optional ",
        IFSelectEditValue::Editable => "Editable ",
        IFSelectEditValue::EditProtected => "Protected",
        IFSelectEditValue::EditComputed => "Computed ",
        IFSelectEditValue::EditRead => "ReadOnly ",
        IFSelectEditValue::EditDynamic => "Dynamic  ",
    }
}

/// The list status of a value in `PrintDefs`.
fn list_label(maxls: i32) -> String {
    match maxls {
        0 => String::from(" (List) "),
        m if m > 0 => format!(" (List <= {m} Items) "),
        _ => String::from(" "),
    }
}

/// The text of an `AsciiString` (lossy for non UTF-8 bytes).
fn text(s: &AsciiString) -> String {
    String::from_utf8_lossy(s.as_bytes()).into_owned()
}

/// `IFSelect_Editor`, abstract: `Label`, `Recognize`, `StringValue`, `Load` and `Apply`
/// are required; the protected constructor is [`IFSelectEditorBase::new`], whose
/// `bind_self` an implementation calls once it has its handle. The non-virtual native
/// members are default methods (do not override them).
pub trait IFSelectEditor: Transient {
    /// The embedded fields.
    fn editor_base(&self) -> &IFSelectEditorBase;
    /// `Label`.
    fn label(&self) -> AsciiString;
    /// `Recognize(form)`: true when the editor can work on the data of `form`.
    fn recognize(&self, form: &Handle<IFSelectEditForm>) -> Result<bool, Failure>;
    /// `StringValue(form, num)`: the value `num` from the data of `form`.
    fn string_value(
        &self,
        form: &Handle<IFSelectEditForm>,
        num: i32,
    ) -> Result<Handle<HAsciiString>, Failure>;
    /// `Load(form, ent, model)`: loads the values of `form` from `ent` and `model`.
    fn load(
        &self,
        form: &Handle<IFSelectEditForm>,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure>;
    /// `Apply(form, ent, model)`: applies the edited values of `form` to `ent` and `model`.
    fn apply(
        &self,
        form: &Handle<IFSelectEditForm>,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure>;
    /// `Form(readonly, undoable = true)`: a complete form on this editor, its label the
    /// label of the editor.
    fn form(&self, readonly: bool, undoable: bool) -> Result<Handle<IFSelectEditForm>, Failure> {
        IFSelectEditForm::new(
            &self.editor_base().this()?,
            readonly,
            undoable,
            &text(&self.label()),
        )
    }
    /// `ListEditor(num)`: a list editor on the value `num` (null when not a list).
    fn list_editor(&self, num: i32) -> Result<Handle<IFSelectListEditor>, Failure> {
        let max = self.max_list(num);
        if max < 0 {
            return Ok(Handle::default());
        }
        Ok(Handle::new(IFSelectListEditor::with_definition(
            &self.typed_value(num)?,
            max,
        )))
    }
    /// `ListValue(form, num)`: the list value `num` of `form`; `None` (null) by default.
    fn list_value(
        &self,
        _form: &Handle<IFSelectEditForm>,
        _num: i32,
    ) -> Result<Option<Vec<Handle<HAsciiString>>>, Failure> {
        Ok(None)
    }
    /// `Update(form, num, newval, enforce)`: the other values updated after a change of
    /// the value `num`; true (nothing to update) by default.
    fn update(
        &self,
        _form: &Handle<IFSelectEditForm>,
        _num: i32,
        _newval: &Handle<HAsciiString>,
        _enforce: bool,
    ) -> Result<bool, Failure> {
        Ok(true)
    }
    /// `UpdateList(form, num, newlist, enforce)`: as `Update` for a list; true by default.
    fn update_list(
        &self,
        _form: &Handle<IFSelectEditForm>,
        _num: i32,
        _newlist: Option<&[Handle<HAsciiString>]>,
        _enforce: bool,
    ) -> Result<bool, Failure> {
        Ok(true)
    }
    /// `SetNbValues(nbval)` (protected): `Standard_OutOfRange` beyond the capacity.
    fn set_nb_values(&self, nbval: i32) -> Result<(), Failure> {
        let mut st = self.editor_base().state.write();
        if nbval > st.thevalues.len() as i32 {
            return Err(capacity_error(nbval, st.thevalues.len()));
        }
        st.thenbval = nbval;
        Ok(())
    }
    /// `SetValue(num, typval, shortname = "", accessmode = IFSelect_Editable)`: nothing
    /// out of 1..NbValues; `Standard_NullObject` for a null value (a native null
    /// dereference).
    fn set_value(
        &self,
        num: i32,
        typval: &Handle<InterfaceTypedValue>,
        shortname: &str,
        accessmode: IFSelectEditValue,
    ) -> Result<(), Failure> {
        let base = self.editor_base();
        let Some(i) = base.rank(num) else {
            return Ok(());
        };
        let tv = typval.get().ok_or_else(|| {
            exceptions::null_object(&format!("IFSelect_Editor::SetValue: null value {num}"))
        })?;
        let (name, label) = (tv.name(), tv.label());
        let mut st = base.state.write();
        if !shortname.is_empty() {
            st.thenames.insert(shortname.as_bytes().to_vec(), num);
        }
        st.themaxsh = st.themaxsh.max(shortname.len() as i32);
        st.themaxco = st.themaxco.max(name.length());
        st.themaxla = st.themaxla.max(label.length());
        st.thenames.insert(name.as_bytes().to_vec(), num);
        st.thevalues[i] = typval.clone();
        st.theshorts[i] = AsciiString::new(shortname);
        st.themodes[i] = accessmode;
        Ok(())
    }
    /// `SetList(num, max = 0)`: the value `num` becomes a list of at most `max` items
    /// (0: no limit, -1: not a list).
    fn set_list(&self, num: i32, max: i32) {
        let base = self.editor_base();
        if let Some(i) = base.rank(num) {
            base.state.write().thelists[i] = max;
        }
    }
    /// `NbValues`.
    fn nb_values(&self) -> i32 {
        self.editor_base().state.read().thenbval
    }
    /// `TypedValue(num)`: `Standard_OutOfRange` out of the capacity.
    fn typed_value(&self, num: i32) -> Result<Handle<InterfaceTypedValue>, Failure> {
        let st = self.editor_base().state.read();
        let cap = st.thevalues.len();
        usize::try_from(num - 1)
            .ok()
            .and_then(|i| st.thevalues.get(i))
            .cloned()
            .ok_or_else(|| {
                exceptions::out_of_range(&format!(
                    "IFSelect_Editor::TypedValue: {num}; expected [1, {cap}]"
                ))
            })
    }
    /// `IsList(num)`.
    fn is_list(&self, num: i32) -> bool {
        self.max_list(num) >= 0
    }
    /// `MaxList(num)`: -1 when not a list or out of range.
    fn max_list(&self, num: i32) -> i32 {
        let base = self.editor_base();
        base.rank(num).map_or(-1, |i| base.state.read().thelists[i])
    }
    /// `Name(num, isshort = false)`: empty out of range; `Standard_NullObject` for a
    /// complete name of a value not set (a native null dereference).
    fn name(&self, num: i32, isshort: bool) -> Result<AsciiString, Failure> {
        let base = self.editor_base();
        let Some(i) = base.rank(num) else {
            return Ok(AsciiString::default());
        };
        if isshort {
            return Ok(base.state.read().theshorts[i].clone());
        }
        let tv = self.typed_value(num)?;
        let live = tv.get().ok_or_else(|| {
            exceptions::null_object(&format!("IFSelect_Editor::Name: value {num} not set"))
        })?;
        Ok(live.name())
    }
    /// `EditMode(num)`: `IFSelect_EditDynamic` out of range.
    fn edit_mode(&self, num: i32) -> IFSelectEditValue {
        let base = self.editor_base();
        base.rank(num).map_or(IFSelectEditValue::EditDynamic, |i| {
            base.state.read().themodes[i]
        })
    }
    /// `NameNumber(name)`: the number of a short or complete name, else of an integer
    /// text in range; 0 when none.
    fn name_number(&self, name: &str) -> i32 {
        if let Some(num) = self
            .editor_base()
            .state
            .read()
            .thenames
            .get(name.as_bytes())
        {
            return *num;
        }
        let res = atoi(name.as_bytes()); // if it's an integer, we try it
        if res < 1 || res > self.nb_values() {
            0
        } else {
            res
        }
    }
    /// `MaxNameLength(what)`: the longest short (-1) or complete (0) name, or label (1).
    fn max_name_length(&self, what: i32) -> i32 {
        let st = self.editor_base().state.read();
        match what {
            -1 => st.themaxsh,
            0 => st.themaxco,
            1 => st.themaxla,
            _ => 0,
        }
    }
    /// `PrintNames(S)`: the numbers, names and labels of the values.
    fn print_names(&self, s: &mut dyn Write) -> io::Result<()> {
        let nb = self.nb_values();
        let (maxsh, maxco) = (self.max_name_length(-1), self.max_name_length(0));
        let mut out = format!(
            "****    Editor : {}\n****    Nb Values = {nb}    ****    Names / Labels\n Num ",
            text(&self.label())
        );
        if maxsh > 0 {
            let _ = write!(
                out,
                "Short{} ",
                InterfaceMsg::blanks_for_text("Short", maxsh)
            );
        }
        let _ = writeln!(
            out,
            "Complete{}  Label",
            InterfaceMsg::blanks_for_text("Complete", maxco)
        );
        for i in 1..=nb {
            let Some(tv) = self.typed_value(i).ok().filter(|t| t.is_some()) else {
                continue;
            };
            let _ = write!(out, "{}{i} ", InterfaceMsg::blanks_for_integer(i, 3));
            if maxsh > 0 {
                let sho = text(&self.name(i, true).unwrap_or_default());
                let _ = write!(out, "{sho}{} ", InterfaceMsg::blanks_for_text(&sho, maxsh));
            }
            let name = text(&tv.name());
            let _ = writeln!(
                out,
                "{name}{}  {}",
                InterfaceMsg::blanks_for_text(&name, maxco),
                text(&tv.label())
            );
        }
        s.write_all(out.as_bytes())
    }
    /// `PrintDefs(S, labels = false)`: the values with their list status, edit mode and
    /// definition, by label or by names.
    fn print_defs(&self, s: &mut dyn Write, labels: bool) -> io::Result<()> {
        let mut out = print_defs_header(self, labels);
        for i in 1..=self.nb_values() {
            let Some(tv) = self.typed_value(i).ok().filter(|t| t.is_some()) else {
                continue;
            };
            let _ = write!(out, " {}{i} ", InterfaceMsg::blanks_for_integer(i, 3));
            out.push_str(&print_defs_names(self, i, &tv, labels));
            let _ = writeln!(
                out,
                " {}{} {}",
                list_label(self.max_list(i)),
                edit_mode_label(self.edit_mode(i)),
                text(&tv.definition())
            );
        }
        s.write_all(out.as_bytes())
    }
}
impl_transient_interface!(dyn IFSelectEditor, "IFSelect_Editor", StandardTransient);

/// The title lines of `PrintDefs`.
fn print_defs_header<E: IFSelectEditor + ?Sized>(editor: &E, labels: bool) -> String {
    let nb = editor.nb_values();
    let what = if labels { "Labels" } else { "Names" };
    let mut out = format!(
        "****    Editor : {}\n****    Nb Values = {nb}    ****    {what}  /  Definitions\n Num ",
        text(&editor.label())
    );
    let (maxsh, maxco) = (editor.max_name_length(-1), editor.max_name_length(0));
    if labels {
        let _ = write!(
            out,
            "Label{}",
            InterfaceMsg::blanks_for_text("Label", editor.max_name_length(1))
        );
    } else {
        if maxsh > 0 {
            let _ = write!(
                out,
                "Short{}",
                InterfaceMsg::blanks_for_text("Short", maxsh + 1)
            );
        }
        let _ = write!(
            out,
            "Complete{}",
            InterfaceMsg::blanks_for_text("Complete", maxco)
        );
    }
    out.push_str("  Edit Mode  &  Definition\n");
    out
}

/// The label, or the short and complete names, of the value `i` in `PrintDefs`.
fn print_defs_names<E: IFSelectEditor + ?Sized>(
    editor: &E,
    i: i32,
    tv: &InterfaceTypedValue,
    labels: bool,
) -> String {
    if labels {
        let label = text(&tv.label());
        return format!(
            "{label}{}",
            InterfaceMsg::blanks_for_text(&label, editor.max_name_length(1))
        );
    }
    let mut out = String::new();
    let maxsh = editor.max_name_length(-1);
    if maxsh > 0 {
        let sho = text(&editor.name(i, true).unwrap_or_default());
        let _ = write!(out, "{sho}{} ", InterfaceMsg::blanks_for_text(&sho, maxsh));
    }
    let name = text(&tv.name());
    let _ = write!(
        out,
        "{name}{}",
        InterfaceMsg::blanks_for_text(&name, editor.max_name_length(0))
    );
    out
}
