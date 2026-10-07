// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the parameters of the session.

//! The parameter editor and forms of the session (`SetParams`) and the trace of the
//! parameters by use (`TraceStatics`).
use super::IFSelectWorkSession;
use super::item_lists::h_text;
use super::params::info;
use crate::if_select::{IFSelectEditForm, IFSelectEditor, IFSelectParamEditor};
use crate::interface::InterfaceTypedValue;
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient},
    tcollection::HAsciiString,
};
use std::fmt::Write as _;

/// The forms of `SetParams`: item name, label and use numbers.
const PARAM_FORMS: [(&str, &str, &[i32]); 6] = [
    ("xst-params-general", "General Parameters", &[1]),
    ("xst-params-load", "Loading Parameters", &[]),
    ("xst-params-send", "Sending Parameters", &[3]),
    ("xst-params-split", "Split Parameters", &[4]),
    ("xst-params-read", "Read(Transfer) Parameters", &[2, 5]),
    ("xst-params-write", "Write(Transfer) Parameters", &[6]),
];

/// The title of a use in `TraceStatics`.
fn use_title(use_: i32) -> Option<&'static str> {
    match use_ {
        1 => Some("*****      General  Parameters       *****"),
        2 => Some("*****            Load  File          *****"),
        3 => Some("*****            Write File          *****"),
        4 => Some("*****            Split File          *****"),
        5 => Some("*****        Transfer (Read)         *****"),
        6 => Some("*****        Transfer (Write)        *****"),
        _ => None,
    }
}

/// A line "label value" of the split names, or "missing" for a null name (never
/// natively: the share-out gives empty names).
pub(crate) fn defined_line(
    out: &mut String,
    label: &str,
    missing: &str,
    value: &Handle<HAsciiString>,
) {
    if value.is_null() {
        let _ = writeln!(out, "{missing}");
    } else {
        let _ = writeln!(out, "{label}{}", h_text(value));
    }
}

impl IFSelectWorkSession {
    /// `SetParams(params, uselist)`: a parameter editor of the typed values of `params`
    /// ("xst-params-edit"), its complete form ("xst-params-all") and a form by use
    /// (1 general, 3 send, 4 split, 2 and 5 read, 6 write). Native quirks kept: the forms
    /// take the 0-based ranks of `uselist` as value numbers, and the loading form stays
    /// empty.
    pub fn set_params(
        &self,
        params: &[Handle<StandardTransient>],
        uselist: &[i32],
    ) -> Result<(), Failure> {
        let editor = IFSelectParamEditor::new(
            (params.len() + uselist.len()) as i32 + 50,
            "Parameter Editor",
        )?;
        for param in params {
            let val = param.cast::<InterfaceTypedValue>();
            if val.is_some() {
                editor.add_value(&val, "")?;
            }
        }
        let dynamic: Handle<dyn IFSelectEditor> = editor.clone().cast();
        self.add_named_item("xst-params-edit", &dynamic.cast(), true)?;
        let paramsall = editor.form(false, true)?;
        self.add_named_item("xst-params-all", &paramsall.cast(), true)?;
        for (name, label, uses) in PARAM_FORMS {
            let nums: Vec<i32> = (0..uselist.len() as i32)
                .filter(|i| uses.contains(&uselist[*i as usize]))
                .collect();
            let form = IFSelectEditForm::with_numbers(&dynamic, &nums, false, true, label)?;
            if !nums.is_empty() {
                self.add_named_item(name, &form.cast(), true)?;
            }
        }
        Ok(())
    }
    /// `TraceStatics(use, mode = 0)`: the title of the use (> 0), the parameters of the
    /// uses it implies, and for the split the file names.
    pub fn trace_statics(&self, use_: i32, mode: i32) -> Result<(), Failure> {
        let mut sout = String::new();
        let title = use_title(use_).filter(|_| use_ > 0 && mode == 0);
        if let Some(t) = title {
            let _ = write!(
                sout,
                "******************************************\n{t}\n******************************************\n\n"
            );
        } else if use_ > 0 && mode == 0 {
            sout.push_str("******************************************\n******************************************\n\n");
        }
        let implied = match use_ {
            5 => Some(-2),
            4 | 6 => Some(-3),
            _ => None,
        };
        if let Some(sub) = implied {
            self.trace_statics(sub, mode)?;
            if mode == 0 {
                sout.push('\n');
            }
        }
        if (use_ == 4 || use_ == -4) && mode == 0 {
            defined_line(
                &mut sout,
                "Prefix        : ",
                "Prefix       not Defined",
                &self.file_prefix()?,
            );
            defined_line(
                &mut sout,
                "Default Root  : ",
                "Default Root not Defined",
                &self.default_file_root()?,
            );
            defined_line(
                &mut sout,
                "Extension     : ",
                "Extension    not defined",
                &self.file_extension()?,
            );
        }
        if use_ > 0 && mode == 0 {
            sout.push_str("******************************************\n\n");
        }
        info(&sout);
        Ok(())
    }
}
