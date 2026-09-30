// Support of the BRepExtrema_Ext* classes (docs/topalgo-design.md section 13) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The results of an indexed Extrema accessor, read once after a replay of the
//! Extrema object, and the sequence reads of the BRepExtrema classes.
use rustycad_tkernel::standard::Failure;

/// What an accessor `f(N)` returned for every `N` of `[1, count]`, plus what it returned
/// just below and just above that range (its exception, or a value when it does not
/// check the index). Example: `IndexedCapture::capture(2, |n| ext.square_distance(n))`.
#[derive(Clone)]
pub(crate) struct IndexedCapture<T: Clone> {
    below: Result<T, Failure>,
    values: Vec<Result<T, Failure>>,
    above: Result<T, Failure>,
}

impl<T: Clone> IndexedCapture<T> {
    /// Calls `accessor` on `0..=count + 1`.
    pub(crate) fn capture(count: i32, accessor: impl Fn(i32) -> Result<T, Failure>) -> Self {
        let count: i32 = count.max(0);
        Self {
            below: accessor(0),
            values: (1..=count).map(&accessor).collect(),
            above: accessor(count + 1),
        }
    }

    /// The result of the accessor for `n`: the captured one inside the range, the one
    /// captured just outside it otherwise.
    pub(crate) fn get(&self, n: i32) -> Result<T, Failure> {
        if n < 1 {
            return self.below.clone();
        }
        match self.values.get(n as usize - 1) {
            Some(value) => value.clone(),
            None => self.above.clone(),
        }
    }
}

/// `NCollection_Sequence::Value(N)`: the item `N` (from 1) or Standard_OutOfRange, the
/// check of the native sequence; `sequence_value("mySqDist", &values, 1)?`.
pub(crate) fn sequence_value<T: Clone>(name: &str, values: &[T], n: i32) -> Result<T, Failure> {
    let item: Option<&T> = usize::try_from(n - 1).ok().and_then(|i| values.get(i));
    item.cloned().ok_or_else(|| {
        Failure::out_of_range(&format!(
            "NCollection_Sequence::Value ({name}): index {n} outside [1, {}]",
            values.len()
        ))
    })
}
