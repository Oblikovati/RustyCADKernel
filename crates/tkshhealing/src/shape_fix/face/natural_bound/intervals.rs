use crate::shape_analysis::adjust_by_period;

/// Removes a periodic interval from the still-free intervals using OCCT's two alignment passes.
pub(super) fn cut_interval(
    intervals: &mut Vec<(f64, f64)>,
    requested: (f64, f64),
    period: f64,
) -> bool {
    if intervals.is_empty() {
        return false;
    }
    for align_to in [requested.1, requested.0] {
        cut_aligned_interval(intervals, requested, period, align_to);
    }
    true
}

fn cut_aligned_interval(
    intervals: &mut Vec<(f64, f64)>,
    requested: (f64, f64),
    period: f64,
    align_to: f64,
) {
    let mut index = 0;
    while index < intervals.len() {
        let interval = intervals[index];
        let shift = adjust_by_period(align_to, (interval.0 + interval.1) * 0.5, period);
        let (start, end) = (requested.0 + shift, requested.1 + shift);
        if end <= interval.0 || start >= interval.1 {
            index += 1;
            continue;
        }
        index = cut_overlap(intervals, index, interval, (start, end));
    }
}

fn cut_overlap(
    intervals: &mut Vec<(f64, f64)>,
    index: usize,
    interval: (f64, f64),
    requested: (f64, f64),
) -> usize {
    if requested.0 > interval.0 {
        if requested.1 < interval.1 {
            intervals.insert(index, interval);
            intervals[index + 1].0 = requested.1;
        }
        intervals[index].1 = requested.0;
        return index + 1;
    }
    if requested.1 < interval.1 {
        intervals[index].0 = requested.1;
        return index + 1;
    }
    intervals.remove(index);
    index
}

/// Selects the midpoint of the longest remaining interval, retaining OCCT's first tie.
pub(super) fn find_best_interval(intervals: &[(f64, f64)]) -> f64 {
    let mut center = 0.0;
    let mut longest = -1.0;
    for (start, end) in intervals {
        let length = end - start;
        if length <= longest {
            continue;
        }
        longest = length;
        center = start + 0.5 * length;
    }
    center
}

#[cfg(test)]
mod tests {
    use super::{cut_interval, find_best_interval};

    #[test]
    fn cut_interval_splits_and_retains_the_two_free_sides() {
        let mut intervals = vec![(0.0, 10.0)];
        assert!(cut_interval(&mut intervals, (3.0, 7.0), 10.0));
        assert_eq!(intervals, [(0.0, 3.0), (7.0, 10.0)]);
    }

    #[test]
    fn cut_interval_removes_wrapped_coverage_in_both_alignments() {
        let mut intervals = vec![(0.0, 10.0)];
        assert!(cut_interval(&mut intervals, (9.0, 11.0), 10.0));
        assert_eq!(intervals, [(1.0, 9.0)]);
    }

    #[test]
    fn longest_interval_keeps_the_first_equal_length_choice() {
        assert_eq!(find_best_interval(&[(0.0, 4.0), (6.0, 10.0)]), 2.0);
    }
}
