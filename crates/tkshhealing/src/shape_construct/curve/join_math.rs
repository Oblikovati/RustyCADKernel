pub(super) fn endpoint_reversals(
    distance_11: f64,
    distance_21: f64,
    distance_12: f64,
    distance_22: f64,
    confusion: f64,
) -> (bool, bool) {
    let minimum_direct = distance_11.min(distance_21);
    let minimum_crossed = distance_12.min(distance_22);
    if (minimum_direct - minimum_crossed).abs() <= confusion || minimum_crossed > minimum_direct {
        return (distance_11 < distance_21, false);
    }
    if minimum_crossed < minimum_direct {
        return (distance_12 < distance_22, true);
    }
    (false, false)
}

#[cfg(test)]
mod tests {
    use super::endpoint_reversals;

    #[test]
    fn endpoint_selection_matches_native_tie_and_crossed_branches() {
        assert_eq!(endpoint_reversals(1., 2., 2., 1., 0.1), (true, false));
        assert_eq!(endpoint_reversals(2., 3., 0.1, 1., 0.1), (true, true));
    }
}
