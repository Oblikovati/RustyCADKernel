// Rust port of DataExchange/TKXSBase/Interface/Interface_InterfaceModel.cxx (the
// non-virtual members on entities) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The non-virtual members of `Interface_InterfaceModel` on its entities: numbering,
//! types, additions with references, reordering, categories and printing.
use super::errors::interface_mismatch;
use super::{
    InterfaceEntityIterator, InterfaceGTool, InterfaceGeneralLib, InterfaceInterfaceModel,
    InterfaceProtocol, InterfaceReportEntity, InterfaceSignTypeBase,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, RuntimeType, StandardTransient, exceptions},
    tcollection::HAsciiString,
};
use std::io::{self, Write};

/// The non-virtual entity members of `Interface_InterfaceModel`, for every model.
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceInterfaceModel, InterfaceModelEntities};
/// fn first(model: &dyn InterfaceInterfaceModel) -> bool { model.value(1).is_ok() }
/// ```
pub trait InterfaceModelEntities: InterfaceInterfaceModel {
    /// `Destroy`: entities, checks and categories (a minimum, as natively).
    fn destroy(&self) {
        self.clear_entities();
        super::interface_model::clear_global_checks(self.model_base());
        self.model_base().write().thecategory.nullify();
    }
    /// `SetProtocol(proto)`: a new GTool for `proto`.
    fn set_protocol(&self, proto: &Handle<dyn InterfaceProtocol>) {
        self.model_base().write().thegtool = Handle::new(InterfaceGTool::with_protocol(proto, 0));
    }
    /// `SetGTool(gtool)`.
    fn set_gtool(&self, gtool: &Handle<InterfaceGTool>) {
        self.model_base().write().thegtool = gtool.clone();
    }
    /// `GTool`.
    fn gtool(&self) -> Handle<InterfaceGTool> {
        self.model_base().read().thegtool.clone()
    }
    /// `DispatchStatus` read (natively a modifiable `bool&`).
    fn dispatch_status(&self) -> bool {
        self.model_base().read().isdispatch
    }
    /// `DispatchStatus` written.
    fn set_dispatch_status(&self, status: bool) {
        self.model_base().write().isdispatch = status;
    }
    /// `NbEntities`.
    fn nb_entities(&self) -> i32 {
        self.model_base().read().theentities.len() as i32
    }
    /// `Contains(anentity)`: an entity of the model, or a report of one.
    fn contains(&self, anentity: &Handle<StandardTransient>) -> bool {
        if self.model_base().read().theentities.contains(anentity) {
            return true;
        }
        let rep = anentity.cast::<InterfaceReportEntity>();
        rep.get().is_some_and(|r| self.contains(&r.concerned()))
    }
    /// `Number(anentity)`: the rank of an entity (or of the entity a report concerns),
    /// 0 when not in the model.
    fn number(&self, anentity: &Handle<StandardTransient>) -> i32 {
        if anentity.is_null() {
            return 0;
        }
        let num = self.model_base().read().theentities.find_index(anentity) as i32;
        if num > 0 {
            return num;
        }
        let rep = anentity.cast::<InterfaceReportEntity>();
        rep.get().map_or(0, |r| self.number(&r.concerned()))
    }
    /// `Value(num)`: `Standard_OutOfRange` out of 1..NbEntities.
    fn value(&self, num: i32) -> Result<Handle<StandardTransient>, Failure> {
        let state = self.model_base().read();
        let index =
            usize::try_from(num).map_err(|_| out_of_entities(num, state.theentities.len()))?;
        state.theentities.find_key(index).cloned()
    }
    /// `NbTypes(ent)`: from the protocol, 1 without protocol.
    fn nb_types(&self, ent: &Handle<StandardTransient>) -> i32 {
        let protocol = self.protocol();
        protocol.get().map_or(1, |p| p.nb_types(ent))
    }
    /// `Type(ent, num = 1)`: from the protocol, else the dynamic type (`Standard_Transient`
    /// for a null entity, which the native code dereferences).
    fn entity_type(&self, ent: &Handle<StandardTransient>, num: i32) -> &'static RuntimeType {
        let protocol = self.protocol();
        match protocol.get() {
            Some(live) => live.entity_type(ent, num),
            None => ent.dynamic_type().unwrap_or_else(
                <StandardTransient as rustycad_tkernel::standard::RuntimeTyped>::static_type,
            ),
        }
    }
    /// `TypeName(ent, complete = true)`: the signature of the GTool, else the type name
    /// (without its package when not complete).
    fn type_name(&self, ent: &Handle<StandardTransient>, complete: bool) -> String {
        let gtool = self.gtool();
        if let Some(live) = gtool.get() {
            return live.sign_value(ent, &self.model_base().this());
        }
        let Some(kind) = ent.dynamic_type() else {
            return String::new();
        };
        let name = kind.name();
        if complete {
            name.to_owned()
        } else {
            InterfaceSignTypeBase::class_name(name).to_owned()
        }
    }
    /// `AddWithRefs(anent, proto, level = 0, listall = false)`: adds `anent` and the
    /// entities it shares and implies, through the library of `proto`; sets the protocol
    /// when the model has none.
    fn add_with_refs_protocol(
        &self,
        anent: &Handle<StandardTransient>,
        proto: &Handle<dyn InterfaceProtocol>,
        level: i32,
        listall: bool,
    ) -> Result<(), Failure> {
        if anent.is_null()
            || (self.model_base().read().theentities.find_index(anent) != 0 && !listall)
        {
            return Ok(());
        }
        let lib = InterfaceGeneralLib::new(proto);
        self.add_with_refs_lib(anent, &lib, level, listall)?;
        if self.protocol().is_null() && proto.is_some() {
            self.set_protocol(proto);
        }
        Ok(())
    }
    /// `AddWithRefs(anent, level = 0, listall = false)`: with the protocol of the model;
    /// `Interface_InterfaceMismatch` without protocol.
    fn add_with_refs(
        &self,
        anent: &Handle<StandardTransient>,
        level: i32,
        listall: bool,
    ) -> Result<(), Failure> {
        let proto = self.protocol();
        if proto.is_null() {
            return Err(interface_mismatch(
                "InterfaceModel : AddWithRefs: the model has no protocol; expected one",
            ));
        }
        self.add_with_refs_protocol(anent, &proto, level, listall)
    }
    /// `AddWithRefs(anent, lib, level = 0, listall = false)`: level 0 adds all levels.
    fn add_with_refs_lib(
        &self,
        anent: &Handle<StandardTransient>,
        lib: &InterfaceGeneralLib,
        level: i32,
        listall: bool,
    ) -> Result<(), Failure> {
        if anent.is_null() {
            return Ok(());
        }
        if self.model_base().read().theentities.find_index(anent) != 0 {
            if !listall {
                return Ok(());
            }
        } else {
            self.add_entity(anent)?;
        }
        let mut iter = InterfaceEntityIterator::new();
        if let Some((module, cn)) = lib.select(anent) {
            module.fill_shared_case(cn, anent, &mut iter);
            module.list_implied_case(cn, anent, &mut iter);
        }
        let lev1 = level - 1;
        if lev1 == 0 {
            return Ok(()); // level = 0 -> all levels; otherwise still n-1
        }
        for entity in iter.content() {
            self.add_with_refs_lib(&entity, lib, lev1, listall)?;
        }
        Ok(())
    }
    /// `ReplaceEntity(nument, anent)`: `Standard_OutOfRange` out of range.
    fn replace_entity(
        &self,
        nument: i32,
        anent: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        let mut state = self.model_base().write();
        let count = state.theentities.len();
        let index = usize::try_from(nument).map_err(|_| out_of_entities(nument, count))?;
        state.theentities.substitute(index, anent.clone())
    }
    /// `ReverseOrders(after = 0)`: reverses the entities after rank `after`, with their
    /// syntactic reports.
    fn reverse_orders(&self, after: i32) -> Result<(), Failure> {
        let nb = self.nb_entities();
        if nb < 2 || after >= nb {
            return Ok(());
        }
        let ents = self.ordered_entities()?;
        self.refill_entities(
            nb,
            (1..=after)
                .chain((after + 1..=nb).rev())
                .map(|i| &ents[(i - 1) as usize]),
        );
        let mut state = self.model_base().write();
        for i in (after + 1..=nb).rev() {
            let i2 = nb + after - i;
            let rep1 = state.thereports.seek(&i).cloned();
            let rep2 = state.thereports.seek(&i2).cloned();
            rebind_report(&mut state.thereports, i2, rep1);
            rebind_report(&mut state.thereports, i, rep2);
        }
        Ok(())
    }
    /// `ChangeOrder(oldnum, newnum, count = 1)`: moves `count` entities from `oldnum` to
    /// `newnum`; `Interface_InterfaceMismatch` when the ranges overlap. Native quirk kept:
    /// the reports are exchanged with a wrong temporary (the report at `i + difnum` is
    /// moved onto itself and the one at `i` is removed).
    fn change_order(&self, oldnum: i32, newnum: i32, count: i32) -> Result<(), Failure> {
        let nb = self.nb_entities();
        if nb < 2 || newnum >= nb || count <= 0 {
            return Ok(());
        }
        let (minum, mxnum) = (oldnum.min(newnum), oldnum.max(newnum));
        if count > mxnum - minum {
            return Err(interface_mismatch(&format!(
                "InterfaceModel : ChangeOrder, Overlap: {count} entities from {oldnum} to {newnum}"
            )));
        }
        let order = changed_order(nb, oldnum, newnum, count, &self.ordered_entities()?)?;
        self.refill_entities(nb, order.iter());
        let difnum = mxnum - minum;
        let mut state = self.model_base().write();
        for i in minum..minum + count {
            let mut rep1 = state.thereports.seek(&i).cloned();
            if let Some(found) = state.thereports.seek(&(i + difnum)) {
                rep1 = Some(found.clone());
            }
            rebind_report(&mut state.thereports, i + difnum, rep1);
            rebind_report(&mut state.thereports, i, None);
        }
        Ok(())
    }
    /// `GetFromTransfer(aniter)`: the entities of `aniter` replace those of the model.
    fn get_from_transfer(&self, aniter: &InterfaceEntityIterator) -> Result<(), Failure> {
        {
            let mut state = self.model_base().write();
            state.theentities.clear(false);
            state
                .theentities
                .resize(aniter.nb_entities().max(0) as usize);
        }
        aniter
            .content()
            .iter()
            .try_for_each(|ent| self.add_entity(ent))
    }
    /// `SetCategoryNumber(num, val)`: false out of 1..NbEntities.
    fn set_category_number(&self, num: i32, val: i32) -> Result<bool, Failure> {
        let nb = self.nb_entities();
        if num < 1 || num > nb {
            return Ok(false);
        }
        let mut state = self.model_base().write();
        let current = state.thecategory.clone();
        if current.is_null() || current.length() < nb {
            let grown = Handle::new(HAsciiString::filled(nb as usize, b' '));
            for i in (1..=current.get().map_or(0, HAsciiString::length)).rev() {
                grown.set_value_char(i, current.value(i)?)?;
            }
            state.thecategory = grown;
        }
        // The category is stored as the character `val + 32`, as natively.
        state.thecategory.set_value_char(num, (val + 32) as u8)?;
        Ok(true)
    }
    /// `CategoryNumber(num)`: 0 when not set.
    fn category_number(&self, num: i32) -> i32 {
        let category = self.model_base().read().thecategory.clone();
        let Some(text) = category.get() else { return 0 };
        if num < 1 || num > text.length() {
            return 0;
        }
        text.value(num).map_or(0, |c| i32::from(c) - 32)
    }
    /// `FillIterator(iter)`: the entities in order.
    fn fill_iterator(&self, iter: &mut InterfaceEntityIterator) {
        let entities: Vec<Handle<StandardTransient>> = self
            .model_base()
            .read()
            .theentities
            .iter()
            .cloned()
            .collect();
        entities.iter().for_each(|ent| iter.get_one_item(ent));
    }
    /// `Entities`.
    fn entities(&self) -> InterfaceEntityIterator {
        let mut iter = InterfaceEntityIterator::new();
        self.fill_iterator(&mut iter);
        iter
    }
    /// `Print(ent, S, mode = 0)`: the number (mode <= 0), ":" (mode 0), and the label
    /// (mode >= 0, "??" for an entity out of the model); "NULL" for null.
    fn print(
        &self,
        ent: &Handle<StandardTransient>,
        s: &mut dyn Write,
        mode: i32,
    ) -> io::Result<()> {
        if ent.is_null() {
            return s.write_all(b"NULL");
        }
        let num = self.number(ent);
        if mode <= 0 {
            write!(s, "{num}")?;
        }
        if mode == 0 {
            s.write_all(b":")?;
        }
        if mode < 0 {
            return Ok(());
        }
        if num > 0 {
            self.print_to_log(ent, s)
        } else {
            s.write_all(b"??")
        }
    }
    /// The entities in order, as the native `NCollection_Array1` copy.
    fn ordered_entities(&self) -> Result<Vec<Handle<StandardTransient>>, Failure> {
        Ok(self
            .model_base()
            .read()
            .theentities
            .iter()
            .cloned()
            .collect())
    }
    /// Clears the entities, reserves `nb` places and adds `order`.
    fn refill_entities<'a>(
        &self,
        nb: i32,
        order: impl Iterator<Item = &'a Handle<StandardTransient>>,
    ) {
        self.model_base().write().theentities.clear(false);
        self.reservate(nb);
        let mut state = self.model_base().write();
        order.for_each(|ent| {
            state.theentities.add(ent.clone());
        });
    }
}

impl<T: InterfaceInterfaceModel + ?Sized> InterfaceModelEntities for T {}

/// The `Standard_OutOfRange` of `FindKey` on the entities.
fn out_of_entities(num: i32, count: usize) -> Failure {
    exceptions::out_of_range(&format!(
        "Interface_InterfaceModel: entity {num}; expected [1, {count}]"
    ))
}

/// Binds `rep` at `num`, or unbinds `num` when there is none.
fn rebind_report(
    reports: &mut rustycad_tkernel::ncollection::DataMap<i32, Handle<StandardTransient>>,
    num: i32,
    rep: Option<Handle<StandardTransient>>,
) {
    match rep.filter(|r| r.is_some()) {
        Some(found) => {
            reports.bind(num, found);
        }
        None => {
            reports.unbind(&num);
        }
    }
}

/// The new order of `ChangeOrder`, filled as the native `ents` array (1-based).
fn changed_order(
    nb: i32,
    oldnum: i32,
    newnum: i32,
    count: i32,
    current: &[Handle<StandardTransient>],
) -> Result<Vec<Handle<StandardTransient>>, Failure> {
    let (minum, mxnum) = (oldnum.min(newnum), oldnum.max(newnum));
    let kount = if oldnum > newnum { count } else { -count };
    let mut ents: Vec<Handle<StandardTransient>> = vec![Handle::default(); nb as usize];
    let mut set = |at: i32, from: i32| -> Result<(), Failure> {
        let source = current
            .get((from - 1) as usize)
            .cloned()
            .ok_or_else(|| out_of_entities(from, current.len()))?;
        let slot = usize::try_from(at - 1).ok().and_then(|i| ents.get_mut(i));
        *slot.ok_or_else(|| out_of_entities(at, nb as usize))? = source;
        Ok(())
    };
    (1..minum).try_for_each(|i| set(i, i))?;
    (mxnum + count..=nb).try_for_each(|i| set(i, i))?;
    (minum..mxnum).try_for_each(|i| set(i + kount, i))?;
    (oldnum..oldnum + count).try_for_each(|i| set(i + (newnum - oldnum), i))?;
    Ok(ents)
}
