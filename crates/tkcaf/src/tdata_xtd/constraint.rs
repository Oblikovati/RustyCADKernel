// Rust port of ApplicationFramework/TKCAF/TDataXtd/TDataXtd_Constraint.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataXtd_Constraint`: a geometric constraint between up to four named shapes, with an
//! optional plane and dimension value.
use super::{TDataXtd, TDataXtdConstraintEnum};
use crate::tnaming::TNamingNamedShape;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, exceptions},
};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdata_std::TDataStdReal;
use rustycad_tklcaf::tdata_std::support::{cast_attribute, find_or_add};
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfChildIterator, TdfDataSet, TdfLabel,
    TdfRelocationTable,
};
use std::io;

/// `TDataXtd_Constraint::GetID()`.
const CONSTRAINT_ID: Guid = guid_from_text("2a96b602-ec8b-11d0-bee7-080009dc3333");

/// The fields of `TDataXtd_Constraint`.
#[derive(Clone)]
struct ConstraintState {
    constraint_type: TDataXtdConstraintEnum,
    value: Handle<TDataStdReal>,
    geometries: [Handle<TNamingNamedShape>; 4],
    plane: Handle<TNamingNamedShape>,
    reversed: bool,
    inverted: bool,
    verified: bool,
}

/// `TDataXtd_Constraint`.
///
/// ```
/// use rustycad_tkcaf::tdata_xtd::{TDataXtdConstraint, TDataXtdConstraintEnum};
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// let constraint = TDataXtdConstraint::set(&data.root()).unwrap();
/// assert_eq!(constraint.get_type(), TDataXtdConstraintEnum::Radius);
/// assert!(constraint.verified() && constraint.nb_geometries() == 0);
/// ```
pub struct TDataXtdConstraint {
    transient: StandardTransient,
    base: TdfAttributeBase,
    state: SharedState<ConstraintState>,
}
impl_transient!(TDataXtdConstraint, "TDataXtd_Constraint", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataXtdConstraint {
    fn default() -> Self {
        Self::new()
    }
}

/// `aShape->Get() == G->Get()` for two non-null named shapes (`IsEqual`).
fn same_shapes(
    first: &Handle<TNamingNamedShape>,
    second: &Handle<TNamingNamedShape>,
) -> Result<bool, Failure> {
    match (first.get(), second.get()) {
        (Some(first), Some(second)) => Ok(first.get()? == second.get()?),
        _ => Ok(false),
    }
}

impl TDataXtdConstraint {
    /// `GetID`.
    pub fn get_id() -> Guid {
        CONSTRAINT_ID
    }
    /// `Set(label)`: finds or adds the constraint.
    pub fn set(label: &TdfLabel) -> Result<Handle<TDataXtdConstraint>, Failure> {
        find_or_add(label, &Self::get_id(), false, Self::new)
    }
    /// `TDataXtd_Constraint()`: a verified `RADIUS` constraint without geometry.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            state: SharedState::new(ConstraintState {
                constraint_type: TDataXtdConstraintEnum::Radius,
                value: Handle::default(),
                geometries: Default::default(),
                plane: Handle::default(),
                reversed: false,
                inverted: false,
                verified: true,
            }),
        }
    }
    /// The four `Set(type, G1, ...)`: the type and the first geometries, unless the type
    /// and the shapes of the given geometries are unchanged (OCC2932). With four geometries
    /// the native check compares the third stored shape with the fourth given one.
    pub fn set_geometries(
        &self,
        constraint_type: TDataXtdConstraintEnum,
        geometries: &[Handle<TNamingNamedShape>],
    ) -> Result<(), Failure> {
        if geometries.is_empty() || geometries.len() > 4 {
            return Err(exceptions::out_of_range(&format!(
                "TDataXtd_Constraint::Set: {} geometries; expected 1 to 4",
                geometries.len()
            )));
        }
        let state = self.state.read().clone();
        if state.constraint_type == constraint_type && self.unchanged(&state, geometries)? {
            return Ok(());
        }
        self.backup()?;
        let mut state = self.state.write();
        state.constraint_type = constraint_type;
        state.geometries[..geometries.len()].clone_from_slice(geometries);
        Ok(())
    }
    /// The OCC2932 test of `Set`: every stored and given geometry is not null and has the
    /// same shape (the native fourth test reads the third stored geometry).
    fn unchanged(
        &self,
        state: &ConstraintState,
        geometries: &[Handle<TNamingNamedShape>],
    ) -> Result<bool, Failure> {
        for (index, given) in geometries.iter().enumerate() {
            let stored = if index == 3 {
                &state.geometries[2]
            } else {
                &state.geometries[index]
            };
            if state.geometries[index].is_null() || !same_shapes(stored, given)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// `Verified()`.
    pub fn verified(&self) -> bool {
        self.state.read().verified
    }
    /// `GetType`.
    pub fn get_type(&self) -> TDataXtdConstraintEnum {
        self.state.read().constraint_type
    }
    /// `IsPlanar`: there is a plane.
    pub fn is_planar(&self) -> bool {
        self.state.read().plane.is_some()
    }
    /// `GetPlane`.
    pub fn get_plane(&self) -> Handle<TNamingNamedShape> {
        self.state.read().plane.clone()
    }
    /// `IsDimension`: there is a value.
    pub fn is_dimension(&self) -> bool {
        self.state.read().value.is_some()
    }
    /// `GetValue`.
    pub fn get_value(&self) -> Handle<TDataStdReal> {
        self.state.read().value.clone()
    }
    /// `NbGeometries`: the geometries before the first null one.
    pub fn nb_geometries(&self) -> i32 {
        let count = self
            .state
            .read()
            .geometries
            .iter()
            .take_while(|g| g.is_some())
            .count();
        i32::try_from(count).unwrap_or(4)
    }
    /// `GetGeometry(Index)` (1 to 4; `Standard_OutOfRange` otherwise, unchecked natively).
    pub fn get_geometry(&self, index: i32) -> Result<Handle<TNamingNamedShape>, Failure> {
        Ok(self.state.read().geometries[slot(index)?].clone())
    }
    /// `ClearGeometries`: backed up unless they are all null.
    pub fn clear_geometries(&self) -> Result<(), Failure> {
        if self.state.read().geometries.iter().all(Handle::is_null) {
            return Ok(());
        }
        self.backup()?;
        self.state.write().geometries = Default::default();
        Ok(())
    }
    /// `SetType(CTR)`: backed up when it changes.
    pub fn set_type(&self, constraint_type: TDataXtdConstraintEnum) -> Result<(), Failure> {
        if self.get_type() == constraint_type {
            return Ok(());
        }
        self.backup()?;
        self.state.write().constraint_type = constraint_type;
        Ok(())
    }
    /// `SetPlane(plane)`: backed up unless both planes have the same shape.
    pub fn set_plane(&self, plane: &Handle<TNamingNamedShape>) -> Result<(), Failure> {
        if same_shapes(&self.get_plane(), plane)? {
            return Ok(());
        }
        self.backup()?;
        self.state.write().plane = plane.clone();
        Ok(())
    }
    /// `SetValue(V)`: backed up unless both values are equal.
    pub fn set_value(&self, value: &Handle<TDataStdReal>) -> Result<(), Failure> {
        let current = self.get_value();
        if let (Some(current), Some(new)) = (current.get(), value.get())
            && current.get() == new.get()
        {
            return Ok(());
        }
        self.backup()?;
        self.state.write().value = value.clone();
        Ok(())
    }
    /// `SetGeometry(Index, G)`: backed up unless both geometries have the same shape.
    pub fn set_geometry(
        &self,
        index: i32,
        geometry: &Handle<TNamingNamedShape>,
    ) -> Result<(), Failure> {
        let position = slot(index)?;
        if same_shapes(&self.state.read().geometries[position].clone(), geometry)? {
            return Ok(());
        }
        self.backup()?;
        self.state.write().geometries[position] = geometry.clone();
        Ok(())
    }
    /// `Verified(status)`: backed up when it changes.
    pub fn set_verified(&self, status: bool) -> Result<(), Failure> {
        self.set_flag(status, |state| &mut state.verified)
    }
    /// `Inverted(status)`: backed up when it changes.
    pub fn set_inverted(&self, status: bool) -> Result<(), Failure> {
        self.set_flag(status, |state| &mut state.inverted)
    }
    /// `Inverted()`.
    pub fn inverted(&self) -> bool {
        self.state.read().inverted
    }
    /// `Reversed(status)`: backed up when it changes.
    pub fn set_reversed(&self, status: bool) -> Result<(), Failure> {
        self.set_flag(status, |state| &mut state.reversed)
    }
    /// `Reversed()`.
    pub fn reversed(&self) -> bool {
        self.state.read().reversed
    }
    fn set_flag(
        &self,
        status: bool,
        flag: fn(&mut ConstraintState) -> &mut bool,
    ) -> Result<(), Failure> {
        if *flag(&mut self.state.read().clone()) == status {
            return Ok(());
        }
        self.backup()?;
        *flag(&mut self.state.write()) = status;
        Ok(())
    }
    /// `CollectChildConstraints(aLabel, TheList)`: every descendant with a constraint.
    pub fn collect_child_constraints(
        label: &TdfLabel,
        list: &mut Vec<TdfLabel>,
    ) -> Result<(), Failure> {
        let mut children = TdfChildIterator::new(label, true)?;
        while children.more() {
            let child = children.value();
            if child.is_attribute(&Self::get_id())? {
                list.push(child);
            }
            children.next();
        }
        Ok(())
    }
}

/// The array slot of a 1-based geometry index.
fn slot(index: i32) -> Result<usize, Failure> {
    usize::try_from(index - 1)
        .ok()
        .filter(|&slot| slot < 4)
        .ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "TDataXtd_Constraint geometry index {index}; expected 1 to 4"
            ))
        })
}

impl TdfAttribute for TDataXtdConstraint {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        CONSTRAINT_ID
    }
    fn restore(&self, attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other =
            cast_attribute::<TDataXtdConstraint>(attribute, "TDataXtd_Constraint::Restore")?;
        let state = rustycad_tklcaf::live(&other, "TDataXtd_Constraint::Restore")?
            .state
            .read()
            .clone();
        *self.state.write() = state;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: the relocated geometries, plane and value, then the type and flags.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TDataXtdConstraint>(into, "TDataXtd_Constraint::Paste")?;
        let target = rustycad_tklcaf::live(&target, "TDataXtd_Constraint::Paste")?;
        let table = rustycad_tklcaf::live(table, "TDataXtd_Constraint::Paste")?;
        let state = self.state.read().clone();
        for (index, geometry) in (1..).zip(&state.geometries) {
            target.set_geometry(index, &relocated(table, geometry))?;
        }
        target.set_plane(&relocated(table, &state.plane))?;
        target.set_value(&relocated(table, &state.value))?;
        target.set_type(state.constraint_type)?;
        target.set_verified(state.verified)?;
        target.set_inverted(state.inverted)?;
        target.set_reversed(state.reversed)
    }
    /// `References`: the geometries (two for a placement mate to faces-angle constraint,
    /// four otherwise), the value and the plane.
    fn references(&self, data_set: &Handle<TdfDataSet>) {
        let Some(data_set) = data_set.get() else {
            return;
        };
        let state = self.state.read().clone();
        let kind = state.constraint_type as i32;
        let placement = kind >= TDataXtdConstraintEnum::Mate as i32
            && kind <= TDataXtdConstraintEnum::FacesAngle as i32;
        let limit = if placement { 2 } else { 4 };
        for geometry in state.geometries.iter().take(limit).filter(|g| g.is_some()) {
            data_set.add_attribute(&geometry.cast());
        }
        if state.value.is_some() {
            data_set.add_attribute(&state.value.cast());
        }
        if state.plane.is_some() {
            data_set.add_attribute(&state.plane.cast());
        }
    }
    /// `Dump`: `Constraint ` and the type name.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Constraint ")?;
        TDataXtd::print_constraint(self.get_type(), stream)
    }
}

/// `RT->HasRelocation(source, target)` into a new null handle.
fn relocated<T: rustycad_tkernel::standard::RuntimeTyped>(
    table: &TdfRelocationTable,
    source: &Handle<T>,
) -> Handle<T>
where
    Handle<T>: Clone,
{
    let mut target: Handle<T> = Handle::default();
    table.has_attribute_relocation_as(&source.cast(), &mut target);
    target
}
