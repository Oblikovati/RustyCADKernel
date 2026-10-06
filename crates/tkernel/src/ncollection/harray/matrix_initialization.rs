//! Matrix construction delegates allocation and assignment to the existing Array2/Array1 engine.
use super::{Array2, ArrayError, HArray2};

impl<T: Default + Send + Sync + 'static> HArray2<T> {
    /// Constructs all defaults, then assigns a borrowed donor in row-major order.
    /// Assignment panic drops initialized cells; the donor remains owned by its caller.
    /// `HArray2::with_value_ref_with_assignment(1, 2, 1, 3, &7, |slot, donor| *slot = *donor)?`.
    pub fn with_value_ref_with_assignment(
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
        donor: &T,
        assign: impl FnMut(&mut T, &T),
    ) -> Result<Self, ArrayError> {
        let mut cells = Array2::with_bounds(rl, ru, cl, cu)?;
        cells.init_ref_with_assignment(donor, assign);
        Ok(Self::from_owned(cells))
    }
}

impl<T: Default + Clone + Send + Sync + 'static> HArray2<T> {
    /// Allocates the rectangle and fills it with `value`; `HArray2::with_value(1, 2, 1, 3, 0)?`.
    pub fn with_value(rl: i32, ru: i32, cl: i32, cu: i32, value: T) -> Result<Self, ArrayError> {
        Self::with_value_ref(rl, ru, cl, cu, &value)
    }
    /// Assigns a borrowed donor into every row-major cell using Clone::clone_from.
    /// `let grid = HArray2::with_value_ref(1, 2, -1, 0, &String::from("OCCT"))?`.
    pub fn with_value_ref(
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
        donor: &T,
    ) -> Result<Self, ArrayError> {
        Self::with_value_ref_with_assignment(rl, ru, cl, cu, donor, T::clone_from)
    }
}
