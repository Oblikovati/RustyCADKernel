// Rust port of ApplicationFramework/TKCAF/TDataXtd/TDataXtd_Pattern.hxx/.cxx and
// TDataXtd_PatternStd.hxx/.cxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataXtd_Pattern` and `TDataXtd_PatternStd`: transformation patterns (linear, circular,
//! rectangular, radial and mirror).
use super::TDataXtdGeometry;
use crate::tnaming::TNamingNamedShape;
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    standard::{Failure, Guid, Handle, RuntimeTyped, SharedState, StandardTransient, exceptions},
};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdata_std::support::{cast_attribute, find_or_add};
use rustycad_tklcaf::tdata_std::{TDataStdInteger, TDataStdReal};
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfDataSet, TdfLabel, TdfRelocationTable,
};
use rustycad_tkmath::gp::{Axis1, Plane3, Transform3, Vector3};
use std::io;

/// `TDataXtd_Pattern::GetID()`, also the `ID` of every pattern.
const PATTERN_ID: Guid = guid_from_text("2a96b618-ec8b-11d0-bee7-080009dc3333");
/// `TDataXtd_PatternStd::GetPatternID()`.
const PATTERN_STD_ID: Guid = guid_from_text("2a96b61b-ec8b-11d0-bee7-080009dc3333");

/// `TDataXtd_Pattern`: an attribute computing a set of transformations.
pub trait TDataXtdPattern: TdfAttribute {
    /// `PatternID`: the kind of pattern.
    fn pattern_id(&self) -> Guid;
    /// `NbTrsfs`: the number of transformations.
    fn nb_trsfs(&self) -> i32;
    /// `ComputeTrsfs(Trsfs)`: fills `NbTrsfs` transformations from the start of `trsfs`.
    fn compute_trsfs(&self, trsfs: &mut [Transform3]) -> Result<(), Failure>;
}
impl_transient_interface!(dyn TDataXtdPattern, "TDataXtd_Pattern", dyn TdfAttribute);

impl dyn TDataXtdPattern {
    /// `TDataXtd_Pattern::GetID`.
    pub fn get_id() -> Guid {
        PATTERN_ID
    }
}

/// The fields of `TDataXtd_PatternStd`.
#[derive(Clone, Default)]
struct PatternStdState {
    signature: i32,
    axis1_reversed: bool,
    axis2_reversed: bool,
    axis1: Handle<TNamingNamedShape>,
    axis2: Handle<TNamingNamedShape>,
    value1: Handle<TDataStdReal>,
    value2: Handle<TDataStdReal>,
    nb1: Handle<TDataStdInteger>,
    nb2: Handle<TDataStdInteger>,
    mirror: Handle<TNamingNamedShape>,
}

/// `TDataXtd_PatternStd`: signature 1 linear, 2 circular, 3 rectangular, 4 radial (rotation
/// then rotation), 5 mirror.
///
/// ```
/// use rustycad_tkcaf::tdata_xtd::{TDataXtdPattern, TDataXtdPatternStd};
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// let pattern = TDataXtdPatternStd::set(&data.root()).unwrap();
/// pattern.set_signature(5).unwrap();
/// assert_eq!((pattern.signature(), pattern.nb_trsfs()), (5, 1));
/// ```
pub struct TDataXtdPatternStd {
    transient: StandardTransient,
    base: TdfAttributeBase,
    state: SharedState<PatternStdState>,
}
impl_transient!(TDataXtdPatternStd, "TDataXtd_PatternStd", dyn TDataXtdPattern, transient,
    interfaces: [dyn TdfAttribute, dyn TDataXtdPattern]);

impl Default for TDataXtdPatternStd {
    fn default() -> Self {
        Self::new()
    }
}

/// `old->Get() == new->Get()` for the OCC2932 tests (natively a null new handle is a
/// null dereference; the port treats it as a change).
fn same_value<T: RuntimeTyped + ?Sized, V: PartialEq>(
    old: &Handle<T>,
    new: &Handle<T>,
    value: impl Fn(&T) -> Result<V, Failure>,
) -> Result<bool, Failure> {
    match (old.get(), new.get()) {
        (Some(old), Some(new)) => Ok(value(old)? == value(new)?),
        _ => Ok(false),
    }
}

/// The shape of a named shape, for [`same_value`].
fn named_shape(named: &TNamingNamedShape) -> Result<rustycad_tkbrep::topods::Shape, Failure> {
    named.get()
}

impl TDataXtdPatternStd {
    /// `GetPatternID`.
    pub fn get_pattern_id() -> Guid {
        PATTERN_STD_ID
    }
    /// `GetID` (inherited from `TDataXtd_Pattern`).
    pub fn get_id() -> Guid {
        PATTERN_ID
    }
    /// `Set(label)`: finds the label's pattern or adds a standard one.
    pub fn set(label: &TdfLabel) -> Result<Handle<TDataXtdPatternStd>, Failure> {
        find_or_add(label, &Self::get_id(), false, Self::new)
    }
    /// `TDataXtd_PatternStd()`: signature 0, nothing set.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            state: SharedState::new(PatternStdState::default()),
        }
    }
    fn read(&self) -> PatternStdState {
        self.state.read().clone()
    }
    /// Backs up and changes the state unless `unchanged` (the OCC2932 corrections).
    fn change(
        &self,
        unchanged: bool,
        apply: impl FnOnce(&mut PatternStdState),
    ) -> Result<(), Failure> {
        if unchanged {
            return Ok(());
        }
        self.backup()?;
        apply(&mut self.state.write());
        Ok(())
    }
    /// `Signature(signature)`.
    pub fn set_signature(&self, signature: i32) -> Result<(), Failure> {
        self.change(self.signature() == signature, |s| s.signature = signature)
    }
    /// `Axis1(Axis1)`.
    pub fn set_axis1(&self, axis: &Handle<TNamingNamedShape>) -> Result<(), Failure> {
        let unchanged = same_value(&self.axis1(), axis, named_shape)?;
        self.change(unchanged, |s| s.axis1 = axis.clone())
    }
    /// `Axis2(Axis2)`.
    pub fn set_axis2(&self, axis: &Handle<TNamingNamedShape>) -> Result<(), Failure> {
        let unchanged = same_value(&self.axis2(), axis, named_shape)?;
        self.change(unchanged, |s| s.axis2 = axis.clone())
    }
    /// `Axis1Reversed(Axis1Reversed)`.
    pub fn set_axis1_reversed(&self, reversed: bool) -> Result<(), Failure> {
        self.change(self.axis1_reversed() == reversed, |s| {
            s.axis1_reversed = reversed
        })
    }
    /// `Axis2Reversed(Axis2Reversed)`.
    pub fn set_axis2_reversed(&self, reversed: bool) -> Result<(), Failure> {
        self.change(self.axis2_reversed() == reversed, |s| {
            s.axis2_reversed = reversed
        })
    }
    /// `Value1(value)`.
    pub fn set_value1(&self, value: &Handle<TDataStdReal>) -> Result<(), Failure> {
        let unchanged = same_value(&self.value1(), value, |real| Ok(real.get()))?;
        self.change(unchanged, |s| s.value1 = value.clone())
    }
    /// `Value2(value)`.
    pub fn set_value2(&self, value: &Handle<TDataStdReal>) -> Result<(), Failure> {
        let unchanged = same_value(&self.value2(), value, |real| Ok(real.get()))?;
        self.change(unchanged, |s| s.value2 = value.clone())
    }
    /// `NbInstances1(NbInstances1)`.
    pub fn set_nb_instances1(&self, count: &Handle<TDataStdInteger>) -> Result<(), Failure> {
        let unchanged = same_value(&self.nb_instances1(), count, |integer| Ok(integer.get()))?;
        self.change(unchanged, |s| s.nb1 = count.clone())
    }
    /// `NbInstances2(NbInstances2)`.
    pub fn set_nb_instances2(&self, count: &Handle<TDataStdInteger>) -> Result<(), Failure> {
        let unchanged = same_value(&self.nb_instances2(), count, |integer| Ok(integer.get()))?;
        self.change(unchanged, |s| s.nb2 = count.clone())
    }
    /// `Mirror(plane)`.
    pub fn set_mirror(&self, plane: &Handle<TNamingNamedShape>) -> Result<(), Failure> {
        let unchanged = same_value(&self.mirror(), plane, named_shape)?;
        self.change(unchanged, |s| s.mirror = plane.clone())
    }
    /// `Signature()`.
    pub fn signature(&self) -> i32 {
        self.state.read().signature
    }
    /// `Axis1()`.
    pub fn axis1(&self) -> Handle<TNamingNamedShape> {
        self.state.read().axis1.clone()
    }
    /// `Axis2()`.
    pub fn axis2(&self) -> Handle<TNamingNamedShape> {
        self.state.read().axis2.clone()
    }
    /// `Axis1Reversed()`.
    pub fn axis1_reversed(&self) -> bool {
        self.state.read().axis1_reversed
    }
    /// `Axis2Reversed()`.
    pub fn axis2_reversed(&self) -> bool {
        self.state.read().axis2_reversed
    }
    /// `Value1()`.
    pub fn value1(&self) -> Handle<TDataStdReal> {
        self.state.read().value1.clone()
    }
    /// `Value2()`.
    pub fn value2(&self) -> Handle<TDataStdReal> {
        self.state.read().value2.clone()
    }
    /// `NbInstances1()`.
    pub fn nb_instances1(&self) -> Handle<TDataStdInteger> {
        self.state.read().nb1.clone()
    }
    /// `NbInstances2()`.
    pub fn nb_instances2(&self) -> Handle<TDataStdInteger> {
        self.state.read().nb2.clone()
    }
    /// `Mirror()`.
    pub fn mirror(&self) -> Handle<TNamingNamedShape> {
        self.state.read().mirror.clone()
    }
}

/// `TDataXtd_Geometry::Axis(named, axis)` reversed on request; the axis stays the default
/// one when the named shape is not an axis (as natively).
fn pattern_axis(named: &Handle<TNamingNamedShape>, reversed: bool) -> Result<Axis1, Failure> {
    let mut axis = Axis1::default();
    TDataXtdGeometry::axis(named, &mut axis)?;
    if reversed {
        axis.reverse();
    }
    Ok(axis)
}

/// A translation along `axis` or a rotation about it, by `step * (index - 1)`.
fn step_transform(
    axis: Axis1,
    rotation: bool,
    step: f64,
    index: i32,
) -> Result<Transform3, Failure> {
    let mut transform = Transform3::default();
    let amount = step * f64::from(index - 1);
    if rotation {
        transform.set_rotation(axis, amount)?;
    } else {
        transform.set_translation(Vector3::from(axis.direction()).scaled(amount));
    }
    Ok(transform)
}

/// The value of a required handle (`Standard_NullObject`; natively a null dereference).
fn required<T: RuntimeTyped + ?Sized, V>(
    handle: &Handle<T>,
    what: &str,
    value: impl Fn(&T) -> V,
) -> Result<V, Failure> {
    handle.get().map(value).ok_or_else(|| {
        exceptions::null_object(&format!(
            "TDataXtd_PatternStd::ComputeTrsfs: null {what}; expected a set attribute"
        ))
    })
}

/// Stores `transform` at the next 1-based position (`Trsfs(++nb)`).
fn push(trsfs: &mut [Transform3], count: &mut usize, transform: Transform3) -> Result<(), Failure> {
    let length = trsfs.len();
    let slot = trsfs.get_mut(*count).ok_or_else(|| {
        exceptions::out_of_range(&format!(
            "TDataXtd_PatternStd::ComputeTrsfs: transformation {} of an array of {length}",
            *count + 1
        ))
    })?;
    *slot = transform;
    *count += 1;
    Ok(())
}

impl TDataXtdPatternStd {
    /// The first direction of `ComputeTrsfs`: `NbInstances1 - 1` steps along/about axis 1.
    fn first_direction(
        &self,
        state: &PatternStdState,
        trsfs: &mut [Transform3],
        count: &mut usize,
    ) -> Result<(), Failure> {
        let axis = pattern_axis(&state.axis1, state.axis1_reversed)?;
        let step = required(&state.value1, "Value1", TDataStdReal::get)?;
        for index in 2..=required(&state.nb1, "NbInstances1", TDataStdInteger::get)? {
            push(
                trsfs,
                count,
                step_transform(axis, state.signature == 2, step, index)?,
            )?;
        }
        Ok(())
    }
    /// The second direction of `ComputeTrsfs` (signatures 3 and 4): each step along/about
    /// axis 2, then composed with each transformation of the first direction.
    fn second_direction(
        &self,
        state: &PatternStdState,
        trsfs: &mut [Transform3],
        count: &mut usize,
    ) -> Result<(), Failure> {
        let axis = pattern_axis(&state.axis2, state.axis2_reversed)?;
        let step = required(&state.value2, "Value2", TDataStdReal::get)?;
        let first = required(&state.nb1, "NbInstances1", TDataStdInteger::get)?;
        for index in 2..=required(&state.nb2, "NbInstances2", TDataStdInteger::get)? {
            let second = step_transform(axis, state.signature == 4, step, index)?;
            push(trsfs, count, second)?;
            for previous in 0..usize::try_from(first - 1).unwrap_or(0) {
                push(trsfs, count, second.multiplied(trsfs[previous]))?;
            }
        }
        Ok(())
    }
    /// The mirror of `ComputeTrsfs` (signature 5 and more): the symmetry about the plane.
    fn mirror_transform(
        &self,
        state: &PatternStdState,
        trsfs: &mut [Transform3],
        count: &mut usize,
    ) -> Result<(), Failure> {
        let mut plane = Plane3::default();
        TDataXtdGeometry::plane(&state.mirror, &mut plane)?;
        let mut transform = Transform3::default();
        transform.set_mirror_plane(plane.position().ax2()?);
        push(trsfs, count, transform)
    }
}

impl TDataXtdPattern for TDataXtdPatternStd {
    fn pattern_id(&self) -> Guid {
        PATTERN_STD_ID
    }
    /// `NbTrsfs`: `NbInstances1 * NbInstances2 - 1` (missing counts are 1), or 1 for a mirror.
    fn nb_trsfs(&self) -> i32 {
        let state = self.read();
        if state.signature >= 5 {
            return 1;
        }
        let first = state.nb1.get().map_or(1, TDataStdInteger::get);
        first * state.nb2.get().map_or(1, TDataStdInteger::get) - 1
    }
    fn compute_trsfs(&self, trsfs: &mut [Transform3]) -> Result<(), Failure> {
        let state = self.read();
        let mut count = 0;
        if state.signature >= 5 {
            return self.mirror_transform(&state, trsfs, &mut count);
        }
        self.first_direction(&state, trsfs, &mut count)?;
        if state.signature == 3 || state.signature == 4 {
            self.second_direction(&state, trsfs, &mut count)?;
        }
        Ok(())
    }
}

/// `RT->HasRelocation(source, target)` into a new null handle.
fn relocated<T: RuntimeTyped + ?Sized>(
    table: &TdfRelocationTable,
    source: &Handle<T>,
) -> Handle<T> {
    let mut target: Handle<T> = Handle::default();
    table.has_attribute_relocation_as(&source.cast(), &mut target);
    target
}

impl TDataXtdPatternStd {
    /// The relocated references of `Paste`, in the native order.
    fn paste_references(
        &self,
        state: &PatternStdState,
        target: &Self,
        table: &TdfRelocationTable,
    ) -> Result<(), Failure> {
        if state.signature >= 5 {
            return target.set_mirror(&relocated(table, &state.mirror));
        }
        target.set_axis1(&relocated(table, &state.axis1))?;
        target.set_value1(&relocated(table, &state.value1))?;
        target.set_nb_instances1(&relocated(table, &state.nb1))?;
        if state.signature <= 2 {
            return Ok(());
        }
        target.set_axis2(&relocated(table, &state.axis2))?;
        target.set_value2(&relocated(table, &state.value2))?;
        target.set_nb_instances2(&relocated(table, &state.nb2))
    }
}

impl TdfAttribute for TDataXtdPatternStd {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        PATTERN_ID
    }
    fn restore(&self, attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other =
            cast_attribute::<TDataXtdPatternStd>(attribute, "TDataXtd_PatternStd::Restore")?;
        let state = rustycad_tklcaf::live(&other, "TDataXtd_PatternStd::Restore")?.read();
        *self.state.write() = state;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: the signature and flags, then the relocated references.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TDataXtdPatternStd>(into, "TDataXtd_PatternStd::Paste")?;
        let target = rustycad_tklcaf::live(&target, "TDataXtd_PatternStd::Paste")?;
        let table = rustycad_tklcaf::live(table, "TDataXtd_PatternStd::Paste")?;
        let state = self.read();
        target.set_signature(state.signature)?;
        target.set_axis1_reversed(state.axis1_reversed)?;
        target.set_axis2_reversed(state.axis2_reversed)?;
        self.paste_references(&state, target, table)
    }
    /// `References`: the axis, value and count of each direction, or the mirror plane.
    fn references(&self, data_set: &Handle<TdfDataSet>) {
        let Some(data_set) = data_set.get() else {
            return;
        };
        let state = self.read();
        let references: Vec<Handle<dyn TdfAttribute>> = if state.signature >= 5 {
            vec![state.mirror.cast()]
        } else if state.signature > 2 {
            vec![
                state.axis1.cast(),
                state.value1.cast(),
                state.nb1.cast(),
                state.axis2.cast(),
                state.value2.cast(),
                state.nb2.cast(),
            ]
        } else {
            vec![state.axis1.cast(), state.value1.cast(), state.nb1.cast()]
        };
        references
            .iter()
            .for_each(|attribute| data_set.add_attribute(attribute));
    }
    /// `Dump`: the class name only.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "TDataXtd_PatternStd")
    }
}
