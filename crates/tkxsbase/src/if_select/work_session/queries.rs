// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the check and parent queries.

//! The check status of the entities (own, or propagated from the entities they
//! share) and the parenthood of two entities.
use super::IFSelectWorkSession;
use crate::interface::{InterfaceCheckIterator, InterfaceIntList, InterfaceModelEntities};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions};

/// The propagated status of a check status `val`, for a fail (`quoi` 1) or a warning.
fn propagated(val: u8, quoi: bool) -> u8 {
    match val {
        b' ' => {
            if quoi {
                b'3'
            } else {
                b'6'
            }
        }
        b'1' => {
            if quoi {
                b'4'
            } else {
                b'7'
            }
        }
        b'2' => {
            if quoi {
                b'5'
            } else {
                b'8'
            }
        }
        b'3' => {
            if quoi {
                b' '
            } else {
                b'6'
            }
        }
        b'4' => {
            if quoi {
                b' '
            } else {
                b'7'
            }
        }
        b'5' => {
            if quoi {
                b' '
            } else {
                b'8'
            }
        }
        _ => b' ',
    }
}

/// `IFSelect_QueryProp(list, ana, num, quoi)`: the status propagated, depth first, to
/// the entities listed for `num`. Native defect kept: the list is empty (its filling
/// from the graph is commented out natively), so nothing is propagated.
fn query_prop(
    list: &mut InterfaceIntList,
    ana: &mut [u8],
    num: i32,
    quoi: bool,
) -> Result<(), Failure> {
    list.set_number(num)?;
    for i in 1..=list.length() {
        if i > 1 {
            list.set_number(num)?; // because recursive call + depth first
        }
        let n = list.value(i)?;
        let slot = usize::try_from(n - 1)
            .ok()
            .filter(|s| *s < ana.len())
            .ok_or_else(|| {
                exceptions::out_of_range(&format!(
                    "IFSelect_QueryProp: {n}; expected [1, {}]",
                    ana.len()
                ))
            })?;
        let val = propagated(ana[slot], quoi);
        if val == b' ' {
            continue;
        }
        ana[slot] = val;
        query_prop(list, ana, n, quoi)?;
    }
    Ok(())
}

/// `TCollection_AsciiString::SetValue(where, what)`: `Standard_OutOfRange` out of
/// 1..Length.
fn set_status(ana: &mut [u8], num: i32, val: u8) -> Result<(), Failure> {
    let len = ana.len();
    let slot = usize::try_from(num - 1)
        .ok()
        .filter(|s| *s < len)
        .ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "IFSelect_WorkSession::QueryCheckList: entity {num}; expected [1, {len}]"
            ))
        })?;
    ana[slot] = val;
    Ok(())
}

impl IFSelectWorkSession {
    /// `QueryCheckList(chl)`: the check status of each entity ('1' warning, '2' fail),
    /// then propagated. `Standard_OutOfRange` for a global check (number 0), natively a
    /// position out of the status string.
    pub fn query_check_list(&self, chl: &InterfaceCheckIterator) -> Result<(), Failure> {
        if !self.is_loaded() {
            return Ok(());
        }
        let nb = self.nb_starting_entities();
        let mut ana = vec![b' '; nb as usize + 1];
        chl.start();
        while chl.more() {
            let (num, ach) = (chl.number()?, chl.value()?);
            if ach.has_failed() {
                set_status(&mut ana, num, b'2')?;
            } else if ach.has_warnings() {
                set_status(&mut ana, num, b'1')?;
            }
            chl.next();
        }
        let mut list = InterfaceIntList::new(); // = thegraph->Graph().SharingNums(0);
        for (stats, quoi) in [(b"147", false), (b"258", true)] {
            for i in 1..=nb {
                if stats.contains(&ana[i as usize - 1]) {
                    query_prop(&mut list, &mut ana, i, quoi)?;
                }
            }
        }
        self.state.write().thecheckana = ana;
        Ok(())
    }
    /// `QueryCheckStatus(ent)`: 0 none, 1 warning, 2 fail, +10 for a propagated fail and
    /// +20 for a propagated warning; -1 when unknown.
    pub fn query_check_status(&self, ent: &Handle<StandardTransient>) -> i32 {
        if !self.is_loaded() {
            return -1;
        }
        let num = self.model().get().map_or(0, |m| m.number(ent));
        let st = self.state.read();
        if num == 0 || st.thecheckana.len() < num as usize {
            return -1;
        }
        match st.thecheckana[num as usize - 1] {
            b'1' => 1,
            b'2' => 2,
            b'3' => 10,
            b'4' => 11,
            b'5' => 12,
            b'6' => 20,
            b'7' => 21,
            b'8' => 22,
            _ => 0,
        }
    }
    /// `QueryParent(entdad, entson)`: 0 for the same entity, the depth of `entdad` above
    /// `entson` through the sharing entities, -1 when not a parent.
    pub fn query_parent(
        &self,
        entdad: &Handle<StandardTransient>,
        entson: &Handle<StandardTransient>,
    ) -> Result<i32, Failure> {
        let (ndad, nson) = (self.starting_number(entdad), self.starting_number(entson));
        if ndad < 1 || nson < 1 {
            return Ok(-1);
        }
        if ndad == nson {
            return Ok(0);
        }
        let hgraph = self.current_graph().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::QueryParent: no graph")
        })?;
        let list = hgraph.graph().sharings(entson)?.content();
        for ent in &list {
            if ent == entdad {
                return Ok(1);
            }
            let stat = self.query_parent(entdad, ent)?;
            if stat >= 0 {
                return Ok(stat + 1);
            }
        }
        Ok(-1) // not yet implemented ...
    }
}
