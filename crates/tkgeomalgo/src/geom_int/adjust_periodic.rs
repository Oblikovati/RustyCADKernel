//! `GeomInt`: the package class, whose only member adjusts a periodic parameter.

/// `GeomInt::AdjustPeriodic(thePar, theParMin, theParMax, thePeriod, theNewPar,
/// theOffset, theEps)`: a parameter more than `eps` outside `[par_min, par_max]` is
/// shifted by the whole periods (towards the other bound) of its distance to that bound.
/// Returns `(adjusted, new_par, offset)`, `adjusted` being `offset > 0`.
///
/// ```
/// use rustycad_tkgeomalgo::geom_int::adjust_periodic;
/// let (adjusted, u, offset) = adjust_periodic(-7.0, 0.0, 6.0, 6.0, 0.0);
/// assert!(adjusted);
/// assert_eq!((u, offset), (5.0, 12.0));
/// ```
pub fn adjust_periodic(
    par: f64,
    par_min: f64,
    par_max: f64,
    period: f64,
    eps: f64,
) -> (bool, f64, f64) {
    let mut offset: f64 = 0.0;
    let mut new_par: f64 = par;
    let b_min: bool = par_min - par > eps;
    let b_max: bool = par - par_max > eps;
    if b_min || b_max {
        let dp: f64 = if b_min { par_max - par } else { par_min - par };
        // `modf(dp / thePeriod, &aNbPer)`: the integral part.
        let nb_per: f64 = (dp / period).trunc();
        offset = nb_per * period;
        new_par += offset;
    }
    (offset > 0.0, new_par, offset)
}
