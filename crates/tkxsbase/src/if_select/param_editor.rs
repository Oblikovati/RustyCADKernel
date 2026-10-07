// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_ParamEditor (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_ParamEditor`: an editor of typed values taken as they are (for instance
//! static parameters), loaded from and applied to the values themselves.
use super::{IFSelectEditForm, IFSelectEditValue, IFSelectEditor, IFSelectEditorBase};
use crate::interface::{
    InterfaceInterfaceModel, InterfaceParamType, InterfaceStatic, InterfaceTypedValue,
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::{AsciiString, HAsciiString},
};

/// `IFSelect_ParamEditor`.
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectEditor, IFSelectParamEditor};
/// let editor = IFSelectParamEditor::new(10, "").unwrap();
/// assert_eq!(editor.label().as_bytes(), b"Param Editor");
/// assert_eq!(editor.nb_values(), 0);
/// ```
pub struct IFSelectParamEditor {
    transient: StandardTransient,
    base: IFSelectEditorBase,
    thelabel: AsciiString,
}
impl_transient!(IFSelectParamEditor, "IFSelect_ParamEditor", dyn IFSelectEditor, transient, interfaces: [dyn IFSelectEditor]);

/// The value `num` of an editor, `Standard_NullObject` when not set (a native null
/// dereference).
fn live_value(
    editor: &IFSelectParamEditor,
    num: i32,
) -> Result<Handle<InterfaceTypedValue>, Failure> {
    let tv = editor.typed_value(num)?;
    if tv.is_null() {
        return Err(exceptions::null_object(&format!(
            "IFSelect_ParamEditor: value {num} not set"
        )));
    }
    Ok(tv)
}

impl IFSelectParamEditor {
    /// `IFSelect_ParamEditor(nbmax = 100, label = "")`: room for `nbmax` values, none
    /// yet; the label defaults to "Param Editor". The editor is bound to its handle.
    pub fn new(nbmax: i32, label: &str) -> Result<Handle<Self>, Failure> {
        let label = if label.is_empty() {
            "Param Editor"
        } else {
            label
        };
        let editor = Handle::new(Self {
            transient: StandardTransient::default(),
            base: IFSelectEditorBase::new(nbmax),
            thelabel: AsciiString::new(label),
        });
        editor.set_nb_values(0)?;
        editor.base.bind_self(&editor.clone().cast());
        Ok(editor)
    }
    /// `AddValue(val, shortname = "")`: one more value, editable; `Standard_OutOfRange`
    /// beyond the room.
    pub fn add_value(
        &self,
        val: &Handle<InterfaceTypedValue>,
        shortname: &str,
    ) -> Result<(), Failure> {
        self.set_nb_values(self.nb_values() + 1)?;
        self.set_value(
            self.nb_values(),
            val,
            shortname,
            IFSelectEditValue::Editable,
        )
    }
    /// `AddConstantText(val, shortname, completename = "")`: one more value, a read only
    /// text named `completename` (else `shortname`).
    pub fn add_constant_text(
        &self,
        val: &str,
        shortname: &str,
        completename: &str,
    ) -> Result<(), Failure> {
        let name = if completename.is_empty() {
            shortname
        } else {
            completename
        };
        let tv = InterfaceTypedValue::new(name, InterfaceParamType::ParamText, "")?;
        tv.set_c_string_value(val)?;
        self.set_nb_values(self.nb_values() + 1)?;
        self.set_value(
            self.nb_values(),
            &Handle::new(tv),
            shortname,
            IFSelectEditValue::EditRead,
        )
    }
    /// `StaticEditor(list, label = "")`: an editor of the static parameters named in
    /// `list` (the unknown names are skipped); null for a null list.
    pub fn static_editor(
        list: Option<&[Handle<HAsciiString>]>,
        label: &str,
    ) -> Result<Handle<Self>, Failure> {
        let Some(names) = list else {
            return Ok(Handle::default());
        };
        let editor = Self::new(names.len() as i32 + 10, label)?;
        for name in names {
            let text = name
                .get()
                .map(|n| String::from_utf8_lossy(n.string().as_bytes()).into_owned())
                .unwrap_or_default();
            let val = InterfaceStatic::static_named(&text);
            if val.is_some() {
                editor.add_value(&val.cast(), "")?;
            }
        }
        Ok(editor)
    }
}

impl IFSelectEditor for IFSelectParamEditor {
    fn editor_base(&self) -> &IFSelectEditorBase {
        &self.base
    }
    /// `Label`.
    fn label(&self) -> AsciiString {
        self.thelabel.clone()
    }
    /// `Recognize(form)`: true (no constraint).
    fn recognize(&self, _form: &Handle<IFSelectEditForm>) -> Result<bool, Failure> {
        Ok(true)
    }
    /// `StringValue(form, num)`: the text of the value `num`.
    fn string_value(
        &self,
        _form: &Handle<IFSelectEditForm>,
        num: i32,
    ) -> Result<Handle<HAsciiString>, Failure> {
        Ok(live_value(self, num)?.h_string_value())
    }
    /// `Load(form, ent, model)`: the texts of the values loaded in `form`.
    fn load(
        &self,
        form: &Handle<IFSelectEditForm>,
        _ent: &Handle<StandardTransient>,
        _model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        for i in 1..=self.nb_values() {
            form.load_value(i, &live_value(self, i)?.h_string_value());
        }
        Ok(true)
    }
    /// `Apply(form, ent, model)`: the modified texts of `form` set to the values.
    fn apply(
        &self,
        form: &Handle<IFSelectEditForm>,
        _ent: &Handle<StandardTransient>,
        _model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        for i in 1..=self.nb_values() {
            if form.is_modified(i)? {
                live_value(self, i)?.set_h_string_value(&form.edited_value(i)?);
            }
        }
        Ok(true)
    }
}
