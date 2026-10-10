//! libstdc++ 13 `std::sort`: introsort with median-of-three pivots, a heap-sort
//! depth fallback, and a final 16-element guarded insertion sort.
use super::heap;

/// `_S_threshold`: partitions at or below this size are left to insertion sort.
const THRESHOLD: usize = 16;

/// Sorts with the exact libstdc++ `std::sort(first, last, comp)` element moves.
///
/// Equivalent elements end in the same (unstable) order as the GCC oracle.
/// Example: `sort_by(&mut indices, |a, b| keys[*a] < keys[*b])`.
pub fn sort_by<E: Copy>(values: &mut [E], mut less: impl FnMut(&E, &E) -> bool) {
    let result = try_sort_by(values, |a, b| {
        Ok::<bool, std::convert::Infallible>(less(a, b))
    });
    match result {
        Ok(()) => (),
        Err(never) => match never {},
    }
}

/// Stops at a failing comparison, retaining the native partial element moves.
/// Example: `try_sort_by(&mut indices, |a, b| criterion(*a, *b))?`.
pub fn try_sort_by<E: Copy, SortError>(
    values: &mut [E],
    mut less: impl FnMut(&E, &E) -> Result<bool, SortError>,
) -> Result<(), SortError> {
    if values.is_empty() {
        return Ok(());
    }
    // `std::__lg(n) * 2`: floor(log2(n)) doubled.
    let depth_limit = 2 * values.len().ilog2() as usize;
    introsort_loop(values, depth_limit, &mut less)?;
    final_insertion_sort(values, &mut less)
}

fn introsort_loop<E: Copy, SortError>(
    values: &mut [E],
    mut depth_limit: usize,
    less: &mut impl FnMut(&E, &E) -> Result<bool, SortError>,
) -> Result<(), SortError> {
    let mut last = values.len();
    while last > THRESHOLD {
        if depth_limit == 0 {
            heap::partial_sort(&mut values[..last], less)?;
            return Ok(());
        }
        depth_limit -= 1;
        let cut = partition_pivot(&mut values[..last], less)?;
        introsort_loop(&mut values[cut..last], depth_limit, less)?;
        last = cut;
    }
    Ok(())
}

fn partition_pivot<E: Copy, SortError>(
    values: &mut [E],
    less: &mut impl FnMut(&E, &E) -> Result<bool, SortError>,
) -> Result<usize, SortError> {
    let last = values.len();
    move_median_to_first(values, [1, last / 2, last - 1], less)?;
    unguarded_partition(values, 1, last, less)
}

/// `__move_median_to_first(result=0, a, b, c)`.
fn move_median_to_first<E: Copy, SortError>(
    values: &mut [E],
    [a, b, c]: [usize; 3],
    less: &mut impl FnMut(&E, &E) -> Result<bool, SortError>,
) -> Result<(), SortError> {
    let chosen = if less(&values[a], &values[b])? {
        if less(&values[b], &values[c])? {
            b
        } else if less(&values[a], &values[c])? {
            c
        } else {
            a
        }
    } else if less(&values[a], &values[c])? {
        a
    } else if less(&values[b], &values[c])? {
        c
    } else {
        b
    };
    values.swap(0, chosen);
    Ok(())
}

/// `__unguarded_partition` around the pivot stored at index 0.
fn unguarded_partition<E: Copy, SortError>(
    values: &mut [E],
    mut first: usize,
    mut last: usize,
    less: &mut impl FnMut(&E, &E) -> Result<bool, SortError>,
) -> Result<usize, SortError> {
    loop {
        while less(&values[first], &values[0])? {
            first += 1;
        }
        last -= 1;
        while less(&values[0], &values[last])? {
            last -= 1;
        }
        if first >= last {
            return Ok(first);
        }
        values.swap(first, last);
        first += 1;
    }
}

fn final_insertion_sort<E: Copy, SortError>(
    values: &mut [E],
    less: &mut impl FnMut(&E, &E) -> Result<bool, SortError>,
) -> Result<(), SortError> {
    if values.len() <= THRESHOLD {
        insertion_sort(values, less)?;
        return Ok(());
    }
    insertion_sort(&mut values[..THRESHOLD], less)?;
    for index in THRESHOLD..values.len() {
        unguarded_linear_insert(values, index, less)?;
    }
    Ok(())
}

fn insertion_sort<E: Copy, SortError>(
    values: &mut [E],
    less: &mut impl FnMut(&E, &E) -> Result<bool, SortError>,
) -> Result<(), SortError> {
    for index in 1..values.len() {
        if less(&values[index], &values[0])? {
            // `move_backward(first, i, i + 1)` then place the new minimum first.
            values[..=index].rotate_right(1);
        } else {
            unguarded_linear_insert(values, index, less)?;
        }
    }
    Ok(())
}

fn unguarded_linear_insert<E: Copy, SortError>(
    values: &mut [E],
    mut last: usize,
    less: &mut impl FnMut(&E, &E) -> Result<bool, SortError>,
) -> Result<(), SortError> {
    let value = values[last];
    let mut next = last - 1;
    while less(&value, &values[next])? {
        values[last] = values[next];
        last = next;
        // Upstream decrements past the front here without reading it; stopping is
        // equivalent for every strict weak ordering (the front is a sentinel).
        let Some(previous) = next.checked_sub(1) else {
            break;
        };
        next = previous;
    }
    values[last] = value;
    Ok(())
}
