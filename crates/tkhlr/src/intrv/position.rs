// Rust translation of ModelingAlgorithms/TKHLR/Intrv/Intrv_Position.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intrv_Position: where an interval lies relative to another one.

/// Intrv_Position: the 13 positions of an interval relative to another one, from
/// completely before it to completely after it. The discriminants are the native
/// enumerator values.
///
/// ```
/// use rustycad_tkhlr::intrv::{Interval, Position};
/// let other = Interval::from_bounds(1.0, 2.0);
/// assert_eq!(Interval::from_bounds(-1.0, 0.0).position(&other), Position::Before);
/// assert_eq!(Position::After as i32, 12);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Position {
    /// Ends before the other starts.
    Before = 0,
    /// Ends where the other starts.
    JustBefore = 1,
    /// Starts before the other and ends inside it.
    OverlappingAtStart = 2,
    /// Starts before the other and ends where it ends.
    JustEnclosingAtEnd = 3,
    /// Starts before the other and ends after it.
    Enclosing = 4,
    /// Starts where the other starts and ends inside it.
    JustOverlappingAtStart = 5,
    /// Starts and ends where the other does.
    Similar = 6,
    /// Starts where the other starts and ends after it.
    JustEnclosingAtStart = 7,
    /// Starts and ends inside the other.
    Inside = 8,
    /// Starts inside the other and ends where it ends.
    JustOverlappingAtEnd = 9,
    /// Starts inside the other and ends after it.
    OverlappingAtEnd = 10,
    /// Starts where the other ends.
    JustAfter = 11,
    /// Starts after the other ends.
    After = 12,
}
