// Rust port of DataExchange/TKXSBase/Interface/Interface_Graph.cxx (HasShareErrors,
// Shareds, Sharings, TypedSharings, RootEntities, Name) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The queries of an `Interface_Graph` on sharing relations.
use super::graph::{GRAPH_SHARE_ERROR, redefined_content};
use super::{InterfaceEntityIterator, InterfaceGraph, InterfaceModelEntities, InterfaceShareTool};
use rustycad_tkernel::{
    standard::{Failure, Handle, RuntimeType, StandardTransient, exceptions},
    tcollection::HAsciiString,
};

impl InterfaceGraph {
    /// `HasShareErrors(ent)`: an entity out of the model has errors; false without
    /// statuses.
    pub fn has_share_errors(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        if self.thestats.is_null() {
            return Ok(false);
        }
        let num = self.entity_number(ent);
        if num == 0 {
            return Ok(true);
        }
        self.theflags.value(num, GRAPH_SHARE_ERROR)
    }
    /// `Shareds(ent)`: the entities `ent` shares, from its module (evaluated each time);
    /// empty for an entity out of the model. A model without GTool is dereferenced
    /// natively: `Standard_NullObject`.
    pub fn shareds(
        &self,
        ent: &Handle<StandardTransient>,
    ) -> Result<InterfaceEntityIterator, Failure> {
        let mut iter = InterfaceEntityIterator::new();
        let num = self.entity_number(ent);
        if num == 0 {
            return Ok(iter);
        }
        let model = self.live_model()?;
        let current = redefined_content(model, num, ent);
        let gtool = model.gtool();
        let live = gtool.get().ok_or_else(|| {
            exceptions::null_object(
                "Interface_Graph::Shareds: the model has no GTool; expected one",
            )
        })?;
        if let Some((module, cn)) = live.select(&current, false) {
            module.fill_shared(model, cn, &current, &mut iter);
        }
        Ok(iter)
    }
    /// `GetShareds(ent)`: `Shareds(ent).Content()`.
    pub fn get_shareds(
        &self,
        ent: &Handle<StandardTransient>,
    ) -> Result<Vec<Handle<StandardTransient>>, Failure> {
        Ok(self.shareds(ent)?.content())
    }
    /// `GetSharings(ent)`: the entities sharing `ent`; `None` (natively null) for an
    /// entity out of the model.
    pub fn get_sharings(
        &self,
        ent: &Handle<StandardTransient>,
    ) -> Result<Option<Vec<Handle<StandardTransient>>>, Failure> {
        let num = self.entity_number(ent);
        if num == 0 {
            return Ok(None);
        }
        let numbers = self.sharing_numbers(num)?;
        let sharings: Result<Vec<_>, Failure> = numbers.iter().map(|n| self.entity(*n)).collect();
        Ok(Some(sharings?))
    }
    /// `Sharings(ent)`.
    pub fn sharings(
        &self,
        ent: &Handle<StandardTransient>,
    ) -> Result<InterfaceEntityIterator, Failure> {
        let mut iter = InterfaceEntityIterator::new();
        if let Some(list) = self.get_sharings(ent)? {
            iter.add_list(&list);
        }
        Ok(iter)
    }
    /// `TypedSharings(ent, type)`: the first entities of type `type` met going up the
    /// sharings of `ent` (`ent` itself when it has that type).
    pub fn typed_sharings(
        &self,
        ent: &Handle<StandardTransient>,
        atype: &RuntimeType,
    ) -> Result<InterfaceEntityIterator, Failure> {
        let mut iter = InterfaceEntityIterator::new();
        add_typed_sharings(ent, atype, &mut iter, self.size(), self)?;
        Ok(iter)
    }
    /// `RootEntities`: the entities no entity shares.
    pub fn root_entities(&self) -> Result<InterfaceEntityIterator, Failure> {
        let mut iter = InterfaceEntityIterator::new();
        let table = self.thesharings.clone().unwrap_or_default();
        for (index, sharing) in table.iter().enumerate() {
            if sharing.is_empty() {
                iter.add_item(&self.entity(index as i32 + 1)?);
            }
        }
        Ok(iter)
    }
    /// `Name(ent)`: the name a module gives to an entity out of the model (null for an
    /// entity of the model, or without GTool or module).
    pub fn name(&self, ent: &Handle<StandardTransient>) -> Result<Handle<HAsciiString>, Failure> {
        let Some(model) = self.themodel.get() else {
            return Ok(Handle::default());
        };
        if model.number(ent) != 0 {
            return Ok(Handle::default());
        }
        let gtool = model.gtool();
        let Some(live) = gtool.get() else {
            return Ok(Handle::default());
        };
        let Some((module, cn)) = live.select(ent, false) else {
            return Ok(Handle::default());
        };
        let sht = InterfaceShareTool::from_graph(self)?;
        Ok(module.name(cn, ent, &sht))
    }
    /// The numbers of the entities sharing entity `num`.
    pub(super) fn sharing_numbers(&self, num: i32) -> Result<Vec<i32>, Failure> {
        let table = self.thesharings.as_ref().ok_or_else(|| {
            exceptions::null_object(
                "Interface_Graph: no sharing table; expected an evaluated graph",
            )
        })?;
        let index = usize::try_from(num - 1).ok().filter(|i| *i < table.len());
        let index = index.ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "Interface_Graph: entity {num}; expected [1, {}]",
                table.len()
            ))
        })?;
        Ok(table[index].clone())
    }
}

/// `AddTypedSharings` (static helper of `TypedSharings`). Native quirk kept: the limit
/// passed down is the count of sharings of the level, not the graph size.
fn add_typed_sharings(
    ent: &Handle<StandardTransient>,
    atype: &RuntimeType,
    iter: &mut InterfaceEntityIterator,
    n: i32,
    graph: &InterfaceGraph,
) -> Result<(), Failure> {
    if ent.is_null() {
        return Ok(());
    }
    if ent.is_kind(atype) {
        iter.add_item(ent);
        return Ok(());
    }
    if iter.nb_entities() > n {
        return Ok(());
    }
    let Some(list) = graph.get_sharings(ent)? else {
        return Ok(());
    };
    let nb = list.len() as i32;
    for sharing in &list {
        add_typed_sharings(sharing, atype, iter, nb, graph)?;
    }
    Ok(())
}
