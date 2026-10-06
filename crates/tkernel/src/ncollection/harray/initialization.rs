//! Native value construction borrows its donor and assigns after all defaults exist.
use super::{Array1, ArrayError, HArray1};

impl<T: Default + Send + Sync + 'static> HArray1<T> {
    /// Constructs defaults then assigns a borrowed donor without requiring copy construction.
    /// Assignment panic drops all initialized slots; the borrowed donor remains owned by its caller.
    /// `HArray1::with_value_ref_with_assignment(1, 3, &42, |slot, donor| *slot = *donor)?`.
    pub fn with_value_ref_with_assignment(
        lower: i32,
        upper: i32,
        donor: &T,
        assign: impl FnMut(&mut T, &T),
    ) -> Result<Self, ArrayError> {
        let mut values = Array1::with_bounds(lower, upper)?;
        values.init_ref_with_assignment(donor, assign);
        Ok(Self::from_owned(values))
    }
}

impl<T: Default + Clone + Send + Sync + 'static> HArray1<T> {
    /// Allocates the bounds and fills every slot with `value`; `HArray1::with_value(1, 3, 0.5)?`.
    pub fn with_value(lower: i32, upper: i32, value: T) -> Result<Self, ArrayError> {
        Self::with_value_ref(lower, upper, &value)
    }

    /// Uses Clone::clone_from as the native copy-assignment policy while retaining the donor.
    /// `let shared = HArray1::with_value_ref(1, 3, &String::from("OCCT"))?`.
    pub fn with_value_ref(lower: i32, upper: i32, donor: &T) -> Result<Self, ArrayError> {
        Self::with_value_ref_with_assignment(lower, upper, donor, T::clone_from)
    }
}
