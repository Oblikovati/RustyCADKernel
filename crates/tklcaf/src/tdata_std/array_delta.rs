// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_DeltaOnModificationOfIntArray,
// TDataStd_DeltaOnModificationOfRealArray, TDataStd_DeltaOnModificationOfByteArray and
// TDataStd_DeltaOnModificationOfExtStringArray (.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The modification deltas of the array attributes (ocaf-design.md section 7). The four
//! native classes are the same code over different items: each is a struct over one
//! [`ArrayDeltaState`].
use super::array_core::{ArrayAttribute, array, new_default_array};
use super::{TDataStdByteArray, TDataStdExtStringArray, TDataStdIntegerArray, TDataStdRealArray};
use crate::live;
use crate::tdf::{
    TdfAttribute, TdfAttributeDelta, TdfAttributeDeltaBase, TdfAttributeState,
    TdfDeltaOnModification, TdfLabel,
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HArray1,
    standard::{Failure, Handle, StandardTransient},
};

/// The private fields `myIndxes`, `myValues`, `myUp1` and `myUp2`: the indexes the
/// modification changed with their old items, and the old and new upper bounds. Empty
/// lists stand for the native null arrays.
pub(crate) struct ArrayDeltaState<T> {
    indexes: Vec<i32>,
    values: Vec<T>,
    up1: i32,
    up2: i32,
}

impl<T> Default for ArrayDeltaState<T> {
    fn default() -> Self {
        Self {
            indexes: Vec::new(),
            values: Vec::new(),
            up1: 0,
            up2: 0,
        }
    }
}

/// The constructor body: compares the backup `old` with the current attribute of its ID
/// on `label`, records the differences, then drops the backup's array (`RemoveArray`).
fn record<A: ArrayAttribute>(
    old: &Handle<A>,
    label: &TdfLabel,
) -> Result<ArrayDeltaState<A::Item>, Failure> {
    let old = live(old, A::TYPE_NAME)?;
    let mut current: Handle<A> = Handle::default();
    if !label.find_attribute_as(&old.id(), &mut current)? {
        return Ok(ArrayDeltaState::default());
    }
    let (old_items, current_items) = (array(old), array(live(&current, A::TYPE_NAME)?));
    let (Some(arr1), Some(arr2)) = (old_items.get(), current_items.get()) else {
        return Ok(ArrayDeltaState::default());
    };
    let mut state = ArrayDeltaState::default();
    if old_items != current_items {
        state = differences::<A>(&arr1.array1(), &arr2.array1())?;
    }
    old.array_state().write().value = Handle::default();
    Ok(state)
}

/// The changed indexes of `arr1` (old) against `arr2` (current), with the old items; the
/// indexes past the new upper bound count as changed.
fn differences<A: ArrayAttribute>(
    arr1: &rustycad_tkernel::ncollection::Array1<'static, A::Item>,
    arr2: &rustycad_tkernel::ncollection::Array1<'static, A::Item>,
) -> Result<ArrayDeltaState<A::Item>, Failure> {
    let (up1, up2) = (arr1.upper(), arr2.upper());
    let item = |array: &rustycad_tkernel::ncollection::Array1<'static, A::Item>, index: i32| {
        array
            .value(index)
            .cloned()
            .map_err(|e| e.into_failure(A::TYPE_NAME))
    };
    let mut indexes = Vec::new();
    for index in arr1.lower()..=up1.min(up2) {
        if item(arr1, index)? != item(arr2, index)? {
            indexes.push(index);
        }
    }
    indexes.extend(up2.saturating_add(1)..=up1);
    let values = indexes
        .iter()
        .map(|&index| item(arr1, index))
        .collect::<Result<_, _>>()?;
    Ok(ArrayDeltaState {
        indexes,
        values,
        up1,
        up2,
    })
}

/// `Apply`: the backup attribute is added back when its label lost the attribute;
/// otherwise the current attribute backs up and takes the old upper bound and items.
fn apply<A: ArrayAttribute>(
    state: &ArrayDeltaState<A::Item>,
    attribute: &Handle<dyn TdfAttribute>,
    label: &TdfLabel,
) -> Result<(), Failure> {
    let backup = attribute.cast::<A>();
    let Some(backup_attribute) = backup.get() else {
        return Ok(());
    };
    let mut current: Handle<A> = Handle::default();
    if !label.find_attribute_as(&backup_attribute.id(), &mut current)? {
        return label.add_attribute(&backup.cast(), false);
    }
    let current = live(&current, A::TYPE_NAME)?;
    current.backup()?;
    if state.up1 == state.up2 && state.indexes.is_empty() {
        return Ok(());
    }
    let shared = array(current);
    let Some(items) = shared.get() else {
        return Ok(());
    };
    if state.up1 == state.up2 {
        return write_old_items::<A>(state, &shared);
    }
    let copy_bound = state.up1.min(state.up2);
    let resized = resized_copy::<A>(&items.array1(), state.up1, copy_bound)?;
    write_old_items::<A>(state, &resized)?;
    current.array_state().write().value = resized;
    Ok(())
}

/// A new array `(lower, up1)` with the items of `items` up to `copy_bound`.
fn resized_copy<A: ArrayAttribute>(
    items: &rustycad_tkernel::ncollection::Array1<'static, A::Item>,
    up1: i32,
    copy_bound: i32,
) -> Result<Handle<HArray1<A::Item>>, Failure> {
    let resized = new_default_array::<A>(items.lower(), up1)?;
    {
        let mut target = live(&resized, A::TYPE_NAME)?.change_array1();
        for index in items.lower()..=copy_bound.min(items.upper()) {
            let item = items
                .value(index)
                .cloned()
                .map_err(|e| e.into_failure(A::TYPE_NAME))?;
            target
                .set_value(index, item)
                .map_err(|e| e.into_failure(A::TYPE_NAME))?;
        }
    }
    Ok(resized)
}

/// Writes the recorded old items back at their indexes.
fn write_old_items<A: ArrayAttribute>(
    state: &ArrayDeltaState<A::Item>,
    target: &Handle<HArray1<A::Item>>,
) -> Result<(), Failure> {
    let mut target = live(target, A::TYPE_NAME)?.change_array1();
    for (index, item) in state.indexes.iter().zip(&state.values) {
        target
            .set_value(*index, item.clone())
            .map_err(|e| e.into_failure(A::TYPE_NAME))?;
    }
    Ok(())
}

/// Declares one array delta class.
macro_rules! array_delta {
    ($(#[$doc:meta])* $name:ident, $occt:literal, $attribute:ty) => {
        $(#[$doc])*
        pub struct $name {
            transient: StandardTransient,
            base: TdfAttributeDeltaBase,
            state: ArrayDeltaState<<$attribute as ArrayAttribute>::Item>,
        }
        impl_transient!($name, $occt, dyn TdfDeltaOnModification, transient,
            interfaces: [dyn TdfAttributeDelta, dyn TdfDeltaOnModification]);
        impl $name {
            /// The constructor `(OldAtt)`: records what the modification of the backup
            /// `old_attribute` changed, already in its bound handle.
            pub fn new(old_attribute: &Handle<$attribute>) -> Result<Handle<Self>, Failure> {
                let base = TdfAttributeDeltaBase::new(&old_attribute.cast());
                let label = old_attribute.get().map(|old| old.label()).unwrap_or_default();
                let state = record(old_attribute, &label)?;
                Ok(TdfAttributeDeltaBase::share(Self {
                    transient: StandardTransient::default(),
                    base,
                    state,
                }))
            }
        }
        impl TdfAttributeDelta for $name {
            fn attribute_delta_base(&self) -> &TdfAttributeDeltaBase {
                &self.base
            }
            /// `Apply`: restores the old bounds and items on the current attribute.
            fn apply(&self) -> Result<(), Failure> {
                apply::<$attribute>(&self.state, &self.attribute(), &self.label())
            }
        }
        impl TdfDeltaOnModification for $name {}
    };
}

array_delta!(
    /// `TDataStd_DeltaOnModificationOfIntArray`.
    TDataStdDeltaOnModificationOfIntArray,
    "TDataStd_DeltaOnModificationOfIntArray",
    TDataStdIntegerArray
);
array_delta!(
    /// `TDataStd_DeltaOnModificationOfRealArray`.
    TDataStdDeltaOnModificationOfRealArray,
    "TDataStd_DeltaOnModificationOfRealArray",
    TDataStdRealArray
);
array_delta!(
    /// `TDataStd_DeltaOnModificationOfByteArray`.
    TDataStdDeltaOnModificationOfByteArray,
    "TDataStd_DeltaOnModificationOfByteArray",
    TDataStdByteArray
);
array_delta!(
    /// `TDataStd_DeltaOnModificationOfExtStringArray`.
    TDataStdDeltaOnModificationOfExtStringArray,
    "TDataStd_DeltaOnModificationOfExtStringArray",
    TDataStdExtStringArray
);
