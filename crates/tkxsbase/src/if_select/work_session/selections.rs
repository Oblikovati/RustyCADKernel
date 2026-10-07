// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the edition of the selections.

//! The inputs of the selections (sources, extraction direction, controls,
//! combinations), the pointed selections and the selections given by name.
use super::IFSelectWorkSession;
use crate::if_select::{
    IFSelectSelectCombine, IFSelectSelectControl, IFSelectSelectDeduct, IFSelectSelectExtract,
    IFSelectSelectModelEntities, IFSelectSelectPointed, IFSelectSelectSignature, IFSelectSelection,
    IFSelectSignCounterOps, IFSelectSignature, IFSelectSignatureList,
};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};

/// The parts of a selection name "name(signature)": the name, and the text between the
/// last '(' and the closing ')' when both are past the start.
pub(super) struct SelectionName {
    pub(super) name: String,
    pub(super) sign: Option<String>,
}

/// Splits `selname` as `GiveSelection` does (the last '(' and the ')' closing the
/// outermost level).
pub(super) fn split_selection_name(selname: &str) -> SelectionName {
    let bytes = selname.as_bytes();
    let (mut np, mut nf, mut nivp) = (None, None, 0);
    for (n, b) in bytes.iter().enumerate() {
        if *b == b'(' {
            np = Some(n);
            nivp += 1;
        }
        if *b == b')' {
            nivp -= 1;
            if nivp <= 0 {
                nf = Some(n);
            }
        }
    }
    let mut nomsel = bytes.to_vec();
    if let Some(f) = nf {
        nomsel[f] = 0;
    }
    if let Some(p) = np {
        nomsel[p] = 0;
    }
    let text = |from: usize| -> String {
        let part = &nomsel[from.min(nomsel.len())..];
        let end = part.iter().position(|b| *b == 0).unwrap_or(part.len());
        String::from_utf8_lossy(&part[..end]).into_owned()
    };
    let sign = match (np, nf) {
        (Some(p), Some(f)) if p > 0 && f > 0 => Some(text(p + 1)),
        _ => None,
    };
    SelectionName {
        name: text(0),
        sign,
    }
}

impl IFSelectWorkSession {
    /// True when `sel` is an item.
    fn is_item(&self, sel: &Handle<dyn IFSelectSelection>) -> bool {
        self.item_ident(&sel.cast()) != 0
    }
    /// `NbSources(sel)`: 1 for an extraction or a deduction, 2 for a control, the
    /// inputs of a combination, 0 otherwise.
    pub fn nb_sources(&self, sel: &Handle<dyn IFSelectSelection>) -> i32 {
        if !self.is_item(sel) {
            return 0;
        }
        if sel.cast::<dyn IFSelectSelectDeduct>().is_some() {
            return 1;
        }
        if sel.cast::<dyn IFSelectSelectControl>().is_some() {
            return 2;
        }
        sel.cast::<dyn IFSelectSelectCombine>()
            .get()
            .map_or(0, |c| c.nb_inputs())
    }
    /// `Source(sel, num = 1)`: the input `num` of `sel`; null otherwise;
    /// `Standard_OutOfRange` for a combination without input `num`.
    pub fn source(
        &self,
        sel: &Handle<dyn IFSelectSelection>,
        num: i32,
    ) -> Result<Handle<dyn IFSelectSelection>, Failure> {
        if !self.is_item(sel) {
            return Ok(Handle::default());
        }
        if let Some(d) = sel.cast::<dyn IFSelectSelectDeduct>().get() {
            return Ok(d.input());
        }
        if let Some(c) = sel.cast::<dyn IFSelectSelectControl>().get() {
            return Ok(match num {
                1 => c.main_input(),
                2 => c.second_input(),
                _ => Handle::default(),
            });
        }
        match sel.cast::<dyn IFSelectSelectCombine>().get() {
            Some(c) => c.input_at(num),
            None => Ok(Handle::default()),
        }
    }
    /// `IsReversedSelectExtract(sel)`: true for a reversed extraction item.
    pub fn is_reversed_select_extract(&self, sel: &Handle<dyn IFSelectSelection>) -> bool {
        self.is_item(sel)
            && sel
                .cast::<dyn IFSelectSelectExtract>()
                .get()
                .is_some_and(|x| !x.is_direct())
    }
    /// `ToggleSelectExtract(sel)`: the direction of an extraction item reversed.
    pub fn toggle_select_extract(&self, sel: &Handle<dyn IFSelectSelection>) -> bool {
        if !self.is_item(sel) {
            return false;
        }
        let ext = sel.cast::<dyn IFSelectSelectExtract>();
        let Some(x) = ext.get() else {
            return false;
        };
        x.set_direct(!x.is_direct());
        true
    }
    /// `SetInputSelection(sel, input)`: the input of an extraction or deduction item
    /// (`input` an item or null).
    pub fn set_input_selection(
        &self,
        sel: &Handle<dyn IFSelectSelection>,
        input: &Handle<dyn IFSelectSelection>,
    ) -> bool {
        if !self.is_item(sel) || (input.is_some() && !self.is_item(input)) {
            return false;
        }
        let deduct = sel.cast::<dyn IFSelectSelectDeduct>();
        let Some(d) = deduct.get() else {
            return false;
        };
        d.set_input(input);
        true
    }
    /// `SetControl(sel, sc, formain = true)`: the main or second input of a control item.
    pub fn set_control(
        &self,
        sel: &Handle<dyn IFSelectSelection>,
        sc: &Handle<dyn IFSelectSelection>,
        formain: bool,
    ) -> bool {
        let dsel = sel.cast::<dyn IFSelectSelectControl>();
        let Some(c) = dsel.get().filter(|_| self.is_item(sel) && self.is_item(sc)) else {
            return false;
        };
        if formain {
            c.set_main_input(sc)
        } else {
            c.set_second_input(sc)
        }
        true
    }
    /// `CombineAdd(selcomb, seladd, atnum = 0)`: the count of inputs after the addition,
    /// 0 when not items.
    pub fn combine_add(
        &self,
        selcomb: &Handle<dyn IFSelectSelection>,
        seladd: &Handle<dyn IFSelectSelection>,
        atnum: i32,
    ) -> i32 {
        let csel = selcomb.cast::<dyn IFSelectSelectCombine>();
        let Some(c) = csel
            .get()
            .filter(|_| self.is_item(selcomb) && self.is_item(seladd))
        else {
            return 0;
        };
        c.add(seladd, atnum);
        c.nb_inputs()
    }
    /// `CombineRemove(selcomb, selrem)`: the last occurrence of `selrem` removed; true
    /// for items (even when absent).
    pub fn combine_remove(
        &self,
        selcomb: &Handle<dyn IFSelectSelection>,
        selrem: &Handle<dyn IFSelectSelection>,
    ) -> Result<bool, Failure> {
        let csel = selcomb.cast::<dyn IFSelectSelectCombine>();
        let Some(c) = csel
            .get()
            .filter(|_| self.is_item(selcomb) && self.is_item(selrem))
        else {
            return Ok(false);
        };
        for i in (1..=c.nb_inputs()).rev() {
            if c.input_at(i)? == *selrem {
                c.remove_at(i);
                return Ok(true);
            }
        }
        Ok(true)
    }
    /// `NewSelectPointed(list, name)`: a new pointed selection of `list` added under
    /// `name` (null when refused).
    pub fn new_select_pointed(
        &self,
        list: Option<&[Handle<StandardTransient>]>,
        name: &str,
    ) -> Result<Handle<dyn IFSelectSelection>, Failure> {
        let sel = IFSelectSelectPointed::new();
        if list.is_some() {
            sel.add_list(list);
        }
        let handle: Handle<dyn IFSelectSelection> = Handle::new(sel).cast();
        let added = self.add_named_item(name, &handle.cast(), true)?;
        Ok(if added == 0 {
            Handle::default()
        } else {
            handle
        })
    }
    /// `SetSelectPointed(sel, list, mode)`: the list set (0), added (> 0) or removed
    /// (< 0) in a pointed selection.
    pub fn set_select_pointed(
        &self,
        sel: &Handle<dyn IFSelectSelection>,
        list: Option<&[Handle<StandardTransient>]>,
        mode: i32,
    ) -> bool {
        let sp = sel.cast::<IFSelectSelectPointed>();
        let Some(p) = sp.get().filter(|_| list.is_some()) else {
            return false;
        };
        if mode == 0 {
            p.clear();
        }
        if mode >= 0 {
            p.add_list(list);
        } else {
            p.remove_list(list);
        }
        true
    }
    /// `GiveSelection(selname)`: the selection named `selname`, or for
    /// "name(text)" a signature selection of the signature or counter `name` matching
    /// `text` on the model entities (null with a message when neither).
    pub fn give_selection(&self, selname: &str) -> Handle<dyn IFSelectSelection> {
        let parts = split_selection_name(selname);
        let item = self.named_item(&parts.name);
        let Some(text) = parts.sign else {
            return item.cast();
        };
        let sign = item.cast::<dyn IFSelectSignature>();
        let cnt = item
            .cast::<IFSelectSignatureList>()
            .cast::<dyn IFSelectSignCounterOps>();
        let selsign = if sign.is_some() {
            IFSelectSelectSignature::with_signature(&sign, &text, false)
        } else if cnt.is_some() {
            IFSelectSelectSignature::with_counter(&cnt, &text, false)
        } else {
            println!("{selname} : neither Signature nor Counter");
            return Handle::default();
        };
        selsign.set_input(&Handle::new(IFSelectSelectModelEntities::new()).cast()); // by default
        Handle::new(selsign).cast()
    }
}
