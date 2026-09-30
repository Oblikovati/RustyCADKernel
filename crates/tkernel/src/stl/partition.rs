//! libstdc++ 13 `std::partition` for bidirectional (and random-access) iterators.

/// Moves elements satisfying `predicate` to the front with the libstdc++
/// two-ended swap scan and returns the split offset.
///
/// The relative order inside each part is the (unstable) libstdc++ order.
/// Example: `let split = partition(&mut values, |v| *v % 2 == 0);`.
pub fn partition<E>(values: &mut [E], mut predicate: impl FnMut(&E) -> bool) -> usize {
    let mut first = 0;
    let mut last = values.len();
    loop {
        while first != last && predicate(&values[first]) {
            first += 1;
        }
        if first == last {
            return first;
        }
        last -= 1;
        while first != last && !predicate(&values[last]) {
            last -= 1;
        }
        if first == last {
            return first;
        }
        values.swap(first, last);
        first += 1;
    }
}
