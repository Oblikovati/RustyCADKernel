// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_EditForm (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the edition, printing and application.

//! The edition, printing and application members of [`IFSelectEditForm`].
use super::edit_form::{FormItem, slot_out_of_range};
use super::{IFSelectEditForm, IFSelectEditValue, IFSelectListEditor, IFSelectTextList};
use crate::interface::{InterfaceInterfaceModel, InterfaceMsg, InterfaceParamType};
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::HAsciiString,
};
use std::fmt::Write as _;
use std::io::Write;

/// The status of a modified value, and of a touched one.
const MODIFIED: i32 = 1;
const TOUCHED: i32 = 2;

/// The text of a value, `(NULL)` for a null one.
fn value_text(val: &Handle<HAsciiString>) -> String {
    val.get().map_or_else(
        || String::from("(NULL)"),
        |t| String::from_utf8_lossy(t.string().as_bytes()).into_owned(),
    )
}

/// `PrintList(list, S, alsolist)`: the count of items, then the items.
fn print_list(list: &IFSelectTextList, out: &mut String, alsolist: bool) {
    let Some(items) = list else {
        out.push_str("(NULL LIST)\n");
        return;
    };
    let _ = writeln!(out, "(List : {} Items)", items.len());
    if !alsolist {
        return;
    }
    for (i, item) in items.iter().enumerate() {
        let _ = writeln!(out, "  [{}]\t{}", i + 1, value_text(item));
    }
}

impl IFSelectEditForm {
    /// The rank to modify `num` at, `None` when the form is read only or `num` absent.
    fn editable_rank(&self, num: i32) -> Option<i32> {
        if self.state.read().themodifs.len() == 1 {
            return None;
        }
        Some(self.rank_from_number(num)).filter(|t| *t != 0)
    }
    /// Records the item `tnum` with its status; `Standard_OutOfRange` beyond the arrays.
    fn record(&self, tnum: i32, status: i32, item: FormItem) -> Result<(), Failure> {
        let mut st = self.state.write();
        let upper = st.thestatus.len() - 1;
        let i = usize::try_from(tnum)
            .ok()
            .filter(|i| *i <= upper)
            .ok_or_else(|| slot_out_of_range("Modify", tnum, upper))?;
        st.thestatus[i] = status;
        st.themodifs[i] = item;
        Ok(())
    }
    /// True when a new value in the mode `acc` is refused (null and not optional, or
    /// protected or computed without `enforce`).
    fn refuses(acc: IFSelectEditValue, isnull: bool, enforce: bool) -> bool {
        (isnull && acc != IFSelectEditValue::Optional)
            || (!enforce
                && matches!(
                    acc,
                    IFSelectEditValue::EditProtected | IFSelectEditValue::EditComputed
                ))
    }
    /// True when `newval` satisfies the definition of the value `num` (an identifier
    /// naming an entity of the model).
    fn satisfies(&self, num: i32, newval: &Handle<HAsciiString>) -> Result<bool, Failure> {
        let typval = self.theeditor.typed_value(num)?;
        let tv = typval.get().ok_or_else(|| {
            exceptions::null_object(&format!(
                "IFSelect_EditForm::Modify: value {num} not defined"
            ))
        })?;
        if !tv.satisfies(newval) {
            return Ok(false);
        }
        let Some(text) = newval
            .get()
            .filter(|_| tv.param_type() == InterfaceParamType::ParamIdent)
        else {
            return Ok(true);
        };
        let model = self.model();
        let label = String::from_utf8_lossy(text.string().as_bytes()).into_owned();
        Ok(model
            .get()
            .is_some_and(|m| m.next_number_for_label(&label, 0, false) > 0))
    }
    /// `Modify(num, newval, enforce = false)`: the value `num` modified when its mode, its
    /// definition and the editor (`Update`) accept it.
    pub fn modify(
        &self,
        num: i32,
        newval: &Handle<HAsciiString>,
        enforce: bool,
    ) -> Result<bool, Failure> {
        self.state.write().thetouched = 0;
        let Some(tnum) = self.editable_rank(num) else {
            return Ok(false);
        };
        if Self::refuses(self.theeditor.edit_mode(num), newval.is_null(), enforce)
            || !self.satisfies(num, newval)?
        {
            return Ok(false);
        }
        if !self.theeditor.update(&self.this(), num, newval, enforce)? {
            return Ok(false);
        }
        self.record(tnum, MODIFIED, FormItem::from_text(newval))?;
        Ok(true)
    }
    /// `ModifyList(num, edited, enforce = false)`: the list `num` modified from a touched
    /// list editor when its mode and the editor (`UpdateList`) accept it.
    pub fn modify_list(
        &self,
        num: i32,
        edited: &Handle<IFSelectListEditor>,
        enforce: bool,
    ) -> Result<bool, Failure> {
        let Some(led) = edited.get().filter(|l| l.is_touched()) else {
            return Ok(false);
        };
        let newlist = led.edited_values();
        self.state.write().thetouched = 0;
        let Some(tnum) = self.editable_rank(num) else {
            return Ok(false);
        };
        let acc = self.theeditor.edit_mode(num);
        if matches!(
            acc,
            IFSelectEditValue::EditRead | IFSelectEditValue::EditDynamic
        ) || Self::refuses(acc, newlist.is_none(), enforce)
        {
            return Ok(false);
        }
        if !self
            .theeditor
            .update_list(&self.this(), num, newlist.as_deref(), enforce)?
        {
            return Ok(false);
        }
        self.record(tnum, MODIFIED, FormItem::from_list(newlist.as_deref()))?;
        Ok(true)
    }
    /// `ModifyListValue(num, list, enforce = false)`: `ModifyList` with a list editor
    /// loaded with `list`.
    pub fn modify_list_value(
        &self,
        num: i32,
        list: Option<&[Handle<HAsciiString>]>,
        enforce: bool,
    ) -> Result<bool, Failure> {
        let led = self.list_editor(num)?;
        let Some(live) = led.get() else {
            return Ok(false);
        };
        if !live.load_edited(list) {
            return Ok(false);
        }
        self.modify_list(num, &led, enforce)
    }
    /// Touches the item `num` (set by the editor while it updates the others).
    fn touch_item(&self, num: i32, item: FormItem) -> Result<bool, Failure> {
        let Some(tnum) = self.editable_rank(num) else {
            return Ok(false);
        };
        self.record(tnum, TOUCHED, item)?;
        self.state.write().thetouched += 1;
        Ok(true)
    }
    /// `Touch(num, newval)`: the value `num` set without check (by the editor).
    pub fn touch(&self, num: i32, newval: &Handle<HAsciiString>) -> Result<bool, Failure> {
        self.touch_item(num, FormItem::from_text(newval))
    }
    /// `TouchList(num, newlist)`: the list `num` set without check (by the editor).
    pub fn touch_list(
        &self,
        num: i32,
        newlist: Option<&[Handle<HAsciiString>]>,
    ) -> Result<bool, Failure> {
        self.touch_item(num, FormItem::from_list(newlist))
    }
    /// `ClearEdit(num = 0)`: the edit status of all values (0) or of `num`. Native quirk
    /// kept: the bound test reads the number, not the rank.
    pub fn clear_edit(&self, num: i32) {
        let tnum = if num == 0 {
            0
        } else {
            self.rank_from_number(num)
        };
        let mut st = self.state.write();
        let nb = st.thestatus.len() as i32 - 1;
        if num == 0 {
            st.thestatus.iter_mut().skip(1).for_each(|s| *s = 0);
        } else if tnum > 0
            && num <= nb
            && let Some(s) = st.thestatus.get_mut(tnum as usize)
        {
            *s = 0;
        }
    }
    /// `PrintDefs(S)`: the label and the extraction of the form.
    pub fn print_defs(&self, s: &mut dyn Write) -> std::io::Result<()> {
        let nbv = self.nb_values(true);
        let label = String::from_utf8_lossy(self.label().as_bytes()).into_owned();
        let mut out = format!("***** EditForm,  Label : {label}\n");
        if self.is_complete() {
            let _ = writeln!(out, "Complete, {nbv} Values");
        } else {
            let _ = writeln!(out, "Extraction on {nbv} Values : (extracted<-editor)");
            for iv in 1..=nbv {
                let _ = write!(out, "  {iv}<-{}", self.number_from_rank(iv));
            }
            out.push('\n');
        }
        out.push_str("*****\n");
        s.write_all(out.as_bytes())
    }
    /// The loaded data line of `PrintValues`.
    fn print_loaded(&self, out: &mut String) {
        let (loaded, ent, model) = (self.is_loaded(), self.entity(), self.model());
        if !loaded {
            out.push_str("*****         Values are NOT loaded            *****\n");
            return;
        }
        match (model.get(), ent.is_some()) {
            (None, false) => out.push_str("*****  No loaded data"),
            (None, true) => {
                let name = ent.dynamic_type().map_or("", |t| t.name());
                let _ = write!(out, "*****  No loaded Model. Loaded object : type {name}");
            }
            (Some(_), false) => out.push_str("*****  No loaded entity"),
            (Some(m), true) => {
                out.push_str("*****  Loaded entity : ");
                let mut text = Vec::new();
                let _ = m.print_label(&ent, &mut text);
                out.push_str(&String::from_utf8_lossy(&text));
            }
        }
    }
    /// `PrintValues(S, what, names, alsolist = false)`: the original (`what < 0`) or
    /// edited (`what > 0`) values, or the modifications (`what = 0`), by complete or
    /// short names.
    pub fn print_values(
        &self,
        s: &mut dyn Write,
        what: i32,
        names: bool,
        alsolist: bool,
    ) -> Result<(), Failure> {
        let label = String::from_utf8_lossy(self.label().as_bytes()).into_owned();
        let mut out = String::from("****************************************************\n");
        let _ = writeln!(
            out,
            "*****  {label}{}*****",
            InterfaceMsg::blanks_for_text(&label, 40)
        );
        out.push_str("*****                                          *****\n");
        self.print_loaded(&mut out);
        out.push_str("\n****************************************************\n\n");
        let (mut nams, mut maxnam) = (
            names,
            self.theeditor.max_name_length(if names { 0 } else { -1 }),
        );
        if maxnam == 0 {
            (maxnam, nams) = (self.theeditor.max_name_length(0), true);
        }
        out.push_str(if what != 0 {
            "Mod N0 Name               Value\n"
        } else {
            " N0 Name               Value\n"
        });
        let nbv = self.nb_values(true);
        let mut nbmod = 0;
        for iv in 1..=nbv {
            let jv = self.number_from_rank(iv);
            let name =
                String::from_utf8_lossy(self.theeditor.name(jv, !nams)?.as_bytes()).into_owned();
            let head = format!(
                "{}{iv} {name}{}",
                InterfaceMsg::blanks_for_integer(iv, 3),
                InterfaceMsg::blanks_for_text(&name, maxnam)
            );
            if what != 0 {
                self.print_value_line(&mut out, &head, jv, what, alsolist)?;
            } else if self.is_modified(jv)? {
                nbmod += 1;
                self.print_modification(&mut out, &head, jv, maxnam, alsolist)?;
            }
        }
        if what == 0 {
            let _ = writeln!(out, "On {nbv} Values, {nbmod} Modified");
        }
        s.write_all(out.as_bytes())
            .map_err(|e| exceptions::program_error(&format!("IFSelect_EditForm::PrintValues: {e}")))
    }
    /// One line of `PrintValues` for `what != 0`.
    fn print_value_line(
        &self,
        out: &mut String,
        head: &str,
        jv: i32,
        what: i32,
        alsolist: bool,
    ) -> Result<(), Failure> {
        let _ = write!(
            out,
            "{}{head}  ",
            if self.is_modified(jv)? { "* " } else { "  " }
        );
        if self.theeditor.is_list(jv) {
            let list = if what < 0 {
                self.original_list(jv)?
            } else {
                self.edited_list(jv)?
            };
            print_list(&list, out, alsolist);
            return Ok(());
        }
        let val = if what < 0 {
            self.original_value(jv)?
        } else {
            self.edited_value(jv)?
        };
        let _ = writeln!(out, "{}", value_text(&val));
        Ok(())
    }
    /// The lines of a modification in `PrintValues` for `what = 0`.
    fn print_modification(
        &self,
        out: &mut String,
        head: &str,
        jv: i32,
        maxnam: i32,
        alsolist: bool,
    ) -> Result<(), Failure> {
        if self.theeditor.is_list(jv) {
            let _ = write!(out, "{head} ORIG:");
            print_list(&self.original_list(jv)?, out, alsolist);
            let _ = write!(
                out,
                "{}MOD :",
                InterfaceMsg::blanks_for_text("", maxnam + 5)
            );
            print_list(&self.edited_list(jv)?, out, alsolist);
            return Ok(());
        }
        let _ = writeln!(out, "{head} ORIG:{}", value_text(&self.original_value(jv)?));
        let _ = writeln!(
            out,
            "{} MOD :{}",
            InterfaceMsg::blanks_for_text("", maxnam + 4),
            value_text(&self.edited_value(jv)?)
        );
        Ok(())
    }
    /// `Apply`: `ApplyData` on the loaded data; the edit status is cleared unless kept.
    pub fn apply(&self) -> Result<bool, Failure> {
        let stat = self.apply_data(&self.entity(), &self.model())?;
        if stat && !self.edit_keep_status() {
            self.clear_edit(0);
        }
        Ok(stat)
    }
    /// `Recognize`: the editor recognizes the form.
    pub fn recognize(&self) -> Result<bool, Failure> {
        self.theeditor.recognize(&self.this())
    }
    /// `ApplyData(ent, model)`: the editor applies the form to `ent` and `model`.
    pub fn apply_data(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure> {
        self.theeditor.apply(&self.this(), ent, model)
    }
    /// `Undo`: the modified values get back their originals, then `Apply`; false when
    /// read only or not undoable.
    pub fn undo(&self) -> Result<bool, Failure> {
        {
            let mut st = self.state.write();
            if st.thestatus.len() == 1 || st.theorigs.len() == 1 {
                return Ok(false);
            }
            for i in 1..st.thestatus.len() {
                if st.thestatus[i] != 0 {
                    st.themodifs[i] = st.theorigs.get(i).cloned().unwrap_or_default();
                }
            }
        }
        self.apply()
    }
}
