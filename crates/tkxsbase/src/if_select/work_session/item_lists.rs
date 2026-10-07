// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the labels and lists of items.

//! The labels of the items ("Kind:label") and the lists of items by kind, by name or
//! by label.
use super::IFSelectWorkSession;
use crate::if_select::{
    IFSelectDispatch, IFSelectEditForm, IFSelectEditor, IFSelectGeneralModifier, IFSelectIntParam,
    IFSelectModifier, IFSelectSelection, IFSelectSignature, IFSelectSignatureList,
    IFSelectSignatureListOps, IFSelectTransformer,
};
use rustycad_tkernel::{
    standard::{Handle, RuntimeType, StandardTransient},
    tcollection::HAsciiString,
};

/// The text of a handle text (empty for null).
pub(crate) fn h_text(h: &Handle<HAsciiString>) -> String {
    h.get()
        .map(|t| String::from_utf8_lossy(t.string().as_bytes()).into_owned())
        .unwrap_or_default()
}

/// `HAsciiString::Search(what)`: the 1-based position of `what` in `text`, -1 when
/// absent.
pub(crate) fn search(text: &str, what: &str) -> i32 {
    text.find(what).map_or(-1, |p| p as i32 + 1)
}

/// The label of a modifier, a dispatch, a transformer, an edit form or an editor.
fn tool_label(var: &Handle<StandardTransient>) -> Option<String> {
    let text = |a: rustycad_tkernel::tcollection::AsciiString| {
        String::from_utf8_lossy(a.as_bytes()).into_owned()
    };
    if let Some(m) = var.cast::<dyn IFSelectGeneralModifier>().get() {
        let kind = if var.cast::<dyn IFSelectModifier>().is_some() {
            "ModelModifier:"
        } else {
            "FileModifier:"
        };
        return Some(format!("{kind}{}", text(m.label())));
    }
    if let Some(d) = var.cast::<dyn IFSelectDispatch>().get() {
        return Some(format!("Dispatch:{}", text(d.label())));
    }
    if let Some(t) = var.cast::<dyn IFSelectTransformer>().get() {
        return Some(format!("Transformer:{}", text(t.label())));
    }
    if let Some(l) = var
        .cast::<IFSelectSignatureList>()
        .cast::<dyn IFSelectSignatureListOps>()
        .get()
    {
        return Some(format!("Counter:{}", l.name()));
    }
    if let Some(s) = var.cast::<dyn IFSelectSignature>().get() {
        return Some(format!("Signature:{}", s.name()));
    }
    if let Some(f) = var.cast::<IFSelectEditForm>().get() {
        return Some(format!("EditForm:{}", text(f.label())));
    }
    var.cast::<dyn IFSelectEditor>()
        .get()
        .map(|e| format!("Editor:{}", text(e.label())))
}

impl IFSelectWorkSession {
    /// `ItemLabel(id)`: the kind and label of an item ("Text:", "Integer:",
    /// "Selection:", ... "VariableType:" and its type); null when no item.
    pub fn item_label(&self, id: i32) -> Handle<HAsciiString> {
        let var = self.item(id);
        if var.is_null() {
            return Handle::default();
        }
        let label = if let Some(text) = var.cast::<HAsciiString>().get() {
            format!("Text:{}", String::from_utf8_lossy(text.string().as_bytes()))
        } else if let Some(intpar) = var.cast::<IFSelectIntParam>().get() {
            format!("Integer:{}", intpar.value())
        } else if let Some(sel) = var.cast::<dyn IFSelectSelection>().get() {
            format!(
                "Selection:{}",
                String::from_utf8_lossy(sel.label().as_bytes())
            )
        } else if let Some(label) = tool_label(&var) {
            label
        } else {
            format!(
                "VariableType:{}",
                var.dynamic_type().map_or("", |t| t.name())
            )
        };
        Handle::new(HAsciiString::new(&label))
    }
    /// `ItemIdents(type)`: the idents of the items of a kind (the removed ones included).
    pub fn item_idents(&self, kind: &RuntimeType) -> Vec<i32> {
        let st = self.state.read();
        (1..=st.theitems.len() as i32)
            .filter(|i| st.theitems[*i as usize - 1].0.is_kind(kind))
            .collect()
    }
    /// `ItemNames(type)`: the names of the items of a kind, in the native map order.
    pub fn item_names(&self, kind: &RuntimeType) -> Vec<Handle<HAsciiString>> {
        let st = self.state.read();
        st.thenames
            .items()
            .filter(|(_, item)| item.is_kind(kind))
            .map(|(name, _)| Handle::new(HAsciiString::new(name.as_bytes())))
            .collect()
    }
    /// `ItemNamesForLabel(label)`: the labels of the items whose label contains `label`
    /// (all for empty). Native quirk kept: the labels are listed, not the names.
    pub fn item_names_for_label(&self, label: &str) -> Vec<Handle<HAsciiString>> {
        let mut list = Vec::new();
        for i in 1..=self.max_ident() {
            let lab = h_text(&self.item_label(i));
            if lab.is_empty() || (!label.is_empty() && search(&lab, label) <= 0) {
                continue;
            }
            list.push(Handle::new(HAsciiString::new(&lab)));
        }
        list
    }
    /// `NextIdentForLabel(label, id, mode = 0)`: the next ident after `id` whose label is
    /// `label` (mode 0), starts with it (1) or contains it (2); 0 when none.
    pub fn next_ident_for_label(&self, label: &str, id: i32, mode: i32) -> i32 {
        for i in id + 1..=self.max_ident() {
            let lab = self.item_label(i);
            if lab.is_null() {
                continue;
            }
            let text = h_text(&lab);
            let found = match mode {
                0 => text == label,
                1 => search(&text, label) == 1,
                2 => search(&text, label) > 0,
                _ => false,
            };
            if found {
                return i;
            }
        }
        0 // here : not found
    }
}
