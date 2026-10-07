// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the lists given by text.

//! The lists of entities given by an object, by entity numbers or labels, or by
//! selection names applied in chain ("sel1 sel2 (1,2)").
use super::{IFSelectTransientList, IFSelectWorkSession};
use crate::if_select::IFSelectSelection;
use crate::interface::InterfaceModelEntities;
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient, exceptions},
    tcollection::HAsciiString,
};
use std::collections::BTreeSet;

/// The entity identifiers of "(id1, id2 ...)": the texts between the separators, the
/// spaces skipped, up to the first ')' after a non empty identifier.
pub(super) fn parenthesized_ids(selname: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut entid = String::new();
    for c in selname.chars().skip(1) {
        if c == ' ' {
            continue;
        }
        if c != ',' && c != ')' {
            entid.push(c);
            continue;
        }
        if entid.is_empty() {
            continue;
        }
        ids.push(std::mem::take(&mut entid));
        if c == ')' {
            break;
        }
    }
    ids
}

impl IFSelectWorkSession {
    /// `GiveList(obj)`: the result of a selection, all the entities of the model, an
    /// entity of the model, or the list a text gives; `None` for null. A native list
    /// object has no counterpart (lists are values here).
    pub fn give_list(
        &self,
        obj: &Handle<StandardTransient>,
    ) -> Result<IFSelectTransientList, Failure> {
        if obj.is_null() {
            return Ok(None);
        }
        let sel = obj.cast::<dyn IFSelectSelection>();
        if sel.is_some() {
            return Ok(Some(self.eval_selection(&sel)?.content()));
        }
        let model = self.model();
        if let Some(m) = model
            .get()
            .filter(|_| obj.object_address() == model.object_address())
        {
            return (1..=m.nb_entities())
                .map(|i| m.value(i))
                .collect::<Result<Vec<_>, _>>()
                .map(Some);
        }
        if self.starting_number(obj) > 0 {
            return Ok(Some(vec![obj.clone()]));
        }
        if let Some(text) = obj.cast::<HAsciiString>().get() {
            return self.give_list_texts(&String::from_utf8_lossy(text.string().as_bytes()), "");
        }
        Ok(Some(Vec::new()))
    }
    /// `GiveList(first, second = "")`: the list `first` gives, applied to the list
    /// `second` gives (leading spaces skipped); `None` for an empty `first`.
    pub fn give_list_texts(
        &self,
        first: &str,
        second: &str,
    ) -> Result<IFSelectTransientList, Failure> {
        if first.is_empty() {
            return Ok(None);
        }
        if let Some(rest) = first.strip_prefix(' ') {
            return self.give_list_texts(rest, second);
        }
        if let Some(rest) = second.strip_prefix(' ') {
            return self.give_list_texts(first, rest);
        }
        let list = if second.is_empty() {
            None
        } else {
            self.give_list_texts(second, "")?
        };
        self.give_list_from(first, list)
    }
    /// `GiveListFromList(selname, ent)`: the list `selname` gives applied to the list of
    /// `ent` (see `give_list`).
    pub fn give_list_from_list(
        &self,
        selname: &str,
        ent: &Handle<StandardTransient>,
    ) -> Result<IFSelectTransientList, Failure> {
        let list = if selname.starts_with('(') || self.number_from_label(selname, 0) > 0 {
            None
        } else {
            self.give_list(ent)?
        };
        self.give_list_from(selname, list)
    }
    /// The body of `GiveListFromList` on a list: "(ids)", an entity number or label, or
    /// selection names applied in chain from the right.
    fn give_list_from(
        &self,
        selname: &str,
        list: IFSelectTransientList,
    ) -> Result<IFSelectTransientList, Failure> {
        if selname.starts_with('(') {
            return Ok(Some(self.entities_of_ids(&parenthesized_ids(selname))));
        }
        let num = self.number_from_label(selname, 0);
        if num > 0 {
            return self.give_list(&self.starting_entity(num));
        }
        let (nomsel, rest) = match selname.find(' ') {
            Some(nb) => (&selname[..nb], Some(&selname[nb + 1..])),
            None => (selname, None),
        };
        if nomsel.is_empty() {
            return Ok(list);
        }
        let sel = self.give_selection(nomsel);
        if sel.is_null() {
            println!("Neither Entity Number/Label nor Selection :{nomsel}");
            return Ok(list);
        }
        let list = match rest {
            Some(r) => self.give_list_from(r, list)?,
            _ => list,
        };
        match list {
            None => self.selection_result(&sel),
            Some(l) => self.selection_result_from_list(&sel, Some(&l)),
        }
    }
    /// The entities of the identifiers (numbers or labels), each once.
    fn entities_of_ids(&self, ids: &[String]) -> Vec<Handle<StandardTransient>> {
        let mut numap = BTreeSet::new();
        let mut list = Vec::new();
        for id in ids {
            let num = self.number_from_label(id, 0);
            if num <= 0 || !numap.insert(num) {
                continue;
            }
            let anent = self.starting_entity(num);
            if anent.is_some() {
                list.push(anent);
            }
        }
        list
    }
    /// `GiveListCombined(l1, l2, mode)`: `None` when a list is null. Native defect kept:
    /// the result list is never allocated, so two lists raise `Standard_NullObject`.
    pub fn give_list_combined(
        &self,
        l1: Option<&[Handle<StandardTransient>]>,
        l2: Option<&[Handle<StandardTransient>]>,
        mode: i32,
    ) -> Result<IFSelectTransientList, Failure> {
        if l1.is_none() || l2.is_none() {
            return Ok(None);
        }
        Err(exceptions::null_object(&format!(
            "IFSelect_WorkSession::GiveListCombined (mode {mode}): the result list is never allocated natively"
        )))
    }
}
